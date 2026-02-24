use axum::{
    response::Redirect,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use tower_http::services::ServeDir;

mod db;
mod handlers;
mod models;
mod pressure;

// ============ 应用启动 ============
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    db::init_pool().await.expect("Failed to init pool");

    let app = Router::new()
        // 根路径重定向到登录页
        .route("/", get(|| async { Redirect::to("/login.html") }))
        // 认证接口
        .route("/api/login", post(handlers::login_handler))
        // 井数据接口
        .route("/api/getWellData", post(handlers::get_well_data_handler))
        // 单个计算接口
        .route("/api/calculateZ", post(handlers::calculate_z_handler))
        .route("/api/calculateBg", post(handlers::calculate_bg_handler))
        .route("/api/calculateCg", post(handlers::calculate_cg_handler))
        .route(
            "/api/calculateDensity",
            post(handlers::calculate_density_handler),
        )
        .route(
            "/api/calculateNiandu",
            post(handlers::calculate_niandu_handler),
        )
        .route("/api/calculatePwbs", post(handlers::calculate_pwbs_handler))
        // 批量计算接口
        .route(
            "/api/calculateBatchPVT",
            post(handlers::calculate_batch_pvt_handler),
        )
        .route(
            "/api/calculateBatchPb",
            post(handlers::calculate_batch_pb_handler),
        )
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
