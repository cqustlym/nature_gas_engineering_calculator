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
    const pressures = content.getData().map(row => row[0]);

    // 获取 wellInfo 表中的相关数据
    const wellInfoData = wellInfo.getData()[0];
    const pc = wellInfoData[3]; // Pc 的值
    const tc = wellInfoData[4]; // Tc 的值
    const t = wellInfoData[1];  // 井底温度
    const rg = wellInfoData[2]; // rg 的值
    const n2 = wellInfoData[5]; // N2 的值
    const co2 = wellInfoData[6]; // CO2 的值
    const h2s = wellInfoData[7]; // H2S 的值

    // 逐行处理 pressures 数据
    const calculateAndUpdate = async (index, pressure) => {
        // 准备发送到后端的数据
        const requestData = {
            pressures: [parseFloat(pressure)], // 将字符串转换为浮点数
            pc: pc,
            tc: tc,
            t: t,
            rg: rg,
            n2: n2,
            co2: co2,
            h2s: h2s
        };

        console.log('Request data:', requestData); // 调试信息

        // 计算 Z 值
        try {
            const responseZ = await fetch('/api/calculateZ', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(requestData)
            });

            if (!responseZ.ok) {
                throw new Error('Network response was not ok');
            }

            const zValues = await responseZ.json();
            const zValue = zValues[0];

            // 更新 content 表中的第二列
            content.setDataAtCell(index, 1, zValue);

            // 计算 P/Z 并更新第三列
            const pOverZ = parseFloat(pressure) / zValue;
            content.setDataAtCell(index, 2, pOverZ);

            // 计算 Bg
            try {
                const responseBg = await fetch('/api/calculateBg', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify(requestData)
                });

                if (!responseBg.ok) {
                    throw new Error('Network response was not ok');
                }

                const bgValues = await responseBg.json();
                const bgValue = bgValues[0];

                // 更新 content 表中的第四列
                content.setDataAtCell(index, 3, bgValue);
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
                    body: JSON.stringify(requestData)
                });

                if (!responseNiandu.ok) {
                    throw new Error('Network response was not ok');
                }

                const nianduValues = await responseNiandu.json();
                const nianduValue = nianduValues[0];

                // 更新 content 表中的第五列
                content.setDataAtCell(index, 4, nianduValue);
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
                    body: JSON.stringify(requestData)
                });

                if (!responseCg.ok) {
                    throw new Error('Network response was not ok');
                }

                const cgValues = await responseCg.json();
                const cgValue = cgValues[0];

                // 更新 content 表中的第六列
                content.setDataAtCell(index, 5, cgValue);
            } catch (error) {
                console.error('Error calculating Cg values:', error);
                alert('计算 Cg 值时出错，请检查输入或联系管理员。');
            }

            // 计算 密度
            try {
                const responseDensity = await fetch('/api/calculateDensity', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify(requestData)
                });

                if (!responseDensity.ok) {
                    throw new Error('Network response was not ok');
                }

                const densityValues = await responseDensity.json();
                const densityValue = densityValues[0];

                // 更新 content 表中的第七列
                content.setDataAtCell(index, 6, densityValue);
            } catch (error) {
                console.error('Error calculating density values:', error);
                alert('计算 密度 值时出错，请检查输入或联系管理员。');
            }
        } catch (error) {
            console.error('Error calculating Z values:', error);
            alert('计算 Z 值时出错，请检查输入或联系管理员。');
        }
    };

    // 逐行处理 pressures 数据
    pressures.forEach((pressure, index) => {
        if (pressure !== null && pressure !== undefined && pressure !== '') {
            calculateAndUpdate(index, pressure);
        }
    });
});