mod format;
mod repl;
mod sink;
mod version;

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
    let printer = ExternalPrinter::<String>::new(32);
    let sink = ReedlineSink::new(printer.sender(), "mush-reedline");

    // ── Conductor ─────────────────────────────────────────────────────
    let (mut conductor, _audio_tx, _midi_tx) =
        Conductor::new(addrs::CMD_INPROC, addrs::PUB_INPROC).expect("failed to bind conductor");

    let mut handle = conductor.subscribe();
    let ctx = conductor.context().clone();

    tokio::spawn(async move {
        conductor.run().await;
    });

    // ── Subscription task ─────────────────────────────────────────────
    tokio::spawn(async move {
        while let Some(update) = handle.recv().await {
            sink.emit(update);
        }
    });

    // ── ZMQ client ────────────────────────────────────────────────────
    let mut client = ZmqClient::connect(&ctx, addrs::CMD_INPROC, addrs::PUB_INPROC)
        .expect("failed to connect ZMQ client");

    println!(
        "Welcome to mush — Audio/MIDI Music shell v{}. Type 'help' for commands.",
        version::short()
    );

    // ── Reedline ──────────────────────────────────────────────────────
    let mut editor = Reedline::create().with_external_printer(printer);
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("mush".to_string()),
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

