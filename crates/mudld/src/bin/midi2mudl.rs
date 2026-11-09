//! `midi2mudl` — convert a Standard MIDI File (.mid) to MUDL (.kdl).
//!
//! Usage: midi2mudl <file.mid>
//!
//! Writes the KDL representation to stdout.

use anyhow::{Context, Result};
use std::{env, fs};

fn main() -> Result<()> {
    let path = env::args()
        .nth(1)
        .context("Usage: midi2mudl <file.mid>")?;

    let bytes = fs::read(&path).with_context(|| format!("Cannot read {path}"))?;
    let smf   = midly::Smf::parse(&bytes).context("Failed to parse MIDI file")?;
    let doc   = mudld::midi_to_kdl(&smf);

    println!("{doc}");
    Ok(())
}