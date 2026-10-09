use clap::Parser;
use mash_audio_core::audio::engine::{AudioEngine, AudioResponse};
use mush_commands::AudioCommand;
use mush_core::Response;

#[derive(Parser, Debug)]
#[command(no_binary_name = true, name = "mash")]
struct Cli {
    #[command(subcommand)]
    command: AudioCommand,
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
            AudioCommand::Version => to_response(engine.version()),
            AudioCommand::Devices => to_response(engine.devices()),
            AudioCommand::DevicesRefresh => to_response(engine.devices_refresh()),
            AudioCommand::Load { path } => to_response(engine.load(&path)),
            AudioCommand::Save { path } => to_response(engine.save(&path)),
            AudioCommand::Play => to_response(engine.play()),
            AudioCommand::Record => to_response(engine.record()),
            AudioCommand::Stop => to_response(engine.stop()),
        },
        Err(e) => Response::Output(e.render().to_string()),
    }
}
