//! Logging configuration with human-readable output formatting.
//!
//! Provides beautiful, colorized console output for easy reading of Docker logs.

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize the logging system with human-readable formatting.
///
/// Features:
/// - Colorized output for different log levels
/// - Target module filtering
/// - Environment-based log level configuration (RUST_LOG)
pub fn init() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default log levels
        EnvFilter::new("vanmoi=info,tower_http=info,sqlx=warn")
    });

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_ansi(true); // Enable colors for Docker logs

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
