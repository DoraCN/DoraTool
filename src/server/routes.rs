use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::infra::state::AppState;
use crate::server::apis;

/// 创建应用路由
/// 接收共享状态 AppState，并将其注入到所有路由中
pub fn create_router(state: Arc<AppState>) -> Router {
    // 定义 CORS (允许跨域，方便前端开发调试)
    let cors = CorsLayer::permissive();

    Router::new()
        // --- 静态页面 (UI) ---
        // 访问根路径 / 时，返回 HTML 界面
        .route("/", get(apis::web::index_page))
        // --- API 接口 ---
        // 获取设备列表
        .route("/api/devices", get(apis::usb::list_devices))
        // 保存规则配置
        .route("/api/rules", post(apis::usb::save_rules))
        // --- 中间件 ---
        .layer(TraceLayer::new_for_http()) // HTTP 请求日志
        .layer(cors) // 跨域支持
        // --- 状态注入 ---
        // 这一步非常关键：它让 Handler 能够通过 State(state) 访问数据
        .with_state(state)
}
