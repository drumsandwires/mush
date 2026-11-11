//! `mush-midi-codec` — symmetric MIDI message codec.
//!
//! Converts between human-readable symbolic strings and raw MIDI wire bytes.
//!
//! # Parse
//! ```
//! let bytes = mush_midi_codec::parse("NoteOn C4 mf").unwrap();
//! assert_eq!(bytes, vec![0x90, 60, 80]);
//! ```
//!
//! # Format
//! ```
//! let s = mush_midi_codec::format(&[0x90, 60, 80], mush_midi_codec::Style::Long).unwrap();
//! assert_eq!(s, "NoteOn C4 mf");
//! ```

mod error;
mod format;
mod note;
mod parse;
mod status;
mod velocity;

pub use error::{FormatError, ParseError};
pub use format::format;
pub use parse::parse;

/// Output style for [`format`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Style {
    /// Full symbolic names — e.g. `"NoteOn C4 mf"`.
    /// Default for file output and save previews.
    #[default]
    Long,
    /// Abbreviated names — e.g. `"Non C4 mf"`.
    /// Default for live monitor display.
    Short,
}