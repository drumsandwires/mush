use clap::{Parser, Subcommand};
use mish_midi_core::midi::engine::{MidiEngine, MidiResponse};
use mush_core::Response;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(no_binary_name = true, name = "mish")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Show mish-midi-core version
    Version,
    /// List available MIDI ports
    Ports,
    /// Refresh and list available MIDI ports
    PortsRefresh,
    /// Load a MIDI file
    Load { path: PathBuf },
    /// Save current session
    Save { path: PathBuf },
    /// Start MIDI playback
    Play,
    /// Start MIDI recording
    Record,
    /// Stop playback or recording
    Stop,
}

fn to_response(r: MidiResponse) -> Response {
    match r {
        MidiResponse::Output(s) => Response::Output(s),
        MidiResponse::Empty => Response::Empty,
        MidiResponse::Exit(s) => Response::Exit(s),
    }
}

pub fn evaluate(engine: &MidiEngine, input: &str) -> Response {
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
        Ok(cli) => match cli.command {
            Command::Version => to_response(engine.version()),
            Command::Ports => to_response(engine.ports()),
            Command::PortsRefresh => to_response(engine.ports_refresh()),
            Command::Load { path } => to_response(engine.load(&path)),
            Command::Save { path } => to_response(engine.save(&path)),
            Command::Play => to_response(engine.play()),
            Command::Record => to_response(engine.record()),
            Command::Stop => to_response(engine.stop()),
        },
        Err(e) => Response::Output(e.render().to_string()),
    }
}
