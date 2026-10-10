// mush/tests/src/mish_tests.rs
mod common {
    include!("lib.rs");
}
use common::spawn_repl;

#[test]
fn mish_help() {
    let mut repl = spawn_repl("mish", "mish> ").expect("failed to spawn mish-shell");
    let out = String::from_utf8_lossy(&repl.execute("help").unwrap()).into_owned();
    assert!(out.contains("MIDI commands"), "got: {out}");
    repl.exit().ok();
}

#[test]
fn mish_version() {
    let mut repl = spawn_repl("mish", "mish> ").expect("failed to spawn mish-shell");
    let out = String::from_utf8_lossy(&repl.execute("version").unwrap()).into_owned();
    assert!(out.contains("mish-midi-core"), "got: {out}");
    repl.exit().ok();
}

#[test]
fn mish_unknown_command() {
    let mut repl = spawn_repl("mish", "mish> ").expect("failed to spawn mish-shell");
    let out = String::from_utf8_lossy(&repl.execute("bogus").unwrap()).into_owned();
    assert!(out.contains("unknown command"), "got: {out}");
    repl.exit().ok();
}
