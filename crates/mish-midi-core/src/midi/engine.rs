use crate::midi::display::list_enumerated_ports;
use crate::midi::enumerate::enumerate_ports;
use enum_iterator::{Sequence, all};
use std::fmt;
use std::sync::OnceLock;

/// The result of evaluating a MIDI command.
/// Deliberately decoupled from any REPL or plugin response type —
/// each frontend maps this into its own output representation.
#[derive(Debug, PartialEq)]
pub enum MidiResponse {
    Output(String),
    Empty,
    Exit(String),
    /// Command not recognised by this domain.
    Unknown,
}

#[derive(Debug, PartialEq, Sequence)]
pub enum MidiTasks {
    Help,
    Version,
    Devices,
    Load,
    Save,
    Play,
    Record,
    Stop,
}

impl fmt::Display for MidiTasks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// MIDI command evaluator.
/// Holds domain state (connected ports, active sequences, etc.)
pub struct MidiEngine {
    // e.g. connected_port: Option<MidiPort>
}

impl MidiEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate(&self, input: &str) -> MidiResponse {
        match input.trim() {
            "" => MidiResponse::Empty,
            "help" => MidiResponse::Output(format!(
                "Midi commands: {}",
                all::<MidiTasks>()
                    .map(|d| d.to_string().to_lowercase())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            "version" => MidiResponse::Output("mish-midi-core v0.1.0".to_string()),
            "ports" => {
                // MidiError is an internal detail; surface it as a user-facing string.
                match enumerate_ports() {
                    Err(e) => MidiResponse::Output(format!("error enumerating ports: {e}")),
                    Ok(ports) => match list_enumerated_ports(&ports) {
                        Some(listing) => MidiResponse::Output(listing),
                        None => MidiResponse::Output("no MIDI ports found".to_string()),
                    },
                }
            }
            "play" => MidiResponse::Output("Playing...".to_string()),
            "stop" => MidiResponse::Output("Stopped.".to_string()),
            _ => MidiResponse::Unknown,
        }
    }
}

impl Default for MidiEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help() {
        assert!(matches!(
            MidiEngine::new().evaluate("help"),
            MidiResponse::Output(_)
        ));
    }

    #[test]
    fn test_unknown() {
        assert_eq!(MidiEngine::new().evaluate("bogus"), MidiResponse::Unknown);
    }

    #[test]
    fn test_empty() {
        assert_eq!(MidiEngine::new().evaluate(""), MidiResponse::Empty);
    }

    #[test]
    fn test_ports_returns_output() {
        // We can't assert on the content (hardware-dependent),
        // but we can assert the shape of the response.
        assert!(matches!(
            MidiEngine::new().evaluate("ports"),
            MidiResponse::Output(_)
        ));
    }
}
