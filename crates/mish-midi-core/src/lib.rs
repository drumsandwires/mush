// mush/crates/mish-midi-core/src/lib.rs

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
            "help" => MidiResponse::Output(
                "MIDI commands: help, version, ports, play, stop".to_string(),
            ),
            "version" => MidiResponse::Output("mish-midi-core v0.1.0".to_string()),
            "ports" => MidiResponse::Output("ports: [none connected]".to_string()),
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
}
