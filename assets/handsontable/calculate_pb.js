// 井底压力计算 - 使用公共函数库

const CONTENT_HEADERS = ['井口压力(MPa)', '井底压力(MPa)', 'Z', 'P/Z', '1/Bg', 'μ', 'Cg'];

document.getElementById('okBtn').addEventListener('click', () => {
    const wellNo = document.getElementById('wellNo').value.trim();
    HotUtils.loadWellData(wellNo, null, null, CONTENT_HEADERS);
});

// 计算按钮的事件监听器
document.getElementById('calculate').addEventListener('click', () => {
    if (!HotUtils.checkTablesInitialized()) return;

    const params = HotUtils.getWellInfoParams();
    if (!HotUtils.validateWellInfoParams(params)) return;

    const pressuresRaw = HotUtils.getPressuresFromTable();
    const requests = pressuresRaw.map((p, i) => ({ p, i })).filter(x => x.p !== null);

    if (requests.length === 0) {
        alert('没有有效的井口压力输入。');
        return;
    }

    const batchReq = {
        pts: requests.map(x => x.p),
        well_no: params.wellNo,
        rg: params.rg,
        pc: params.pc,
        tc: params.tc,
        h: params.md,
        tts: params.th,
        tws: params.t,
        n2: params.n2,
        co2: params.co2,
        h2s: params.h2s
    };

    fetch('/api/calculateBatchPb', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(batchReq)
    })
        .then(resp => { if (!resp.ok) throw new Error('Network error'); return resp.json(); })
        .then(results => {
            const newData = pressuresRaw.map((orig, idx) => {
                if (orig == null || orig === '') return [orig, null, null, null, null, null, null];
                const foundIndex = requests.findIndex(r => r.i === idx);
                if (foundIndex === -1) return [orig, null, null, null, null, null, null];
                const r = results[foundIndex];
                const invBg = r.bg !== 0 ? 1 / r.bg : null;
                return [parseFloat(orig), r.pwbs, r.z, r.p_over_z, invBg, r.niandu, r.cg];
            });
            HotUtils.content().loadData(newData);
        })
        .catch(err => { console.error(err); alert('批量计算出错'); });
});