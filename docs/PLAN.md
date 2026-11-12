# mush MVP — Implementation Task Checklist

---

## 1. Stabilise

### 1.1 Audit

- [x] 1.1.1 Run baseline test suite, record output
- [x] 1.1.2 Run baseline clippy, record output
- [x] 1.1.3 Document workspace dependency graph
- [x] 1.1.4 Document `mush-core` public API
- [x] 1.1.5 Document `mush-commands` contents, make merge/keep decision
- [x] 1.1.6 Document `mish-midi-core` module contents
- [x] 1.1.7 Document `mash-audio-core` module contents
- [x] 1.1.8 Document shell evaluator call graph
- [x] 1.1.9 Document plugin command inventory
- [x] 1.1.10 Document test coverage and gaps
- [x] 1.1.11 Record all dependency versions

### 1.2 Baseline

- [x] 1.2.1 Confirm all existing tests pass
- [x] 1.2.2 Add `tests/fixtures/test.mid`
- [x] 1.2.3 Confirm `trycmd` snapshots are current
- [x] 1.2.4 Confirm nushell plugin tests pass

---

## 2. Foundation — `mush-core`

### 2.1 Workspace Dependencies

- [x] 2.1.1 Add `tokio` to workspace `Cargo.toml`
- [x] 2.1.2 Add `serde` to workspace `Cargo.toml`
- [x] 2.1.3 Add `rmp-serde` to workspace `Cargo.toml`
- [x] 2.1.4 Add `tmq` to workspace `Cargo.toml`
- [x] 2.1.5 Add `crossbeam` to workspace `Cargo.toml`
- [x] 2.1.6 Add `midly` to workspace `Cargo.toml`
- [x] q2.1.7 Add `libc` to workspace `Cargo.toml`
- [x] 2.1.8 Confirm `resolver = "3"` in workspace root
- [x] 2.1.9 Confirm `edition = "2024"` in all crate `Cargo.toml` files
- [x] 2.1.10 Run `cargo build --workspace` — confirm clean

### 2.2 Transport State

- [x] 2.2.1 Create `mush-core/src/transport.rs`
- [x] 2.2.2 Define `HasSamplePos` trait
- [x] 2.2.3 Define `ProjectMeta` struct
- [x] 2.2.4 Define `TransportState` struct
- [x] 2.2.5 Implement `TransportState::new()`
- [x] 2.2.6 Implement `TransportState::play()`
- [x] 2.2.7 Implement `TransportState::stop()`
- [x] 2.2.8 Implement `TransportState::bpm()`
- [x] 2.2.9 Implement `TransportState::set_bpm()`
- [x] 2.2.10 Implement `TransportState::seek()`
- [x] 2.2.11 Add unit tests
- [x] 2.2.12 Wire `transport` module into `lib.rs`

### 2.3 Commands and Updates

- [x] 2.3.1 Define `TransportCommand` enum with serde derives
- [x] 2.3.2 Create `mush-core/src/update.rs`
- [x] 2.3.3 Define `AudioUpdate` enum with serde derives
- [x] 2.3.4 Define `MidiUpdate` enum with serde derives
- [x] 2.3.5 Define `EngineUpdate` enum with serde derives
- [x] 2.3.6 Wire `update` module into `lib.rs`

### 2.4 Traits

- [x] 2.4.1 Create `mush-core/src/sink.rs`
- [x] 2.4.2 Define `EngineSink` trait
- [x] 2.4.3 Create `mush-core/src/midi_output.rs`
- [x] 2.4.4 Define `MidiOutput` trait
- [x] 2.4.5 Wire `sink` and `midi_output` modules into `lib.rs`

### 2.5 Platform Shims

- [x] 2.5.1 Create `mush-core/src/rt_priority.rs`
- [x] 2.5.2 Implement `elevate_current_thread()` — Linux
- [x] 2.5.3 Implement `elevate_current_thread()` — macOS
- [x] 2.5.4 Implement `elevate_current_thread()` — no-op fallback
- [x] 2.5.5 Wire `rt_priority` module into `lib.rs`

---

## 3. MIDI Engine — `mish-midi-core`

### 3.1 Timeline

- [x] 3.1.1 Create `mish-midi-core/src/midi/timeline.rs`
- [x] 3.1.2 Define `TimelineEvent` struct
- [x] 3.1.3 Define `EventPayload` enum
- [x] 3.1.4 Implement `HasSamplePos` for `TimelineEvent`
- [x] 3.1.5 Implement `TempoMap`
- [x] 3.1.6 Implement `TempoMap::ticks_to_samples()`
- [x] 3.1.7 Implement `midi_message_to_payload()`
- [x] 3.1.8 Implement `load_midi_bytes()`
- [x] 3.1.9 Implement `load_midi()`
- [x] 3.1.10 Add `gen_test_mid` binary
- [x] 3.1.11 Add unit tests
- [x] 3.1.12 Wire `timeline` module into `mod.rs`

### 3.2 MIDI Output and Engine

- [x] 3.2.1 Implement `DesktopMidiOutput` struct
- [x] 3.2.2 Implement `DesktopMidiOutput::open_first_port()`
- [x] 3.2.3 Implement `MidiOutput` for `DesktopMidiOutput`
- [x] 3.2.4 Implement `MidiEngine::spawn()`
- [x] 3.2.5 Implement `MidiEngine::run_loop()`
- [x] 3.2.6 Handle 2-byte vs 3-byte MIDI messages
- [x] 3.2.7 Add `thread::yield_now()` on empty queue

### 3.3 Clock

- [x] 3.3.1 Create `mish-midi-core/src/midi/clock.rs`
- [x] 3.3.2 Define `Clock` enum — `Cpal` and `Software` variants
- [x] 3.3.3 Implement `Clock::start()`
- [x] 3.3.4 Implement `Clock::try_cpal()`
- [x] 3.3.5 Implement cpal callback — silence output
- [x] 3.3.6 Implement cpal callback — MIDI dispatch
- [x] 3.3.7 Implement cpal callback — playhead advancement
- [x] 3.3.8 Implement cpal callback — periodic status report
- [x] 3.3.9 Implement `Clock::start_software()`
- [x] 3.3.10 Implement `dispatch_events()` — shared by both paths
- [x] 3.3.11 Implement `Drop` for `Clock`
- [x] 3.3.12 Add `TODO Phase 3` comment at mixing stub
- [x] 3.3.13 Wire `clock` module into `mod.rs`

### 3.4 MIDI Device Enumeration

- [x] 3.4.1 Extend `enumerate.rs` — define `MidiPortInfo` with serde
- [x] 3.4.2 Implement `list_midi_ports() -> Vec<MidiPortInfo>`
- [x] 3.4.3 Update `MidiUpdate::PortList` to use `Vec<MidiPortInfo>`
- [x] 3.4.4 Merge `DevicesRefresh` into `Devices` — remove cache

---

## 4. Audio Device Enumeration — `mash-audio-core`

### 4.1 Device Types

- [x] 4.1.1 Define `AudioDeviceInfo` struct with serde derives
- [x] 4.1.2 Define `AudioHostInfo` struct with serde derives
- [x] 4.1.3 Add missing fields — `is_default`, `sample_rates`, `index`

### 4.2 Enumeration

- [x] 4.2.1 Implement `list_devices() -> AudioHostInfo`
- [x] 4.2.2 Identify default output device
- [x] 4.2.3 Enumerate supported sample rates per device
- [x] 4.2.4 Sort sample rates ascending
- [x] 4.2.5 Update `AudioUpdate::DeviceList` to use `AudioHostInfo`
- [x] 4.2.6 Merge `DevicesRefresh` into `Devices` — remove cache
- [x] 4.2.7 Add unit tests
- [x] 4.2.8 Update affected existing tests

---

## 5. Transport Layer — `mush-core`

### 5.1 ZMQ Transport

- [x] 5.1.1 Create `mush-core/src/zmq_transport.rs`
- [x] 5.1.2 Implement `ZmqTransport::bind()`
- [x] 5.1.3 Implement `ZmqTransport::recv_command()`
- [x] 5.1.4 Implement `ZmqTransport::send_update()`
- [x] 5.1.5 Handle DEALER/ROUTER identity frames
- [x] 5.1.6 Wire `zmq_transport` module into `lib.rs`

### 5.2 ZMQ Client

- [x] 5.2.1 Create `mush-core/src/zmq_client.rs`
- [x] 5.2.2 Implement `ZmqClient::connect()`
- [x] 5.2.3 Implement `ZmqClient::send_command()`
- [x] 5.2.4 Implement `ZmqClient::recv_update()`
- [x] 5.2.5 Wire `zmq_client` module into `lib.rs`

### 5.3 Serialisation Tests

- [x] 5.3.1 Add msgpack roundtrip test — all `TransportCommand` variants
- [x] 5.3.2 Add msgpack roundtrip test — all `EngineUpdate` variants
- [x] 5.3.3 Add ZMQ roundtrip test — command over `inproc://`
- [x] 5.3.4 Add ZMQ roundtrip test — update over `inproc://`
- [x] 5.3.5 Add ZMQ roundtrip test — multiple commands in sequence

---

## 6. TheConductor — `mush-core`

### 6.0 TransportCommand Extensions

- [x] 6.0.1 Add `MidiEvent` struct to `transport.rs`
- [x] 6.0.2 Add `QueueEvent(MidiEvent)` variant to `TransportCommand`
- [x] 6.0.3 Add `LoadMudl(PathBuf)` variant to `TransportCommand`

### 6.1 Subscription Channel

- [x] 6.1.1 Define `Subscription` struct
- [x] 6.1.2 Define `SubscriptionHandle` struct
- [x] 6.1.3 Implement `Subscription::send()`
- [x] 6.1.4 Implement `SubscriptionHandle::recv()`
- [x] 6.1.5 Implement dropped handle detection and cleanup

### 6.2 TheConductor

- [x] 6.2.1  Create `mush-core/src/conductor.rs`
- [x] 6.2.2  Define `TheConductor` struct
- [x] 6.2.3  Implement `TheConductor::new()`
- [x] 6.2.4  Implement `TheConductor::run()`
- [x] 6.2.5  Implement `tokio::select!` loop
- [x] 6.2.6  Implement `handle_command()` — `Play`, `Stop`, `SetBpm`, `Seek`
- [x] 6.2.7  Implement `handle_command()` — `LoadMidi`
- [x] 6.2.8  Implement `handle_command()` — `LoadMudl` (reserved — NotImplemented)
- [x] 6.2.9  Implement `handle_command()` — `QueueEvent`
- [x] 6.2.10 Implement `handle_command()` — `AudioDevices`
- [x] 6.2.11 Implement `handle_command()` — `MidiDevices`
- [x] 6.2.12 Implement `emit_all()` — fan-out to ZMQ PUB + all Subscriptions
- [x] 6.2.13 Implement subscription cleanup — `retain()` on dead handles
- [x] 6.2.14 Switch clock status_tx to `tokio::sync::mpsc`
- [x] 6.2.15 Wire `conductor` module into `lib.rs`

### 6.3 TheConductor Tests

- [x] 6.3.1  Test `Play` command sets `is_playing`
- [x] 6.3.2  Test `Stop` command clears `is_playing`
- [x] 6.3.3  Test `SetBpm` updates transport
- [x] 6.3.4  Test update reaches ZMQ subscriber
- [x] 6.3.5  Test update reaches `SubscriptionHandle`
- [x] 6.3.6  Test closed subscription is removed cleanly
- [x] 6.3.7  Test `LoadMidi` populates timeline
- [x] 6.3.8  Test `AudioDevices` emits device list
- [x] 6.3.9  Test `MidiDevices` emits port list
- [x] 6.3.10 Test multiple subscriptions all receive update

---

## 7. Shell Migration

### 7.1 Formatting

- [x] 7.1.1  Create `mish-shell/src/format.rs`
- [x] 7.1.2  Implement `format_update()` — `TransportStarted`
- [x] 7.1.3  Implement `format_update()` — `TransportStopped`
- [x] 7.1.4  Implement `format_update()` — `PlayheadMoved` (suppress)
- [x] 7.1.5  Implement `format_update()` — `DeviceList`
- [x] 7.1.6  Implement `format_update()` — `PortList`
- [x] 7.1.7  Implement `format_update()` — errors
- [x] 7.1.8  Implement `format_update()` — `LoadMudl` acknowledgement
- [x] 7.1.9  Unit tests for all format functions
- [x] 7.1.10 Create `mash-shell/src/format.rs` — same pattern
- [x] 7.1.11 Create `mush-shell/src/format.rs` — combined MIDI + audio

### 7.2 Sink

- [x] 7.2.1 Create `mish-shell/src/sink.rs`
- [x] 7.2.2 Define `ReedlineSink` struct
- [x] 7.2.3 Implement `EngineSink` for `ReedlineSink`
  — `emit()` must be non-blocking
  — use `external_print` not blocking I/O
- [x] 7.2.4 Repeat for `mash-shell` and `mush-shell`

### 7.3 `mish-shell` Migration

- [x] 7.3.1 Rename `evaluator.rs` → `evaluator_legacy.rs`
- [x] 7.3.2 Create `repl.rs` — parse input, send `TransportCommand`
- [x] 7.3.3 Wire `ZmqClient` into `repl.rs`
  — `connect(addrs::CMD_INPROC, addrs::PUB_INPROC)` for MVP
- [x] 7.3.4 Rewrite `main.rs` — wire all components
- [x] 7.3.5 Confirm existing `trycmd` tests pass
- [x] 7.3.6 Update snapshots if formatting changed
- [ ] 7.3.7 Delete `evaluator_legacy.rs`

### 7.4 `mash-shell` Migration

- [x] 7.4.1 Follow same pattern as 7.3
- [x] 7.4.2 Wire `ZmqClient` — inproc transport
- [x] 7.4.3 Confirm existing tests pass

### 7.5 `mush-shell` Migration

- [x] 7.5.1 Follow same pattern as 7.3
- [x] 7.5.2 Full MIDI + audio command set — superset of `mish` and `mash`
- [x] 7.5.3 Wire `ZmqClient` — inproc transport
- [x] 7.5.4 Developed after 7.3 and 7.4 are stable
- [x] 7.5.5 Confirm existing tests pass

---

## 8. Plugin Migration

### 8.0 Deferred from Task 7 ✅ Complete

- [x] 8.0.1  `mish-shell/src/main.rs` — subscribe + sink task
- [x] 8.0.2  `mash-shell/src/main.rs` — subscribe + sink task
- [x] 8.0.3  `mush-shell/src/main.rs` — subscribe + sink task
- [x] 8.0.4  Confirmed `mish> play` → `▶ transport started` in REPL
- [x] 8.0.F  Fix: shared ZMQ Context required for inproc transport
- [x] 8.0.5  Update `trycmd` snapshots — `mish-shell`
- [x] 8.0.6  Update `trycmd` snapshots — `mash-shell`
- [x] 8.0.7  Update `trycmd` snapshots — `mush-shell`
- [x] 8.0.8  Add `EngineUpdate::Midi(MidiUpdate::LoadMudlAck)` variant

### 8.1 `nu_plugin_midi` Migration — Structural ✅ Complete

- [x] 8.1.1  Embed `Conductor` in `main.rs`
- [x] 8.1.2  Remove `MidiEngine` from `MidiPlugin` struct
- [x] 8.1.3  Wire `ZmqClient` into plugin commands via shared state

### 8.1 `nu_plugin_midi` Migration — Commands ⏸ Deferred to post-8.3

Commands cannot be meaningfully implemented until `handle_command` is
complete (8.3). Stubs that call `recv_update()` will hang.

- [x] 8.1.4  `midi ports`
     — `TransportCommand::MidiDevices` → `MidiUpdate::PortList`
- [x] 8.1.5  `midi port` (no arg)
     — return active port name from `TransportState`
- [x] 8.1.6  `midi port <name>`
     — set active port in `TransportState`
- [x] 8.1.7  `midi play`
     — `PipelineData::Empty` → `TransportCommand::Play`
     — `PipelineData::ListStream` → `QueueEvent` per record + `Play`
- [x] 8.1.8  `midi stop`
     — `TransportCommand::Stop`
- [x] 8.1.9  `midi load <path>`
     — `.mid` → `TransportCommand::LoadMidi(path)`
     — `.mudl` → `TransportCommand::LoadMudl(path)`
     — extension determines variant
- [x] 8.1.10 `midi send <msg>`
     — `TransportCommand::SendMessage(bytes)`
- [x] 8.1.11 `midi monitor`
     — `recv_update()` loop → `ListStream`
     — autosave pre-roll to `.mudl` in cwd (optional path arg)
- [x] 8.1.12 `midi record`
     — on-demand bounded capture
     — `TransportCommand::Record` start/stop
- [x] 8.1.13 `midi save <path>`
     — `.mid` or `.mudl` determined by extension
     — `TransportCommand::Save(path)`
- [x] 8.1.14 Confirm plugin tests pass

### 8.2 `nu_plugin_live_audio` Migration ⏸ Deferred to post-8.3

- [x] 8.2.1  Embed `Conductor` in `main.rs`
- [x] 8.2.2  Remove `AudioEngine` from `AudioPlugin` struct
- [x] 8.2.3  Wire `ZmqClient` into plugin commands via shared state
- [x] 8.2.4  `audio hosts`
     — `TransportCommand::AudioHosts` → `AudioUpdate::HostList`
- [x] 8.2.5  `audio host` (no arg)
     — return active host name from `TransportState`
- [x] 8.2.6  `audio host <name>`
     — set active host in `TransportState`
- [x] 8.2.7  `audio devices`
     — `TransportCommand::AudioDevices` → `AudioUpdate::DeviceList`
- [x] 8.2.8  `audio device` (no arg)
     — return active device name from `TransportState`
- [x] 8.2.9  `audio device <name>`
     — set active device in `TransportState`
- [x] 8.2.10 Confirm plugin tests pass

### 8.3 Wire Clock, Timeline, and MIDI Output into Conductor

This is the critical path. All of 8.1 and 8.2 unblock once this is done.

#### 8.3.1 Audit `mish-midi-core`

- [x] Confirm what exists: `clock.rs`, `timeline.rs`, `engine.rs`,
  `midir` wiring
- [x] Identify what moves to `mush-core` vs stays in `mish-midi-core`
- [x] Confirm `midir` output port open/close lifecycle

#### 8.3.2 Extend `TransportState`

- [x] Add `active_midi_port: Arc<RwLock<Option<String>>>`
     — set by `midi port <name>`
     — read by `Conductor` when opening `midir` connection
- [x] Add `active_audio_host: Arc<RwLock<Option<String>>>`
- [x] Add `active_audio_device: Arc<RwLock<Option<String>>>`

#### 8.3.3 Extend `TransportCommand`

- [x] Add `MidiPort(Option<String>)`
     — `None` → query active port
     — `Some(name)` → set active port
- [x] Add `AudioHost(Option<String>)`
- [x] Add `AudioDevice(Option<String>)`
- [x] Add `AudioHosts`
     — enumerate available hosts
- [x] Add `SendMessage(Vec<u8>)`
     — raw MIDI bytes, bypasses timeline
- [x] Add `Record(bool)`
     — `true` → start bounded capture
     — `false` → stop and return events
- [x] Add `Save(PathBuf)`
     — save timeline to `.mid` or `.mudl`

#### 8.3.4 Extend `EngineUpdate`

- [x] Add `MidiUpdate::PortList` — already exists ✅
- [x] Add `MidiUpdate::ActivePort(Option<String>)`
- [x] Add `AudioUpdate::HostList(String)`
- [x] Add `AudioUpdate::ActiveHost(Option<String>)`
- [x] Add `AudioUpdate::ActiveDevice(Option<String>)`
- [x] Add `MidiUpdate::RecordStopped(Vec<MidiEvent>)`
- [x] Add `MidiUpdate::SaveAck(String)`

#### 8.3.5 Add timeline to `Conductor`

- [x] Add `timeline: Arc<RwLock<Vec<TimelineEvent>>>`
- [x] Add `midi_queue: Arc<ArrayQueue<[u8; 3]>>`
- [x] `handle_command(QueueEvent(event))` → append to timeline,
  sort by `beat`
- [x] `handle_command(LoadMidi(path))` → parse via `midly`
  → populate timeline → emit `MidiUpdate::LoadMudlAck`
- [x] `handle_command(LoadMudl(path))` → parse KDL
  → populate timeline → emit `MidiUpdate::LoadMudlAck`

#### 8.3.6 Add MIDI output to `Conductor`

- [x] Open `midir::MidiOutput` on startup
- [x] `handle_command(MidiDevices)` → enumerate ports
  → emit `MidiUpdate::PortList`
- [x] `handle_command(MidiPort(None))` → emit `MidiUpdate::ActivePort`
- [x] `handle_command(MidiPort(Some(name)))` → open connection
  → store in `Conductor` → update `TransportState::active_midi_port`
  → emit `MidiUpdate::ActivePort`
- [x] `handle_command(SendMessage(bytes))` → write directly to
  open `MidiOutputConnection` → bypasses timeline

#### 8.3.7 Wire `Clock` into `Conductor`

- [x] Add `clock: Option<Clock>` to `Conductor`
- [x] `handle_command(Play)`:
  — construct `ClockArgs` from `Conductor` fields
  — `Clock::start(args)` → store in `self.clock`
  — emit `AudioUpdate::TransportStarted`
- [x] `handle_command(Stop)`:
  — drop `self.clock`
  — emit `AudioUpdate::TransportStopped`
- [x] `handle_command(Seek(pos))` → write `TransportState` atomics
- [x] `handle_command(SetBpm(bpm))` → write `TransportState` atomics

#### 8.3.8 Add pre-roll buffer to `Conductor`

- [x] Add `pre_roll: Arc<ArrayQueue<MidiEvent>>`
  — fixed large capacity (e.g. 65_536 events)
- [x] Fed by `midi monitor` — every incoming MIDI event pushed here
- [x] Autosave to `.mudl` in cwd when buffer wraps
  — autosave path overridable via `midi monitor <path>` arg
- [x] `handle_command(Record(true))` → begin bounded capture
  alongside pre-roll
- [x] `handle_command(Record(false))` → stop capture
  → emit `MidiUpdate::RecordStopped(events)`

#### 8.3.9 Wire audio device enumeration into `Conductor`

- [x] `handle_command(AudioHosts)` → enumerate `cpal` hosts
  → emit `AudioUpdate::HostList`
- [x] `handle_command(AudioDevices)` → enumerate devices on active host
  → emit `AudioUpdate::DeviceList`
- [x] `handle_command(AudioHost(None))` → emit `AudioUpdate::ActiveHost`
- [x] `handle_command(AudioHost(Some(name)))` → set active host
  → emit `AudioUpdate::ActiveHost`
- [x] `handle_command(AudioDevice(None))` → emit `AudioUpdate::ActiveDevice`
- [x] `handle_command(AudioDevice(Some(name)))` → set active device
  → emit `AudioUpdate::ActiveDevice`

#### 8.3.10 Implement `Save`

- [x] `handle_command(Save(path))`:
  — `.mid` extension → write via `midly`
  — `.mudl` extension → write KDL
  — emit `MidiUpdate::SaveAck(path)`

#### 8.3.11 Update `format_update`

- [ ] Add arms for all new `EngineUpdate` variants added in 8.3.4
- [ ] Add unit tests for each new arm

#### 8.3.12.a — Add mush-midi-codec crate ✅

- [x] Create crates/mush-midi-codec/ with workspace registration
- [x] error.rs — ParseError, FormatError via thiserror
- [x] status.rs — StatusKind enum, symbol table, wire encoding
- [x] note.rs — scientific pitch notation ↔ MIDI note number (C-1…G9)
- [x] velocity.rs — dynamic marking symbols ↔ MIDI velocity (pppp…ffff, pf)
- [x] parse.rs — &str → Vec<u8>, numeric + symbolic + mixed + SysEx
- [x] format.rs — Vec<u8> → String, Style::Long / Style::Short
- [x] lib.rs — public API, doc examples
- [x] 87 unit tests + 2 doc tests, all passing
- [x] Round-trip guarantee: parse(format(bytes, Long)?) == Ok(bytes)
- [x] No engine, ZMQ, audio, or shell dependencies


#### 8.3.12b — Wire MVP command surface

mish-shell

    Rename Devices → Ports (sends MidiDevices)
    Add Port [<name>] — get/set active MIDI output port (MidiPort)
    Add Send <msg> — parse via mush-midi-codec, send SendMessage
    Wire Save <path> — replace stub with SaveMidi(path)
    Wire Record — replace stub with Record { start: true, path: None }
    Wire Monitor [<path>] — replace stub with Monitor { path }

mash-shell

    Add Hosts — list audio hosts (AudioHosts)
    Add Host [<name>] — get/set active audio host (AudioHost)
    Add Device [<name>] — get/set active audio device (AudioDevice)
    Wire Save <path> — replace stub
    Wire Record — replace stub
    Wire Monitor — replace stub

mush-shell

    Rename midi devices → midi ports (MidiDevices)
    Add midi port [<name>] — get/set active MIDI output port
    Add midi send <msg> — parse via mush-midi-codec, send SendMessage
    Wire midi save <path> — replace stub
    Wire midi record — replace stub
    Wire midi monitor [<path>] — replace stub
    Add audio hosts — list audio hosts (AudioHosts)
    Add audio host [<name>] — get/set active audio host
    Add audio device [<name>] — get/set active audio device
    Wire audio save <path> — replace stub
    Wire audio record — replace stub
    Wire audio monitor — replace stub

nu_plugin_midi

    Verify Monitor is not a stub
    Verify Record is not a stub
    Add midi load <path> — LoadMidi(path)
    Add midi save <path> — SaveMidi(path)
    Add midi ports — MidiDevices
    Add midi port [<name>] — MidiPort(None/Some)
    Add midi send <msg> — mush-midi-codec parse → SendMessage
    Wire pipeline input on midi play — QueueEvent per row

nu_plugin_live_audio

    Create lib.rs, move AudioPlugin struct out of main.rs
    Verify Play, Stop, Record, Devices, DevicesRefresh are not stubs
    Add audio hosts — AudioHosts
    Add audio host [<name>] — AudioHost(None/Some)
    Add audio device [<name>] — AudioDevice(None/Some)

#### 8.3.12 Confirm MVP command surface end-to-end

- [ ] `midi ports` — returns structured port list
- [ ] `midi port` — returns active port name
- [ ] `midi port <name>` — sets active port
- [ ] `midi load song.mid` — loads SMF file
- [ ] `midi load song.mudl` — loads MUDL file
- [ ] `midi play` — audible output on connected synth
- [ ] `midi stop` — stops playback cleanly
- [ ] `open song.mudl | midi play` — pipeline play
- [ ] `open song.mudl | where note > 60 | midi play` — filtered pipeline
- [ ] `midi send <msg>` — raw message delivered to port
- [ ] `midi monitor` — streams live events, autosaves pre-roll
- [ ] `midi record` — on-demand bounded capture
- [ ] `midi save song.mudl` — saves timeline
- [ ] `midi save song.mid` — saves as SMF
- [ ] `audio hosts` — returns host list
- [ ] `audio host` — returns active host
- [ ] `audio host <name>` — sets active host
- [ ] `audio devices` — returns device list
- [ ] `audio device` — returns active device
- [ ] `audio device <name>` — sets active device
- [ ] All commands functional in `mish-shell`, `mash-shell`,
  `mush-shell`, `nu_plugin_midi`, `nu_plugin_live_audio`

----

## 9. Integration and Cleanup

### 9.1 Cleanup

- [ ] 9.1.1  Assess `display.rs` in `mish-midi-core` — move or keep
- [ ] 9.1.2  Assess `display.rs` in `mash-audio-core` — move or keep
- [ ] 9.1.3  Assess `tasks.rs` in `mish-midi-core` — delete or keep
- [ ] 9.1.4  Assess `tasks.rs` in `mash-audio-core` — delete or keep
- [ ] 9.1.5  Delete `mush-commands` crate
   — superseded by `TransportCommand` in `mush-core`
   — remove from workspace `Cargo.toml`
   — confirm no remaining dependents before deletion
- [ ] 9.1.6  Delete all `evaluator_legacy.rs` files
- [ ] 9.1.7  Remove deprecated `MidiEngine::devices_refresh()`
   — last callers removed in task 7.3
- [ ] 9.1.8  Remove deprecated `MidiEngine::invalidate_devices_cache()`
   — last callers removed in task 7.3
- [ ] 9.1.9  Confirm `commands/ports_refresh.rs` deleted (8.1.5)
- [ ] 9.1.10 Confirm `commands/devices_refresh.rs` deleted (8.2.3)

### 9.2 CI

- [ ] 9.2.1 Add `libzmq3-dev` install step — Linux
  — must be first step, cargo build fails silently without it
- [ ] 9.2.2 Add `zeromq` install step — macOS (`brew install zeromq`)
- [ ] 9.2.3 Add `cargo nextest` install step
- [ ] 9.2.4 Add `cargo nextest run --workspace` step
- [ ] 9.2.5 Add `cargo clippy --workspace -- -D warnings` step
- [ ] 9.2.6 Add `cargo fmt --check` step
- [ ] 9.2.7 Add ZMQ inproc roundtrip test (deferred from 5.3.3–5.3.5)
- [ ] 9.2.8 Add ZMQ TCP roundtrip test
- [ ] 9.2.9 Add MUDL file format validation test

### 9.3 Definition of Done

- [ ] 9.3.1  `cargo nextest run --workspace` — all pass
- [ ] 9.3.2  `cargo clippy --workspace -- -D warnings` — clean
- [ ] 9.3.3  `cargo fmt --check` — clean
- [ ] 9.3.4  `mish-shell` REPL — all MIDI commands functional
- [ ] 9.3.5  `mish-shell` — audible MIDI output on connected synth
- [ ] 9.3.6  `mash-shell` REPL — all audio device commands functional
- [ ] 9.3.7  `mush-shell` REPL — full MIDI + audio command set functional
- [ ] 9.3.8  `nu_plugin_midi` — all commands functional in Nushell
- [ ] 9.3.9  `nu_plugin_live_audio` — all commands functional in Nushell
- [ ] 9.3.10 `midi monitor` — streams and cancels cleanly
- [ ] 9.3.11 `open song.mudl | midi play` — audible output on connected synth
- [ ] 9.3.12 `open song.mudl | where note > 60 | midi play` — filtered pipeline works
- [ ] 9.3.13 ZMQ inproc — command + update roundtrip in same process
- [ ] 9.3.14 ZMQ TCP — shell in separate process sends command, receives update
- [ ] 9.3.15 All deprecated methods removed
- [ ] 9.3.16 All legacy evaluator files deleted
- [ ] 9.3.17 All `display.rs` files assessed and resolved
