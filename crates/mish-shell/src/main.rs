mod format;
mod repl;
mod version;

use clap::{CommandFactory, Parser};
use mush_core::shell::Shell;
use mush_core::zmq_client::ZmqClient;
use mush_core::{Response, addrs};
use mush_tracing::tracing;
use std::path::PathBuf;

/// mish — MIDI shell
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Log level: error, warn, info, debug, trace [default: info]
    #[arg(long, value_name = "LEVEL", default_value = "info")]
    log_level: tracing::Level,

    /// Log file path [default: ~/.local/share/drumsandwires/<bin>.log]
    #[arg(long, value_name = "PATH")]
    log_file: Option<PathBuf>,

    /// Enable tokio-console on the given port [default: 6669]
    #[arg(long, default_value = "false")]
    tokio_console: bool,

    /// tokio-console port [default: 6669]
    #[arg(long, value_name = "PORT")]
    console_port: Option<u16>,
}

struct MishShell;

impl Shell for MishShell {
    fn name(&self) -> &'static str {
        env!("CARGO_BIN_NAME")
    }
    fn prompt(&self) -> &'static str {
        env!("CARGO_BIN_NAME")
    }
    fn sink_name(&self) -> &'static str {
        concat!(env!("CARGO_BIN_NAME"), "-reedline")
    }
    fn banner(&self) -> String {
        format!(
            "Welcome to {} — MIDI shell. Type 'help' for commands.",
            version::short()
        )
    }
    async fn evaluate(&self, client: &mut ZmqClient, input: &str) -> Response {
        repl::evaluate(client, input).await
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let app_name = Cli::command().get_name().to_owned();

    // Resolve tracing config from CLI flags with env var fallbacks:
    //   MUSH_LOG_LEVEL, MUSH_LOG_FILE, MUSH_CONSOLE, MUSH_CONSOLE_PORT
    let config = mush_tracing::resolve_config(
        cli.log_level,
        cli.log_file
            .or_else(|| mush_tracing::default_logfile(&app_name)),
        cli.tokio_console,
        cli.console_port,
    );

    // Initialise tracing (stderr + optional file + optional tokio-console).
    //
    // tokio-console investigation notes (Stage 6, parked):
    //
    //   - `.cargo/config.toml` sets `RUSTFLAGS="--cfg tokio_unstable"` ✓
    //   - `mush-tracing` has a `tokio-console` feature that adds
    //     `ConsoleLayer` to the tracing-subscriber registry ✓
    //   - CLI flags `--tokio-console` / `--console-port` wired through
    //     `resolve_config` into `TracingConfig` ✓
    //   - Port binds and CLI connects (confirmed via `ss -tlnp`) ✓
    //   - h2 CANCEL on connection: caused by console-api version mismatch
    //     between `tokio-console` 0.1.14 (needs console-api 0.9) and
    //     released `console-subscriber` 0.3 (ships console-api 0.7) ✓ fixed
    //     by pinning console-subscriber to git rev b59758e5a6effe3a9df3ec84
    //   - TUI connects but shows no task events — root cause unresolved.
    //     Suspected: ConsoleLayer::spawn() / subscriber registry interaction.
    //     Parked until console-subscriber cuts a release matching CLI 0.1.14.
    let _guard = mush_tracing::init(config);

    tracing::info!(shell = app_name, version = version::compact(), "starting");
    tracing::debug!(version = version::long(), "full version");

    mush_core::shell::run(MishShell, addrs::CMD_INPROC, addrs::PUB_INPROC, 44_100)
        .await
        .expect("shell exited with error");
}
