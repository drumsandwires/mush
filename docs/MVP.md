# mush MVP Implementation Plan

## MIDI-Focused MVP with Audio Callback Clock

---

## Scope

### MVP

```
IN  — mush-core (TransportState, EngineMonitor, ZmqTransport, PluginChannel)
IN  — mish-midi-core (timeline, MidiEngine, Clock via cpal silence)
IN  — mash-audio-core (device enumeration only — no playback)
IN  — mish-shell (ZmqClient, reedline REPL)
IN  — nu_plugin_midi (PluginChannel, midi play/stop/monitor)
IN  — audio devices command (cpal device enumeration only)
IN  — tests (PTY + unit)

OUT — audio mixing, AudioClip, mix_into
OUT — audio file loading (symphonia, rubato)
OUT — mash-audio-core engine (playback)
OUT — mash-shell / mush-shell migration
OUT — nu_plugin_live_audio migration
OUT — WASM
OUT — DAWproject
OUT — TUI / GUI
```

### Phase 2 (separate plan)

```
IN  — AudioClip + mix_into
IN  — AudioLoader (symphonia + rubato)
IN  — mash-audio-core engine (cpal output with mixing)
IN  — mash-shell binary
IN  — nu_plugin_live_audio
IN  — audio load / play / seek commands
IN  — DAWproject load/save
```

---

## Existing Project Structure

```
mush/
├── Cargo.lock
├── Cargo.toml
├── crates/
│   ├── mash-audio-core/
│   │   └── src/audio/
│   │       ├── display.rs
│   │       ├── engine.rs
│   │       ├── enumerate.rs
│   │       ├── tasks.rs
│   │       └── types.rs
│   ├── mash-shell/
│   │   └── src/
│   │       ├── evaluator.rs
│   │       ├── lib.rs
│   │       └── main.rs
│   ├── mish-midi-core/
│   │   └── src/midi/
│   │       ├── display.rs
│   │       ├── engine.rs
│   │       ├── enumerate.rs
│   │       ├── tasks.rs
│   │       └── types.rs
│   ├── mish-shell/
│   │   └── src/
│   │       ├── evaluator.rs
│   │       ├── lib.rs
│   │       └── main.rs
│   ├── mush-commands/
│   │   └── src/lib.rs
│   ├── mush-core/
│   │   └── src/lib.rs
│   ├── mush-shell/
│   │   └── src/
│   │       ├── evaluator.rs
│   │       ├── lib.rs
│   │       └── main.rs
│   ├── nu_plugin_live_audio/
│   │   └── src/commands/
│   │       ├── devices.rs
│   │       ├── devices_refresh.rs
│   │       ├── play.rs
│   │       ├── record.rs
│   │       └── stop.rs
│   └── nu_plugin_midi/
│       └── src/commands/
│           ├── play.rs
│           ├── ports.rs
│           ├── ports_refresh.rs
│           ├── record.rs
│           └── stop.rs
└── tests/
    ├── cmd/
    │   ├── mash-banner.toml
    │   ├── mish-banner.toml
    │   └── mush-banner.toml
    └── _src/
        ├── mash_tests.rs
        ├── mish_tests.rs
        ├── mush_tests.rs
        ├── nu_plugin_midi_tests.rs
        └── repl_test.rs
```

---

## Target Structure (MVP complete)

```
mush/
├── Cargo.toml
├── rust-toolchain.toml
├── .cargo/config.toml
├── crates/
│   ├── mush-core/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── transport.rs          ← NEW
│   │       ├── update.rs             ← NEW
│   │       ├── shell.rs              ← NEW
│   │       ├── midi_output.rs        ← NEW
│   │       ├── rt_priority.rs        ← NEW
│   │       ├── monitor.rs            ← NEW
│   │       ├── zmq_transport.rs      ← NEW
│   │       └── zmq_client.rs         ← NEW
│   ├── mish-midi-core/
│   │   └── src/midi/
│   │       ├── display.rs            ← MOVE to shell
│   │       ├── engine.rs             ← EXTEND (RT thread)
│   │       ├── enumerate.rs          ← KEEP
│   │       ├── tasks.rs              ← ASSESS (may delete)
│   │       ├── types.rs              ← EXTEND
│   │       ├── timeline.rs           ← NEW
│   │       ├── clock.rs              ← NEW
│   │       └── file_io.rs            ← NEW
│   ├── mash-audio-core/
│   │   └── src/audio/
│   │       ├── display.rs            ← MOVE to shell
│   │       ├── engine.rs             ← STUB (Phase 2)
│   │       ├── enumerate.rs          ← EXTEND (serde)
│   │       ├── tasks.rs              ← ASSESS (may delete)
│   │       └── types.rs              ← EXTEND (serde, fields)
│   ├── mish-shell/
│   │   └── src/
│   │       ├── main.rs               ← REWRITE
│   │       ├── repl.rs               ← NEW (replaces evaluator)
│   │       ├── sink.rs               ← NEW
│   │       ├── format.rs             ← NEW
│   │       └── evaluator_legacy.rs   ← RENAME (delete after migration)
│   ├── mash-shell/                   ← DEFER to Phase 2
│   ├── mush-shell/                   ← DEFER to Phase 2
│   ├── nu_plugin_midi/
│   │   └── src/
│   │       ├── main.rs               ← EXTEND
│   │       ├── lib.rs                ← EXTEND
│   │       └── commands/
│   │           ├── play.rs           ← REWRITE
│   │           ├── stop.rs           ← REWRITE
│   │           ├── ports.rs          ← REWRITE
│   │           ├── ports_refresh.rs  ← REWRITE
│   │           ├── record.rs         ← STUB
│   │           └── monitor.rs        ← NEW (streaming)
│   └── nu_plugin_live_audio/
│       └── src/commands/
│           ├── devices.rs            ← REWRITE
│           ├── devices_refresh.rs    ← REWRITE
│           ├── play.rs               ← STUB
│           ├── stop.rs               ← STUB
│           └── record.rs             ← STUB
└── tests/
    ├── cmd/                          ← UPDATE snapshots
    └── _src/
        ├── mish_tests.rs             ← EXTEND
        ├── nu_plugin_midi_tests.rs   ← EXTEND
        └── ...
```

---

## Architectural Overview

```
Control Layer
    mish-shell (ZmqClient · reedline)
    nu_plugin_midi (PluginChannel · mpsc)
         │ TransportCommand
         ▼
ZeroMQ Transport Layer
    DEALER/ROUTER · PUB/SUB · inproc:// · rmp-serde
         │
         ▼
Async Bridge Layer
    EngineMonitor · Tokio Runtime
    ZmqTransport · plugin_cmd_rx · plugin_upd_tx
         │ write atomics / spawn_blocking
         ▼
Transport State
    Arc<TransportState>
    playhead · is_playing · bpm · next_event_idx · meta
         │ read atomics
         ▼
Real-Time Layer  [ISR-equivalent]
    Clock · cpal silence callback · SCHED_FIFO
    crossbeam::ArrayQueue · MidiEngine · midir
         │
         ▼
Hardware
    MIDI Interface · midir
    Audio Interface · cpal (enumeration only at MVP)
```

---

## Phase 0 — Stabilise Existing Code

**Goal:** Establish a clean, fully-tested baseline before any
architectural changes. Every existing test must pass at the end of
Phase 0 and remain passing throughout all subsequent phases.

---

### Slice 0a — Codebase Audit

**Entry:** Existing codebase as-is.
**Exit:** Completed audit document, all findings recorded.
**Time budget:** 2–4 hours.

See the standalone **Slice 0a Audit Document** for the full audit
procedure and recording templates.

#### Outputs

```
audit/
    baseline-test-output.txt
    baseline-clippy-output.txt
    cargo-tree.txt
    deps-depth-1.txt
    slice-0a-findings.md
```

---

### Slice 0b — Verify Baseline Tests

**Entry:** Slice 0a complete.
**Exit:** All existing tests pass and are recorded.

#### Tasks

- [ ] Run full test suite, confirm all pass:

```bash
cargo test --workspace 2>&1 | tee audit/baseline-test-output.txt
```

- [ ] Run clippy, record warnings:

```bash
cargo clippy --workspace 2>&1 | tee audit/baseline-clippy-output.txt
```

- [ ] Confirm `trycmd` snapshot files are up to date:

```bash
cargo test --test shell
```

- [ ] Confirm nushell plugin tests pass:

```bash
cargo test --test plugin
```

- [ ] Add `tests/fixtures/test.mid` if not present — short known-good
  MIDI file, 4 bars at 120 BPM.

#### Acceptance

```
All tests pass.
No unexpected clippy errors.
test.mid fixture present.
```

---

### Slice 0c — Assess `mush-commands`

**Entry:** Slice 0a complete.
**Exit:** Decision recorded, no code changes yet.

#### Tasks

- [ ] Read `crates/mush-commands/src/lib.rs` fully
- [ ] Determine whether it contains:
  - [ ] Shared command parsing logic
  - [ ] Shared command dispatch logic
  - [ ] Display / formatting logic
  - [ ] Direct engine dependencies
- [ ] Record decision:

| Option | Condition | Action |
|---|---|---|
| Merge into `mush-core` | Contains only shared types with no display logic | Merge in Slice 1a |
| Keep separate | Contains display logic or shell-specific concerns | Keep, extend in place |

#### Acceptance

```
Decision recorded in audit/slice-0a-findings.md.
No code changes made.
```

---

## Phase 1 — Insert New Architectural Layers

These slices add new infrastructure **without modifying the existing
shells or plugins**. The existing direct-call path continues to work
throughout Phase 1.

---

### Slice 1a — Extend `mush-core` with Transport Types

**Entry:** Phase 0 complete.
**Exit:** New types compile, existing code unaffected, unit tests pass.

#### New Files

- `crates/mush-core/src/transport.rs`
- `crates/mush-core/src/update.rs`
- `crates/mush-core/src/shell.rs`
- `crates/mush-core/src/midi_output.rs`
- `crates/mush-core/src/rt_priority.rs`

#### Tasks

**`transport.rs`**

- [ ] `TransportState` struct:
  - [ ] `playhead: Arc<AtomicU64>`
  - [ ] `is_playing: Arc<AtomicBool>`
  - [ ] `bpm_milli: Arc<AtomicU64>`
  - [ ] `next_event_idx: Arc<AtomicUsize>`
  - [ ] `meta: Arc<RwLock<ProjectMeta>>`
- [ ] `TransportState::new(bpm: f64) -> Arc<Self>`
- [ ] `TransportState::play()`
- [ ] `TransportState::stop()`
- [ ] `TransportState::bpm() -> f64`
- [ ] `TransportState::set_bpm(bpm: f64)`
- [ ] `TransportState::seek<E: HasSamplePos>(target, timeline)`
- [ ] `HasSamplePos` trait — `fn sample_pos(&self) -> u64`
- [ ] `TransportCommand` enum — `#[derive(Debug, Serialize, Deserialize)]`:
  - [ ] `Play`
  - [ ] `Stop`
  - [ ] `Seek(u64)`
  - [ ] `SetBpm(f64)`
  - [ ] `LoadMidi(PathBuf)`
  - [ ] `AudioDevices`
  - [ ] `MidiDevices`
- [ ] `ProjectMeta` struct — `#[derive(Debug, Default, Clone)]`

**`update.rs`**

- [ ] `AudioUpdate` enum — `#[derive(Debug, Clone, Serialize, Deserialize)]`:
  - [ ] `PlayheadMoved(u64)`
  - [ ] `TransportStarted`
  - [ ] `TransportStopped`
  - [ ] `DeviceList(AudioHostInfo)` — forward declare, filled by Slice 1c
  - [ ] `Error(String)`
- [ ] `MidiUpdate` enum — `#[derive(Debug, Clone, Serialize, Deserialize)]`:
  - [ ] `EventDispatched { channel: u8, note: u8, vel: u8 }`
  - [ ] `PortList(Vec<MidiPortInfo>)` — forward declare, filled by Slice 1b
  - [ ] `Error(String)`
- [ ] `EngineUpdate` enum — `#[derive(Debug, Clone, Serialize, Deserialize)]`:
  - [ ] `Audio(AudioUpdate)`
  - [ ] `Midi(MidiUpdate)`

**`shell.rs`**

- [ ] `Shell` trait:
  - [ ] `fn emit(&self, update: EngineUpdate)`
  - [ ] `fn name(&self) -> &str`

**`midi_output.rs`**

- [ ] `MidiOutput` trait:
  - [ ] `fn send(&mut self, bytes: &[u8]) -> Result<(), String>`

**`rt_priority.rs`**

- [ ] `pub fn elevate_current_thread()`
- [ ] `#[cfg(target_os = "linux")]` — `SCHED_FIFO`
- [ ] `#[cfg(target_os = "macos")]` — `thread_policy_set`
- [ ] `#[cfg(target_os = "windows")]` — `SetThreadPriority`
- [ ] All other targets — no-op

**`Cargo.toml` additions**

```toml
[dependencies]
zeromq    = { workspace = true }
rmp-serde = { workspace = true }
serde     = { workspace = true }
tokio     = { workspace = true }
crossbeam = { workspace = true }
cpal      = { workspace = true }
```

#### Unit Tests

```rust
// transport.rs
#[test] fn play_stop()
#[test] fn bpm_roundtrip()
#[test] fn seek_updates_both_atomics()
#[test] fn seek_idx_written_before_playhead()
#[test] fn transport_command_msgpack_roundtrip()
#[test] fn engine_update_msgpack_roundtrip()
```

#### Acceptance

```bash
cargo build -p mush-core
cargo test -p mush-core
cargo test --workspace   # all existing tests still pass
```

---

### Slice 1b — Extend `mish-midi-core`

**Entry:** Slice 1a complete.
**Exit:** Timeline tests pass, clock compiles, existing tests unaffected.

#### New Files

- `crates/mish-midi-core/src/midi/timeline.rs`
- `crates/mish-midi-core/src/midi/clock.rs`
- `crates/mish-midi-core/src/midi/file_io.rs`

#### Tasks

**Assess existing files first (from Slice 0a findings)**

- [ ] `types.rs` — add `HasSamplePos` impl if event type exists,
  otherwise add `TimelineEvent` to `timeline.rs`
- [ ] `engine.rs` — assess overlap with planned `MidiEngine` RT thread.
  Extend in place or add new RT thread alongside existing code.
- [ ] `tasks.rs` — if superseded by `EngineMonitor`, mark for deletion
  in Slice 2h. Do not delete yet.

**`timeline.rs`**

- [ ] `TimelineEvent` struct — `sample_pos: u64`, `payload: EventPayload`
- [ ] `EventPayload` enum — `Midi([u8; 3])`, `TempoChange(f64)`
- [ ] `HasSamplePos` impl for `TimelineEvent`
- [ ] `build_timeline(smf, sample_rate, initial_bpm) -> Vec<TimelineEvent>`:
  - [ ] Handle `Timing::Metrical` and `Timing::Timecode`
  - [ ] Expand delta-ticks to absolute samples
  - [ ] Handle `MetaMessage::Tempo` inline
  - [ ] Handle `NoteOn`, `NoteOff`, `Controller`, `ProgramChange`,
    `PitchBend`
  - [ ] Merge all tracks, sort by `sample_pos`

**`file_io.rs`**

- [ ] `load_midi_bytes(bytes, sample_rate, bpm) -> Result<Vec<TimelineEvent>>`
- [ ] `load_midi(path, sample_rate, bpm) -> Result<Vec<TimelineEvent>>`

**`engine.rs` extensions**

- [ ] `DesktopMidiOutput` — wraps `midir::MidiOutputConnection`,
  impls `MidiOutput` trait
- [ ] `MidiEngine::spawn(queue, status_tx)`:
  - [ ] `open_connection()` — list ports, connect to first
  - [ ] `run_loop()` — spin on `ArrayQueue`, send bytes, emit `MidiUpdate`
  - [ ] `rt_priority::elevate_current_thread()` on entry
  - [ ] `thread::yield_now()` when queue empty
  - [ ] ProgramChange (0xCn) sends 2 bytes, all others 3

**`clock.rs`**

- [ ] `Clock` enum — `Cpal(cpal::Stream)`, `Software(JoinHandle<()>)`
- [ ] `Clock::start(playhead, is_playing, next_event_idx, timeline,
  midi_queue, status_tx) -> Clock`
- [ ] `run_cpal(...)`:
  - [ ] Open default output device
  - [ ] `out.fill(0.0)` — silence only
  - [ ] Elevate RT priority on first callback invocation
  - [ ] Dispatch due `TimelineEvent`s into `ArrayQueue`
  - [ ] Advance `playhead` and `next_event_idx`
  - [ ] Emit `AudioUpdate::PlayheadMoved` every ~100ms
  - [ ] `// TODO Phase 2 Slice C: mix AudioClips here`
- [ ] `run_software(...)`:
  - [ ] `thread::sleep`-based ticker at 512/44100 cadence
  - [ ] Same dispatch logic as cpal callback
  - [ ] Fallback only — used when no audio device available
- [ ] `dispatch_and_advance(...)` — extracted pure function, shared
  by both clock implementations, independently testable

**`types.rs` additions**

- [ ] `MidiPortInfo` struct — `#[derive(Debug, Clone, Serialize, Deserialize)]`:
  - [ ] `index: usize`
  - [ ] `name: String`
  - [ ] `is_default: bool`
- [ ] `list_midi_ports() -> Vec<MidiPortInfo>`

#### Unit Tests

```rust
// timeline.rs
#[test] fn timeline_is_non_empty()
#[test] fn timeline_is_sorted()
#[test] fn timeline_contains_midi_events()
#[test] fn tempo_change_updates_timing()
#[test] fn different_sample_rates_scale_correctly()

// engine.rs
#[test] fn queue_pop_order_is_fifo()
#[test] fn program_change_sends_two_bytes()

// clock.rs
#[test] fn playhead_advances_when_playing()
#[test] fn playhead_does_not_advance_when_stopped()
#[test] fn due_events_are_dispatched()
#[test] fn future_events_are_not_dispatched()
```

#### Acceptance

```bash
cargo test -p mish-midi-core
cargo test --workspace   # all existing tests still pass
```

---

### Slice 1c — Extend `mash-audio-core` with Device Types

**Entry:** Slice 1a complete.
**Exit:** Device types serialise over msgpack, existing enumeration
wrapped, unit tests pass.

#### Tasks

**Assess existing files first (from Slice 0a findings)**

- [ ] `types.rs` — does it define device structs? If yes, add serde
  derives and missing fields. If no, add new types.
- [ ] `enumerate.rs` — wrap or extend return type to produce
  `AudioHostInfo`.

**`types.rs` additions / extensions**

- [ ] `AudioDeviceInfo` — `#[derive(Debug, Clone, Serialize, Deserialize)]`:
  - [ ] `index: usize`
  - [ ] `name: String`
  - [ ] `channels: u16`
  - [ ] `sample_rates: Vec<u32>`
  - [ ] `is_default: bool`
- [ ] `AudioHostInfo` — `#[derive(Debug, Clone, Serialize, Deserialize)]`:
  - [ ] `name: String`
  - [ ] `default: Option<String>`
  - [ ] `devices: Vec<AudioDeviceInfo>`

**`enumerate.rs` extensions**

- [ ] `list_devices() -> AudioHostInfo`:
  - [ ] Use `cpal::default_host()`
  - [ ] Identify default output device
  - [ ] Enumerate output devices
  - [ ] For each device: name, max channels, supported sample rates
  - [ ] Filter sample rates to common values within supported range:
    44100, 48000, 88200, 96000, 192000
  - [ ] Sort sample rates ascending
  - [ ] Mark default device

#### Unit Tests

```rust
#[test] fn list_devices_does_not_panic()
#[test] fn device_info_is_serialisable()
#[test] fn default_device_is_marked()
#[test] fn sample_rates_are_sorted()
#[test] fn audio_host_info_msgpack_roundtrip()
```

#### Acceptance

```bash
cargo test -p mash-audio-core
cargo test --workspace   # all existing tests still pass
```

---

### Slice 1d — ZeroMQ Transport

**Entry:** Slice 1a complete.
**Exit:** `TransportCommand` and `EngineUpdate` round-trip over
`inproc://`, unit tests pass.

#### New Files

- `crates/mush-core/src/zmq_transport.rs`
- `crates/mush-core/src/zmq_client.rs`

#### Tasks

**`zmq_transport.rs`** — engine side

- [ ] `ZmqTransport` struct — `router: RouterSocket`, `pub_: PubSocket`
- [ ] `ZmqTransport::bind(cmd_addr, pub_addr) -> Result<Self>`
- [ ] `ZmqTransport::recv_command() -> Option<TransportCommand>`:
  - [ ] ROUTER recv — frames: `[identity, empty, payload]`
  - [ ] msgpack decode payload
  - [ ] Log decode errors, return `None`
- [ ] `ZmqTransport::send_update(update: &EngineUpdate) -> Result<()>`:
  - [ ] msgpack encode
  - [ ] PUB send

**`zmq_client.rs`** — frontend side

- [ ] `ZmqClient` struct — `dealer: DealerSocket`, `sub: SubSocket`
- [ ] `ZmqClient::connect(cmd_addr, pub_addr) -> Result<Self>`
- [ ] `ZmqClient::connect_engine(cmd_addr, pub_addr) -> Result<()>`:
  - [ ] Connect additional DEALER + SUB endpoints
  - [ ] Commands fan out to all engines
  - [ ] Updates received from all engines
- [ ] `ZmqClient::send_command(cmd: &TransportCommand) -> Result<()>`
- [ ] `ZmqClient::recv_update() -> Result<EngineUpdate>`

**Address scheme**

| Topology | Address example |
|---|---|
| Same process | `inproc://mush-cmd` |
| Same host | `ipc:///tmp/mush-cmd` |
| Network | `tcp://192.168.1.10:5555` |

#### Unit Tests

```rust
#[tokio::test] async fn command_roundtrip_inproc()
#[tokio::test] async fn update_roundtrip_inproc()
#[tokio::test] async fn all_command_variants_roundtrip()
#[tokio::test] async fn all_update_variants_roundtrip()
#[tokio::test] async fn multiple_commands_in_sequence()
```

#### Acceptance

```bash
cargo test -p mush-core -- zmq
cargo test --workspace   # all existing tests still pass
```

---

### Slice 1e — EngineMonitor + PluginChannel

**Entry:** Slices 1b + 1c + 1d complete.
**Exit:** Monitor routes commands to `TransportState`, fans updates to
all sinks, streaming subscriptions work.

#### New File

- `crates/mush-core/src/monitor.rs`

#### Tasks

**`SubscriptionHandle`**

- [ ] `pub rx: mpsc::Receiver<EngineUpdate>`

**`PluginChannel`**

- [ ] `cmd_tx: mpsc::Sender<TransportCommand>`
- [ ] `sub_tx: mpsc::Sender<mpsc::Sender<EngineUpdate>>`
- [ ] `fn send(&self, cmd: TransportCommand) -> Result<(), String>`
- [ ] `fn subscribe(&self, buffer: usize) -> Result<SubscriptionHandle>`

**`EngineMonitor<S: Shell>`**

- [ ] Fields:
  - [ ] `transport: Arc<TransportState>`
  - [ ] `zmq: ZmqTransport`
  - [ ] `plugin_cmd_rx: mpsc::Receiver<TransportCommand>`
  - [ ] `sub_rx: mpsc::Receiver<mpsc::Sender<EngineUpdate>>`
  - [ ] `subscriptions: Vec<mpsc::Sender<EngineUpdate>>`
  - [ ] `audio_rx: std::sync::mpsc::Receiver<AudioUpdate>`
  - [ ] `midi_rx: std::sync::mpsc::Receiver<MidiUpdate>`
  - [ ] `sinks: Vec<S>`
- [ ] `EngineMonitor::new(..., num_plugins: usize) -> (Self, Vec<PluginChannel>)`
- [ ] `EngineMonitor::run(self)`:
  - [ ] Bridge `audio_rx` / `midi_rx` via `spawn_blocking`
  - [ ] `tokio::select!` over:
    - [ ] `sub_rx` — new subscription registrations
    - [ ] `zmq.recv_command()` — external frontends
    - [ ] `plugin_cmd_rx` — Nushell plugins
    - [ ] `audio_fwd_rx` — RT audio status
    - [ ] `midi_fwd_rx` — RT MIDI status
- [ ] `handle_command(cmd)`:
  - [ ] `Play` → `transport.play()`
  - [ ] `Stop` → `transport.stop()`
  - [ ] `SetBpm` → `transport.set_bpm()`
  - [ ] `Seek` → `transport.seek()`
  - [ ] `LoadMidi` → `spawn_blocking` → `load_midi()` →
    `RwLock::write()` → recalculate `next_event_idx` →
    emit `EngineUpdate`
  - [ ] `AudioDevices` → `list_devices()` →
    emit `EngineUpdate::Audio(AudioUpdate::DeviceList(...))`
  - [ ] `MidiDevices` → `list_midi_ports()` →
    emit `EngineUpdate::Midi(MidiUpdate::PortList(...))`
- [ ] `emit_all(update)`:
  - [ ] Local sinks — `sink.emit(update.clone())`
  - [ ] ZMQ PUB — `zmq.send_update(&update)`
  - [ ] Active subscriptions — `tx.try_send(update.clone())`
  - [ ] `subscriptions.retain(|tx| tx.try_send(...).is_ok())`

#### Unit Tests

```rust
#[tokio::test] async fn play_command_sets_is_playing()
#[tokio::test] async fn stop_command_clears_is_playing()
#[tokio::test] async fn set_bpm_updates_transport()
#[tokio::test] async fn engine_update_reaches_zmq_subscriber()
#[tokio::test] async fn engine_update_reaches_plugin_subscription()
#[tokio::test] async fn closed_subscription_is_removed()
#[tokio::test] async fn load_midi_populates_timeline()
#[tokio::test] async fn audio_devices_emits_device_list()
#[tokio::test] async fn midi_devices_emits_port_list()
#[tokio::test] async fn multiple_subscriptions_all_receive_update()
```

#### Acceptance

```bash
cargo test -p mush-core -- monitor
cargo test --workspace   # all existing tests still pass
```

---

## Phase 2 — Migrate Existing Shells and Plugins

Each migration is done one binary at a time. The existing direct-call
path continues to work until the migration of that binary is complete.

---

### Slice 2a — Audit Evaluators

**Entry:** Phase 1 complete.
**Exit:** Migration map complete, decisions recorded.

#### Tasks

- [ ] Read `evaluator.rs` in each shell crate
- [ ] Produce migration map:

| Shell | Command string | Engine call | `TransportCommand` |
|---|---|---|---|
| mish | | | |
| mash | | | |
| mush | | | |

- [ ] List commands with no `TransportCommand` equivalent — decide:
  **stub**, **defer**, or **implement**
- [ ] List display calls in each evaluator — confirm move to `format.rs`
- [ ] Assess `record` commands in both plugins — record decision

#### Acceptance

```
Migration map complete.
All decisions recorded.
No code changes made.
```

---

### Slice 2b — Add `format.rs` to Each Shell

**Entry:** Slice 2a complete.
**Exit:** Formatting extracted into pure functions, unit-testable.

#### Tasks — each shell crate

- [ ] Create `src/format.rs`:
  - [ ] `pub fn format_update(update: &EngineUpdate) -> Vec<String>`
  - [ ] `fn format_audio(update: &AudioUpdate) -> Vec<String>`
i  - [ ] `fn format_midi(update: &MidiUpdate) -> Vec<String>`
  - [ ] `fn format_device_list(info: &AudioHostInfo) -> Vec<String>`
  - [ ] `fn format_port_list(ports: &[MidiPortInfo]) -> Vec<String>`
  - [ ] `PlayheadMoved` — returns empty `Vec` (suppressed at MVP)
  - [ ] `EventDispatched` — returns empty `Vec` (suppressed at MVP)
  - [ ] `TransportStarted` — returns `vec!["▶  playing"]`
  - [ ] `TransportStopped` — returns `vec!["■  stopped"]`
  - [ ] Errors — returns `vec!["error: {msg}"]`

- [ ] Create `src/sink.rs`:
  - [ ] `ReedlineSink` struct — `printer: ExternalPrinter<String>`
  - [ ] `ReedlineSink::new() -> (ExternalPrinter<String>, Self)`
  - [ ] `impl Shell for ReedlineSink`:
    - [ ] `fn name(&self) -> &str`
    - [ ] `fn emit(&self, update: EngineUpdate)` — calls `format::format_update()`, prints each line

- [ ] Assess existing `display.rs` in core crates:
  - [ ] If display logic belongs in the shell — move to `format.rs`
  - [ ] If it contains domain helpers (e.g. note number → name) — keep in core
  - [ ] Mark for deletion in Slice 2h if fully superseded

#### Unit Tests — `format.rs`

```rust
#[test] fn transport_started_formats_correctly()
#[test] fn transport_stopped_formats_correctly()
#[test] fn playhead_moved_produces_no_output()
#[test] fn device_list_includes_header_and_devices()
#[test] fn device_list_marks_default_device()
#[test] fn port_list_includes_all_ports()
#[test] fn audio_error_formats_correctly()
#[test] fn midi_error_formats_correctly()
```

#### Acceptance

```bash
cargo test -p mish-shell -- format
cargo test --workspace   # all existing tests still pass
```

---

### Slice 2c — Migrate `mish-shell`

**Entry:** Slices 1e + 2b complete.
**Exit:** `mish` binary uses `ZmqClient` + `EngineMonitor`. All
existing `trycmd` tests pass.

#### Tasks

- [ ] Rename `src/evaluator.rs` → `src/evaluator_legacy.rs`
- [ ] Create `src/repl.rs`:
  - [ ] `pub async fn run(mut client: ZmqClient, printer: ExternalPrinter<String>)`
  - [ ] Parse `play`, `stop`, `bpm <n>`, `load <path>`, `seek <n>`,
    `midi devices`, `audio devices`, `quit`
  - [ ] Send via `client.send_command()`
  - [ ] Spawn task: `client.recv_update()` → `printer.print(format_update(...))`
- [ ] Rewrite `src/main.rs`:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport   = TransportState::new(120.0);
    let timeline    = Arc::new(RwLock::new(Vec::new()));
    let midi_queue  = Arc::new(ArrayQueue::new(1024));

    let (audio_tx, audio_rx) = std::sync::mpsc::channel();
    let (midi_tx,  midi_rx)  = std::sync::mpsc::channel();

    MidiEngine::spawn(Arc::clone(&midi_queue), midi_tx);

    let _clock = Clock::start(
        Arc::clone(&transport.playhead),
        Arc::clone(&transport.is_playing),
        Arc::clone(&transport.next_event_idx),
        Arc::clone(&timeline),
        Arc::clone(&midi_queue),
        audio_tx,
    );

    let zmq = ZmqTransport::bind(
        "inproc://mush-cmd",
        "inproc://mush-upd",
    ).await?;

    let (printer, sink) = ReedlineSink::new();

    let (monitor, _plugins) = EngineMonitor::new(
        Arc::clone(&transport),
        zmq,
        audio_rx,
        midi_rx,
        vec![sink],
        0,
    );

    let client = ZmqClient::connect(
        "inproc://mush-cmd",
        "inproc://mush-upd",
    ).await?;

    tokio::join!(
        monitor.run(),
        repl::run(client, printer),
    );

    Ok(())
}
```

- [ ] Verify all existing `trycmd` tests pass:

```bash
cargo test --test shell -- mish
```

- [ ] If snapshot tests fail due to formatting changes — update
  snapshots in a single commit:

```bash
TRYCMD=overwrite cargo test --test shell -- mish
```

#### Manual Acceptance Test

```
mish> audio devices
host:    CoreAudio
default: MacBook Pro Speakers

idx   name                      ch    sample rates
───   ──────────────────────    ──    ────────────
0     MacBook Pro Speakers      2     44100, 48000 ◀ default
1     BlackHole 2ch             2     44100, 48000

mish> midi devices
ports:
0     IAC Driver Bus 1
1     Focusrite USB MIDI

mish> load tests/fixtures/test.mid
mish> bpm 140
mish> play
[audible MIDI at 140 BPM]
mish> seek 44100
mish> stop
mish> quit
```

---

### Slice 2d — Migrate `nu_plugin_midi`

**Entry:** Slice 1e complete.
**Exit:** Plugin uses `PluginChannel`. All existing plugin tests pass.

#### Tasks

- [ ] Rewrite `src/main.rs` to wire engine + monitor + plugin channel:

```rust
fn main() {
    // Engine setup — same as mish-shell main.rs
    // ...

    let (monitor, mut plugin_channels) = EngineMonitor::new(
        Arc::clone(&transport),
        zmq,
        audio_rx,
        midi_rx,
        vec![],   // no local sinks for plugin
        1,        // one plugin channel
    );

    let channel = plugin_channels.remove(0);
    let plugin  = MidiPlugin::new(channel);

    // Run monitor in background thread.
    std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(monitor.run());
    });

    serve_plugin(&plugin, nu_plugin::MsgPackSerializer);
}
```

- [ ] Rewrite `commands/play.rs` — `channel.send(TransportCommand::Play)`
- [ ] Rewrite `commands/stop.rs` — `channel.send(TransportCommand::Stop)`
- [ ] Rewrite `commands/ports.rs` — `channel.send(TransportCommand::MidiDevices)`,
  wait for `MidiUpdate::PortList`, return as `PipelineData`
- [ ] Rewrite `commands/ports_refresh.rs` — same as `ports.rs`
- [ ] Stub `commands/record.rs` — return `LabeledError::new("not yet implemented")`
- [ ] Add `commands/monitor.rs` — `MidiMonitorCommand`:
  - [ ] `channel.subscribe(256)` → `SubscriptionHandle`
  - [ ] `std::iter::from_fn` blocking on `handle.rx.blocking_recv()`
  - [ ] `engine_update_to_value()` — `EngineUpdate` → `nu_protocol::Value`
  - [ ] Return `PipelineData::ListStream`
- [ ] Add `commands/audio_devices.rs` — `AudioDevicesCommand`:
  - [ ] `channel.send(TransportCommand::AudioDevices)`
  - [ ] Open short-lived subscription, wait for `AudioUpdate::DeviceList`
  - [ ] Return as `PipelineData::Value` (list of records)

#### `engine_update_to_value()`

```rust
fn engine_update_to_value(update: EngineUpdate, span: Span) -> Value {
    match update {
        EngineUpdate::Midi(MidiUpdate::EventDispatched { channel, note, vel }) => {
            Value::record(record! {
                "type"    => Value::string("midi",      span),
                "channel" => Value::int(channel as i64, span),
                "note"    => Value::int(note    as i64, span),
                "vel"     => Value::int(vel     as i64, span),
            }, span)
        }
        EngineUpdate::Audio(AudioUpdate::PlayheadMoved(pos)) => {
            Value::record(record! {
                "type"     => Value::string("playhead", span),
                "position" => Value::int(pos as i64,    span),
            }, span)
        }
        _ => Value::nothing(span),
    }
}
```

#### Manual Acceptance Test

```nushell
plugin add target/debug/nu_plugin_midi

midi play
midi stop
midi ports
# table of MIDI ports

audio devices
# table of audio devices

midi monitor | take 16 | select channel note vel
midi monitor | where note > 60 | first 8

midi play
midi monitor | each { |e| print $e }
```

#### Acceptance

```bash
cargo test --test plugin -- midi
cargo test --workspace   # all existing tests still pass
```

---

### Slice 2e — Migrate `mash-shell`

**Entry:** Slices 1e + 2b complete.
**Exit:** `mash` binary uses new layers. All existing tests pass.

#### Tasks

- [ ] Follow same pattern as Slice 2c
- [ ] Rename `evaluator.rs` → `evaluator_legacy.rs`
- [ ] Create `repl.rs` — extends `mish` commands with:
  - [ ] `audio devices` (already in `mish` — confirm consistent output)
- [ ] Create `format.rs` — extends `mish` format with any
  `mash`-specific display differences
- [ ] Rewrite `main.rs` — same wiring as `mish-shell`
- [ ] Verify existing tests:

```bash
cargo test --test shell -- mash
```

---

### Slice 2f — Migrate `nu_plugin_live_audio`

**Entry:** Slice 1e complete.
**Exit:** Plugin uses `PluginChannel`. All existing plugin tests pass.

#### Tasks

- [ ] Follow same pattern as Slice 2d
- [ ] Rewrite `commands/devices.rs` — `TransportCommand::AudioDevices`,
  return structured `PipelineData`
- [ ] Rewrite `commands/devices_refresh.rs` — same as `devices.rs`
- [ ] Stub `commands/play.rs` — `not yet implemented` (Phase 2)
- [ ] Stub `commands/stop.rs` — `not yet implemented` (Phase 2)
- [ ] Stub `commands/record.rs` — `not yet implemented` (Phase 2)
- [ ] Verify existing tests:

```bash
cargo test --test plugin -- audio
```

---

### Slice 2g — Migrate `mush-shell`

**Entry:** Slices 2c + 2e complete.
**Exit:** `mush` binary uses new layers, combines MIDI + audio commands.

#### Tasks

- [ ] Follow same pattern as Slice 2c
- [ ] `format.rs` covers both MIDI and audio updates
- [ ] `repl.rs` combines all commands from `mish` and `mash`
- [ ] Verify existing tests:

```bash
cargo test --test shell -- mush
```

---

### Slice 2h — Cleanup

**Entry:** All migrations complete, all tests pass.
**Exit:** Legacy code removed, architecture clean.

#### Tasks

- [ ] Delete `evaluator_legacy.rs` from each shell crate
- [ ] Assess `display.rs` in `mish-midi-core` and `mash-audio-core`:
  - [ ] Delete if fully superseded by shell `format.rs`
  - [ ] Reduce to domain helpers if any remain useful
- [ ] Assess `tasks.rs` in `mish-midi-core` and `mash-audio-core`:
  - [ ] Delete if fully superseded by `EngineMonitor`
  - [ ] Keep if still used
- [ ] Assess `mush-commands`:
  - [ ] Merge into `mush-core` if appropriate (per Slice 0c decision)
  - [ ] Otherwise clean up any dead code
- [ ] Run full suite:

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

---

## Phase 3 — Audio Playback (future)

---

### Slice A — `AudioClip`

**Entry:** MVP complete.
**Exit:** `mix_into()` unit tests pass, no hardware required.

#### Tasks — `mash-audio-core`

- [ ] `AudioClip` struct — `samples: Vec<f32>`, `channels: usize`,
  `start_sample: u64`
- [ ] `HasSamplePos` impl for `AudioClip`
- [ ] `mix_into(out, frame_start, channels)` — additive mix

#### Unit Tests

```rust
#[test] fn mix_into_silence_produces_clip_samples()
#[test] fn mix_into_clips_are_additive()
#[test] fn mix_into_clip_before_window_is_silent()
#[test] fn mix_into_clip_after_window_is_silent()
#[test] fn mix_into_partial_overlap_start()
#[test] fn mix_into_partial_overlap_end()
#[test] fn mix_into_channel_count_mismatch_is_safe()
```

---

### Slice B — `AudioLoader`

**Entry:** Slice A complete.
**Exit:** Audio files decode and resample correctly.

#### Tasks — `mash-audio-core`

- [ ] `loader::load_clip(path, start_sample, engine_sample_rate)`
  — symphonia decode + rubato resample
- [ ] `file_io::load_audio(path, start_sample, engine_sample_rate)`
- [ ] `file_io::load_dawproject_audio(path, engine_sample_rate)`

#### Unit Tests

```rust
#[test] fn load_wav_produces_non_empty_clip()
#[test] fn load_flac_produces_non_empty_clip()
#[test] fn resample_44100_to_48000_scales_length()
```

---

### Slice C — Audio Engine

**Entry:** Slices A + B complete.
**Exit:** Audio files play through speakers alongside MIDI.

#### Tasks — `mash-audio-core`

- [ ] Replace `out.fill(0.0)` stub in `clock.rs` with `mix_into()` loop:

```rust
// Replace:
out.fill(0.0);
// TODO Phase 2 Slice C: mix AudioClips here.

// With:
out.fill(0.0);
if let Ok(clips) = audio_clips.try_read() {
    for clip in clips.iter() {
        clip.mix_into(out, frame_start, channels);
    }
}
```

- [ ] Add `audio_clips: Arc<RwLock<Vec<AudioClip>>>` to `TransportState`
- [ ] Add `LoadAudio` to `TransportCommand`
- [ ] Add `LoadAudio` handling in `EngineMonitor`
- [ ] Add `AudioUpdate::ClipLoaded` variant
- [ ] Add `TransportCommand::LoadProject`, `SaveProject`

---

### Slice D — DAWproject

**Entry:** Slice C complete.

- [ ] `load_dawproject_midi(path, sample_rate, bpm)`
- [ ] `load_dawproject_audio(path, engine_sample_rate)`
- [ ] `save_dawproject(path, timeline, clips, meta)`
- [ ] `EngineMonitor` — `LoadProject`, `SaveProject` handling

---

### Slice E — `mash-shell` Audio Commands

**Entry:** Slice C complete.

- [ ] `load audio <path>`
- [ ] `load project <path>`
- [ ] `save project <path>`

---

### Slice F — `nu_plugin_live_audio` Full Implementation

**Entry:** Slice C complete.

- [ ] Implement stubbed `play`, `stop`, `record` commands
- [ ] Add `audio load`, `audio load project`, `audio save project`
- [ ] Add `audio monitor` streaming command

---

### Slice G — Audio Integration Tests

**Entry:** Slices E + F complete.

- [ ] `tests/fixtures/test.wav`
- [ ] `tests/fixtures/test.dawproject`
- [ ] `mash_pty.rs` — load audio, play, seek, save/reload project

---

## Complete Slice Summary

| Slice | Crate | Deliverable | Phase |
|---|---|---|---|
| 0a | all | codebase audit | 0 |
| 0b | tests | baseline verified, fixtures added | 0 |
| 0c | mush-commands | merge/keep decision | 0 |
| 1a | mush-core | transport types, serde, traits, rt_priority | 1 |
| 1b | mish-midi-core | timeline, MidiEngine, Clock | 1 |
| 1c | mash-audio-core | device types + serde | 1 |
| 1d | mush-core | ZmqTransport + ZmqClient + tests | 1 |
| 1e | mush-core | EngineMonitor + PluginChannel + subscriptions | 1 |
| 2a | all shells | evaluator audit, migration map | 2 |
| 2b | all shells | format.rs + ReedlineSink | 2 |
| 2c | mish-shell | migrate to ZmqClient + EngineMonitor | 2 |
| 2d | nu_plugin_midi | migrate to PluginChannel | 2 |
| 2e | mash-shell | migrate to ZmqClient + EngineMonitor | 2 |
| 2f | nu_plugin_live_audio | migrate to PluginChannel | 2 |
| 2g | mush-shell | migrate to ZmqClient + EngineMonitor | 2 |
| 2h | all | cleanup legacy code | 2 |
| A | mash-audio-core | AudioClip + mix_into | 3 |
| B | mash-audio-core | AudioLoader (symphonia + rubato) | 3 |
| C | mash-audio-core | audio engine with mixing | 3 |
| D | mash-audio-core | DAWproject load/save | 3 |
| E | mash-shell | audio commands | 3 |
| F | nu_plugin_live_audio | full audio plugin | 3 |
| G | tests | audio integration tests | 3 |

---

## Dependency Graph

```
Phase 0 (audit + stabilise)
    └── Phase 1 (insert new layers)
            ├── Slice 1a (mush-core types)        ← foundation
            │       ├── Slice 1b (timeline+clock) ← parallel with 1c, 1d
            │       ├── Slice 1c (device types)   ← parallel with 1b, 1d
            │       └── Slice 1d (ZMQ)            ← parallel with 1b, 1c
            │               └── Slice 1e (EngineMonitor) ← needs 1b+1c+1d
            └── Phase 2 (migrate existing code)
                    └── Slice 2a (audit evaluators)
                            └── Slice 2b (format.rs)
                                    ├── Slice 2c (mish-shell)    ─┐
                                    ├── Slice 2d (nu_plugin_midi) ─┤ parallel
                                    ├── Slice 2e (mash-shell)    ─┤
                                    ├── Slice 2f (nu_plugin_live_audio) ─┘
                                    └── Slice 2g (mush-shell) ← needs 2c+2e
                                            └── Slice 2h (cleanup)
                                                    └── Phase 3 (audio)
                                                            ├── Slice A (AudioClip)    ─┐ parallel
                                                            └── Slice B (AudioLoader)  ─┘
                                                                    └── Slice C (engine)
                                                                            ├── Slice D (DAWproject)
                                                                            ├── Slice E (mash-shell)
                                                                            └── Slice F (nu_plugin_live_audio)
                                                                                    └── Slice G (tests)
```

---

## Hour Estimates

| Phase | Optimistic | Realistic |
|---|---|---|
| Phase 0 | 4h | 9h |
| Phase 1 | 17h | 34h |
| Phase 2 | 19h | 38h |
| Testing (throughout) | 8h | 18h |
| **Total** | **48h** | **99h** |

### 4-Week Schedule at 25h/week

| Week | Slices | Target hours | Deliverable |
|---|---|---|---|
| 1 | 0a–0c, 1a–1c | ~25h | Audit complete, new types compile, timeline + device types tested |
| 2 | 1d–1e | ~25h | ZMQ roundtrip tested, EngineMonitor routes commands end-to-end |
| 3 | 2a–2d | ~25h | mish-shell and nu_plugin_midi fully migrated, existing tests pass |
| 4 | 2e–2h | ~25h | All shells and plugins migrated, full test suite passes |

---

## Key Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| ZeroMQ crate bugs or missing features | Medium | High | Spike `inproc://` roundtrip in week 1 before committing |
| `EngineMonitor` subscription race conditions | Medium | High | Write unit tests before implementation |
| `cpal` platform differences (Linux/macOS/Windows) | Medium | Medium | Test clock in isolation on target platform early |
| `trycmd` snapshot brittleness | High | Low | Update all snapshots in single commit at end of each migration slice |
| `record` commands have no `TransportCommand` equivalent | High | Low | Stub with `not yet implemented` in Slice 2d/2f, decide in Slice 2a |
| `tasks.rs` overlap with `EngineMonitor` larger than expected | Unknown | High | Resolve in Slice 0a audit before writing any new code |

---

## Definition of Done

The MVP is complete when:

- [ ] `cargo test --workspace` passes with no failures
- [ ] `cargo clippy --workspace -- -D warnings` passes with no warnings
- [ ] `cargo fmt --check` passes
- [ ] `mish` REPL accepts `play`, `stop`, `bpm`, `load`, `seek`,
  `midi devices`, `audio devices`, `quit`
- [ ] `mish` produces audible MIDI output on a connected synth
- [ ] `nu_plugin_midi` commands work in Nushell:
  `midi play`, `midi stop`, `midi ports`, `audio devices`,
  `midi monitor`
- [ ] `midi monitor` streams events correctly and cancels cleanly
  on `Ctrl+C`
- [ ] All existing `trycmd` and plugin framework tests pass
- [ ] Legacy `evaluator.rs` files deleted
- [ ] `display.rs` files in core crates assessed and cleaned up
