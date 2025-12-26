use std::path::Path;
use tracing_appender::rolling;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};

pub fn init_logging(
    data_dir: &Path,
    current_db: Option<&str>,
) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    // Determine log directory
    let logs_dir = match current_db {
        Some(db) => data_dir.join(db).join("logs"),
        None => data_dir.join("logs"),
    };
    std::fs::create_dir_all(&logs_dir).ok();

    // Rolling daily log file
    let file_appender = rolling::daily(&logs_dir, "meridb.log");
    let (nb_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Env filter: prefer MERIDB_LOG; fallback to RUST_LOG; default to info
    let env_filter = EnvFilter::try_from_env("MERIDB_LOG")
        .or_else(|_| EnvFilter::try_from_env("RUST_LOG"))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Format: time, level, target, message
    let file_layer = fmt::layer()
        .with_writer(nb_writer)
        .with_ansi(false)
        .with_target(true)
        .with_level(true);

    let enable_stdout = EnvFilter::try_from_env("MERIDB_DEV").is_ok()
        || EnvFilter::try_from_env("MERIDB_LOG_STDOUT").is_ok();

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer);

    if enable_stdout {
        let stdout_layer = fmt::layer()
            .with_writer(std::io::stdout)
            .with_ansi(true)
            .with_target(true)
            .with_level(true);
        registry.with(stdout_layer).init();
    } else {
        registry.init();
    }

    Some(guard) // Keep guard alive in main; drop on exit to flush logs
}
