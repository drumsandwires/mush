mod format;
mod repl;
mod sink;
mod version;

use tracing::{debug, trace};
use tracing_subscriber;

use mush_core::Response;
use mush_core::addrs;
use mush_core::conductor::Conductor;
use mush_core::zmq_client::ZmqClient;
use reedline::{DefaultPrompt, DefaultPromptSegment, ExternalPrinter, Reedline, Signal};

use mush_core::sink::EngineSink;
use sink::ReedlineSink;

#[tokio::main]
async fn main() {
    // Initialise tracing — respects RUST_LOG at runtime.
    // e.g. RUST_LOG=mush_core=debug,mish_shell=debug
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // ── ExternalPrinter ───────────────────────────────────────────────
    // Capacity of 32: enough to buffer a burst of updates without
    // blocking the Conductor. Messages beyond 32 are dropped by try_send.
    let printer = ExternalPrinter::<String>::new(32);
    let sink = ReedlineSink::new(printer.sender(), "mish-reedline");

    // ── Conductor ─────────────────────────────────────────────────────
    // subscribe() must be called before run() — run() consumes the
    // conductor, subscribe() requires &mut self.
    let (mut conductor, _audio_tx, _midi_tx) =
        Conductor::new(addrs::CMD_INPROC, addrs::PUB_INPROC).expect("failed to bind conductor");

    let mut handle = conductor.subscribe();
    let ctx = conductor.context().clone(); // ← shared context

    // Spawn the conductor loop — runs until shutdown.
    tokio::spawn(async move {
        conductor.run().await;
    });

    // Yield to the tokio runtime to allow the conductor task to
    // enter its select! loop before the ZMQ client connects.
    tokio::task::yield_now().await;

    // ── Subscription task ─────────────────────────────────────────────
    // Drives EngineUpdates from the SubscriptionHandle into the
    // ReedlineSink, which forwards them to the ExternalPrinter.
    // Runs concurrently with the REPL loop — never blocks readline.
    tokio::spawn(async move {
        debug!("sink task started");
        trace!("[sink task] started"); // ← temporary  // ← temporary
        while let Some(update) = handle.recv().await {
            trace!("[sink task] received: {update:?}"); // ← temporary
            sink.emit(update);
        }
        debug!("sink task exited");
    });

    // ── ZMQ client ────────────────────────────────────────────────────
    let mut client = ZmqClient::connect(&ctx, addrs::CMD_INPROC, addrs::PUB_INPROC)
        .expect("failed to connect ZMQ client");

    println!(
        "Welcome to mish — MIDI shell v{}. Type 'help' for commands.",
        version::short()
    );

    // ── Reedline ──────────────────────────────────────────────────────
    // with_external_printer wires the printer's receiver into the
    // reedline poll loop — updates appear above the prompt cleanly.
    let mut editor = Reedline::create().with_external_printer(printer);
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("mish".to_string()),
        DefaultPromptSegment::Empty,
    );

    loop {
        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => {
                let input = line.trim();
                match input {
                    "quit" | "exit" => {
                        println!("Bye!");
                        break;
                    }
                    _ => match repl::evaluate(&mut client, input).await {
                        Response::Output(msg) => println!("{msg}"),
                        Response::Empty => {}
                        Response::Exit(msg) => {
                            println!("{msg}");
                            break;
                        }
                    },
                }
            }
            Ok(Signal::CtrlD) => {
                println!("Bye!");
                break;
            }
            Ok(Signal::CtrlC) => println!("(interrupted)"),
            Err(e) => {
                eprintln!("error: {e}");
                break;
            }
            Ok(_) => {}
        }
    }
}

