# Design

This document describes the architecture, crate structure, key abstractions, and data flows of the **mush** project. It is the primary reference for contributors and for AI-assisted development sessions. Keep it current as the project evolves.

---

## Purpose

**mush** is a Rust-based MIDI and audio workstation engine with multiple shell and plugin interfaces. At MVP it is a single-binary application per shell. Post-MVP the engine will be extracted into a standalone server (`mush-server`) with local or remote shell clients.

---

## Diagrams

| Diagram | Purpose |
|---------|---------|
| [`mush_crate_map.puml`](./docs/diagrams/mush_crate_map.puml) | Crate dependencies and roles |
| [`mush_data_flow.puml`](./docs/diagrams/mush_data_flow.puml) | End-to-end data flow |
| [`mush_architecture.puml`](./docs/diagrams/mush_architecture.puml) | Detailed single-binary architecture |

---

## Crate Map

| Crate | Type | Role |
|-------|------|------|
| `mush-core` | lib | Conductor, transport state, clock, MIDI engine, audio engine. Embedded in all shells at MVP. |
| `mush-midi-codec` | lib | CLI parser/formatter for raw MIDI messages. Symbolic, numeric, note name, and velocity name input formats. |
| `mudld` | lib | `.mid` ↔ `.mudl` (KDL) file codec. Standalone — no runtime dependency on `mush-core`. |
| `mish-shell` | bin | Interactive MIDI REPL. Reedline. |
| `mash-shell` | bin | Interactive audio REPL. Reedline. |
| `mush-shell` | bin | Combined MIDI + audio REPL. Reedline. Post-MVP DAW functionality. |
| `nu_plugin_midi` | bin | Nushell plugin — MIDI commands. |
| `nu_plugin_live_audio` | bin | Nushell plugin — live audio commands. |
| `tests` | — | Workspace integration tests. `assert_cmd` / `try_cmd`. |

---

## UI Variants

There are five UI entry points. All embed `mush-core` via the ZMQ inproc transport at MVP.

| Shell | Interface | Scope |
|-------|-----------|-------|
| `mish-shell` | Reedline REPL | MIDI |
| `nu_plugin_midi` | Nushell plugin | MIDI |
| `mash-shell` | Reedline REPL | Audio |
| `nu_plugin_live_audio` | Nushell plugin | Audio |
| `mush-shell` | Reedline REPL | MIDI + Audio + DAW (post-MVP) |

---

## Key Abstractions

| Type | Crate | Role |
|------|-------|------|
| `Conductor` | `mush-core` | Central Tokio `select!` loop. Receives `TransportCommand`, dispatches to clock, MIDI engine, and transport state. Fan-out of `EngineUpdate` to subscribers. |
| `TransportState` | `mush-core` | `Arc`-wrapped atomic state. Playhead position, BPM, is_playing, timeline, next event index. Read directly by RT layer. |
| `Timeline<P>` | `mush-core` | Sorted, sample-accurate event sequence. Generic over payload (`MidiPayload` / `AudioPayload`). Written by `LoadMidi`, `LoadMudl`, and `QueueEvent`. Read by `Clock` via `Arc<RwLock<Timeline>>`. Never held across an audio callback. |
| `Clock` | `mush-core` | Drives playback timing. cpal audio callback (primary) or software fallback. Reads `Timeline` and pushes `MidiEvent` to the lock-free queue. |
| `MidiEngine` | `mush-core` | High-priority thread. Owns `MidiEngineRt` (SCHED_FIFO). Pops `MidiEvent` from the lock-free queue and writes to the MIDI interface. |
| `AudioEngine` | `mush-core` | TBD at MVP. Placeholder alongside `MidiEngine`. |
| `ZmqClient` | `mush-core` | DEALER + SUB sockets. Shell-side transport. Sends `TransportCommand`, receives `EngineUpdate`. |
| `ZmqTransport` | `mush-core` | ROUTER + PUB sockets. Engine-side transport. Receives `TransportCommand`, publishes `EngineUpdate`. |
| `Subscription` / `SubscriptionHandle` | `mush-core` | In-process mpsc fan-out of `EngineUpdate` to all 5 shell variants. |
| `MidiEvent` | `mush-core` | Unit of work passed through the lock-free queue from `Clock` to `MidiEngineRt`. |
| `TransportCommand` | `mush-core` | Commands from shell to conductor: `Play`, `Stop`, `Seek`, `SetBpm`, `LoadMidi`, `QueueEvent`, `LoadMudl` (reserved). |
| `EngineUpdate` | `mush-core` | Status fan-out from conductor to shells: `AudioUpdate`, `MidiUpdate`. |

---

## Data Flows

### Command path (shell → hardware)

1. User input
2. → Shell REPL / Nushell plugin
3. → ZmqClient (DEALER inproc)
4. → ZmqTransport (ROUTER inproc)
5. → Conductor (tokio::select!)
6. → TransportState (write atomics) · Clock (Play/Stop/Seek/SetBpm) · MidiEngine (LoadMidi/QueueEvent)
7. → Clock (cpal callback) — reads Timeline<MidiPayload> → crossbeam::ArrayQueue<MidiEvent>
8. → MidiEngineRt (SCHED_FIFO) → midir → MIDI Interface

### send command path

1. User input
2. → Shell REPL / Nushell plugin
3. → mush-midi-codec (parse symbolic/numeric input)
4. → ZmqClient (DEALER inproc)
5. → ZmqTransport (ROUTER inproc)
6. → Conductor
7. → MidiEngine → MidiEngineRt → MIDI Interface

### File load/save path

**load:**
1. User input → Conductor
2. → Conductor reads file bytes
3. → mudld (bytes → Timeline<MidiPayload>)
4. → TransportState

**save:**
1. → Conductor
2. → mudld (Timeline<MidiPayload> → bytes)
3. → Conductor writes file bytes

### Status path (hardware → shell)

1. Clock → AudioUpdate (mpsc) → Conductor
2. MidiEngine → MidiUpdate (mpsc) → Conductor
3. Conductor → EngineUpdate → ZmqTransport (PUB inproc) → ZmqClient (SUB) → Shell
4. Conductor → Subscription (mpsc) → SubscriptionHandle → Shell

---

## External Dependencies

| Dependency | Role |
|------------|------|
| `tokio` | Async runtime for the Conductor layer |
| `tmq` | ZeroMQ bindings (DEALER · ROUTER · PUB · SUB) for inproc transport |
| `cpal` | Audio I/O — drives the Clock callback |
| `midir` | MIDI port enumeration and I/O |
| `midly` | SMF (.mid) file parsing |
| `wmidi` | MIDI message types |
| `reedline` | REPL line editor for mish, mash, mush shells |
| `nu-plugin` / `nu-protocol` | Nushell plugin interface |
| `crossbeam` / `crossbeam-queue` | Lock-free `ArrayQueue` between Clock and MidiEngineRt |
| `serde` / `rmp-serde` | Serialisation for IPC and file formats |
| `clap` | CLI argument parsing |
| `shadow-rs` | Build-time version metadata |
| `tracing` / `tracing-subscriber` | Structured diagnostics and logging |
| `thiserror` / `anyhow` | Error handling |

---

## MVP → Post-MVP Split

At MVP, `mush-core` is embedded directly in each shell binary. The ZMQ transport layer uses `inproc://` addresses, keeping everything in a single process.

Post-MVP (Model A), the Conductor is extracted into a standalone `mush-server` binary:

- ZMQ addresses change from `inproc://mush-cmd` / `inproc://mush-pub` to `tcp://5252` / `tcp://5253`
- Shell binaries become thin clients — no change to shell or conductor code required
- Enables remote shells and multi-client sessions

---

## Known Constraints

- **ALSA stderr noise** — ALSA prints enumeration warnings to stderr on Linux. Suppressed via `libc` fd redirect. See #36, #62.
- **Real-time thread** — `MidiEngineRt` runs at `SCHED_FIFO` priority. No allocations, no locks on the RT path. The `crossbeam::ArrayQueue` is the only communication channel between the Clock and the RT thread.
- **Timeline lock** — `Timeline` is protected by `RwLock`. Write locks (`LoadMidi`, `LoadMudl`, `QueueEvent`) are never held across an audio callback. Violating this constraint will cause priority inversion on the RT thread.
- **Port state persistence** — Active MIDI port selection is not currently persisted across commands. See #80.
- **Transport clock reset** — Clock position is not reset on file load. See #82.
- **AudioEngine** — Placeholder at MVP. `mash-shell` and `nu_plugin_live_audio` are partially implemented.


