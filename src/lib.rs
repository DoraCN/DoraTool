use std::sync::Arc;

use anyhow::Result;
use tokio::net::TcpListener;
use tracing::info;

use crate::{core::usb, infra::state::AppState};

pub mod cli;
pub mod core;
pub mod infra;
pub mod server;

pub async fn run() -> Result<()> {
    // 1. 初始化日志系统 (保存到当前目录下的 logs 文件夹)
    // _guard 必须存在于 main 的整个生命周期
    let _guard = infra::logs::init("./logs");

    info!("系统启动中...");

    // init logs
    // tracing_subscriber::fmt::init();

    // init infra
    let paths = infra::config::AppPaths::new()?;
    let rules = infra::config::load_rules(&paths.config_file)?;

    // app state
    let state = Arc::new(AppState::new(paths.config_file, rules));

    usb::manager::start_background_monitor(state.as_ref().clone());

    info!("Web Server listening on port 3000...");
    let app = server::routes::create_router(state.clone());
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app).await?;

    Ok(())
}
