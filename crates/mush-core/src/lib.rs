use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};
use std::io::{self, BufRead, IsTerminal, Write};

pub mod addrs;
pub mod conductor;
pub mod midi_output;
pub mod rt_priority;
pub mod sink;
pub mod subscription;
pub mod transport;
pub mod update;
pub mod version;
pub mod zmq_client;
pub mod zmq_transport;

/// The response returned by a [`Shell`] implementation after
/// evaluating one ar
#[derive(Debug, PartialEq)]
pub enum Response {
    Output(String),
    Empty,
    Exit(String),
}

pub trait Shell {
    /// The prompt string shown to the user, e.g. `"mish> "`.
    fn prompt(&self) -> &str;

    /// Evaluate one line of user input and return a [`Response`].
    /// The engine handles `quit`/`exit` before calling this.
    fn evaluate(&self, input: &str) -> Response;

    /// Called once before the REPL loop starts.
    fn on_start(&self) {}

    /// Called once after the REPL loop ends.
    fn on_exit(&self) {}
}

pub fn run<S: Shell>(shell: S) {
    shell.on_start();

    if std::io::stdin().is_terminal() {
        run_interactive(&shell);
    } else {
        run_batch(&shell);
    }

    shell.on_exit();
}

fn run_interactive<S: Shell>(shell: &S) {
    let mut editor = Reedline::create();
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic(shell.prompt().to_string()),
        DefaultPromptSegment::Empty,
    );

    loop {
        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => match line.trim() {
                "quit" | "exit" => {
                    println!("Bye!");
                    break;
                }
                input => match shell.evaluate(input) {
                    Response::Output(msg) => println!("{msg}"),
                    Response::Empty => {}
                    Response::Exit(msg) => {
                        println!("{msg}");
                        break;
                    }
                },
            },
            Ok(Signal::CtrlD) => {
                println!("Bye!");
                break;
            }
            Ok(Signal::CtrlC) => println!("(interrupted)"),
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            Ok(_) => {}
        }
    }
}

fn run_batch<S: Shell>(shell: &S) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.expect("failed to read line");
        let trimmed = line.trim();

        match trimmed {
            "quit" | "exit" => {
                writeln!(out, "Bye!").unwrap();
                break;
            }
            "" => {}
            input => match shell.evaluate(input) {
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
