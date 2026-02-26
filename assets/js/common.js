// 公共函数库 - 气藏工程计算
// 用于PVT、井底压力、井口压力计算的公共函数

let wellInfo = null;
let content = null;

// 井信息表列头配置（通用）
const WELLINFO_HEADERS = ['井号', '中部井深(m)', '井口温度(k)', '井底温度(k)', 'rg', 'Pc(MPa)', 'Tc(k)', 'N<sub>2</sub>(%)', 'CO<sub>2</sub>(%)', 'H<sub>2</sub>S(%)'];

// Handsontable 通用配置
const HOT_CONFIG = {
    language: 'zh-CN',
    licenseKey: 'non-commercial-and-evaluation',
    height: 100, // 设置表格高度，启用滚动
    width: 'auto',
    theme: 'material',
    // 固定表头
    fixedRowsTop: 0, // 固定顶部1行（表头）
    stretchH: 'all', // 列宽自适应
    // 性能优化
    renderAllRows: false,
    autoRowSize: false,
    autoColSize: false
};

// 初始化井信息表格
function initWellInfoTable(wellDataArray, customHeaders = null) {
    const container = document.getElementById('wellInfo');
    if (!container) return null;

    return new Handsontable(container, {
        data: wellDataArray,
        rowHeaders: true,
        colHeaders: customHeaders || WELLINFO_HEADERS,
        language: 'zh-CN',
        licenseKey: 'non-commercial-and-evaluation',
        height: 'auto',
        width: 'auto',
        theme: 'material',
        fixedRowsTop: 0,
        stretchH: 'all',
        renderAllRows: false,
        autoRowSize: false,
        autoColSize: false,
        className: 'htCenter' // 所有单元格居中
    });
}

// 初始化计算结果表格
function initContentTable(colHeaders, rowCount = 20) {
    const container = document.getElementById('content');
    if (!container) return null;

    const data = Array.from({ length: rowCount }, () => Array(7).fill(null));
    return new Handsontable(container, {
        data: data,
        rowHeaders: true,
        colHeaders: colHeaders,
        language: 'zh-CN',
        licenseKey: 'non-commercial-and-evaluation',
        height: 500,
        width: 'auto',
        theme: 'material',
        fixedRowsTop: 0, //固定多少行数据
        stretchH: 'all',
        renderAllRows: false,
        autoRowSize: false,
        autoColSize: false,
        className: 'htCenter' // 所有单元格居中
    });
}

// 销毁表格实例
function destroyTables() {
    if (wellInfo) {
        wellInfo.destroy();
        wellInfo = null;
    }
    if (content) {
        content.destroy();
        content = null;
    }
}

// 获取井数据并初始化表格
function loadWellData(wellNo, onSuccess, wellInfoColHeaders = null, contentColHeaders = null) {
    console.log('loadWellData called:', wellNo);
    console.log('params:', { wellInfoColHeaders, contentColHeaders });

    if (!wellNo || !wellNo.trim()) {
        alert('请输入井号！');
        return;
    }

    destroyTables();

    fetch(`/api/getWellData?wellNo=${encodeURIComponent(wellNo)}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ well_no: wellNo })
    })
        .then(response => {
            if (!response.ok) throw new Error('Network response was not ok');
            return response.json();
        })
        .then(data => {
            const wellDataArray = data.map(item => [
                item.wellname,
                item.md,
                item.th,
                item.tb,
                item.rg,
                item.pc,
                item.tc,
                item.n2,
                item.co2,
                item.h2s
            ]);

            wellInfo = initWellInfoTable(wellDataArray, wellInfoColHeaders);
            content = initContentTable(contentColHeaders);

            if (onSuccess) onSuccess(data);
        })
        .catch(error => {
            console.error('Error fetching well data:', error);
            alert('无法获取井数据，请检查输入或联系管理员。');
        });
}

// 从表格获取井信息参数
function getWellInfoParams() {
    if (!wellInfo) return null;

    const data = wellInfo.getSourceData()[0];
    if (!data) return null;

    return {
        wellNo: data[0],
        md: parseFloat(data[1]),
        th: parseFloat(data[2]),
        t: parseFloat(data[3]),
        rg: parseFloat(data[4]),
        pc: parseFloat(data[5]),
        tc: parseFloat(data[6]),
        n2: parseFloat(data[7]),
        co2: parseFloat(data[8]),
        h2s: parseFloat(data[9])
    };
}

// 从表格获取压力数组
function getPressuresFromTable() {
    if (!content) return [];

    return content.getSourceData()
        .map(row => row[0])
        .map(v => (v === null || v === undefined || v === '' ? null : parseFloat(v)))
        .map(v => (Number.isNaN(v) ? null : v));
}

// 验证井信息参数
function validateWellInfoParams(params) {
    const { pc, tc, t, rg, n2, co2, h2s } = params;

    if (isNaN(pc) || !isFinite(pc) || isNaN(tc) || !isFinite(tc) ||
        isNaN(t) || !isFinite(t) || isNaN(rg) || !isFinite(rg) ||
        isNaN(n2) || !isFinite(n2) || isNaN(co2) || !isFinite(co2) || isNaN(h2s) || !isFinite(h2s)) {
        alert('井信息参数必须是有效有限数字，请检查输入。');
        return false;
    }

    if (pc <= 0 || tc <= 0 || t <= 0 || rg <= 0) {
        alert('Pc、Tc、井底温度、rg必须大于0。');
        return false;
    }

    if (n2 < 0 || n2 > 100 || co2 < 0 || co2 > 100 || h2s < 0 || h2s > 100) {
        alert('N2、CO2、H2S百分比必须在0-100之间。');
        return false;
    }

    return true;
}

// 处理批量计算响应并更新表格
function handleBatchResults(pressuresRaw, requests, results, resultMapper) {
    const newData = pressuresRaw.map((orig, idx) => {
        if (orig === null || orig === undefined || orig === '') {
            return [orig, null, null, null, null, null, null];
        }
        const foundIndex = requests.findIndex(r => r.i === idx);
        if (foundIndex === -1) {
            return [orig, null, null, null, null, null, null];
        }
        const r = results[foundIndex];
        return resultMapper(orig, r);
    });

    content.loadData(newData);
}

// 检查表格是否已初始化
function checkTablesInitialized() {
    if (!content || !wellInfo) {
        alert('请先点击"确定"生成表格！');
        return false;
    }
    return true;
}

// 导出函数供外部使用
window.HotUtils = {
    wellInfo: () => wellInfo,
    content: () => content,
    destroyTables,
    loadWellData,
    getWellInfoParams,
    getPressuresFromTable,
    validateWellInfoParams,
    handleBatchResults,
    checkTablesInitialized
};