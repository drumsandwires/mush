/// The result of evaluating an Audio command.
use enum_iterator::{Sequence, all};
use std::fmt;
use std::sync::OnceLock;

use crate::audio::display::list_hosts_and_devices;
use crate::audio::enumerate::enumerate_hosts_and_devices;
use crate::audio::types::{AudioDeviceError, AudioSystemInfo};

#[derive(Debug, PartialEq)]
pub enum AudioResponse {
    Output(String),
    Empty,
    Exit(String),
    /// Command not recognised by this domain.
    Unknown,
}

#[derive(Debug, PartialEq, Sequence)]
pub enum AudioTasks {
    Help,
    Version,
    Devices,
    Load,
    Save,
    Play,
    Record,
    Stop,
}

impl fmt::Display for AudioTasks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Audio command evaluator.
/// Holds domain state (active device, recording session, etc.)
pub struct AudioEngine {
    /// Populated on the first `devices` command via OnceCell interior
    /// mutability — allows evaluate() to take &self instead of &mut self.
    devices_cache: OnceLock<AudioSystemInfo>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            devices_cache: OnceLock::new(),
        }
    }

    pub fn evaluate(&self, input: &str) -> AudioResponse {
        match input.trim() {
            "" => AudioResponse::Empty,
            "help" => AudioResponse::Output(format!(
                "Audio commands: {}",
                all::<AudioTasks>()
                    .map(|d| d.to_string().to_lowercase())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            "version" => AudioResponse::Output("mash-audio-core v0.1.0".to_string()),
            "devices" => AudioResponse::Output(self.devices_listing()),
            "load" => AudioResponse::Output("Loading...".to_string()),
            "save" => AudioResponse::Output("Saving...".to_string()),
            "record" => AudioResponse::Output("Recording...".to_string()),
            "play" => AudioResponse::Output("Playing back...".to_string()),
            "stop" => AudioResponse::Output("Stopping...".to_string()),
            _ => AudioResponse::Unknown,
        }
    }

    /// Returns the formatted device listing, populating the cache on first call.
    fn devices_listing(&self) -> String {
        let info = self.devices_cache.get_or_init(|| {
            enumerate_hosts_and_devices().unwrap_or_else(|e: AudioDeviceError| {
                eprintln!("mash-audio-core: device enumeration failed: {e}");
                AudioSystemInfo { hosts: vec![] }
            })
        });
        list_hosts_and_devices(info)
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
