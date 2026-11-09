//! `mudld` — MIDI ↔ MUDL (KDL) conversion library.
//!
//! Provides two public entry points:
//!
//! - [`midi_to_kdl`] — convert a parsed [`midly::Smf`] to a [`kdl::KdlDocument`]
//! - [`kdl_to_smf`]  — convert a parsed [`kdl::KdlDocument`] to a [`midly::Smf`]
//!
//! # Memory note
//! `midly::Smf<'a>` borrows its byte slices (SysEx, meta text, etc.)
//! from a backing buffer. Where the KDL source is the backing store,
//! those slices are promoted to `'static` via [`leak_bytes`].
//!
//! TODO: replace `leak_bytes` with an arena allocator (e.g. `bumpalo`)
//! so long-running processes do not accumulate leaked allocations.

pub mod kdl_to_midi;
pub mod midi_to_kdl;

pub use kdl_to_midi::kdl_to_smf;
pub use midi_to_kdl::midi_to_kdl;