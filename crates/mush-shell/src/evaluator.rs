use clap::{Parser, Subcommand};
use mash_audio_core::audio::engine::{AudioEngine, AudioResponse};
use mish_midi_core::midi::engine::{MidiEngine, MidiResponse};
use mush_commands::{AudioCommand, MidiCommand};
use mush_core::Response;

#[derive(Parser, Debug)]
#[command(no_binary_name = true, name = "mush")]
struct Cli {
    #[command(subcommand)]
    domain: Domain,
}

#[derive(Subcommand, Debug)]
enum Domain {
    /// Audio commands
    Audio {
        #[command(subcommand)]
        command: AudioCommand,
    },
    /// MIDI commands
    Midi {
        #[command(subcommand)]
        command: MidiCommand,
    },
}

fn audio_to_response(r: AudioResponse) -> Response {
    match r {
        AudioResponse::Output(s) => Response::Output(s),
        AudioResponse::Empty => Response::Empty,
        AudioResponse::Exit(s) => Response::Exit(s),
    }
}

fn midi_to_response(r: MidiResponse) -> Response {
    match r {
        MidiResponse::Output(s) => Response::Output(s),
        MidiResponse::Empty => Response::Empty,
        MidiResponse::Exit(s) => Response::Exit(s),
    }
}

pub fn evaluate(audio: &AudioEngine, midi: &MidiEngine, input: &str) -> Response {
    if input.is_empty() {
        return Response::Empty;
    }

    let tokens = match shlex::split(input) {
        Some(t) => t,
        None => {
            return Response::Output(
                "error: malformed input (unclosed quote or trailing backslash)".into(),
            );
        }
    };

    match Cli::try_parse_from(tokens) {
        Ok(cli) => match cli.domain {
            Domain::Audio { command } => match command {
                AudioCommand::Version => audio_to_response(audio.version()),
                AudioCommand::Devices => audio_to_response(audio.devices()),
                AudioCommand::DevicesRefresh => audio_to_response(audio.devices_refresh()),
                AudioCommand::Load { path } => audio_to_response(audio.load(&path)),
                AudioCommand::Save { path } => audio_to_response(audio.save(&path)),
                AudioCommand::Play => audio_to_response(audio.play()),
                AudioCommand::Record => audio_to_response(audio.record()),
                AudioCommand::Monitor => audio_to_response(audio.monitor()),
                AudioCommand::Stop => audio_to_response(audio.stop()),
            },
            Domain::Midi { command } => match command {
                MidiCommand::Version => midi_to_response(midi.version()),
                MidiCommand::Devices => midi_to_response(midi.devices()),
                MidiCommand::DevicesRefresh => midi_to_response(midi.devices_refresh()),
                MidiCommand::Load { path } => midi_to_response(midi.load(&path)),
                MidiCommand::Save { path } => midi_to_response(midi.save(&path)),
                MidiCommand::Play => midi_to_response(midi.play()),
                MidiCommand::Record => midi_to_response(midi.record()),
                MidiCommand::Monitor => midi_to_response(midi.monitor()),
                MidiCommand::Stop => midi_to_response(midi.stop()),
            },
        },
        Err(e) => Response::Output(e.render().to_string()),
    }
}
