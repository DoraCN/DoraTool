use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::core::usb::models::DeviceConfig;

/// 应用路径管理
/// 负责计算跨平台的标准路径 (Linux: ~/.local/share, ~/.config 等)
pub struct AppPaths {
    pub config_file: PathBuf, // ~/.config/dora-tool/usb_rules.json
    pub log_dir: PathBuf,     // ~/.local/share/dora-tool/
    pub pid_file: PathBuf,    // ~/.local/share/dora-tool/dora-tool.pid
}

impl AppPaths {
    pub fn new() -> Result<Self> {
        // 使用 "directories" crate 获取符合系统规范的路径
        // Linux: /home/user/.local/share/dora-tool
        // Mac:   /Users/user/Library/Application Support/com.dora.tool
        let proj_dirs = ProjectDirs::from("com", "dora", "dora-tool")
            .context("无法确定用户主目录，请检查系统环境")?;

        let config_dir = proj_dirs.config_dir();
        let data_dir = proj_dirs.data_local_dir();

        // 1. 确保目录存在 (mkdir -p)
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).context("无法创建配置目录")?;
        }
        if !data_dir.exists() {
            fs::create_dir_all(data_dir).context("无法创建数据目录")?;
        }

        info!("Config Dir: {:?}", config_dir);
        info!("Data Dir:   {:?}", data_dir);

        Ok(Self {
            config_file: config_dir.join("usb_rules.json"),
            log_dir: data_dir.to_path_buf(),
            pid_file: data_dir.join("dora-tool.pid"),
        })
    }
}

/// 加载规则配置
/// 如果文件不存在，返回空列表而不是报错
pub fn load_rules(path: &Path) -> Result<Vec<DeviceConfig>> {
    if !path.exists() {
        info!("配置文件不存在，将使用空规则列表初始化: {:?}", path);
        return Ok(Vec::new());
    }

    let content =
        fs::read_to_string(path).with_context(|| format!("无法读取配置文件: {:?}", path))?;

    // 如果文件为空，也返回空列表
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let rules: Vec<DeviceConfig> =
        serde_json::from_str(&content).with_context(|| "解析 JSON 配置文件失败，请检查格式")?;

    info!("已加载 {} 条规则", rules.len());
    Ok(rules)
}
