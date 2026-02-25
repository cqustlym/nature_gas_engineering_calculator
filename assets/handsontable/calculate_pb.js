// 用来保存当前两个表格实例
let wellInfo = null;
let content = null;

// 事件监听器
document.getElementById('okBtn').addEventListener('click', () => {
    console.log('Button clicked'); // 调试信息

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

    console.log('Fetching well data for wellNo:', wellNo); // 调试信息

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
            console.log('Received well data:', data); // 调试信息

            // 将数据转换为二维数组
            const wellDataArray = data.map(item => [
                item.wellname,
                item.md, // 中部井深
                item.th, // 井口温度
                item.tb, // 井底温度
                item.rg,
                item.pc,
                item.tc,
                item.n2,
                item.co2,
                item.h2s
            ]);

            console.log('Converted well data array:', wellDataArray); // 调试信息

            // === 第一个表：井信息 ===
            console.log('Initializing wellInfo table'); // 调试信息
            wellInfo = new Handsontable(document.getElementById('wellInfo'), {
                data: wellDataArray, // 使用转换后的二维数组
                rowHeaders: true,
                colHeaders: ['井号', '中部井深(m)', '井口温度(k)', '井底温度(k)', 'rg', 'Pc(MPa)', 'Tc(k)', 'N<sub>2</sub>(%)', 'CO<sub>2</sub>(%)', 'H<sub>2</sub>S(%)'],
                language: 'zh-CN',
                licenseKey: 'non-commercial-and-evaluation',
                height: 'auto',
                width: 'auto',
                theme: 'material'
            });

            // === 第二个表：计算区域 ===
            console.log('Initializing content table'); // 调试信息
            const data2 = Array.from({ length: 7 }, () => Array(7).fill(null));
            content = new Handsontable(document.getElementById('content'), {
                data: data2,                       // 注意这里用 data2
                rowHeaders: true,
                colHeaders: ['井口压力(MPa)', '井底压力(MPa)', 'Z', 'P/Z', '1/Bg', 'μ', 'Cg'], // 更新表头
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
    console.log('Calculate button clicked'); // 调试信息

    if (!content || !wellInfo) {
        alert('请先点击“确定”生成表格！');
        return;
    }

    // 获取 content 表中的第一列数据（井口压力）
    const pressuresRaw = content.getSourceData().map(row => row[0]);

    // 获取 wellInfo 表中的相关数据
    const wellInfoData = wellInfo.getSourceData()[0];
    const pc = parseFloat(wellInfoData[5]); // Pc (MPa)
    const tc = parseFloat(wellInfoData[6]); // Tc (k)
    const t = parseFloat(wellInfoData[3]); // 井底温度 (k)
    const rg = parseFloat(wellInfoData[4]); // rg (m)
    const n2 = parseFloat(wellInfoData[7]); // N2 (%)
    const co2 = parseFloat(wellInfoData[8]); // CO2 (%)
    const h2s = parseFloat(wellInfoData[9]); // H2S (%)
    const md = parseFloat(wellInfoData[1]); // 中部井深 (m)
    const th = parseFloat(wellInfoData[2]); // 井口温度 (k)

    const ptsArr = pressuresRaw
        .map(v => (v === null || v === undefined || v === '' ? null : parseFloat(v)))
        .map(v => (Number.isNaN(v) ? null : v));

    const requests = ptsArr.map((p, i) => ({ p, i })).filter(x => x.p !== null);
    if (requests.length === 0) {
        alert('没有有效的井口压力输入。');
        return;
    }

    const batchReq = {
        pts: requests.map(x => x.p),
        well_no: wellInfoData[0],
        rg: rg,
        pc: pc,
        tc: tc,
        h: md,
        tts: th,
        tws: t,
        n2: n2,
        co2: co2,
        h2s: h2s
    };

    fetch('/api/calculateBatchPb', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(batchReq)
    })
        .then(resp => {
            if (!resp.ok) throw new Error('Network response not ok');
            return resp.json();
        })
        .then(results => {
            // results 对应 requests 的顺序
            const newData = pressuresRaw.map((orig, idx) => {
                if (orig === null || orig === undefined || orig === '') return [orig, null, null, null, null, null, null];
                const foundIndex = requests.findIndex(r => r.i === idx);
                if (foundIndex === -1) return [orig, null, null, null, null, null, null];
                const r = results[foundIndex];
                // 注意 UI 需要第5列是 1/Bg
                const invBg = r.bg !== 0 ? 1 / r.bg : null;
                return [parseFloat(orig), r.pwbs, r.z, r.p_over_z, invBg, r.niandu, r.cg];
            });

            content.loadData(newData);
        })
        .catch(err => {
            console.error('Batch PB calculation error:', err);
            alert('批量计算出错，请检查输入或联系管理员。');
        });
});