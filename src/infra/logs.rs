use time::UtcOffset;
use time::macros::format_description;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, time::OffsetTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// 初始化日志系统
/// 返回一个 WorkerGuard，**必须**在 main 函数中持有它直到程序结束，
/// 否则日志可能无法写入文件。
pub fn init(log_dir: &str) -> WorkerGuard {
    // 1. 设置文件滚动策略 (按天滚动)
    // 文件名格式: dora-tool.2023-10-27.log
    let file_appender = tracing_appender::rolling::daily(log_dir, "dora-tool.log");

    // 2. 包装成非阻塞写入器 (Non-blocking)
    // 这一点至关重要！它确保文件 I/O 不会阻塞你的 USB 扫描线程或 Web 线程。
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // 3. 设置时间格式 (使用本地时间)
    // 需要处理时区偏移，这里简单处理，生产环境可能需要更严谨的时区获取
    let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    let timer = OffsetTime::new(
        offset,
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    );

    // 4. 定义控制台输出层 (Console Layer)
    // 显示彩色，格式紧凑
    let console_layer = fmt::layer()
        .with_timer(timer.clone())
        .with_target(true) // 显示模块路径
        .with_thread_names(true) // 显示线程名 (方便调试后台线程)
        .with_writer(std::io::stdout);

    // 5. 定义文件输出层 (File Layer)
    // 去除颜色 (ANSI Code)，记录更详细的信息
    let file_layer = fmt::layer()
        .with_timer(timer)
        .with_ansi(false) // 文件里不要颜色代码
        .with_writer(non_blocking);

    // 6. 注册全局订阅者
    // RUST_LOG=info 环境变量可以控制级别，默认 info
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    // 返回 guard，这就好比是一个“保镖”，只要它活着，后台写入线程就活着
    guard
}
