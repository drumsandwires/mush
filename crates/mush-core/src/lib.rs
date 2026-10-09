// mush/crates/mush-core/src/lib.rs
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

/// The response returned by a [`Shell`] implementation after
/// evaluating one line of user input.
#[derive(Debug, PartialEq)]
pub enum Response {
    Output(String),
    Empty,
    Exit(String),
}

/// Implement this trait to create a shell that runs inside the mush REPL engine.
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

/// Runs the REPL loop for any type that implements [`Shell`].
pub fn run<S: Shell>(shell: S) {
    shell.on_start();

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

    shell.on_exit();
}
