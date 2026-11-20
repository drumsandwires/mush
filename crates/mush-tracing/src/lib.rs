pub use tracing;

use dirs;
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub struct TracingConfig {
    pub log_level: Level,
    pub log_file: Option<PathBuf>,
    pub tokio_console: bool,
    pub console_port: u16,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            log_level: Level::INFO,
            log_file: None,
            tokio_console: false,
            console_port: 6669,
        }
    }
}

pub fn resolve_config(
    log_level: Level,
    log_file: Option<PathBuf>,
    console: bool,
    console_port: Option<u16>,
) -> TracingConfig {
    let tokio_console = console
        || std::env::var("MUSH_CONSOLE")
            .map(|v| matches!(v.as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);

    let console_port = console_port
        .or_else(|| {
            std::env::var("MUSH_CONSOLE_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
        })
        .unwrap_or(6669);

    let log_file = log_file.or_else(|| std::env::var("MUSH_LOG_FILE").ok().map(PathBuf::from));

    TracingConfig {
        log_level,
        log_file,
        tokio_console,
        console_port,
    }
}

pub fn default_logfile(name: &str) -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("drumsandwires").join(format!("{}.log", name)))
}

/// Initialises the tracing subscriber.
///
/// The console layer must be the first layer added to the registry —
/// `Box<dyn Layer<S>>` only implements `Layer<S>` for the exact `S` it
/// was constructed with. Adding it first means `S = Registry` throughout.
///
/// Returns a `WorkerGuard` that must be held for the lifetime of `main`.
pub fn init(config: TracingConfig) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let filter = EnvFilter::builder()
        .with_default_directive(config.log_level.into())
        .from_env_lossy();

    let stderr_layer = fmt::layer().compact().with_writer(std::io::stderr);

    // ── File layer ────────────────────────────────────────────────────────────
    let guard = if let Some(path) = config.log_file {
        let dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        let filename = path.file_name().expect("log file path has no filename");
        let appender = tracing_appender::rolling::never(dir, filename);
        let (non_blocking, guard) = tracing_appender::non_blocking(appender);

        let file_layer = fmt::layer()
            .compact()
            .with_ansi(false)
            .with_writer(non_blocking);

        // Console layer added first so S = Registry at the point of insertion.
        #[cfg(feature = "tokio-console")]
        let registry = {
            let base = tracing_subscriber::registry().with(config.tokio_console.then(|| {
                console_subscriber::ConsoleLayer::builder()
                    .server_addr(([127, 0, 0, 1], config.console_port))
                    .spawn()
            }));
            base.with(filter).with(stderr_layer).with(file_layer)
        };

        #[cfg(not(feature = "tokio-console"))]
        let registry = tracing_subscriber::registry()
            .with(filter)
            .with(stderr_layer)
            .with(file_layer);

        registry.init();
        Some(guard)
    } else {
        #[cfg(feature = "tokio-console")]
        let registry = {
            let base = tracing_subscriber::registry().with(config.tokio_console.then(|| {
                console_subscriber::ConsoleLayer::builder()
                    .server_addr(([127, 0, 0, 1], config.console_port))
                    .spawn()
            }));
            base.with(filter).with(stderr_layer)
        };

        #[cfg(not(feature = "tokio-console"))]
        let registry = tracing_subscriber::registry()
            .with(filter)
            .with(stderr_layer);

        registry.init();
        None
    };

    guard
}
