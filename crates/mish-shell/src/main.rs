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

    let log_file = cli
        .log_file
        .or_else(|| mush_tracing::default_logfile(&app_name));

    let _guard = mush_tracing::init(mush_tracing::TracingConfig {
        log_level: cli.log_level,
        log_file,
    });

    tracing::info!(shell = app_name, version = version::compact(), "starting");
    tracing::debug!(version = version::long(), "full version");

    mush_core::shell::run(MishShell, addrs::CMD_INPROC, addrs::PUB_INPROC, 44_100)
        .await
        .expect("shell exited with error");
}
