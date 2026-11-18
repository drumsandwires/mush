# mush

A collection of command-line tools and shells for working with MIDI and audio data. Developed as part of the [Let's Get Rusty](letsgetrusty.com) Rust Accelerator program.

Currently intended as a diagnostic and debugging tool for use in other projects but with ambitions to become a command-line based digital audio workstation (DAW) one day, when it grows up.

There are three [`reedline`](github.com/nushell/reedline) based shells: `mish`, `mash` and `mush`. These cover MIDI, audio and unified MIDI/Audio respectively. The same functionality is exposed as a pair of [`nushell`](https://www.nushell.sh/) plugins: `nu_plugin_midi` and `nu_plugin_live_audio`.

Note that current development is focussed on implementing MIDI functionality.

## Functionality

The shells and plugins provide commands to:

- `version` — display version information.
- `port` — get or set the active device port.
- `ports` — enumerate available device ports.
- `load` — load file data.
- `play` — play loaded or recorded data.
- `record` — record live data.
- `save` — save recorded data.
- `stop` — stop playback or recording.
- `monitor` — monitor live data.
- `send` — send MIDI data.
- `receive` — receive MIDI data including SysEx.

For both MIDI and audio data.

## Structure

Developed as a Rust workspace, the project is decomposed into core domain logic, codec, and shell/plugin crates. See [`DESIGN.md`](./DESIGN.md) for the full architecture.

- 📁 `mush/`
  - 📁 `crates/`
    - 📁 `mush-core/` — Conductor, transport state, clock, MIDI engine, audio engine
    - 📁 `mush-midi-codec/` — MIDI message CLI parser/formatter
    - 📁 `mudld/` — `.mid` ↔ `.mudl` (KDL) file codec
    - 📁 `mish-shell/` — Standalone MIDI Reedline REPL → `mish`
    - 📁 `mash-shell/` — Standalone audio Reedline REPL → `mash`
    - 📁 `mush-shell/` — Unified MIDI + audio Reedline REPL → `mush`
    - 📁 `nu_plugin_midi/` — Nushell MIDI plugin → `nu_plugin_midi`
    - 📁 `nu_plugin_live_audio/` — Nushell audio plugin → `nu_plugin_live_audio`
  - 📁 `docs/` — Architecture diagrams and supplementary documentation
  - 📁 `tests/` — Workspace-level integration tests

## Dependencies

| Crate | Depends On | Purpose |
|-------|------------|---------|
| `mush-core` | `tokio` · `tmq` · `cpal` · `midir` · `midly` · `wmidi` · `crossbeam` · `serde` · `rmp-serde` · `tracing` | Conductor, transport, clock, MIDI and audio engines |
| `mush-midi-codec` | `wmidi` · `thiserror` | MIDI message CLI parser/formatter |
| `mudld` | `midly` · `serde` · `rmp-serde` · `thiserror` | `.mid` ↔ `.mudl` file codec |
| `mish-shell` | `mush-core` · `mush-midi-codec` · `reedline` · `clap` · `shadow-rs` · `tracing-subscriber` | MIDI interactive shell |
| `mash-shell` | `mush-core` · `reedline` · `clap` · `shadow-rs` · `tracing-subscriber` | Audio interactive shell |
| `mush-shell` | `mush-core` · `mush-midi-codec` · `reedline` · `clap` · `shadow-rs` · `tracing-subscriber` | Combined MIDI + audio interactive shell |
| `nu_plugin_midi` | `mush-core` · `mush-midi-codec` · `nu-plugin` · `nu-protocol` · `shadow-rs` · `tracing-subscriber` | Exposes MIDI as Nushell commands |
| `nu_plugin_live_audio` | `mush-core` · `nu-plugin` · `nu-protocol` · `shadow-rs` · `tracing-subscriber` | Exposes live audio as Nushell commands |
| `tests` *(integration)* | `mish-shell` · `mash-shell` · `mush-shell` · `nu_plugin_midi` · `nu_plugin_live_audio` | Integration test harness |


## Prerequisites

- [Rust](https://rustup.rs/) stable toolchain — edition 2024 minimum
- [Nushell](https://www.nushell.sh/) — required for plugin development and testing
- [FluidSynth](https://www.fluidsynth.org/) — required for MIDI audio testing
- [PlantUML](https://plantuml.com/) — required for local diagram rendering
- [cargo-nextest](https://nexte.st/) — test runner
- [mask](https://github.com/jacobdeichert/mask) — task runner

Install task runner and test runner:

```sh
    cargo install mask
    cargo install cargo-nextest
```

## Plan

Development is currently focussed on MIDI functionality with the goal of publishing a usable MIDI shell and plugin. Milestones 1–4 are complete with remedial work in progress. Current work is Milestone 5.

| Milestone | Status | Goals |
| --------- | ------ | ----- |
| 1 | ✅ Complete | Workspace & core scaffold, enumerate devices |
| 2 | ✅ Complete | File commands in both frontends |
| 3 | ✅ Complete | Live MIDI device I/O: `monitor` |
| 4 | ✅ Complete | `send` + SysEx |
| 5 | 🔧 In progress | Full test, remedial fixes, tracing and diagnostics |
| 6 | ⏳ Pending | Publish |

## Usage

### Shells

Run the standalone MIDI shell:

```sh
    cargo run --bin mish
```

Run the standalone audio shell:

```sh
    cargo run --bin mash
```

Run the unified MIDI + audio shell:

```sh
    cargo run --bin mush
```

The `help` command in each shell lists the available commands. For each command `help <command>` displays detailed help.

### Nushell Plugin

Build and register the MIDI plugin:

```sh
    cargo build --package nu_plugin_midi --release
    plugin add target/release/nu_plugin_midi
    plugin use nu_plugin_midi
```

Build and register the audio plugin:

```sh
    cargo build --package nu_plugin_live_audio --release
    plugin add target/release/nu_plugin_live_audio
    plugin use nu_plugin_live_audio
```

The `plugin list` command displays the status of registered plugins and available sub-commands. For each command adding `--help` displays detailed help.

## Documentation

This README provides instructions for working with the implementation and outputs of the project.

Other documentation:

| Document | Purpose |
| -------- | ------- |
| [`DESIGN.md`](./DESIGN.md) | High-level architecture, crate map, key abstractions, data flows, and external dependencies. Reference document for contributors and for AI-assisted development sessions. |
| [`STYLE.md`](./STYLE.md) | Defines the project documentation and coding standards. |
| [`CHANGES.md`](./CHANGES.md) | Details of work items fixed/implemented in a build/release and any known issues. |
| [`DEVELOP.md`](./DEVELOP.md) | Details of tools, versions and procedures necessary for developing with this project. |
| [`BUILD.md`](./BUILD.md) | How to build this project. |
| [`TEST.md`](./TEST.md) | How to test this project. Details of manual unit, integration, and smoke tests. How to run automated tests. |
| [`RELEASE.md`](./RELEASE.md) | How to release this project. |


## Development Tasks

The following are executable tasks using `mask --maskfile README.md <task>` (aliased as `miso` - "make it so"). 

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
cargo nextest run --package tests --profile ci
```

## buildest

> Build all workspace crates

```sh
cargo build --workspace && cargo nextest run --workspace
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
cargo nextest run --package tests --profile ci
```

### test shell (shell)

> Run mish integration tests only

```sh
cargo build --bin $shell
cargo nextest run --package $shell-tests --test ${shell}-tests --profile ci
```

### test plugin

> Check nu_plugin_midi binary exists and was built successfully

```sh
cargo build --package nu_plugin_midi
cargo nextest run --package mush-tests --test nu_plugin_midi_tests --profile ci
```

## ci

> Full CI pipeline — build, unit tests, integration tests

```sh
set -e
cargo build --workspace
cargo nextest run --package mish-midi-core --profile unit
cargo nextest run --package mash-audio-core --profile unit
cargo nextest run --package tests --profile ci
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

## plugin-add (plugin)

> Build, register and list `nu_plugin_midi` with nushell (release build)

```sh
set -e
cargo build --package nu_plugin_${plugin} --release
nu -c "plugin add target/release/nu_plugin_$plugin; plugin list | where name == $plugin"
```

## plugin-rm (plugin)

> Build, register and list `nu_plugin_midi` with nushell (release build)

```sh
set -e
nu -c "do -i {plugin rm $plugin}; plugin list | where name == $plugin"
```

## plugin-ls (plugin)

> Build, register and list `nu_plugin_midi` with nushell (release build)

```sh
set -e
nu -c "plugin list | where name == $plugin"
```



## plugin-test (plugin)

> Build, register and list `nu_plugin_midi` with nushell (release build)

```sh
set -e
cargo build --package nu_plugin_${plugin} --release
nu -c "do -i {plugin rm $plugin}; plugin add target/release/nu_plugin_$plugin; plugin list | where name == $plugin"
```

## pr-check

```sh
echo Fmt...
cargo fmt --check
echo Clippy...
cargo clippy --workspace
echo Tests..
cargo nextest run --workspace
```

## fluidsynth

```sh
fluidsynth -a pulseaudio -m alsa_seq -o midi.autoconnect=1 /usr/share/sounds/sf2/default-GM.sf2
```
