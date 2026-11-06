mod common {
    include!("lib.rs");
}
use common::{cargo_bin, clean, spawn_repl};

#[test]
fn mash_help() {
    // Diagnostics — remove once fixed
    eprintln!("cwd = {:?}", std::env::current_dir());
    eprintln!(
        "CARGO_MANIFEST_DIR (compile time) = {}",
        env!("CARGO_MANIFEST_DIR")
    );
    eprintln!(
        "CARGO_MANIFEST_DIR (runtime) = {:?}",
        std::env::var("CARGO_MANIFEST_DIR")
    );
    eprintln!(
        "CARGO_TARGET_DIR (runtime) = {:?}",
        std::env::var("CARGO_TARGET_DIR")
    );
    let path = cargo_bin("mash");
    eprintln!("cargo_bin(mash) = {path:?}");
    eprintln!("exists = {}", path.exists());

    let mut repl = spawn_repl("mash", "mash>").expect("failed to spawn mash-shell");
    let out = clean(&repl.execute("help").unwrap());
    assert!(out.contains("Audio commands"), "got: {out}");
    repl.exit().ok();
}

#[test]
fn mash_version() {
    let mut repl = spawn_repl("mash", "mash> ").expect("failed to spawn mash-shell");
    let out = clean(&repl.execute("version").unwrap());
    assert!(out.contains("mash-audio-core"), "got: {out}");
    repl.exit().ok();
}

#[test]
fn mash_unknown_command() {
    let mut repl = spawn_repl("mash", "mash> ").expect("failed to spawn mash-shell");
    let out = clean(&repl.execute("bogus").unwrap());
    assert!(out.contains("unknown command"), "got: {out}");
    repl.exit().ok();
}
