// mush/crates/mush-shell/src/main.rs
use mash_audio_core::{AudioEngine, AudioResponse};
use mish_midi_core::{MidiEngine, MidiResponse};
use mush_core::{Response, Shell, run};

struct MushShell {
    midi: MidiEngine,
    audio: AudioEngine,
}

impl Shell for MushShell {
    fn prompt(&self) -> &str {
        "mush"
    }

    fn evaluate(&self, input: &str) -> Response {
        // Namespaced: "midi <cmd>" or "audio <cmd>"
        if let Some(cmd) = input.strip_prefix("midi ") {
            return from_midi(self.midi.evaluate(cmd), cmd);
        }
        if let Some(cmd) = input.strip_prefix("audio ") {
            return from_audio(self.audio.evaluate(cmd), cmd);
        }

        // Unprefixed: try midi then audio
        match self.midi.evaluate(input) {
            MidiResponse::Unknown => {}
            r => return from_midi(r, input),
        }
        match self.audio.evaluate(input) {
            AudioResponse::Unknown => {}
            r => return from_audio(r, input),
        }

        Response::Output(format!("mush: unknown command: {input}"))
    }

    fn on_start(&self) {
        println!("Welcome to mush — MIDI + Audio shell.");
        println!("Prefix commands with 'midi' or 'audio', or use them unprefixed.");
        println!("Type 'help' for commands, 'quit' to exit.");
    }
}

fn from_midi(r: MidiResponse, cmd: &str) -> Response {
    match r {
        MidiResponse::Output(s) => Response::Output(s),
        MidiResponse::Empty => Response::Empty,
        MidiResponse::Exit(s) => Response::Exit(s),
        MidiResponse::Unknown => Response::Output(format!("mush/midi: unknown command: {cmd}")),
    }
}

fn from_audio(r: AudioResponse, cmd: &str) -> Response {
    match r {
        AudioResponse::Output(s) => Response::Output(s),
        AudioResponse::Empty => Response::Empty,
        AudioResponse::Exit(s) => Response::Exit(s),
        AudioResponse::Unknown => Response::Output(format!("mush/audio: unknown command: {cmd}")),
    }
}

fn main() {
    run(MushShell {
        midi: MidiEngine::new(),
        audio: AudioEngine::new(),
    });
}
