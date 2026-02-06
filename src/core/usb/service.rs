use crate::core::usb::models::{DeviceConfig, DeviceView};
use usb_resolver::RawDeviceInfo; // 假设 RawDeviceInfo 在这里可用，或者从 models 引入

/// 纯业务逻辑：将“原始设备数据”与“配置规则”进行匹配，生成“视图数据”
///
/// 注意：这个函数不进行任何 IO 操作 (不调用 scan)，只进行内存计算。
/// 这使得它非常快，且易于单元测试。
pub fn match_raw_to_views(
    raw_devices: &[RawDeviceInfo],
    rules: &[DeviceConfig],
) -> Vec<DeviceView> {
    let mut views = Vec::new();

    // 遍历传入的原始设备快照
    for raw in raw_devices {
        let mut matched_role = None;

        // O(M*N) 的匹配逻辑 (通常 M, N 很小，完全没问题)
        for rule in rules {
            // 1. 硬件 ID 匹配 (u16 比对)
            if raw.vid == rule.vid && raw.pid == rule.pid {
                // 2. 序列号匹配 (如果规则配了 SN)
                let serial_match = match (&rule.serial, &raw.serial) {
                    (Some(r), Some(v)) => r == v,
                    (None, _) => true, // 规则没配 SN，视为匹配
                    _ => false,        // 规则配了但设备没有，不匹配
                };

                // 3. 物理路径匹配 (如果规则配了 Path)
                let path_match = rule.port_path == raw.port_path;

                // 4. 判定
                if serial_match && path_match {
                    matched_role = Some(rule.role.clone());
                    break; // 找到规则，跳出内层循环
                }
            }
        }

        // --- 转换逻辑 ---
        // 利用之前在 models.rs 实现的 From<RawDeviceInfo>
        // 注意：这里可能需要 raw.clone()，因为 DeviceView 拥有数据的所有权
        let mut view = DeviceView::from(raw.clone());
        view.role = matched_role;

        views.push(view);
    }

    views
}
