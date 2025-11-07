use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Json, Redirect}, // 导入 IntoResponse
    routing::{get, post},
};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use std::env;
use tower_http::services::ServeDir;
use tracing::{error, info};
mod pressure; // 引入 pressure 模块

// ---------------- 全局连接池 ----------------
static POOL: Lazy<MySqlPool> = Lazy::new(|| {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
            MySqlPoolOptions::new()
                .max_connections(500)
                .connect(&url)
                .await
                .expect("connect db error")
        })
    })
});

// ---------------- 登录请求体 ----------------
#[derive(Deserialize)]
struct LoginReq {
    username: String,
    password: String,
}

// ---------------- 登录接口 ----------------
async fn login_handler(Json(req): Json<LoginReq>) -> Result<&'static str, StatusCode> {
    let rec = sqlx::query!(
        "SELECT username FROM users WHERE username = ? AND password = ?",
        req.username,
        req.password
    )
    .fetch_optional(&*POOL)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match rec {
        Some(_) => Ok("ok"),                   // 200
        None => Err(StatusCode::UNAUTHORIZED), // 401
    }
}

// ---------------- 获取井数据请求体 ----------------
#[derive(Deserialize)]
struct WellDataReq {
    well_no: String,
}

// ---------------- 获取井数据接口 ----------------
async fn get_well_data_handler(
    Json(req): Json<WellDataReq>,
) -> Result<Json<Vec<WellData>>, StatusCode> {
    let well_no = &req.well_no;
    let rows = sqlx::query!(
        "SELECT wellname, md, th, tb, rg, pc, tc, n2, co2, h2s FROM gaswell WHERE wellname = ?",
        well_no
    )
    .fetch_all(&*POOL)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if rows.is_empty() {
        return Err(StatusCode::NOT_FOUND); // Return 404 if no well data found
    }

    let well_data: Vec<WellData> = rows
        .into_iter()
        .map(|row| WellData {
            wellname: row.wellname.clone().expect("REASON"), // 直接克隆 String
            md: row.md.unwrap_or_default(),                  // 中部井深
            th: row.th.unwrap_or_default(),                  // 井口温度
            tb: row.tb.unwrap_or_default(),                  // 井底温度
            rg: row.rg.unwrap_or_default(),
            pc: row.pc.unwrap_or_default(),
            tc: row.tc.unwrap_or_default(),
            n2: row.n2.unwrap_or_default(),
            co2: row.co2.unwrap_or_default(),
            h2s: row.h2s.unwrap_or_default(),
        })
        .collect();

    Ok(Json(well_data))
}

// ---------------- 井数据结构体 ----------------
#[derive(Serialize)]
struct WellData {
    wellname: String,
    md: f64, // 中部井深
    th: f64, // 井口温度
    tb: f64, // 井底温度
    rg: f64,
    pc: f64,
    tc: f64,
    n2: f64,
    co2: f64,
    h2s: f64,
}

// ---------------- 计算 Z 值请求体 ----------------
#[derive(Deserialize, Debug)]
struct CalculateZReq {
    pressures: Vec<f64>,
    pc: f64,
    tc: f64,
    t: f64,
}

// ---------------- 计算 Z 值接口 ----------------
async fn calculate_z_handler(Json(req): Json<CalculateZReq>) -> Result<Json<Vec<f64>>, StatusCode> {
    info!("Received request: {:?}", req);

    let pressures = req.pressures;
    let pc = req.pc;
    let tc = req.tc;
    let t = req.t;

    let z_values: Vec<f64> = pressures
        .iter()
        .map(|&p| {
            let result = pressure::z(pc, tc, t, p);
            info!("Calculated z value for pressure {}: {}", p, result);
            result
        })
        .collect();

    Ok(Json(z_values))
}

// ---------------- 计算 Bg 请求体 ----------------
#[derive(Deserialize, Debug)]
struct CalculateBgReq {
    pressures: Vec<f64>,
    pc: f64,
    tc: f64,
    t: f64,
}

// ---------------- 计算 Bg 接口 ----------------
async fn calculate_bg_handler(
    Json(req): Json<CalculateBgReq>,
) -> Result<Json<Vec<f64>>, StatusCode> {
    info!("Received request: {:?}", req);

    let pressures = req.pressures;
    let pc = req.pc;
    let tc = req.tc;
    let t = req.t;

    let bg_values: Vec<f64> = pressures
        .iter()
        .map(|&p| pressure::bg(pc, tc, t, p))
        .collect();

    Ok(Json(bg_values))
}

// ---------------- 计算 Cg 请求体 ----------------
#[derive(Deserialize, Debug)]
struct CalculateCgReq {
    pressures: Vec<f64>,
    pc: f64,
    tc: f64,
    t: f64,
}

// ---------------- 计算 Cg 接口 ----------------
async fn calculate_cg_handler(
    Json(req): Json<CalculateCgReq>,
) -> Result<Json<Vec<f64>>, StatusCode> {
    info!("Received request: {:?}", req);

    let pressures = req.pressures;
    let pc = req.pc;
    let tc = req.tc;
    let t = req.t;

    let cg_values: Vec<f64> = pressures
        .iter()
        .map(|&p| pressure::cg(pc, tc, t, p))
        .collect();

    Ok(Json(cg_values))
}

// ---------------- 计算 密度 请求体 ----------------
#[derive(Deserialize, Debug)]
struct CalculateDensityReq {
    pressures: Vec<f64>,
    pc: f64,
    tc: f64,
    t: f64,
    rg: f64,
}

// ---------------- 计算 密度 接口 ----------------
async fn calculate_density_handler(
    Json(req): Json<CalculateDensityReq>,
) -> Result<Json<Vec<f64>>, StatusCode> {
    info!("Received request: {:?}", req);

    let pressures = req.pressures;
    let pc = req.pc;
    let tc = req.tc;
    let t = req.t;
    let rg = req.rg;

    let density_values: Vec<f64> = pressures
        .iter()
        .map(|&p| pressure::density(rg, pc, tc, t, p))
        .collect();

    Ok(Json(density_values))
}

// ---------------- 计算 粘度 请求体 ----------------
#[derive(Deserialize, Debug)]
struct CalculateNianduReq {
    pressures: Vec<f64>,
    pc: f64,
    tc: f64,
    t: f64,
    rg: f64,
    n2: f64,
    co2: f64,
    h2s: f64,
}

// ---------------- 计算 粘度 接口 ----------------
async fn calculate_niandu_handler(
    Json(req): Json<CalculateNianduReq>,
) -> Result<Json<Vec<f64>>, StatusCode> {
    info!("Received request: {:?}", req);

    let pressures = req.pressures;
    let pc = req.pc;
    let tc = req.tc;
    let t = req.t;
    let rg = req.rg;
    let n2 = req.n2;
    let co2 = req.co2;
    let h2s = req.h2s;

    let niandu_values: Vec<f64> = pressures
        .iter()
        .map(|&p| pressure::niandu(rg, pc, tc, t, p, n2, co2, h2s))
        .collect();

    Ok(Json(niandu_values))
}

// ---------------- 计算 井底静压 接口 ----------------

// 定义请求结构体
#[derive(Deserialize, Serialize, Debug)]
struct CalculatePwbsReq {
    well_no: String,
    rg: f64,
    pc: f64,
    tc: f64,
    h: f64,
    tts: f64,
    tws: f64,
    pts: f64,
}
async fn calculate_pwbs_handler(
    Json(req): Json<CalculatePwbsReq>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Received request: {:?}", req);

    let pwbs_value = pressure::pwbs(req.rg, req.pc, req.tc, req.h, req.tts, req.tws, req.pts);

    Ok(Json(vec![pwbs_value]))
}

// ---------------- 启动入口 ----------------
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok(); // ① 先加载 .env
    let _ = &*POOL; // ② 再使用全局池

    let app = Router::new()
        // 根路径重定向到登录页
        .route("/", get(|| async { Redirect::to("/login.html") }))
        // 登录接口
        .route("/api/login", post(login_handler))
        // 获取井数据接口
        .route("/api/getWellData", post(get_well_data_handler))
        // 计算 Z 值接口
        .route("/api/calculateZ", post(calculate_z_handler))
        // 计算 Bg 接口
        .route("/api/calculateBg", post(calculate_bg_handler))
        // 计算 Cg 接口
        .route("/api/calculateCg", post(calculate_cg_handler))
        // 计算 密度 接口
        .route("/api/calculateDensity", post(calculate_density_handler))
        // 计算 粘度 接口
        .route("/api/calculateNiandu", post(calculate_niandu_handler))
        // 计算 井底静压 接口
        .route("/api/calculatePwbs", post(calculate_pwbs_handler))
        // 静态文件
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/login.html", ServeDir::new("assets/html/login.html"))
        .nest_service("/index.html", ServeDir::new("assets/html/index.html"))
        .nest_service(
            "/calculate_pvt.html",
            ServeDir::new("assets/html/calculate_pvt.html"),
        )
        .nest_service(
            "/calculate_pb.html",
            ServeDir::new("assets/html/calculate_pb.html"),
        )
        .nest_service(
            "/calculate_ph.html",
            ServeDir::new("assets/html/calculate_ph.html"),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("→ 打开 http://localhost:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

// 查询数据库
async fn query_db(_pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let rows = sqlx::query!("SELECT * FROM gaswell")
        .fetch_all(_pool)
        .await?;
    for row in rows {
        println!("{:#?}", row);
    }
    Ok(())
}

// 用户注册
async fn register(pool: &MySqlPool, username: &str, password: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO users (username, password) VALUES (?, ?)",
        username,
        password
    )
    .execute(pool)
    .await?;
    println!("{username} 注册成功");
    Ok(())
}
