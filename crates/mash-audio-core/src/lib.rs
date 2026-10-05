// mush/crates/mash-core/src/lib.rs

/// The result of evaluating an Audio command.
#[derive(Debug, PartialEq)]
pub enum AudioResponse {
    Output(String),
    Empty,
    Exit(String),
    /// Command not recognised by this domain.
    Unknown,
}

/// Audio command evaluator.
/// Holds domain state (active device, recording session, etc.)
pub struct AudioEngine {
    // e.g. active_device: Option<Device>
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate(&self, input: &str) -> AudioResponse {
        match input.trim() {
            "" => AudioResponse::Empty,
            "help" => AudioResponse::Output(
                "Audio commands: help, version, list-devices, record, playback".to_string(),
            ),
            "version" => AudioResponse::Output("mash-core v0.1.0".to_string()),
            "list-devices" => AudioResponse::Output("devices: [default]".to_string()),
            "record" => AudioResponse::Output("Recording...".to_string()),
            "playback" => AudioResponse::Output("Playing back...".to_string()),
            _ => AudioResponse::Unknown,
        }
    }
}

impl Default for AudioEngine {
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
            AudioEngine::new().evaluate("help"),
            AudioResponse::Output(_)
        ));
    }

    #[test]
    fn test_unknown() {
        assert_eq!(AudioEngine::new().evaluate("bogus"), AudioResponse::Unknown);
    }

    #[test]
    fn test_empty() {
        assert_eq!(AudioEngine::new().evaluate(""), AudioResponse::Empty);
    }
}
