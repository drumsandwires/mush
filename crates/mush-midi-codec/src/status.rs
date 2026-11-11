/// A recognised MIDI status type (channel-voice or SysEx).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    NoteOff,
    NoteOn,
    PolyPressure,
    ControlChange,
    ProgramChange,
    ChannelPressure,
    PitchBend,
    SysEx,
}

impl StatusKind {
    /// Number of data bytes expected after the status byte.
    /// SysEx returns 0 — it is variable length and handled separately.
    pub fn data_len(self) -> usize {
        match self {
            Self::NoteOff         => 2,
            Self::NoteOn          => 2,
            Self::PolyPressure    => 2,
            Self::ControlChange   => 2,
            Self::ProgramChange   => 1,
            Self::ChannelPressure => 1,
            Self::PitchBend       => 2,
            Self::SysEx           => 0, // variable
        }
    }

    /// Base status nibble (high nibble, channel bits zeroed).
    /// Returns `None` for SysEx (no channel nibble).
    pub fn base_status(self) -> Option<u8> {
        match self {
            Self::NoteOff         => Some(0x80),
            Self::NoteOn          => Some(0x90),
            Self::PolyPressure    => Some(0xA0),
            Self::ControlChange   => Some(0xB0),
            Self::ProgramChange   => Some(0xC0),
            Self::ChannelPressure => Some(0xD0),
            Self::PitchBend       => Some(0xE0),
            Self::SysEx           => None,
        }
    }

    /// Long symbolic name.
    pub fn long_name(self) -> &'static str {
        match self {
            Self::NoteOff         => "NoteOff",
            Self::NoteOn          => "NoteOn",
            Self::PolyPressure    => "PolyPressure",
            Self::ControlChange   => "ControlChange",
            Self::ProgramChange   => "ProgramChange",
            Self::ChannelPressure => "ChannelPressure",
            Self::PitchBend       => "PitchBend",
            Self::SysEx           => "SysEx",
        }
    }

    /// Short symbolic name.
    pub fn short_name(self) -> &'static str {
        match self {
            Self::NoteOff         => "Noff",
            Self::NoteOn          => "Non",
            Self::PolyPressure    => "PP",
            Self::ControlChange   => "CC",
            Self::ProgramChange   => "PC",
            Self::ChannelPressure => "CP",
            Self::PitchBend       => "PB",
            Self::SysEx           => "SysEx",
        }
    }

    /// Parse a status symbol string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "noteoff" | "noff"          => Some(Self::NoteOff),
            "noteon"  | "non"           => Some(Self::NoteOn),
            "polypressure" | "pp"       => Some(Self::PolyPressure),
            "controlchange" | "cc"      => Some(Self::ControlChange),
            "programchange" | "pc"      => Some(Self::ProgramChange),
            "channelpressure" | "cp"    => Some(Self::ChannelPressure),
            "pitchbend" | "pb"          => Some(Self::PitchBend),
            "sysex"                     => Some(Self::SysEx),
            _                           => None,
        }
    }

    /// Decode a raw status byte into (StatusKind, channel).
    /// Channel is 0-indexed (0–15). Returns None for unrecognised bytes.
    pub fn from_status_byte(byte: u8) -> Option<(Self, u8)> {
        if byte == 0xF0 {
            return Some((Self::SysEx, 0));
        }
        let kind = match byte & 0xF0 {
            0x80 => Self::NoteOff,
            0x90 => Self::NoteOn,
            0xA0 => Self::PolyPressure,
            0xB0 => Self::ControlChange,
            0xC0 => Self::ProgramChange,
            0xD0 => Self::ChannelPressure,
            0xE0 => Self::PitchBend,
            _    => return None,
        };
        Some((kind, byte & 0x0F))
    }
}