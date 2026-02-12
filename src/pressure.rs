use anyhow::Result;
use sqlx::MySqlPool;

//Dranchuk,Purris和Robinson法计算z
//pc临界压力  tc临界温度  t井底温度  p压力
pub fn z<Pc, Tc, T, P>(pc: Pc, tc: Tc, t: T, p: P) -> f64
where
    Pc: Into<f64>,
    Tc: Into<f64>,
    T: Into<f64>,
    P: Into<f64>,
{
    let pc = pc.into();
    let tc = tc.into();
    let t = t.into();
    let p = p.into();

    const A1: f64 = 0.31506237;
    const A2: f64 = -1.0467099;
    const A3: f64 = -0.57832729;
    const A4: f64 = 0.53530771;
    const A5: f64 = -0.61232032;
    const A6: f64 = -0.10488813;
    const A7: f64 = 0.68157001;
    const A8: f64 = 0.68446549;

    let ppr = p / pc;
    let tpr = t / tc;

    if ppr >= 0.1 && ppr <= 14.0 {
        let mut luopr = 0.27 * ppr / tpr;
        for _ in 0..25 {
            let fl = luopr - (0.27 * ppr) / tpr
                + (A1 + A2 / tpr + A3 / tpr.powi(3)) * luopr.powi(2)
                + (A4 + A5 / tpr) * luopr.powi(3)
                + (A5 * A6 * luopr.powi(6)) / tpr
                + (A7 * luopr.powi(3) / tpr.powi(3))
                    * (1.0 + A8 * luopr.powi(2))
                    * (-A8 * luopr.powi(2)).exp();

            let dfl = 1.0
                + (A1 + A2 / tpr + A3 / tpr.powi(3)) * (2.0 * luopr)
                + (A4 + A5 / tpr) * (3.0 * luopr.powi(2))
                + (A5 * A6 / tpr) * (6.0 * luopr.powi(5))
                + (A7 / tpr.powi(3)) * (3.0 * luopr.powi(2) + A8 * (3.0 * luopr.powi(4)))
                - A8.powi(2) * (2.0 * luopr.powi(6)) * (-A8 * luopr.powi(2)).exp();

            luopr -= fl / dfl;
        }
        0.27 * ppr / (luopr * tpr)
    } else {
        let tt = 1.0 / tpr;
        let mut yy = 0.06125 * ppr * tt * (-1.2 * (1.0 - tt).powi(2)).exp();
        for _ in 0..25 {
            let fh = -0.06125 * ppr * tt * (-1.2 * (1.0 - tt).powi(2)).exp()
                + (yy + yy.powi(2) + yy.powi(3) - yy.powi(4)) / (1.0 - yy).powi(3)
                - (14.76 * tt - 9.76 * tt.powi(2) + 4.58 * tt.powi(3)) * yy.powi(2)
                + (90.7 * tt - 242.2 * tt.powi(2) + 42.4 * tt.powi(3)) * yy.powf(2.18 + 2.82 * tt);

            let dh = (1.0 + 4.0 * yy + 4.0 * yy.powi(2) - 4.0 * yy.powi(3) + yy.powi(4))
                / (1.0 - yy).powi(4)
                - (29.52 * tt - 19.52 * tt.powi(2) + 9.16 * tt.powi(3)) * yy
                + (2.18 + 2.82 * tt)
                    * (90.7 * tt - 242.2 * tt.powi(2) + 42.4 * tt.powi(3))
                    * yy.powf(1.18 + 2.82 * tt);

            yy -= fh / dh;
        }
        0.06125 * ppr * tt * (-1.2 * (1.0 - tt).powi(2)).exp() / yy
    }
}

pub async fn query_z(
    pool: &MySqlPool,
    wellname: &str,
) -> Result<(Option<f64>, Option<f64>, Option<f64>)> {
    let rec = sqlx::query!(
        r#"
        SELECT pc, tc, tb
        FROM qcs.gaswell
        WHERE wellname = ?
        "#,
        wellname
    )
    .fetch_optional(pool) // 0 或 1 行
    .await?;

    match rec {
        Some(r) => Ok((r.pc, r.tc, r.tb)), // 数据库 NULL -> Rust None
        None => Ok((None, None, None)),
    }
}

//计算天然气体积系数
pub fn bg<Pc, Tc, T, P>(pc: Pc, tc: Tc, t: T, p: P) -> f64
where
    Pc: Into<f64>,
    Tc: Into<f64>,
    T: Into<f64>,
    P: Into<f64>,
{
    let pc = pc.into();
    let tc = tc.into();
    let t = t.into();
    let p = p.into();
    0.0003447 * z(pc, tc, t, p) * t / p
}

//计算粘度μ
// 'Lee,Gonzalez和Eakin法计算粘度μ,单位cp
// '杨继盛“采气工艺基础”（旧）40页
// '酸性气体修正（1986年）
pub fn niandu<Rg, Pc, Tc, T, P, Yn2, Yco2, Yh2s>(
    rg: Rg,
    pc: Pc,
    tc: Tc,
    t: T,
    p: P,
    yn2: Yn2,
    yco2: Yco2,
    yh2s: Yh2s,
) -> f64
where
    Rg: Into<f64>,
    Pc: Into<f64>,
    Tc: Into<f64>,
    T: Into<f64>,
    P: Into<f64>,
    Yn2: Into<f64>,
    Yco2: Into<f64>,
    Yh2s: Into<f64>,
{
    let rg: f64 = rg.into();
    let pc: f64 = pc.into();
    let tc: f64 = tc.into();
    let t: f64 = t.into();
    let p: f64 = p.into();
    let yn2: f64 = yn2.into();
    let yco2: f64 = yco2.into();
    let yh2s: f64 = yh2s.into();

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

    let llluopr = density(rg, pc, tc, t, p); // 计算密度
    let niandu = k * (x * llluopr.powf(y)).exp(); //niandu----粘度
    niandu
}

//计算天然气压缩系数
pub fn cg<Pc, Tc, T, P>(pc: Pc, tc: Tc, t: T, p: P) -> f64
where
    Pc: Into<f64>,
    Tc: Into<f64>,
    T: Into<f64>,
    P: Into<f64>,
{
    let pc: f64 = pc.into();
    let tc: f64 = tc.into();
    let t: f64 = t.into();
    let p: f64 = p.into();
    // 定义常数
    let a1 = 0.31506237;
    let a2 = -1.0467099;
    let a3 = -0.57832729;
    let a4 = 0.53530771;
    let a5 = -0.61232032;
    let a6 = -0.10488813;
    let a7 = 0.68157001;
    let a8 = 0.68446549;

    // 计算无量纲压力和温度
    let ppr = p / pc;
    let tpr = t / tc;

    // 初始化luopr
    let mut luopr = 0.27 * ppr / tpr;

    // 使用牛顿迭代法求解luopr
    for _ in 0..30 {
        let fluopr = luopr - (0.27 * ppr / tpr)
            + (a1 + a2 / tpr + a3 / tpr.powi(3)) * luopr.powi(2)
            + (a4 + a5 / tpr) * luopr.powi(3)
            + (a5 * a6 * luopr.powi(6) / tpr)
            + (a7 * luopr.powi(3) / tpr.powi(3))
                * (1.0 + a8 * luopr.powi(2))
                * (-a8 * luopr.powi(2)).exp();
        let dfluopr = 1.0
            + (a1 + a2 / tpr + a3 / tpr.powi(3)) * (2.0 * luopr)
            + (a4 + a5 / tpr) * (3.0 * luopr.powi(2))
            + (a5 * a6 / tpr) * (6.0 * luopr.powi(5))
            + (a7 / tpr.powi(3)) * (3.0 * luopr.powi(2) + a8 * (3.0 * luopr.powi(4)))
            - a8.powi(2) * (2.0 * luopr.powi(6)) * (-a8 * luopr.powi(2)).exp();
        luopr = luopr - fluopr / dfluopr;
    }

    // 计算压缩系数z
    let z = 0.27 * ppr / (luopr * tpr);

    // 计算dzlt
    let dzlt = (a1 + a2 / tpr + a3 / tpr.powi(3))
        + 2.0 * (a4 + a5 / tpr) * luopr
        + 5.0 * a5 * a6 * luopr.powi(4) / tpr
        + (2.0 * a7 * luopr / tpr.powi(3))
            * (1.0 + a8 * luopr.powi(2) - a8.powi(2) * luopr.powi(4))
            * (-a8 * luopr.powi(2)).exp();

    // 计算cpr
    let cpr = 1.0 / ppr - (0.27 / (z.powi(2) * tpr)) * (dzlt / (1.0 + luopr * dzlt / z));

    // 返回压缩系数cg
    cpr / pc
}

pub fn density<Rg, Pc, T, P>(rg: Rg, pc: Pc, tc: Rg, t: T, p: P) -> f64
where
    Pc: Into<f64>,
    Rg: Into<f64>,
    T: Into<f64>,
    P: Into<f64>,
{
    let pc: f64 = pc.into();
    let rg: f64 = rg.into();
    let p: f64 = p.into();
    let t = t.into();

    // 'Dranchuk,Purris和Robinson法计算z
    // 'Lee,Gonzalez和Eakin法计算密度ρ
    // '杨继盛“采气工艺基础”（旧）40页
    // 'Dranchuk,Purris和Robinson法计算z
    let zz = z(pc, tc, t, p);

    let lluopr = 3.4844 * p * rg / (zz * t);
    lluopr //'lluopr----密度
}
//平均温度和平均压缩系数计算法计算井底压力(静气柱）
//pws:pressure wellbore shut-in 关井状态下的井筒压力
//pts:pressure at tubing surface 井口压力
//tts:tubing temperature at surface"，即井口管柱温度。
//tws:tubing temperature at some specific depth"，即井筒温度或特定深度的管柱温度。
pub fn pws(rg: f64, pc: f64, tc: f64, h: f64, tts: f64, tws: f64, pts: f64) -> f64 {
    let mut pws = pts + pts * h / 12192.0; // 给定一个初值
    for _ in 0..100 {
        let p = (pts + pws) / 2.0; //平均井筒压力
        let t = (tts + tws) / 2.0; //平均井筒温度
        let zz = z(pc, tc, t, p); //调用函数计算压缩系数z
        pws = pts * ((0.03415 * rg * h) / (zz * t)).exp(); // 更新pws
    }
    return pws;
}

//平均温度和平均压缩系数计算法计算井口压力(静气柱）
pub fn pts(rg: f64, pc: f64, tc: f64, h: f64, tts: f64, tws: f64, pws: f64) -> f64 {
    let mut pts = pws - pws * h / 12192.0; // 初始化pts值

    for _ in 0..=50 {
        let p = (pts + pws) / 2.0;
        let t = (tts + tws) / 2.0;

        let zz = z(pc, tc, t, p); // 调用z函数计算压缩系数

        // 更新pts值
        pts = pws / ((0.03415 * rg * h) / (zz * t)).exp();
    }
    pts
}

pub fn fy(
    rg: f64,
    pc: f64,
    tc: f64,
    tts: f64,
    tws: f64,
    p: f64,
    yn2: f64,
    yco2: f64,
    yh2s: f64,
    d1: f64,
    q: f64,
    ee: f64,
) -> f64 {
    let kn2 = yn2 * (0.00005 * rg + 0.000047) * 100.0;
    let kco2 = yco2 * (0.000078 * rg + 0.00001) * 100.0;
    let kh2s = yh2s * (0.000058 * rg - 0.000018) * 100.0;

    let t = (tts + tws) / 2.0;
    let k = (0.0001 * (9.4 + 0.02 * 28.97 * rg) * (9.0 * t / 5.0).powf(1.5))
        / (209.0 + 19.0 * 28.97 * rg + 9.0 * t / 5.0)
        + kn2
        + kco2
        + kh2s;
    let x = 3.5 + 986.0 / (9.0 * t / 5.0) + 0.01 * 28.97 * rg;
    let y = 2.4 - 0.2 * x;

    let zz = z(pc, tc, t, p);
    let lluopr = 0.0014926 * (144.9275 * p) * (28.97 * rg) / (zz * (9.0 * t / 5.0)); // lluopr----密度
    let zhandu = k * (lluopr.powf(y)).exp(); // zhandu----粘度

    let re = 179.39789 * q * rg / (d1 * zhandu); // re----雷诺数
    1.0 / ((1.14 - 2.0 * ((ee / d1 + 21.25 / re.powf(0.9)).log10() / 10.0f64.log10().log10()))
        .powf(2.0))
}

//计算油管采气时的井底流动压力Pwf(利用平均温度和平均压缩系数)
//yn2:N2摩尔分数  yco2:CO2摩尔分数  yh2s:H2S摩尔分数  d1:油管直径，单位：m  ee:粗糙系数
//d1、h1---第1段油管直径和下入长度
//d2、h2---第2段油管直径和下入长度
//d3、h3---产层直径和油管底部至中部井深的长度
pub fn pwf<PC, TC, YN2, YCO2, YH2S, D1, D2, D3, H1, H2, H3, H, TTS, TWS, Q, PTF, EE>(
    rg: f64, // 已经确定是 f64，就不泛型了
    pc: PC,
    tc: TC,
    yn2: YN2,
    yco2: YCO2,
    yh2s: YH2S,
    d1: D1,
    d2: D2,
    d3: D3,
    h1: H1,
    h2: H2,
    h3: H3,
    h: H,
    tts: TTS,
    tws: TWS,
    q: Q,
    ptf: PTF,
    ee: EE,
) -> f64
where
    PC: Into<f64>,
    TC: Into<f64>,
    YN2: Into<f64>,
    YCO2: Into<f64>,
    YH2S: Into<f64>,
    D1: Into<f64>,
    D2: Into<f64>,
    D3: Into<f64>,
    H1: Into<f64>,
    H2: Into<f64>,
    H3: Into<f64>,
    H: Into<f64>,
    TTS: Into<f64>,
    TWS: Into<f64>,
    Q: Into<f64>,
    PTF: Into<f64>,
    EE: Into<f64>,
{
    // 统一转成 f64
    let pc = pc.into();
    let tc = tc.into();
    let yn2 = yn2.into();
    let yco2 = yco2.into();
    let yh2s = yh2s.into();
    let d1 = d1.into();
    let d2 = d2.into();
    let d3 = d3.into();
    let h1 = h1.into();
    let h2 = h2.into();
    let h3 = h3.into();
    let h = h.into();
    let tts = tts.into();
    let tws = tws.into();
    let q = q.into();
    let ptf = ptf.into();
    let ee = ee.into();

    let tj = 0.8742 * q + 20.22 + 273.15;
    let t_avg = (tts + tws) / 2.0;

    // 第1段
    let t1 = h1 * (tws - tj) / h + tj;
    let pwf1 = if d1 > 0.0 {
        let mut pwf1 = ptf + ptf * h1 / 12192.0;
        for _ in 1..=15 {
            let p = (pwf1 + ptf) / 2.0;
            let zz = z(pc, tc, t_avg, p);
            let ffy = fy(rg, pc, tc, tts, tws, p, yn2, yco2, yh2s, d1, q, ee);
            let s = 0.03415 * rg * h1 / (t_avg * zz);
            pwf1 = (pwf1.powi(2) * (2.0_f64).powf(2.0 * s)
                + 1.324e-10 * ffy * (q * t_avg * zz).powi(2) * ((2.0_f64).powf(2.0 * s) - 1.0)
                    / d1.powi(5))
            .sqrt();
        }
        pwf1
    } else {
        ptf
    };

    // 第2段
    let t2 = (h1 + h2) * (tws - tj) / h + tj;
    let pwf2 = if d2 > 0.0 {
        let mut pwf2 = pwf1 + pwf1 * h2 / 12192.0;
        for _ in 1..=15 {
            let p = (pwf1 + pwf2) / 2.0;
            let zz = z(pc, tc, (t1 + t2) / 2.0, p);
            let ffy = fy(rg, pc, tc, tts, tws, p, yn2, yco2, yh2s, d2, q, ee);
            let s = 0.03415 * rg * h2 / ((t1 + t2) / 2.0 * zz);
            pwf2 = (pwf1.powi(2) * (2.0_f64).powf(2.0 * s)
                + 1.324e-10
                    * ffy
                    * (q * (t1 + t2) / 2.0 * zz).powi(2)
                    * ((2.0_f64).powf(2.0 * s) - 1.0)
                    / d2.powi(5))
            .sqrt();
        }
        pwf2
    } else {
        pwf1
    };

    // 第3段
    let _t3 = (h1 + h2 + h3) * (tws - tj) / h + tj;
    let pwf3 = if d3 > 0.0 {
        let mut pwf3 = pwf2 + pwf2 * h3 / 12192.0;
        for _ in 1..=15 {
            let p = (pwf2 + pwf3) / 2.0;
            let zz = z(pc, tc, (t2 + tws) / 2.0, p);
            let ffy = fy(rg, pc, tc, tts, tws, p, yn2, yco2, yh2s, d3, q, ee);
            let s = 0.03415 * rg * h3 / ((t2 + tws) / 2.0 * zz);
            pwf3 = (pwf2.powi(2) * (2.0_f64).powf(2.0 * s)
                + 1.324e-10
                    * ffy
                    * (q * (t2 + tws) / 2.0 * zz).powi(2)
                    * ((2.0_f64).powf(2.0 * s) - 1.0)
                    / d3.powi(5))
            .sqrt();
        }
        pwf3
    } else {
        pwf2
    };
    pwf3
}

///平均温度和平均压缩系数计算法计算井口压力(静气柱）well pressure(bottom shutdown)
/// 静气柱井底压力（平均温度/平均压缩系数法）
/// 参数:
///   rg   – 气体相对密度（空气=1）
///   pc   – 假临界压力，MPa
///   tc   – 假临界温度，K
///   h    – 气柱高度（井深），m
///   tts  – 井口静温，℃
///   tws  – 井底静温，℃
///   pts  – 井口静压，MPa
/// 返回:
///   井底静压，MPa
pub fn pwbs<Rg, Pc, Tc, H, Tts, Tws, Pts>(
    rg: Rg,
    pc: Pc,
    tc: Tc,
    h: H,
    tts: Tts,
    tws: Tws,
    pts: Pts,
) -> f64
where
    Rg: Into<f64>,
    Pc: Into<f64>,
    Tc: Into<f64>,
    H: Into<f64>,
    Tts: Into<f64>,
    Tws: Into<f64>,
    Pts: Into<f64>,
{
    let rg: f64 = rg.into();
    let pc: f64 = pc.into();
    let tc: f64 = tc.into();
    let h: f64 = h.into();
    let tts: f64 = tts.into();
    let tws: f64 = tws.into();
    let pts: f64 = pts.into();

    // 初始估值：线性外推
    let mut pws = pts + pts * h / 12192.0;

    // 固定 30 次迭代，与 VBA 保持一致
    for _ in 0..30 {
        let p = (pts + pws) * 0.5;
        let t = (tts + tws) * 0.5;

        let zz = z(pc, tc, t, p); // 已实现的压缩因子函数
        pws = pts * (0.03415 * rg * h / (zz * t)).exp();
    }
    pws
}
