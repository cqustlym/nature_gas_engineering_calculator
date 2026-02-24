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

    let z = z(pc, tc, t, p);
    let density = 3.4844 * p * rg / (z * t);

    let llluopr = density; // 密度
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

//计算油管采气时的井底流动压力Pwf(利用平均温度和平均压缩系数)
//yn2:N2摩尔分数  yco2:CO2摩尔分数  yh2s:H2S摩尔分数  d1:油管直径，单位：m  ee:粗糙系数
//d1、h1---第1段油管直径和下入长度
//d2、h2---第2段油管直径和下入长度
//d3、h3---产层直径和油管底部至中部井深的长度

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

/// 计算井口压力（反向计算）
/// 已知井底压力，计算井口压力
/// 参数:
///   rg   – 气体相对密度（空气=1）
///   pc   – 假临界压力，MPa
///   tc   – 假临界温度，K
///   h    – 气柱高度（井深），m
///   tts  – 井口静温，℃
///   tws  – 井底静温，℃
///   pwbs – 井底静压，MPa
/// 返回:
///   井口静压，MPa
pub fn ph<Rg, Pc, Tc, H, Tts, Tws, Pwbs>(
    rg: Rg,
    pc: Pc,
    tc: Tc,
    h: H,
    tts: Tts,
    tws: Tws,
    pwbs: Pwbs,
) -> f64
where
    Rg: Into<f64>,
    Pc: Into<f64>,
    Tc: Into<f64>,
    H: Into<f64>,
    Tts: Into<f64>,
    Tws: Into<f64>,
    Pwbs: Into<f64>,
{
    let rg: f64 = rg.into();
    let pc: f64 = pc.into();
    let tc: f64 = tc.into();
    let h: f64 = h.into();
    let tts: f64 = tts.into();
    let tws: f64 = tws.into();
    let pwbs: f64 = pwbs.into();

    // 使用牛顿迭代法反向计算井口压力
    let mut pts = pwbs / (1.0 + h / 12192.0); // 初始估值

    for _ in 0..30 {
        let p_avg = (pts + pwbs) * 0.5;
        let t_avg = (tts + tws) * 0.5;
        let zz = z(pc, tc, t_avg, p_avg);

        // 根据井底压力公式反向推导
        let pts_new = pwbs * (zz * t_avg / (0.03415 * rg * h + zz * t_avg)).exp();

        if (pts_new - pts).abs() < 0.0001 {
            pts = pts_new;
            break;
        }
        pts = pts_new;
    }

    pts
}
