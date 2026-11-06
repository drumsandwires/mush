#[cfg(feature = "reedline-shell")]
use std::io::{BufRead, IsTerminal, Write};

#[cfg(feature = "reedline-shell")]
use reedline::{DefaultPrompt, DefaultPromptSegment, ExternalPrinter, Reedline, Signal};

#[cfg(feature = "reedline-shell")]
use crate::{
    Response,
    conductor::Conductor,
    sink::{EngineSink, ReedlineSink},
    zmq_client::ZmqClient,
};

#[cfg(feature = "reedline-shell")]
pub trait Shell {
    fn name(&self) -> &'static str;
    fn prompt(&self) -> &'static str;
    fn sink_name(&self) -> &'static str; // ← &'static str, not &str
    fn banner(&self) -> String;
    fn evaluate(
        &self,
        client: &mut ZmqClient,
        input: &str,
    ) -> impl std::future::Future<Output = Response> + Send;

    fn on_start(&self) {
        println!("{}", self.banner());
    }
    fn on_exit(&self) {
        println!("Bye!");
    }
}

#[cfg(feature = "reedline-shell")]
pub async fn run<S: Shell + 'static>(
    shell: S,
    cmd_addr: &str,
    pub_addr: &str,
    sample_rate: u32,
) -> Result<(), tmq::TmqError> {
    let printer = ExternalPrinter::<String>::new(32);

    let (mut conductor, _midi_queue, _audio_tx, _midi_tx) =
        Conductor::new(cmd_addr, pub_addr, sample_rate)?;

    let mut handle = conductor.subscribe();
    let ctx = conductor.context().clone();

    tokio::spawn(async move {
        conductor.run().await;
    });

    tokio::task::yield_now().await;

    let sink = ReedlineSink::new(printer.sender(), shell.sink_name());
    tokio::spawn(async move {
        while let Some(update) = handle.recv().await {
            sink.emit(update);
        }
    });

    let mut client = ZmqClient::connect(&ctx, cmd_addr, pub_addr)?;

    shell.on_start();

    if std::io::stdin().is_terminal() {
        run_interactive(&shell, &mut client, printer).await;
    } else {
        run_batch(&shell, &mut client).await;
    }

    shell.on_exit();
    Ok(())
}

#[cfg(feature = "reedline-shell")]
async fn run_interactive<S: Shell>(
    shell: &S,
    client: &mut ZmqClient,
    printer: ExternalPrinter<String>,
) {
    let mut editor = Reedline::create().with_external_printer(printer);
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic(shell.prompt().to_string()),
        DefaultPromptSegment::Empty,
    );

    loop {
        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => {
                let input = line.trim();
                match input {
                    "quit" | "exit" => break,
                    "" => {}
                    _ => match shell.evaluate(client, input).await {
                        Response::Output(msg) => println!("{msg}"),
                        Response::Empty => {}
                        Response::Exit(msg) => {
                            println!("{msg}");
                            break;
                        }
                    },
                }
            }
            Ok(Signal::CtrlD) => break,
            Ok(Signal::CtrlC) => println!("(interrupted)"),
            Err(e) => {
                eprintln!("error: {e}");
                break;
            }
            Ok(_) => {}
        }
    }
}

#[cfg(feature = "reedline-shell")]
async fn run_batch<S: Shell>(shell: &S, client: &mut ZmqClient) {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.expect("failed to read line");
        let input = line.trim().to_string();

        match input.as_str() {
            "quit" | "exit" => {
                writeln!(out, "Bye!").unwrap();
                break;
            }
            "" => {}
            _ => match shell.evaluate(client, &input).await {
                Response::Output(msg) => writeln!(out, "{msg}").unwrap(),
                Response::Empty => {}
                Response::Exit(msg) => {
                    writeln!(out, "{msg}").unwrap();
                    break;
                }
            },
        }
    }
}
