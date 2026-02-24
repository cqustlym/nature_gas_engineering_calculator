use serde::{Deserialize, Serialize};

// ============ 认证 ============
#[derive(Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

// ============ 井数据 ============
#[derive(Deserialize)]
pub struct WellDataReq {
    pub well_no: String,
}

#[derive(Serialize)]
pub struct WellData {
    pub wellname: String,
    pub md: f64,
    pub th: f64,
    pub tb: f64,
    pub rg: f64,
    pub pc: f64,
    pub tc: f64,
    pub n2: f64,
    pub co2: f64,
    pub h2s: f64,
}

// ============ 单个计算请求 ============
#[derive(Deserialize, Debug)]
pub struct CalculateZReq {
    pub pressures: Vec<f64>,
    pub pc: f64,
    pub tc: f64,
    pub t: f64,
}

#[derive(Deserialize, Debug)]
pub struct CalculateBgReq {
    pub pressures: Vec<f64>,
    pub pc: f64,
    pub tc: f64,
    pub t: f64,
}

#[derive(Deserialize, Debug)]
pub struct CalculateCgReq {
    pub pressures: Vec<f64>,
    pub pc: f64,
    pub tc: f64,
    pub t: f64,
}

#[derive(Deserialize, Debug)]
pub struct CalculateDensityReq {
    pub pressures: Vec<f64>,
    pub pc: f64,
    pub tc: f64,
    pub t: f64,
    pub rg: f64,
}

#[derive(Deserialize, Debug)]
pub struct CalculateNianduReq {
    pub pressures: Vec<f64>,
    pub pc: f64,
    pub tc: f64,
    pub t: f64,
    pub rg: f64,
    pub n2: f64,
    pub co2: f64,
    pub h2s: f64,
}

#[derive(Deserialize, Debug)]
pub struct CalculatePwbsReq {
    pub rg: f64,
    pub pc: f64,
    pub tc: f64,
    pub h: f64,
    pub tts: f64,
    pub tws: f64,
    pub pts: f64,
}

// ============ 批量PVT ============
#[derive(Deserialize)]
pub struct CalculateBatchPVTReq {
    pub pressures: Vec<f64>,
    pub pc: f64,
    pub tc: f64,
    pub t: f64,
    pub rg: f64,
    pub n2: f64,
    pub co2: f64,
    pub h2s: f64,
}

#[derive(Serialize)]
pub struct BatchPVTResp {
    pub z: f64,
    pub p_over_z: f64,
    pub bg: f64,
    pub niandu: f64,
    pub cg: f64,
    pub density: f64,
}

// ============ 批量PB ============
#[derive(Deserialize)]
pub struct CalculateBatchPbReq {
    pub pts: Vec<f64>,
    pub rg: f64,
    pub pc: f64,
    pub tc: f64,
    pub h: f64,
    pub tts: f64,
    pub tws: f64,
    pub n2: f64,
    pub co2: f64,
    pub h2s: f64,
}

#[derive(Serialize)]
pub struct BatchPbResp {
    pub pwbs: f64,
    pub z: f64,
    pub p_over_z: f64,
    pub bg: f64,
    pub niandu: f64,
    pub cg: f64,
}

#[derive(Deserialize)]
pub struct CalculateBatchPhReq {
    pub pwbs: Vec<f64>, // 井底压力数组
    pub well_no: String,
    pub rg: f64,
    pub pc: f64,
    pub tc: f64,
    pub h: f64,
    pub tts: f64,
    pub tws: f64,
    pub n2: f64,
    pub co2: f64,
    pub h2s: f64,
}

#[derive(Serialize)]
pub struct BatchPhResp {
    pub ph: f64, // 井口压力
    pub z: f64,
    pub p_over_z: f64,
    pub bg: f64,
    pub niandu: f64,
    pub cg: f64,
}
