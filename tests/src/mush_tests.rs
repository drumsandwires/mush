// mush/tests/src/mush_tests.rs
mod common {
    include!("lib.rs");
}
use common::spawn_repl;

#[test]
fn mush_midi_namespaced() {
    let mut repl = spawn_repl("mush", "mush> ").expect("failed to spawn mush-shell");
    let out = String::from_utf8_lossy(&repl.execute("midi ports").unwrap()).into_owned();
    assert!(out.contains("ports"), "got: {out}");
    repl.exit().ok();
}

#[test]
fn mush_audio_namespaced() {
    let mut repl = spawn_repl("mush", "mush> ").expect("failed to spawn mush-shell");
    let out = String::from_utf8_lossy(&repl.execute("audio devices").unwrap()).into_owned();
    assert!(out.contains("devices"), "got: {out}");
    repl.exit().ok();
}

#[test]
fn mush_unknown_command() {
    let mut repl = spawn_repl("mush", "mush> ").expect("failed to spawn mush-shell");
    let out = String::from_utf8_lossy(&repl.execute("bogus").unwrap()).into_owned();
    assert!(out.contains("unknown command"), "got: {out}");
    repl.exit().ok();
}
