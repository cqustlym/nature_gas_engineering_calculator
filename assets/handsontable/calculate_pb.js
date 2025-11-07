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
    const pressures = content.getData().map(row => row[0]);

    // 获取 wellInfo 表中的相关数据
    const wellInfoData = wellInfo.getData()[0];
    const pc = wellInfoData[5]; // Pc 的值
    const tc = wellInfoData[6]; // Tc 的值
    const t = wellInfoData[3];  // 井底温度
    const rg = wellInfoData[4]; // rg 的值
    const n2 = wellInfoData[7]; // N2 的值
    const co2 = wellInfoData[8]; // CO2 的值
    const h2s = wellInfoData[9]; // H2S 的值
    const md = wellInfoData[1]; // 中部井深
    const th = wellInfoData[2]; // 井口温度

    const calculateAndUpdate = async (index, pressure) => {
        // 准备发送到后端的数据
        const requestData = {
            well_no: wellInfoData[0],
            rg: rg,
            pc: pc,
            tc: tc,
            h: md, // 中部井深
            tts: th, // 井口温度
            tws: t, // 井底温度
            pts: parseFloat(pressure), // 井口压力
        };

        console.log('Request data:', requestData); // 调试信息

        // 计算 Pwbs
        try {
            const responsePwbs = await fetch('/api/calculatePwbs', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(requestData)
            });

            if (!responsePwbs.ok) {
                throw new Error('Network response was not ok');
            }

            const pwbsValue = await responsePwbs.json();

            // 更新 content 表中的第二列（井底压力）
            content.setDataAtCell(index, 1, pwbsValue);

            // 获取 Z 值
            let zValue;
            if (pwbsValue && pwbsValue.z) {
                zValue = pwbsValue.z;
            } else {
                // 如果 Pwbs 返回的值中没有 Z 值，则单独计算 Z 值
                const responseZ = await fetch('/api/calculateZ', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({
                        pressures: [parseFloat(pwbsValue)], // 使用井底压力
                        pc: pc,
                        tc: tc,
                        t: t,
                    })
                });

                if (!responseZ.ok) {
                    throw new Error('Network response was not ok');
                }

                const zValues = await responseZ.json();
                zValue = zValues[0];
            }

            // 更新 content 表中的第三列（Z）
            content.setDataAtCell(index, 2, zValue);

            // 计算 P/Z 并更新第四列
            const pOverZ = parseFloat(pwbsValue) / zValue; // 使用井底压力
            content.setDataAtCell(index, 3, pOverZ);

            // 计算 Bg
            try {
                const responseBg = await fetch('/api/calculateBg', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({
                        pressures: [parseFloat(pwbsValue)], // 使用井底压力
                        pc: pc,
                        tc: tc,
                        t: t,
                    })
                });

                if (!responseBg.ok) {
                    throw new Error('Network response was not ok');
                }

                const bgValues = await responseBg.json();
                const bgValue = bgValues[0];

                // 更新 content 表中的第五列（1/Bg）
                content.setDataAtCell(index, 4, 1 / bgValue);
            } catch (error) {
                console.error('Error calculating Bg values:', error);
                alert('计算 Bg 值时出错，请检查输入或联系管理员。');
            }

            // 计算 粘度 μ
            try {
                const responseNiandu = await fetch('/api/calculateNiandu', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({
                        pressures: [parseFloat(pwbsValue)], // 使用井底压力
                        pc: pc,
                        tc: tc,
                        t: t,
                        rg: rg,
                        n2: n2,
                        co2: co2,
                        h2s: h2s,
                    })
                });

                if (!responseNiandu.ok) {
                    throw new Error('Network response was not ok');
                }

                const nianduValues = await responseNiandu.json();
                const nianduValue = nianduValues[0];

                // 更新 content 表中的第六列（粘度）
                content.setDataAtCell(index, 5, nianduValue);
            } catch (error) {
                console.error('Error calculating Niandu values:', error);
                alert('计算 粘度 值时出错，请检查输入或联系管理员。');
            }

            // 计算 Cg
            try {
                const responseCg = await fetch('/api/calculateCg', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({
                        pressures: [parseFloat(pwbsValue)], // 使用井底压力
                        pc: pc,
                        tc: tc,
                        t: t,
                    })
                });

                if (!responseCg.ok) {
                    throw new Error('Network response was not ok');
                }

                const cgValues = await responseCg.json();
                const cgValue = cgValues[0];

                // 更新 content 表中的第七列（Cg）
                content.setDataAtCell(index, 6, cgValue);
            } catch (error) {
                console.error('Error calculating Cg values:', error);
                alert('计算 Cg 值时出错，请检查输入或联系管理员。');
            }
        } catch (error) {
            console.error('Error calculating Pwbs value:', error);
            alert('计算 Pwbs 值时出错，请检查输入或联系管理员。');
        }
    };

    // 逐行处理 pressures 数据
    pressures.forEach((pressure, index) => {
        if (pressure !== null && pressure !== undefined && pressure !== '') {
            calculateAndUpdate(index, pressure);
        }
    });
});