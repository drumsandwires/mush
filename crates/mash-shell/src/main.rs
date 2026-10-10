mod evaluator;

use mash_audio_core::audio::engine::AudioEngine;
use mush_core::{Response, Shell, run};

struct MashShell(AudioEngine);

impl Shell for MashShell {
    // TODO(#15): Use naming from the workspace for the prompt.
    fn prompt(&self) -> &str {
        "mash"
    }

    fn evaluate(&self, input: &str) -> Response {
        evaluator::evaluate(&self.0, input)
    }

    // TODO(#16): Use naming a versioning from the workspace for the banner.
    fn on_start(&self) {
        println!("Welcome to mash — Audio shell. Type 'help' for commands.");
    }
}

fn main() {
    run(MashShell(AudioEngine::new()));
}
