use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::server::error::ApiError;

// Standardized API response structure
// 统一 API 响应结构
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,

    // HTTP Status (200, 400, 500)
    // HTTP状态码，不序列化到 json body中，只用于控制HTTP header
    #[serde(skip)]
    pub http_status: StatusCode,
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        (self.http_status, Json(self)).into_response()
    }
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            msg: "ok".to_string(),
            data: Some(data),
            http_status: StatusCode::OK,
        }
    }

    pub fn error(code: i32, msg: String) -> Self {
        Self {
            code,
            msg,
            data: None,
            http_status: StatusCode::OK,
        }
    }

    pub fn ok() -> Self {
        Self {
            code: 0,
            msg: "ok".to_string(),
            data: None,
            http_status: StatusCode::OK,
        }
    }

    pub fn server_error(msg: String) -> Self {
        Self {
            code: 500,
            msg,
            data: None,
            http_status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    // Chained calls: Setting the HTTP status code
    // 链式调用：设置 HTTP 状态码
    // 用在ApiError中
    pub fn status(mut self, status: StatusCode) -> Self {
        self.http_status = status;
        self
    }
}

impl Default for ApiResponse<()> {
    fn default() -> Self {
        Self::ok()
    }
}

// Unified return type for the application
// T defaults to (), so you can simply write AppResult when no return value is needed.
// 应用统一返回类型
// T 默认为 ()，这样不需要返回值时可以直接写 AppResult
pub type ApiResult<T = ()> = Result<ApiResponse<T>, ApiError>;
