// 用来保存当前两个表格实例
let wellInfo = null;
let content = null;

// 事件监听器
document.getElementById('okBtn').addEventListener('click', () => {


    // 如果已经存在旧的表格实例，先销毁它们
    if (wellInfo) {
        wellInfo.destroy();
        wellInfo = null;
    }
    if (content) {
        content.destroy();
        content = null;
    }

    // 获取井号
    const wellNo = document.getElementById('wellNo').value.trim();

    if (!wellNo) {
        alert('请输入井号！');
        return;
    }



    // 发送 AJAX 请求
    fetch(`/api/getWellData?wellNo=${encodeURIComponent(wellNo)}`, {
        method: 'POST', // Use POST method
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ well_no: wellNo })
    })
        .then(response => {
            if (!response.ok) {
                throw new Error('Network response was not ok');
            }
            return response.json();
        })
        .then(data => {
            // 将数据转换为二维数组
            const wellDataArray = data.map(item => [
                item.wellname,
                item.tb,
                item.rg,
                item.pc,
                item.tc,
                item.n2,
                item.co2,
                item.h2s
            ]);



            // === 第一个表：井信息（可编辑） ===

            wellInfo = new Handsontable(document.getElementById('wellInfo'), {
                data: wellDataArray, // 使用转换后的二维数组
                rowHeaders: true,
                colHeaders: ['井号', '井底温度(k)', 'rg', 'Pc(MPa)', 'Tc(k)', 'N<sub>2</sub>(%)', 'CO<sub>2</sub>(%)', 'H<sub>2</sub>S(%)'],
                columns: [
                    { readOnly: true },     // 井号只读
                    { readOnly: false, type: 'numeric', numericFormat: { pattern: '0.00' } },    // 井底温度可编辑
                    { readOnly: false, type: 'numeric', numericFormat: { pattern: '0.000' } },   // rg可编辑
                    { readOnly: false, type: 'numeric', numericFormat: { pattern: '0.00' } },    // Pc可编辑
                    { readOnly: false, type: 'numeric', numericFormat: { pattern: '0.0' } },     // Tc可编辑
                    { readOnly: false, type: 'numeric', numericFormat: { pattern: '0.00' } },    // N₂可编辑
                    { readOnly: false, type: 'numeric', numericFormat: { pattern: '0.00' } },    // CO₂可编辑
                    { readOnly: false, type: 'numeric', numericFormat: { pattern: '0.00' } }     // H₂S可编辑
                ],
                language: 'zh-CN',
                licenseKey: 'non-commercial-and-evaluation',
                height: 'auto',
                width: 'auto',
                theme: 'material',
                // 添加编辑后的回调函数
                afterChange: function (changes, source) {
                    if (source === 'edit') {
                        // 参数修改后的处理（可选）
                        console.log('参数已修改:', changes);
                    }
                }
            });

            // 可选：自动重新计算函数
            function autoRecalculate() {
                console.log('参数修改，自动重新计算...');
                // 检查是否有压力数据
                if (content && content.getData().some(row => row[0] !== null && row[0] !== '')) {
                    // 延迟执行，避免频繁计算
                    setTimeout(() => {
                        document.getElementById('calculate').click();
                    }, 500);
                }
            }
            // === 第二个表：计算区域 ===

            const data2 = Array.from({ length: 7 }, () => Array(7).fill(null));
            content = new Handsontable(document.getElementById('content'), {
                data: data2,                       // 注意这里用 data2
                rowHeaders: true,
                colHeaders: ['压力(MPa)', 'Z', 'P/Z', 'Bg', 'μ', 'Cg', 'ρg(g/cm³)'],
                language: 'zh-CN',
                licenseKey: 'non-commercial-and-evaluation',
                height: 'auto',
                width: 'auto',
                theme: 'material'
            });
        })
        .catch(error => {
            console.error('Error fetching well data:', error);
            alert('无法获取井数据，请检查输入或联系管理员。');
        });
});

// 计算按钮的事件监听器
document.getElementById('calculate').addEventListener('click', () => {


    if (!content || !wellInfo) {
        console.error('请先点击"确定"生成表格！');
        throw new Error('Tables not initialized');
    }
    if (wellInfo.getData().length === 0 || !wellInfo.getData()[0]) {
        alert('井信息表为空，请先确定井号并加载数据。');
        return;
    }

    // 获取当前井信息数据（包括用户编辑后的参数）
    const currentWellData = wellInfo.getSourceData()[0];

    // 获取 content 表中的第一列数据
    const pressuresRaw = content.getSourceData().map(row => row[0]);
    const pressures = pressuresRaw
        .map(v => (v === null || v === undefined || v === '' ? null : parseFloat(v)))
        .map(v => (Number.isNaN(v) || !isFinite(v) ? null : v));

    // 获取 wellInfo 表中的相关数据（使用用户编辑后的数据）
    const wellInfoData = currentWellData;

    const pc = parseFloat(wellInfoData[3]); // Pc 的值
    const tc = parseFloat(wellInfoData[4]); // Tc 的值
    const t = parseFloat(wellInfoData[1]);  // 井底温度
    const rg = parseFloat(wellInfoData[2]); // rg 的值
    const n2 = parseFloat(wellInfoData[5]); // N2 的值
    const co2 = parseFloat(wellInfoData[6]); // CO2 的值
    const h2s = parseFloat(wellInfoData[7]); // H2S 的值

    // 验证参数是否为有效数字
    if (isNaN(pc) || !isFinite(pc) || isNaN(tc) || !isFinite(tc) || isNaN(t) || !isFinite(t) || isNaN(rg) || !isFinite(rg) || isNaN(n2) || !isFinite(n2) || isNaN(co2) || !isFinite(co2) || isNaN(h2s) || !isFinite(h2s)) {
        console.error('井信息参数必须是有效有限数字，请检查输入。');
        throw new Error('Invalid parameters');
    }

    // 验证参数范围
    if (pc <= 0 || tc <= 0 || t <= 0 || rg <= 0) {
        console.error('Pc、Tc、井底温度、rg必须大于0。');
        throw new Error('Parameters out of range');
    }
    if (n2 < 0 || n2 > 100 || co2 < 0 || co2 > 100 || h2s < 0 || h2s > 100) {
        console.error('N2、CO2、H2S百分比必须在0-100之间。');
        throw new Error('Percentages out of range');
    }

    // 只对有效 pressure 做批量请求（保留索引以便恢复）
    const requests = pressures
        .map((p, i) => ({ p, i }))
        .filter(x => x.p !== null);

    if (requests.length === 0) {
        console.error('没有有效的压力输入。');
        throw new Error('No valid pressures');
    }

    const batchReq = {
        pressures: requests.map(x => x.p),
        pc: pc,
        tc: tc,
        t: t,
        rg: rg,
        n2: n2,
        co2: co2,
        h2s: h2s
    };



    fetch('/api/calculateBatchPVT', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(batchReq)
    })
        .then(resp => {
            if (!resp.ok) throw new Error('Network response not ok');
            return resp.json();
        })
        .then(results => {
            // results 是按请求顺序返回的计算结果数组
            // 构造新的表格数据：保留压力列，填充其余列
            const newData = pressuresRaw.map((orig, idx) => {
                if (orig === null || orig === undefined || orig === '') return [orig, null, null, null, null, null, null];
                const foundIndex = requests.findIndex(r => r.i === idx);
                if (foundIndex === -1) return [orig, null, null, null, null, null, null];
                const r = results[foundIndex];
                return [parseFloat(orig), r.z, r.p_over_z, r.bg, r.niandu, r.cg, r.density];
            });

            // 一次性加载数据以减少重绘
            content.loadData(newData);
        })
        .catch(err => {
            console.error('PVT计算错误:', err);
            alert('批量计算出错: ' + (err.message || err));
        });
});