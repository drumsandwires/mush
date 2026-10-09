// mush/crates/mash-audio-core/src/audio/types.rs

use thiserror::Error;

// ── Errors ───────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum AudioDeviceError {
    /// Wraps the single unified cpal::Error introduced in 0.18.
    /// Use .kind() on the inner error to match specific ErrorKind variants.
    #[error("audio device error: {0}")]
    CpalError(#[from] cpal::Error),
}

// ── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct AudioSupportedRates {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug)]
pub struct AudioDeviceInfo {
    pub description: String,
    pub is_default_input: bool,
    pub is_default_output: bool,
    pub input_channels: Option<u16>,
    pub output_channels: Option<u16>,
    pub supported_rates: Option<AudioSupportedRates>,
}

#[derive(Debug)]
pub struct AudioHostInfo {
    pub name: String,
    pub devices: Vec<AudioDeviceInfo>,
}

#[derive(Debug)]
pub struct AudioSystemInfo {
    pub hosts: Vec<AudioHostInfo>,
}
