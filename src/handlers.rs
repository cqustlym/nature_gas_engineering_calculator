use crate::models::*;
use crate::pressure;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use rayon::prelude::*;

// ============ 认证处理 ============
pub async fn login_handler(Json(req): Json<LoginReq>) -> Result<&'static str, StatusCode> {
    let rec = sqlx::query!(
        "SELECT username FROM users WHERE username = ? AND password = ?",
        req.username,
        req.password
    )
    .fetch_optional(&*crate::db::get_pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match rec {
        Some(_) => Ok("ok"),
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

// ============ 井数据处理 ============
pub async fn get_well_data_handler(
    Json(req): Json<WellDataReq>,
) -> Result<Json<Vec<WellData>>, StatusCode> {
    let well_no = &req.well_no;
    let rows = sqlx::query!(
        "SELECT wellname, md, th, tb, rg, pc, tc, n2, co2, h2s FROM gaswell WHERE wellname = ?",
        well_no
    )
    .fetch_all(&*crate::db::get_pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if rows.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let well_data: Vec<WellData> = rows
        .into_iter()
        .map(|row| WellData {
            wellname: row.wellname,
            md: row.md.unwrap_or_default(),
            th: row.th.unwrap_or_default(),
            tb: row.tb.unwrap_or_default(),
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

// ============ 单个计算处理 ============
pub async fn calculate_z_handler(
    Json(req): Json<CalculateZReq>,
) -> Result<Json<Vec<f64>>, StatusCode> {
    let z_values: Vec<f64> = req
        .pressures
        .par_iter()
        .map(|&p| pressure::z(req.pc, req.tc, req.t, p))
        .collect();
    Ok(Json(z_values))
}

pub async fn calculate_bg_handler(
    Json(req): Json<CalculateBgReq>,
) -> Result<Json<Vec<f64>>, StatusCode> {
    let bg_values: Vec<f64> = req
        .pressures
        .par_iter()
        .map(|&p| pressure::bg(req.pc, req.tc, req.t, p))
        .collect();
    Ok(Json(bg_values))
}

pub async fn calculate_cg_handler(
    Json(req): Json<CalculateCgReq>,
) -> Result<Json<Vec<f64>>, StatusCode> {
    let cg_values: Vec<f64> = req
        .pressures
        .par_iter()
        .map(|&p| pressure::cg(req.pc, req.tc, req.t, p))
        .collect();
    Ok(Json(cg_values))
}

pub async fn calculate_density_handler(
    Json(req): Json<CalculateDensityReq>,
) -> Result<Json<Vec<f64>>, StatusCode> {
    let density_values: Vec<f64> = req
        .pressures
        .par_iter()
        .map(|&p| pressure::density(req.rg, req.pc, req.tc, req.t, p))
        .collect();
    Ok(Json(density_values))
}

pub async fn calculate_niandu_handler(
    Json(req): Json<CalculateNianduReq>,
) -> Result<Json<Vec<f64>>, StatusCode> {
    let niandu_values: Vec<f64> = req
        .pressures
        .par_iter()
        .map(|&p| pressure::niandu(req.rg, req.pc, req.tc, req.t, p, req.n2, req.co2, req.h2s))
        .collect();
    Ok(Json(niandu_values))
}

pub async fn calculate_pwbs_handler(
    Json(req): Json<CalculatePwbsReq>,
) -> Result<impl IntoResponse, StatusCode> {
    let pwbs_value = pressure::pwbs(req.rg, req.pc, req.tc, req.h, req.tts, req.tws, req.pts);
    Ok(Json(vec![pwbs_value]))
}

// ============ 批量PVT计算 ============
pub async fn calculate_batch_pvt_handler(
    Json(req): Json<CalculateBatchPVTReq>,
) -> Result<Json<Vec<BatchPVTResp>>, StatusCode> {
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
                let density = 3.4844 * p * rg / (z * t);
                let niandu = calculate_viscosity(rg, t, density, n2, co2, h2s);
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

// ============ 批量PB计算 ============
pub async fn calculate_batch_pb_handler(
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
                let niandu = calculate_viscosity(rg, tws, density, n2, co2, h2s);
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

// ============ 工具函数 ============
/// 粘度计算 - 消除代码重复
pub fn calculate_viscosity(rg: f64, t: f64, density: f64, yn2: f64, yco2: f64, yh2s: f64) -> f64 {
    let kn2 = yn2 * (0.00005 * rg + 0.000047) * 100.0;
    let kco2 = yco2 * (0.000078 * rg + 0.00001) * 100.0;
    let kh2s = yh2s * (0.000058 * rg - 0.000018) * 100.0;

    let k = (0.0001 * (9.4 + 0.02 * 28.97 * rg) * (9.0 * t / 5.0).powf(1.5))
        / (209.0 + 19.0 * 28.97 * rg + 9.0 * t / 5.0)
        + kn2
        + kco2
        + kh2s;

    let x = 3.5 + 986.0 / (9.0 * t / 5.0) + 0.01 * 28.97 * rg;
    let y = 2.4 - 0.2 * x;

    k * (x * density.powf(y)).exp()
}
