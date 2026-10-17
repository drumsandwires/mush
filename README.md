# mush

A collection of command-line tools and shells for working with MIDI and audio data. Developed as part of the [Let's Get Rusty](letsgetrusty.com) Rust Accelerator program. 

Currently intended as a diagnostic and debugging tool for use in other projects but with ambitions to become a command-line based digital audio workstation (DAW) one day, when it grows up.

There are three [`reedline`](github.com/nushell/reedline) based shells: `mish`, `mash` and `mush` . These cover MIDI, audio and unified MIDI/Audio respectively. The same functionality is exposed as a pair  `nushell` plugins: `nu_plugin_midi` and `nu_plugin_live_audio`. 

Note that current development is focussed on implementing MIDI functionality.

## Functionality

The shells and plugins provide commands to: 

- enumerate available `devices`.

- `load` file data.

- `play` loaded or recorded data.

- `record` live data.

- `save` recorded data.

- `stop` playback or recording.

- `monitor` live data.

- `send` MIDI data.

- `receive` MIDI data including SysEx.

For both MIDI and audio data.

## Structure

Developed as a workspace, the project is decomposed into core domain logic crates for MIDI, Audio and shell functionality. The crates are used by the shell and plugin crates to expose the domain functionality.

- 📁`mush/`
  - 📁`crates/`
    - 📁`mush-core/` - REPL engine + Shell trait (`reedline`)
    - 📁`mish-midi-core/` - MIDI domain logic (`midir`)
    - 📁`mash-audio-core/` - Audio domain logic (`cpal`)
    - 📁`mish-shell/`  - Standalone MIDI reedline REPL  → `mish`
    - 📁`mash-shell/` - Standalone Audio reedline REPL → `mash`
    - 📁`mush-shell/` - Unified MIDI + Audio REPL → `mush`
    - 📁`nu_plugin_midi/` - Nushell MIDI plugin  → `nu_plugin_midi`
    - 📁`nu_plugin_live_audio/` - Nushell audio plugin → `nu_plugin_live_audio`
  - 📁`tests/`  - Workspace-level PTY integration tests (expectrl)

## Dependencies

| Crate                   | Depends On                                       | Purpose                                |
| ----------------------- | ------------------------------------------------ | -------------------------------------- |
| `mush-core`             | `reedline`                                       | Wraps reedline for use by shell crates |
| `mish-midi-core`        | `midir`                                          | Core MIDI data logic                   |
| `mash-audio-core`       | `cpal`                                           | Core live audio data logic             |
| `mish-shell`            | `mush-core`, `mish-midi-core`                    | MIDI interactive shell                 |
| `mash-shell`            | `mush-core`, `mash-audio-core`                   | Live audio interactive shell           |
| `mush-shell`            | `mush-core`, `mish-midi-core`, `mash-audio-core` | Combined interactive shell             |
| `nu_plugin_midi`        | `mish-midi-core`, `nu-plugin`, `nu-protocol`     | Exposes MIDI as nushell commands       |
| `nu_plugin_live_audio`  | `mash-audio-core`, `nu-plugin`, `nu-protocol`    | Exposes live audio as nushell commands |
| `tests` *(integration)* | `mish-shell`, `mash-shell`, `mush-shell`         | Integration test harness               |

## Prerequisites

- [Rust](https://rustup.rs/) stable toolchain
- [cargo-nextest](https://nexte.st/) for running tests
- [mask](https://github.com/jacobdeichert/mask) for task running development tasks

Install Rust, task runner and test runner:

- `cargo install mask`
- `cargo install cargo-nextest`

### Plan

Development is currently focussed on MIDI functionality with the goal of publishing a usable MIDI shell and plugin.

| Milestone | Goals                                        |
| --------- | -------------------------------------------- |
| 1         | Workspace & Core Scaffold, Enumerate devices |
| 2         | File Commands in Both Frontends              |
| 3         | Live MIDI Device I/O: `midi listen`          |
| 4         | `midi send` + SysEx                          |
| 5         | Full test, rework                            |
| 6         | Publish                                      |

## Usage

### Shells

Run the standalone MIDI shell: `cargo run --bin mish`

Run the standalone Audio shell; `cargo run --bin mash`

Run the unified MIDI + Audio shell: `cargo run --bin mush`

The `help` command in each shell list the available commands.

### Nushell Plugin

Build the MIDI plugin with nushell: `cargo build --package nu_plugin_midi --release`

Register in nushell with: `plugin add target/release/nu_plugin_midi`

Activate the plugin with `plugin use nu_plugin_midi` 

The `plugin list` command displays the status of the plugin and available sub-commands. 

For each command adding `--help` displays the help for that command.

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
# echo >> TODO(#4): 
cargo nextest run --package tests --profile ci
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
# echo TODO(#5): 
cargo nextest run --package tests --profile ci
```

### test shell (shell)

> Run mish integration tests only

```sh
cargo build --bin $shell
# echo TODO(#6): 
cargo nextest run --package $shell-tests --test ${shell}-tests --profile ci
```

### test plugin

> Check nu_plugin_midi binary exists and was built successfully

```sh
cargo build --package nu_plugin_midi
# echo TODO(#7): 
cargo nextest run --package mush-tests --test nu_plugin_midi_tests --profile ci
```

## ci

> Full CI pipeline — build, unit tests, integration tests

```sh
set -e
cargo build --workspace
cargo nextest run --package mish-midi-core --profile unit
cargo nextest run --package mash-audio-core --profile unit
# echo TODO(#8): 
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

## plugin-test (plugin)

> Build, register and list `nu_plugin_midi` with nushell (release build)

```sh
set -e
cargo build --package nu_plugin_${plugin} --release
nu -c "do -i {plugin rm $plugin}; plugin add target/release/nu_plugin_$plugin; plugin list | where name == $plugin"
```
