# mush

A collection of command-line tools and shells for working with MIDI and audio data. Currently intended as a diagnostic and debugging tool for use in other projects but with ambitions to become a command-line based digital audio workstation (DAW) one day, when it grows up.

## Structure

A MIDI and Audio shell workspace — standalone reedline MIDI and audio REPLs, a unified shell, and a nushell plugin, all sharing common domain cores.

- 📁`mush/`
  - 📁`crates/`
    - 📁`mush-core/` - REPL engine + Shell trait (reedline)
    - 📁`mish-midi-core/` - MIDI domain logic (no REPL dependency)
    - 📁`mash-audio-core/` - Audio domain logic (no REPL dependency)
    - 📁`mish-shell/`  - Standalone MIDI reedline REPL  → binary: mish
    - 📁`mash-shell/` - Standalone Audio reedline REPL → binary: mash
    - 📁`mush-shell/` - Unified MIDI + Audio REPL      → binary: mush
    - 📁`nu_plugin_midi/` - Nushell MIDI plugin            → binary: nu_plugin_midi
    - 📁nu_plugin_audio/`: - Nushell audio plugin → binary: nu_plugin_live_audio
  - 📁`tests/`  - Workspace-level PTY integration tests (expectrl)

## Dependencies

| Crate                   | Depends On                                       | Purpose                                |
| ----------------------- | ------------------------------------------------ | -------------------------------------- |
| `mush-core`             | `reedline`                                       | Wraps reedline for use by shell crates |
| `mish-midi-core`        | (`midir`)                                        | Core MIDI data logic                   |
| `mash-audio-core`       | (`cpal`)                                         | Core live audio data logic             |
| `mish-shell`            | `mush-core`, `mish-midi-core`                    | MIDI interactive shell                 |
| `mash-shell`            | `mush-core`, `mash-audio-core`                   | Live audio interactive shell           |
| `mush-shell`            | `mush-core`, `mish-midi-core`, `mash-audio-core` | Combined interactive shell             |
| `nu_plugin_midi`        | `mish-midi-core`, `nu-plugin`, `nu-protocol`     | Exposes MIDI as nushell commands       |
| `nu_plugin_live_audio`  | `mash-audio-core`, `nu-plugin`, `nu-protocol`    | Exposes live audio as nushell commands |
| `tests` *(integration)* | `mish-shell`, `mash-shell`, `mush-shell`         | Integration test harness               |

## Prerequisites

- [Rust](https://rustup.rs/) stable toolchain
- [cargo-nextest](https://nexte.st/) for running tests
- [mask](https://github.com/jacobdeichert/mask) for task running

Install task runner and test runner:

- `cargo install mask`
- `cargo install cargo-nextest`

### Plan

| Milestone | Goals                                        |
| --------- | -------------------------------------------- |
| 1         | Workspace & Core Scaffold, Enumerate devices |
| 2         | File Commands in Both Frontends              |
| 3         | Live Device I/O: `midi listen`               |
| 4         | `midi send` + SysEx                          |
| 5         | Full test, rework                            |
| 6         | Publish                                      |

## Usage

### Shells

Run the standalone MIDI shell: `cargo run --bin mish`

Run the standalone Audio shell; `cargo run --bin mash`

Run the unified MIDI + Audio shell: `cargo run --bin mush`

### Nushell Plugin

Build the MIDI plugin with nushell: `cargo build --package nu_plugin_midi --release`

Register in nushell with: `plugin add target/release/nu_plugin_midi`

Once registered, commands are available in nushell:

- `midi ports`
- `midi play`
- `midi stop`

## Development Tasks

The following are executable tasks using [`mask --maskfile README.md`]() . 

## build

> Build all workspace crates

```sh
cargo build --workspace
```

## run (shell)

> Run a shell binary — usage: mask run [mish|mash|mush]

```sh
cargo run --bin $shell
```

## test

> Run all tests (unit + integration)

```sh
cargo nextest run --package mish-midi-core --profile unit
cargo nextest run --package mash-audio-core --profile unit
echo TODO(#4): cargo nextest run --package mush-tests --profile ci
```

### test unit

> Run domain unit tests only (fast, parallel, no PTY)

```sh
cargo nextest run --package mish-midi-core --profile unit
cargo nextest run --package mash-audio-core --profile unit
```

### test integration

> Run PTY integration tests (sequential, requires built binaries)

```sh
cargo build --workspace
echo TODO(#5): cargo nextest run --package mush-tests --profile ci
```

### test shell (shell)

> Run mish integration tests only

```sh
cargo build --bin $shell
echo TODO(#6): cargo nextest run --package $shell-tests --test ${shell}-tests --profile ci
```

### test plugin

> Check nu_plugin_midi binary exists and was built successfully

```sh
cargo build --package nu_plugin_midi
echo TODO(#7): cargo nextest run --package mush-tests --test nu_plugin_midi_tests --profile ci
```

## ci

> Full CI pipeline — build, unit tests, integration tests

```sh
set -e
cargo build --workspace
cargo nextest run --package mish-midi-core --profile unit
cargo nextest run --package mash-audio-core --profile unit
echo TODO(#8): cargo nextest run --package mush-tests --profile ci
```

## clean

> Remove build artifacts

```sh
cargo clean
```

## check

> Run cargo check across the workspace (fast, no codegen)

```sh
cargo check --workspace
```

## clippy

> Run clippy lints across the workspace

```sh
cargo clippy --workspace -- -D warnings
```

## format

> Format all source files

```sh
cargo fmt --all
```

## format-check

> Check formatting without modifying files (useful in CI)

```sh
cargo fmt --all -- --check
```

## plugin-test (plugin)

> Build, register and list `nu_plugin_midi` with nushell (release build)

```sh
set -e
cargo build --package nu_plugin_${plugin} --release
nu -c "plugin rm ${plugin}; plugin add target/release/nu_plugin_${plugin}; plugin list | where name == ${plugin}; plugin use live_audio; audio devices e> /dev/null"
```
