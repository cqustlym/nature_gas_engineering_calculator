// PVT计算 - 使用公共函数库

const CONTENT_HEADERS = ['压力(MPa)', 'Z', 'P/Z', 'Bg', '粘度', 'Cg', '密度'];

document.getElementById('okBtn').addEventListener('click', () => {
    const wellNo = document.getElementById('wellNo').value.trim();
    HotUtils.loadWellData(wellNo, null, null, CONTENT_HEADERS);
});

document.getElementById('calculate').addEventListener('click', () => {
    if (!HotUtils.checkTablesInitialized()) return;

    const params = HotUtils.getWellInfoParams();
    if (!HotUtils.validateWellInfoParams(params)) return;

    const pressuresRaw = HotUtils.getPressuresFromTable();
    const requests = pressuresRaw.map((p, i) => ({ p, i })).filter(x => x.p !== null);

    if (requests.length === 0) {
        alert('没有有效的压力输入。');
        return;
    }

    const batchReq = {
        pressures: requests.map(x => x.p),
        pc: params.pc,
        tc: params.tc,
        t: params.t,
        rg: params.rg,
        n2: params.n2,
        co2: params.co2,
        h2s: params.h2s
    };

    fetch('/api/calculateBatchPVT', {
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
                return [parseFloat(orig), r.z, r.p_over_z, r.bg, r.niandu, r.cg, r.density];
            });
            HotUtils.content().loadData(newData);
        })
        .catch(err => { console.error(err); alert('批量计算出错'); });
});