// crates/mash-shell/src/main.rs  (MODIFY)
mod evaluator;

use mash_audio_core::audio::engine::AudioEngine;
use mush_core::{Response, Shell, run};

struct MashShell(AudioEngine);

impl Shell for MashShell {
    fn prompt(&self) -> &str {
        "mash> "
    }

    fn evaluate(&self, input: &str) -> Response {
        evaluator::evaluate(&self.0, input)
    }

    fn on_start(&self) {
        println!("Welcome to mash — Audio shell. Type 'help' for commands.");
    }
}

fn main() {
    run(MashShell(AudioEngine::new()));
}
