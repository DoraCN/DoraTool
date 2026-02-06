use serde::{Deserialize, Serialize};
use usb_resolver::RawDeviceInfo;

/// Configuration/Rules
/// Keep as numbers for easy comparison and JSON storage conventions
// 配置文件/规则 用户保存设备配置信息和读取
// 保持数字，方便比对，且符合 JSON 存储习惯
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub role: String, // Require and Unique
    pub vid: u16,
    pub pid: u16,
    pub serial: Option<String>,
    pub port_path: String,
}

/// Front View
// 前端视图 ( 直接以十六进制显示 "0x3290" )
#[derive(Debug, Serialize)]
pub struct DeviceView {
    pub role: Option<String>,
    pub vid: String, // 变更：直接发给前端 "0x3290"
    pub pid: String, // 变更：直接发给前端 "0x2645"
    pub serial: Option<String>,
    pub port_path: String,
    pub system_path: String,
}

impl From<RawDeviceInfo> for DeviceView {
    fn from(value: RawDeviceInfo) -> Self {
        Self {
            role: None,
            vid: format!("0x{:04x}", value.vid),
            pid: format!("0x{:04x}", value.pid),
            serial: value.serial,
            port_path: value.port_path,
            system_path: value.system_path,
        }
    }
}
