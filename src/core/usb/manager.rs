// use std::{thread, time::Duration};

// use tracing::{debug, error, info};
// use usb_resolver::get_monitor;

// use crate::infra::state::AppState;

// // Starts the background USB management task.
// // Note: This function is non-blocking; it will immediately spawn a new thread and return.
// // 启动后台 USB 管理任务
// // 注意：这个函数是非阻塞的，它会立即 spawn 一个新线程并返回
// pub fn start_background_monitor(state: AppState) {
//     let tick = |state: &AppState| -> anyhow::Result<()> {
//         // Get the monitor instance
//         // 获取 monitor 实例,
//         let monitor = get_monitor();

//         // Perform a full scan.
//         // 执行全量扫描
//         let raw_devices = monitor.scan_now()?;

//         debug!("raw devices {:?}", raw_devices);

//         // Update shared state
//         // 更新共享状态
//         {
//             // Acquire the write lock
//             // 获取写锁
//             if let Ok(mut w) = state.live_devices.write() {
//                 // I'm choosing to perform a full replacement here.
//                 // 我这里选择全量替换
//                 // TODO 以后更改为更快捷的方式
//                 *w = raw_devices;
//             }
//         }

//         // Debug
//         debug!("Updated device list.");
//         Ok(())
//     };

//     thread::spawn(move || {
//         info!("USB Background Manager started.");

//         // This loop is responsible for continuously refreshing the device status.
//         // 这里的 loop 负责不断刷新设备状态
//         loop {
//             if let Err(e) = tick(&state) {
//                 error!("USB Manager Error: {}", e);
//             }

//             // Scanning interval: 1 second. This ensures real-time performance without consuming excessive CPU resources.
//             // 扫描间隔：1秒。既保证实时性，又不占用过多 CPU
//             thread::sleep(Duration::from_millis(300));
//         }
//     });
// }

use crossbeam_channel::{select, unbounded};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::core::usb::models::DeviceConfig;
use crate::infra::state::AppState;
use usb_resolver::{DeviceEvent, get_monitor}; // 确保引入

pub fn start_background_monitor(state: AppState) {
    // 1. 启动 Polling 线程 (负责发现新设备，但可能会阻塞)
    let state_for_poll = state.clone();
    thread::spawn(move || {
        run_polling_loop(state_for_poll);
    });

    // 2. 启动 Event 线程 (负责已配置设备的极速热插拔)
    let state_for_event = state.clone();
    thread::spawn(move || {
        run_event_listener(state_for_event);
    });
}

/// 任务 A: 事件监听 (解决拔出卡顿的核心)
fn run_event_listener(state: AppState) {
    info!("🚀 [Thread-Event] USB 热插拔监听已启动 (即时响应)");

    let (tx, rx) = unbounded();
    let monitor = get_monitor();

    // 获取当前的规则快照
    // 注意：如果运行期间修改了规则，这里最好有个机制能更新 monitor，
    // 但为了简单，目前仅使用启动时的规则。

    if let Err(e) = monitor.start(tx) {
        error!("无法启动内核事件监听: {}", e);
        return;
    }

    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    DeviceEvent::Attached(resolved) => {
                        info!("⚡ [Event] 设备极速上线: {}", resolved.system_path);
                        // Attached 通常也会被 Polling 扫到，这里可以不做操作，
                        // 或者为了快，手动触发一次 scan (但 scan 可能会阻塞)
                    }
                    DeviceEvent::Detached(role_name) => {
                        info!("⚡ [Event] 设备极速下线: {}", role_name);

                        // --- 关键操作：绕过阻塞的 Scan，直接操作内存 ---
                        let mut devices = state.live_devices.write().unwrap();
                        let current_rules = state.rules.read().unwrap();

                        // 我们需要找到这个 role 对应的 VID/PID，然后在 raw_devices 里把它删掉
                        if let Some(rule) = current_rules.iter().find(|r| r.role == role_name) {
                            let before_len = devices.len();

                            // 从列表中移除匹配该规则的设备
                            devices.retain(|d| {
                                // 如果 VID/PID 匹配，且序列号(如果有)也匹配，就删掉它
                                let vid_match = d.vid == rule.vid;
                                let pid_match = d.pid == rule.pid;
                                let serial_match = rule.serial == d.serial;

                                // 如果所有条件都符合，说明这就是那个被拔掉的设备，返回 false (删除)
                                !(vid_match && pid_match && serial_match)
                            });

                            let after_len = devices.len();
                            if before_len != after_len {
                                info!("✨ 已从内存中强制移除设备: {} (无需等待 Scan)", role_name);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Event Channel Closed: {}", e);
                break;
            }
        }
    }
}

/// 任务 B: 轮询扫描 (负责兜底和发现未知设备)
fn run_polling_loop(state: AppState) {
    info!("🐢 [Thread-Poll] USB 轮询扫描已启动 (发现新设备)");

    // 这里的 monitor 专门用于 scan
    let monitor = get_monitor();
    let mut last_seen_fingerprints = HashSet::new();

    loop {
        let start = Instant::now();

        // 这一步在设备拔出时会阻塞 5~10 秒
        match monitor.scan_now() {
            Ok(raw_devices) => {
                // 生成指纹用于日志 (同之前逻辑)
                let mut current_fingerprints = HashSet::new();
                for dev in &raw_devices {
                    let key = format!("{:04x}:{:04x}:{:?}", dev.vid, dev.pid, dev.port_path);
                    if !last_seen_fingerprints.contains(&key) {
                        info!("🔍 [Poll] 扫描到设备: (VID:{:04x})", dev.vid);
                    }
                    current_fingerprints.insert(key);
                }
                last_seen_fingerprints = current_fingerprints;

                // 更新内存
                // 注意：这里会直接覆盖 Event 线程的修改，
                // 但因为 Scan 发生了阻塞，当它运行到这里时，raw_devices 里肯定已经没有那个设备了。
                // 所以最终状态是一致的。
                let mut w = state.live_devices.write().unwrap();
                *w = raw_devices;
            }
            Err(e) => error!("Scan failed: {}", e),
        }

        let duration = start.elapsed();
        if duration > Duration::from_secs(1) {
            warn!(
                "⚠️  USB 扫描发生了 I/O 阻塞: {:.2}s (这是正常的 OS 行为，但 Event 线程已提前更新 UI)",
                duration.as_secs_f32()
            );
            // 如果刚刚卡了很久，说明刚发生了拔出，立即进行下一次扫描可能意义不大
            // 且不需要 sleep 太多，因为已经睡了 7 秒了
        } else {
            // 正常情况睡 300ms
            thread::sleep(Duration::from_millis(300));
        }
    }
}
