use clap::{Parser, Subcommand};
use mash_audio_core::audio::engine::{AudioEngine, AudioResponse};
use mush_core::Response;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(no_binary_name = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Show mash-audio-core version
    Version,
    /// List audio hosts and devices
    Devices,
    /// Refresh and list audio hosts and devices
    DevicesRefresh,
    /// Load an audio file
    Load { path: PathBuf },
    /// Save current session
    Save { path: PathBuf },
    /// Play back loaded audio
    Play,
    /// Start recording
    Record,
    /// Stop playback or recording
    Stop,
}

fn to_response(r: AudioResponse) -> Response {
    match r {
        AudioResponse::Output(s) => Response::Output(s),
        AudioResponse::Empty => Response::Empty,
        AudioResponse::Exit(s) => Response::Exit(s),
    }
}

pub fn evaluate(engine: &AudioEngine, input: &str) -> Response {
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
            Command::Devices => to_response(engine.devices()),
            Command::DevicesRefresh => to_response(engine.devices_refresh()),
            Command::Load { path } => to_response(engine.load(&path)),
            Command::Save { path } => to_response(engine.save(&path)),
            Command::Play => to_response(engine.play()),
            Command::Record => to_response(engine.record()),
            Command::Stop => to_response(engine.stop()),
        },
        Err(e) => Response::Output(format!("mash: {e}")),
    }
}
