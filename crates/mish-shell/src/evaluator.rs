use clap::Parser;
use mish_midi_core::midi::engine::{MidiEngine, MidiResponse};
use mush_commands::MidiCommand;
use mush_core::Response;

#[derive(Parser, Debug)]
#[command(no_binary_name = true, name = "mish")]
struct Cli {
    #[command(subcommand)]
    command: MidiCommand,
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
            MidiCommand::Version => to_response(engine.version()),
            MidiCommand::Devices => to_response(engine.devices()),
            MidiCommand::DevicesRefresh => to_response(engine.devices_refresh()),
            MidiCommand::Load { path } => to_response(engine.load(&path)),
            MidiCommand::Save { path } => to_response(engine.save(&path)),
            MidiCommand::Play => to_response(engine.play()),
            MidiCommand::Record => to_response(engine.record()),
            MidiCommand::Monitor => to_response(engine.monitor()),
            MidiCommand::Stop => to_response(engine.stop()),
        },
        Err(e) => Response::Output(e.render().to_string()),
    }
}
