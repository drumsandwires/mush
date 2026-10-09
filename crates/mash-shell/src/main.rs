// mush/crates/mash-shell/src/main.rs
use mash::audio::engine::{AudioEngine, AudioResponse};
use mash_audio_core as mash;

use mush_core::{Response, Shell, run};

struct MashShell(AudioEngine);

impl Shell for MashShell {
    fn prompt(&self) -> &str {
        "mash"
    }

    fn evaluate(&self, input: &str) -> Response {
        match self.0.evaluate(input) {
            AudioResponse::Output(s) => Response::Output(s),
            AudioResponse::Empty => Response::Empty,
            AudioResponse::Exit(s) => Response::Exit(s),
            AudioResponse::Unknown => Response::Output(format!("mash: unknown command: {input}")),
        }
    }

    fn on_start(&self) {
        println!("Welcome to mash — Audio shell. Type 'help' for commands.");
    }
}

fn main() {
    run(MashShell(AudioEngine::new()));
}
