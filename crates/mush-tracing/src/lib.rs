// Re-export tracing so binaries only need mush-tracing as a dependency
pub use tracing;

use std::path::PathBuf;
use std::time::Instant;
use tracing::Level;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub struct TracingConfig {
    pub log_level: Level,
    pub log_file: Option<PathBuf>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            log_level: Level::INFO,
            log_file: None,
        }
    }
}

/// Initialises the tracing subscriber.
///
/// Returns a `WorkerGuard` that must be held for the lifetime of `main`.
/// Dropping it flushes and closes the non-blocking file writer.
/// If no log file is configured, returns `None`.
pub fn init(config: TracingConfig) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let filter = EnvFilter::builder()
        .with_default_directive(config.log_level.into())
        .from_env_lossy();

    let stderr_layer = fmt::layer().compact().with_writer(std::io::stderr);

    let guard = if let Some(path) = config.log_file {
        let dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        let filename = path.file_name().expect("log file path has no filename");

        let appender = tracing_appender::rolling::never(dir, filename);
        let (non_blocking, guard) = tracing_appender::non_blocking(appender);

        let file_layer = fmt::layer()
            .compact()
            .with_ansi(false)
            .with_writer(non_blocking);

        tracing_subscriber::registry()
            .with(filter)
            .with(stderr_layer)
            .with(file_layer)
            .init();

        Some(guard)
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(stderr_layer)
            .init();

        None
    };

    #[cfg(feature = "tokio-console")]
    console_subscriber::init();

    guard
}

/// A diagnostic event captured on the RT path.
/// Must be `Copy` and allocation-free — pushed from audio callback
/// and MIDI RT thread into a lock-free ring buffer.
#[derive(Debug, Clone, Copy)]
pub enum DiagEvent {
    /// `midi_queue.push()` failed — queue was full.
    /// MIDI events are being dropped on the RT path.
    MidiQueueFull { frame: u64 },

    /// `status_tx.try_send()` dropped an update.
    /// Conductor is not draining fast enough.
    StatusChannelFull,

    /// `output.send()` failed in `MidiEngineRt`.
    MidiSendError,

    /// `timeline.try_read()` failed — lock was held.
    /// Timeline was not readable during this buffer period.
    TimelineLockMissed { frame: u64 },
}
