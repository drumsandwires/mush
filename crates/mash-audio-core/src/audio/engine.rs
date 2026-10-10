// crates/mash-audio-core/src/audio/engine.rs  (REPLACE)
use crate::audio::display::list_hosts_and_devices;
use crate::audio::enumerate::enumerate_hosts_and_devices;
use crate::audio::types::{AudioDeviceError, AudioSystemInfo};
use enum_iterator::{Sequence, all};
use std::fmt;
use std::path::Path;
use std::sync::RwLock;

/// The result of evaluating an Audio command.
/// Deliberately decoupled from any REPL or plugin response type —
/// each frontend maps this into its own output representation.
#[derive(Debug, PartialEq)]
pub enum AudioResponse {
    Output(String),
    Empty,
    Exit(String),
}

/// Canonical list of audio tasks — drives help output in any frontend.
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
/// Pure domain logic — no string parsing, no clap, no shlex, no REPL types.
/// Each frontend (REPL, nushell plugin, GUI) calls methods directly.
pub struct AudioEngine {
    /// Refreshable cache of enumerated audio devices.
    /// None = unpopulated or invalidated; Some = ready.
    devices_cache: RwLock<Option<AudioSystemInfo>>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            devices_cache: RwLock::new(None),
        }
    }

    /// Invalidates the devices cache so the next call to `devices()`
    /// re-enumerates from the OS. Call this before any command that
    /// may change the device topology (e.g. connect, disconnect).
    pub fn invalidate_devices_cache(&self) {
        *self.devices_cache.write().unwrap() = None;
    }

    /// Returns a formatted list of all available audio tasks.
    /// Frontends can call this for their own `help` command.
    pub fn help(&self) -> AudioResponse {
        AudioResponse::Output(format!(
            "Audio commands: {}",
            all::<AudioTasks>()
                .map(|t| t.to_string().to_lowercase())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }

    // TODO(#9): Use naming and version from the workspace
    pub fn version(&self) -> AudioResponse {
        AudioResponse::Output("mash-audio-core v0.1.0".to_string())
    }

    /// Lists available audio devices.
    /// Uses the cache when populated; enumerates from OS on cache miss.
    pub fn devices(&self) -> AudioResponse {
        // fast path: cache hit
        {
            let read = self.devices_cache.read().unwrap();
            if let Some(ref info) = *read {
                return AudioResponse::Output(list_hosts_and_devices(info));
            }
        } // read lock dropped here before write below

        // slow path: cache miss — enumerate and populate
        let info = enumerate_hosts_and_devices().unwrap_or_else(|e: AudioDeviceError| {
            eprintln!("mash-audio-core: device enumeration failed: {e}");
            AudioSystemInfo { hosts: vec![] }
        });
        let response = AudioResponse::Output(list_hosts_and_devices(&info));
        match self.devices_cache.write() {
            Ok(mut guard) => *guard = Some(info),
            Err(poisoned) => *poisoned.into_inner() = Some(info),
        }
        response
    }

    /// Refreshes the devices cache unconditionally, then returns the listing.
    /// Use this when you know the device topology may have changed.
    pub fn devices_refresh(&self) -> AudioResponse {
        self.invalidate_devices_cache();
        self.devices()
    }

    pub fn load(&self, path: &Path) -> AudioResponse {
        // TODO(#10): Load an audio file.
        AudioResponse::Output(format!("TBD: Loading '{}'...", path.display()))
    }

    pub fn save(&self, path: &Path) -> AudioResponse {
        // TODO(#11): Save an audio session to a file.
        AudioResponse::Output(format!("TBD: Saving to '{}'...", path.display()))
    }

    pub fn play(&self) -> AudioResponse {
        // TODO(#12): Playback an audio session or file.
        AudioResponse::Output("TBD: Playing back...".to_string())
    }

    pub fn record(&self) -> AudioResponse {
        // TODO(#13): Start recording an audio session.
        AudioResponse::Output("TBD: Recording...".to_string())
    }

    pub fn stop(&self) -> AudioResponse {
        // TODO(#14): Stop playback/recording.
        AudioResponse::Output("TBD: Stopping...".to_string())
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
            AudioEngine::new().help(),
            AudioResponse::Output(_)
        ));
    }

    #[test]
    fn test_version() {
        assert!(matches!(
            AudioEngine::new().version(),
            AudioResponse::Output(_)
        ));
    }

    #[test]
    fn test_devices_returns_output() {
        assert!(matches!(
            AudioEngine::new().devices(),
            AudioResponse::Output(_)
        ));
    }

    #[test]
    fn test_devices_refresh() {
        let engine = AudioEngine::new();
        let _ = engine.devices();
        assert!(matches!(engine.devices_refresh(), AudioResponse::Output(_)));
    }

    #[test]
    fn test_invalidate_then_devices() {
        let engine = AudioEngine::new();
        let _ = engine.devices();
        engine.invalidate_devices_cache();
        assert!(matches!(engine.devices(), AudioResponse::Output(_)));
    }

    #[test]
    fn test_play() {
        assert!(matches!(
            AudioEngine::new().play(),
            AudioResponse::Output(_)
        ));
    }

    #[test]
    fn test_stop() {
        assert!(matches!(
            AudioEngine::new().stop(),
            AudioResponse::Output(_)
        ));
    }

    #[test]
    fn test_load() {
        assert!(matches!(
            AudioEngine::new().load(Path::new("test.wav")),
            AudioResponse::Output(_)
        ));
    }
}
