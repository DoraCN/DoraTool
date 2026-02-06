use axum::response::Html;

/// 返回内嵌的前端页面
pub async fn index_page() -> Html<&'static str> {
    Html(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>DoraTool USB Resolver</title>
        <style>
            :root { --primary: #2563eb; --bg: #f8fafc; --text: #1e293b; }
            body { font-family: -apple-system, sans-serif; background: var(--bg); color: var(--text); padding: 2rem; max-width: 1200px; margin: 0 auto; }
            .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem; }
            button { background: var(--primary); color: white; border: none; padding: 0.5rem 1rem; border-radius: 4px; cursor: pointer; font-weight: 500; }
            button:hover { opacity: 0.9; }
            button.save { background: #16a34a; }

            /* 消息提示框样式 */
            #msg-box { padding: 10px; margin-bottom: 10px; border-radius: 4px; display: none; }
            .error { background: #fee2e2; color: #991b1b; border: 1px solid #f87171; }
            .success { background: #dcfce7; color: #166534; border: 1px solid #4ade80; }

            table { width: 100%; border-collapse: collapse; background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
            th, td { padding: 1rem; text-align: left; border-bottom: 1px solid #e2e8f0; }
            th { background: #f1f5f9; font-weight: 600; }
            input[type="text"] { padding: 0.4rem; border: 1px solid #cbd5e1; border-radius: 4px; width: 100%; box-sizing: border-box; }
            .badge { display: inline-block; padding: 0.25rem 0.5rem; border-radius: 999px; font-size: 0.75rem; font-weight: 600; }
            .badge.bound { background: #dcfce7; color: #166534; }
            .badge.unbound { background: #f1f5f9; color: #64748b; }
            .info-row { font-size: 0.85em; color: #64748b; margin-top: 2px; }
        </style>
    </head>
    <body>
        <div class="header">
            <h1>🔌 DoraTool USB Resolver</h1>
            <div class="actions">
                <button onclick="loadData()">Refresh Devices</button>
                <button class="save" onclick="saveRules()">Save Configuration</button>
            </div>
        </div>

        <div id="msg-box"></div>

        <table>
            <thead>
                <tr>
                    <th style="width: 25%">Role (Unique)</th>
                    <th style="width: 40%">Device Info</th>
                    <th style="width: 20%">Binding Strategy</th>
                    <th style="width: 15%">Status</th>
                </tr>
            </thead>
            <tbody id="device-list">
                </tbody>
        </table>

        <script>
            let currentDevices = [];
            let isAutoRefreshing = true; // 控制自动刷新的开关

            function showMsg(msg, isError) {
                const box = document.getElementById('msg-box');
                box.style.display = 'block';
                box.className = isError ? 'error' : 'success';
                box.innerText = msg;
                setTimeout(() => box.style.display = 'none', 3000);
            }

            async function loadData(silent = false) {
                try {
                    const res = await fetch('/api/devices');
                    const data = await res.json();

                    if(data.code !== 0) {
                        if (!silent) showMsg(data.msg, true);
                        return;
                    }

                    // 注意：根据 ApiResponse 的结构，数据在 data.data.devices
                    // 如果 ApiResponse<Value> 返回的是 { "devices": [...], "saved_rules": [...] }
                    // currentDevices = data.data;

                    // 简单的 Diff 优化：如果数据字符串没变，就不重新渲染 DOM
                    // 防止 input 框输入时焦点丢失
                    const newDevicesStr = JSON.stringify(data.data);
                    const oldDevicesStr = JSON.stringify(currentDevices);

                    if (newDevicesStr !== oldDevicesStr) {
                        currentDevices = data.data;
                        render();
                        // 如果不是静默模式（比如手动点击），提示一下
                        if (!silent) console.log("Data updated");
                    }
                } catch (e) { if (!silent) showMsg('Connection failed: ' + e, true); }
            }

            function render() {
                const tbody = document.getElementById('device-list');
                tbody.innerHTML = currentDevices.map((dev, idx) => `
                    <tr>
                        <td>
                            <input type="text" id="role-${idx}"
                                value="${dev.assigned_role || ''}"
                                placeholder="e.g. top_camera"
                                style="font-weight: bold;">
                        </td>
                        <td>
                            <div class="info-row">
                                VID: <b>${dev.vid}</b> | PID: <b>${dev.pid}</b>
                            </div>
                            <div class="info-row">SN: ${dev.serial || 'N/A'}</div>
                            <div class="info-row">Port: ${dev.port_path}</div>
                        </td>
                        <td>
                            <select id="strategy-${idx}" style="padding:0.4rem">
                                <option value="port" ${dev.port_path ? 'selected' : ''}>Bind by Port Path</option>
                                <option value="serial" ${dev.serial ? '' : 'disabled'}>Bind by Serial</option>
                            </select>
                        </td>
                        <td>
                            <span class="badge ${dev.assigned_role ? 'bound' : 'unbound'}">
                                ${dev.assigned_role ? 'BOUND' : 'NEW'}
                            </span>
                        </td>
                    </tr>
                `).join('');
            }

            let refreshTimer = null;

            function startAutoRefresh() {
                if (refreshTimer) clearInterval(refreshTimer);
                // 每 1000 毫秒 (1秒) 自动拉取一次数据
                refreshTimer = setInterval(() => {
                    if (isAutoRefreshing) {
                        loadData(true); // true 表示静默加载，不弹窗报错
                    }
                }, 1000);
            }

            function pauseRefresh() {
                console.log("Input focused, pausing auto-refresh...");
                isAutoRefreshing = false;
            }

            function resumeRefresh() {
                console.log("Input blurred, resuming auto-refresh...");
                isAutoRefreshing = true;
                // 失去焦点后立即刷新一次，避免数据滞后
                loadData(true);
            }

            // 页面加载完毕后，立即加载一次，并启动自动刷新
            loadData();
            startAutoRefresh();

            async function saveRules() {
                const rules = [];
                const roles = new Set();

                for (let idx = 0; idx < currentDevices.length; idx++) {
                    const dev = currentDevices[idx];
                    const roleInput = document.getElementById(`role-${idx}`).value.trim();

                    if (!roleInput) continue;

                    if (roles.has(roleInput)) {
                        showMsg(`Duplicate role '${roleInput}' detected!`, true);
                        return;
                    }
                    roles.add(roleInput);

                    const strategy = document.getElementById(`strategy-${idx}`).value;

                    // 这里要注意：发给后端的是 DeviceConfig 结构
                    // 前端的 dev.vid 是 String ("0x1234")，但 DeviceConfig 需要 u16
                    // 所以我们需要在前端或者后端处理。
                    // 鉴于之前的设计：前端展示 String，后端保存 u16。
                    // 实际上，DeviceConfig 需要 u16。
                    // 最简单的做法：前端把原始的 hex string 转回 int 发给后端，
                    // 或者后端做宽容处理。

                    // 修正：DeviceView 中的 vid 是 String ("0x...")
                    // 我们可以直接 parse 回去

                    const rule = {
                        role: roleInput,
                        // 去掉 0x 并转为整数
                        vid: parseInt(dev.vid, 16),
                        pid: parseInt(dev.pid, 16),
                        serial: null,
                        port_path: null
                    };

                    if (strategy === 'serial' && dev.serial) {
                        rule.serial = dev.serial;
                    } else if (dev.port_path) {
                        rule.port_path = dev.port_path;
                    }
                    rules.push(rule);
                }

                const res = await fetch('/api/rules', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify(rules)
                });

                const ret = await res.json();
                if(ret.code === 0) {
                    showMsg('Saved successfully!', false);
                    loadData();
                } else {
                    showMsg(ret.message, true);
                }
            }

            loadData();
        </script>
    </body>
    </html>
    "#,
    )
}
