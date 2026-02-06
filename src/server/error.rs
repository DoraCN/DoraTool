use crate::server::response::ApiResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

// 定义错误宏
macro_rules! define_api_error {
    (
        $(
            $(#[$docs:meta])*
            ($variant:ident, $code:expr, $msg:expr, $status:expr);
        )+
    ) => {
        // Global application enumerations
        // 全局应用枚举
        pub enum ApiError {
            $(
                $(#[$docs])*
                $variant,
            )+
        }

        impl ApiError {
            // Get business error code
            // 获取业务错误码
            pub fn code(&self) -> i32 {
                match self {
                    $(
                        Self::$variant => $code,
                    )+
                }
            }

            // Get error information
            // 获取错误信息
            pub fn msg(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => $msg,
                    )+
                }
            }

            // Get the HTTP status code.
            // 获取 HTTP 状态码
            pub fn status(&self) -> StatusCode {
                match self {
                    $(
                        Self::$variant => $status,
                    )+
                }
            }
        }

        // Implementing IntoResponse
        // This means that Result<T, ApiError> can be used directly as the return value of a Handler.
        // 实现 IntoResponse
        // 这意味着 Result<T, ApiError> 可以直接作为 Handler 的返回值
        impl IntoResponse for ApiError {
            fn into_response(self) -> Response {
                // 调用我们之前封装好的 ApiResponse
                ApiResponse::<()>::error(self.code(), self.msg().to_string())
                    .status(self.status())
                    .into_response()
            }
        }
    };
}

// --- A centralized location for configuring error codes. ---
// --- 统一配置错误码的地方 ---
define_api_error! {
    /// 404 Not Found
    (NotFound, 404, "Resource Not Found", StatusCode::NOT_FOUND);

    /// 400 参数错误
    (InvalidParam, 1002, "Invalid Parameters", StatusCode::BAD_REQUEST);

    /// 401 未授权/未登录
    (Unauthorized, 1003, "Unauthorized Access", StatusCode::UNAUTHORIZED);

    /// 403 权限不足
    (PermissionDenied, 1004, "Permission Denied", StatusCode::FORBIDDEN);

    /// 500 数据库错误
    (DbError, 2001, "Database Error", StatusCode::INTERNAL_SERVER_ERROR);

    /// 500 未知错误
    (Unknown, 9999, "Unknown Server Error", StatusCode::INTERNAL_SERVER_ERROR);

    /// 500 唯一性冲突 (例如名称重复)
    (Conflict, 2002, "Resource Already Exists", StatusCode::CONFLICT);
}
