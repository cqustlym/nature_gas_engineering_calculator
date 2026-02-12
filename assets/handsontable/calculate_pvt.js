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
                item.tb,
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
                colHeaders: ['井号', '井底温度(k)', 'rg', 'Pc(MPa)', 'Tc(k)', 'N<sub>2</sub>(%)', 'CO<sub>2</sub>(%)', 'H<sub>2</sub>S(%)'],
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
    console.log('Calculate button clicked'); // 调试信息

    if (!content || !wellInfo) {
        alert('请先点击“确定”生成表格！');
        return;
    }
    // 获取 content 表中的第一列数据
    const pressuresRaw = content.getData().map(row => row[0]);
    const pressures = pressuresRaw
        .map(v => (v === null || v === undefined || v === '' ? null : parseFloat(v)))
        .map(v => (Number.isNaN(v) ? null : v));

    // 获取 wellInfo 表中的相关数据
    const wellInfoData = wellInfo.getData()[0];
    const pc = wellInfoData[3]; // Pc 的值
    const tc = wellInfoData[4]; // Tc 的值
    const t = wellInfoData[1];  // 井底温度
    const rg = wellInfoData[2]; // rg 的值
    const n2 = wellInfoData[5]; // N2 的值
    const co2 = wellInfoData[6]; // CO2 的值
    const h2s = wellInfoData[7]; // H2S 的值

    // 只对有效 pressure 做批量请求（保留索引以便恢复）
    const requests = pressures
        .map((p, i) => ({ p, i }))
        .filter(x => x.p !== null);

    if (requests.length === 0) {
        alert('没有有效的压力输入。');
        return;
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
            console.error('Batch PVT calculation error:', err);
            alert('批量计算出错，请检查输入或联系管理员。');
        });
});