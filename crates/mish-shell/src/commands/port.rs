use mush_core::transport::TransportCommand;

use crate::repl::{send, MishClient};

/// Dispatch a port command to the conductor and print the response.
///
/// - No args        → `MidiPortGet`   — print current in/out assignments
/// - `in <sel>`     → `MidiInputPort` — set input port by name, index, or substring
/// - `out <sel>`    → `MidiOutputPort` — set output port by name, index, or substring
/// - `in` / `out`   → clear the respective port selection
pub async fn handle_port(
    client: &mut MishClient,
    direction: Option<&str>,
    selector: Option<String>,
) -> anyhow::Result<()> {
    let cmd = match direction {
        None => TransportCommand::MidiPortGet,
        Some("in") => TransportCommand::MidiInputPort(selector),
        Some("out") => TransportCommand::MidiOutputPort(selector),
        Some(other) => {
            eprintln!("unknown port direction '{other}' — use 'in' or 'out'");
            return Ok(());
        }
    };
    send(client, cmd).await
}