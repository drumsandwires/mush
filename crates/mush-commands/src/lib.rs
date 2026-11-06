use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum AudioCommand {
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
    /// Monitor live audio
    Monitor,
    /// Stop playback or recording
    Stop,
}

#[derive(Subcommand, Debug)]
pub enum MidiCommand {
    /// Show mish-midi-core version
    Version,
    /// List available MIDI devices
    Devices,
    /// Refresh and list available MIDI devices
    DevicesRefresh,
    /// Load a MIDI file
    Load { path: PathBuf },
    /// Save current session
    Save { path: PathBuf },
    /// Start MIDI playback
    Play,
    /// Start MIDI recording
    Record,
    /// Monitor live MIDI
    Monitor,
    /// Stop playback or recording
    Stop,
}
