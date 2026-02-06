use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{
    core::usb::{
        self,
        models::{DeviceConfig, DeviceView},
    },
    infra::state::AppState,
    server::{
        error::ApiError,
        response::{ApiResponse, ApiResult},
    },
};

pub async fn list_devices(State(state): State<Arc<AppState>>) -> ApiResult<Vec<DeviceView>> {
    // 读取规则
    let rules = { state.rules.read().unwrap().clone() };

    // 读取实施设备
    let raw_devices = { state.live_devices.read().unwrap().clone() };

    // 业务匹配
    let views = usb::service::match_raw_to_views(&raw_devices, &rules);

    Ok(ApiResponse::success(views))
}

pub async fn save_rules(
    State(state): State<Arc<AppState>>,
    Json(new_rules): Json<Vec<DeviceConfig>>,
) -> ApiResult {
    // 这里的 AppResult 默认泛型就是 ()

    // ... 校验逻辑 ...
    if new_rules.is_empty() {
        return Err(ApiError::InvalidParam);
    }

    // ... 序列化和保存逻辑 ...
    let json_str = serde_json::to_string_pretty(&new_rules).map_err(|e| {
        tracing::error!("Serialize error: {}", e);
        ApiError::Unknown
    })?;

    if let Err(e) = std::fs::write(&state.config_path, json_str) {
        // 返回带 HTTP 500 的 ApiResponse
        return Ok(ApiResponse::server_error(format!("写入失败: {}", e)));
    }

    // ... 更新内存 ...
    {
        let mut w = state.rules.write().unwrap();
        *w = new_rules;
    }

    // ==========================================
    // 之前: Ok(ApiResponse::success(()))
    // 现在: 直接调用 ok()
    // ==========================================
    Ok(ApiResponse::ok())
}
