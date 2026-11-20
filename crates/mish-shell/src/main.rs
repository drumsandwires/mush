mod format;
mod repl;
mod version;

use mush_core::shell::Shell;
use mush_core::zmq_client::ZmqClient;
use mush_core::{Response, addrs};
use mush_tracing::tracing;

struct MishShell;

impl Shell for MishShell {
    fn name(&self) -> &'static str {
        "mish"
    }
    fn prompt(&self) -> &'static str {
        "mish"
    }
    fn sink_name(&self) -> &'static str {
        "mish-reedline"
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
    let log_file = mush_tracing::default_logfile("mish");

    let _guard = mush_tracing::init(mush_tracing::TracingConfig {
        log_level: tracing::Level::INFO,
        log_file,
    });

    tracing::info!(shell = "mish", version = version::compact(), "starting");
    tracing::debug!(version = version::long(), "full version");

    mush_core::shell::run(MishShell, addrs::CMD_INPROC, addrs::PUB_INPROC, 44_100)
        .await
        .expect("shell exited with error");
}
