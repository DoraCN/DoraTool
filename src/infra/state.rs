use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use usb_resolver::RawDeviceInfo;

use crate::core::usb::models::DeviceConfig;

#[derive(Debug, Clone)]
pub struct AppState {
    // the path of config file
    // 配置文件的路径
    pub config_path: PathBuf,
    // Persistent rules (Web server modifies, USB worker reads)
    // 持久化规则 (Web server修改, USB worker读取)
    pub rules: Arc<RwLock<Vec<DeviceConfig>>>,
    // Real-time device list (modified by USB worker, read by Web server)
    // Here we store RawDeviceInfo; the Web layer is responsible for rendering it into a view.
    // 实时的设备列表 (USB worker修改, Web server读取)
    // 这里我们存 RawDeviceInfo，Web层负责渲染成 View
    pub live_devices: Arc<RwLock<Vec<RawDeviceInfo>>>,
}

impl AppState {
    // new state
    // 创建新的状态
    pub fn new(config_path: PathBuf, rules: Vec<DeviceConfig>) -> Self {
        Self {
            config_path,
            rules: Arc::new(RwLock::new(rules)),
            live_devices: Arc::new(RwLock::new(Vec::new())),
        }
    }
}
