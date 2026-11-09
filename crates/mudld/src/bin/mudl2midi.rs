//! `mudl2midi` — convert a MUDL (.kdl) file to a Standard MIDI File (.mid).
//!
//! Usage: mudl2midi <file.kdl> <out.mid>

use anyhow::{Context, Result};
use std::{env, fs};

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let kdl_path = args
        .next()
        .context("Usage: mudl2midi <file.kdl> <out.mid>")?;
    let midi_path = args
        .next()
        .context("Usage: mudl2midi <file.kdl> <out.mid>")?;

    let src = fs::read_to_string(&kdl_path).with_context(|| format!("Cannot read {kdl_path}"))?;

    let doc = src.parse::<kdl::KdlDocument>().context("KDL parse error")?;
    let smf = mudld::kdl_to_smf(&doc)?;

    let mut out = Vec::new();
    smf.write_std(&mut out).context("Failed to write MIDI")?;

    fs::write(&midi_path, &out).with_context(|| format!("Cannot write {midi_path}"))?;

    eprintln!("Wrote {} track(s) → {midi_path}", smf.tracks.len());
    Ok(())
}

