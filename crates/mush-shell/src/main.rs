mod format;
mod repl;
mod version;

use mush_core::shell::Shell;
use mush_core::zmq_client::ZmqClient;
use mush_core::{Response, addrs};

struct MushShell;

impl Shell for MushShell {
    fn name(&self) -> &'static str {
        "mush"
    }
    fn prompt(&self) -> &'static str {
        "mush"
    }
    fn sink_name(&self) -> &'static str {
        "mush-reedline"
    }
    fn banner(&self) -> String {
        format!(
            "Welcome to {} — Audio/MIDI Music shell. Type 'help' for commands.",
            version::short()
        )
    }
    async fn evaluate(&self, client: &mut ZmqClient, input: &str) -> Response {
        repl::evaluate(client, input).await
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    mush_core::shell::run(MushShell, addrs::CMD_INPROC, addrs::PUB_INPROC)
        .await
        .expect("shell exited with error");
}

