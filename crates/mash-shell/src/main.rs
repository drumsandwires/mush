mod repl;
mod version;

use mush_core::Response;
use mush_core::addrs;
use mush_core::conductor::Conductor;
use mush_core::zmq_client::ZmqClient;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

#[tokio::main]
async fn main() {
    let (conductor, _audio_tx, _midi_tx) =
        Conductor::new(addrs::CMD_INPROC, addrs::PUB_INPROC).expect("failed to bind conductor");

    tokio::spawn(async move {
        conductor.run().await;
    });

    let mut client = ZmqClient::connect(addrs::CMD_INPROC, addrs::PUB_INPROC)
        .expect("failed to connect ZMQ client");

    println!(
        "Welcome to mash — Audio shell v{} Type 'help' for commands.",
        version::short()
    );

    let mut editor = Reedline::create();
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("mash".to_string()),
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
