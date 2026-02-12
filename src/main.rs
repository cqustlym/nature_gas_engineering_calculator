use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Redirect},
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::env;
use tower_http::services::ServeDir;
mod pressure;

// -------- Batch 请求/响应结构体 --------
#[derive(Deserialize)]
struct CalculateBatchPVTReq {
    pressures: Vec<f64>,
    pc: f64,
    tc: f64,
    t: f64,
    rg: f64,
    n2: f64,
    co2: f64,
    h2s: f64,
}

#[derive(Serialize)]
struct BatchPVTResp {
    z: f64,
    p_over_z: f64,
    bg: f64,
    niandu: f64,
    cg: f64,
    density: f64,
}

#[derive(Deserialize)]
struct CalculateBatchPbReq {
    pts: Vec<f64>,
    well_no: Option<String>,
    rg: f64,
    pc: f64,
    tc: f64,
    h: f64,
    tts: f64,
    tws: f64,
    n2: f64,
    co2: f64,
    h2s: f64,
}

#[derive(Serialize)]
struct BatchPbResp {
    pwbs: f64,
    z: f64,
    p_over_z: f64,
    bg: f64,
    niandu: f64,
    cg: f64,
}

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
            wellname: row.wellname,         // 直接使用 String
            md: row.md.unwrap_or_default(), // 中部井深
            th: row.th.unwrap_or_default(), // 井口温度
            tb: row.tb.unwrap_or_default(), // 井底温度
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
    let pressures = req.pressures;
    let pc = req.pc;
    let tc = req.tc;
    let t = req.t;

    let z_values: Vec<f64> = pressures
        .par_iter()
        .map(|&p| pressure::z(pc, tc, t, p))
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
    let pressures = req.pressures;
    let pc = req.pc;
    let tc = req.tc;
    let t = req.t;

    let bg_values: Vec<f64> = pressures
        .par_iter()
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
    let pressures = req.pressures;
    let pc = req.pc;
    let tc = req.tc;
    let t = req.t;

    let cg_values: Vec<f64> = pressures
        .par_iter()
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
    let pressures = req.pressures;
    let pc = req.pc;
    let tc = req.tc;
    let t = req.t;
    let rg = req.rg;

    let density_values: Vec<f64> = pressures
        .par_iter()
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
    let pressures = req.pressures;
    let pc = req.pc;
    let tc = req.tc;
    let t = req.t;
    let rg = req.rg;
    let n2 = req.n2;
    let co2 = req.co2;
    let h2s = req.h2s;

    let niandu_values: Vec<f64> = pressures
        .par_iter()
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
    let pwbs_value = pressure::pwbs(req.rg, req.pc, req.tc, req.h, req.tts, req.tws, req.pts);

    Ok(Json(vec![pwbs_value]))
}

// 批量 PVT 计算：一次返回每个点的 z, p_over_z, bg, niandu, cg, density
async fn calculate_batch_pvt_handler(
    Json(req): Json<CalculateBatchPVTReq>,
) -> Result<Json<Vec<BatchPVTResp>>, StatusCode> {
    // CPU密集型工作放到 blocking 线程
    let pressures = req.pressures.clone();
    let pc = req.pc;
    let tc = req.tc;
    let t = req.t;
    let rg = req.rg;
    let n2 = req.n2;
    let co2 = req.co2;
    let h2s = req.h2s;

    let result = tokio::task::spawn_blocking(move || {
        pressures
            .into_par_iter()
            .map(|p| {
                let z = pressure::z(pc, tc, t, p);
                let p_over_z = if z != 0.0 { p / z } else { 0.0 };
                let bg = 0.0003447 * z * t / p;

                // density (approx) reuse z to avoid recomputing inside density()
                let density = 3.4844 * p * rg / (z * t);

                // niandu inline (reuse density)
                let kn2 = n2 * (0.00005 * rg + 0.000047) * 100.0;
                let kco2 = co2 * (0.000078 * rg + 0.00001) * 100.0;
                let kh2s = h2s * (0.000058 * rg - 0.000018) * 100.0;
                let k = (0.0001 * (9.4 + 0.02 * 28.97 * rg) * (9.0 * t / 5.0).powf(1.5))
                    / (209.0 + 19.0 * 28.97 * rg + 9.0 * t / 5.0)
                    + kn2
                    + kco2
                    + kh2s;
                let x = 3.5 + 986.0 / (9.0 * t / 5.0) + 0.01 * 28.97 * rg;
                let y = 2.4 - 0.2 * x;
                let niandu = k * (x * density.powf(y)).exp();

                // cg use existing function (may recompute z internally)
                let cg = pressure::cg(pc, tc, t, p);

                BatchPVTResp {
                    z,
                    p_over_z,
                    bg,
                    niandu,
                    cg,
                    density,
                }
            })
            .collect()
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
}

// 批量 PB 计算: 接收多个 pts，返回每个点的 pwbs 及其对应的参数
async fn calculate_batch_pb_handler(
    Json(req): Json<CalculateBatchPbReq>,
) -> Result<Json<Vec<BatchPbResp>>, StatusCode> {
    let pts = req.pts.clone();
    let rg = req.rg;
    let pc = req.pc;
    let tc = req.tc;
    let h = req.h;
    let tts = req.tts;
    let tws = req.tws;
    let n2 = req.n2;
    let co2 = req.co2;
    let h2s = req.h2s;

    let result = tokio::task::spawn_blocking(move || {
        pts.into_par_iter()
            .map(|pt| {
                let pwbs = pressure::pws(rg, pc, tc, h, tts, tws, pt);
                let z = pressure::z(pc, tc, tws, pwbs);
                let p_over_z = if z != 0.0 { pwbs / z } else { 0.0 };
                let bg = 0.0003447 * z * tws / pwbs;

                let density = 3.4844 * pwbs * rg / (z * tws);
                let kn2 = n2 * (0.00005 * rg + 0.000047) * 100.0;
                let kco2 = co2 * (0.000078 * rg + 0.00001) * 100.0;
                let kh2s = h2s * (0.000058 * rg - 0.000018) * 100.0;
                let k = (0.0001 * (9.4 + 0.02 * 28.97 * rg) * (9.0 * tws / 5.0).powf(1.5))
                    / (209.0 + 19.0 * 28.97 * rg + 9.0 * tws / 5.0)
                    + kn2
                    + kco2
                    + kh2s;
                let x = 3.5 + 986.0 / (9.0 * tws / 5.0) + 0.01 * 28.97 * rg;
                let y = 2.4 - 0.2 * x;
                let niandu = k * (x * density.powf(y)).exp();

                let cg = pressure::cg(pc, tc, tws, pwbs);

                BatchPbResp {
                    pwbs,
                    z,
                    p_over_z,
                    bg,
                    niandu,
                    cg,
                }
            })
            .collect()
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
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
        // 批量 PVT 计算接口
        .route("/api/calculateBatchPVT", post(calculate_batch_pvt_handler))
        // 批量 PB 计算接口
        .route("/api/calculateBatchPb", post(calculate_batch_pb_handler))
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
