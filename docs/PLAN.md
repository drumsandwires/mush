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

## 6. Conductor — `mush-core`

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

### 6.2 Conductor

- [ ] 6.2.1  Create `mush-core/src/conductor.rs`
- [ ] 6.2.2  Define `Conductor` struct
- [ ] 6.2.3  Implement `Conductor::new()`
- [ ] 6.2.4  Implement `Conductor::run()`
- [ ] 6.2.5  Implement `tokio::select!` loop
- [ ] 6.2.6  Implement `handle_command()` — `Play`, `Stop`, `SetBpm`, `Seek`
- [ ] 6.2.7  Implement `handle_command()` — `LoadMidi`
- [ ] 6.2.8  Implement `handle_command()` — `LoadMudl` (reserved — NotImplemented)
- [ ] 6.2.9  Implement `handle_command()` — `QueueEvent`
- [ ] 6.2.10 Implement `handle_command()` — `AudioDevices`
- [ ] 6.2.11 Implement `handle_command()` — `MidiDevices`
- [ ] 6.2.12 Implement `emit_all()` — fan-out to ZMQ PUB + all Subscriptions
- [ ] 6.2.13 Implement subscription cleanup — `retain()` on dead handles
- [ ] 6.2.14 Switch clock status_tx to `tokio::sync::mpsc`
- [ ] 6.2.15 Wire `conductor` module into `lib.rs`

### 6.3 Conductor Tests

- [ ] 6.3.1  Test `Play` command sets `is_playing`
- [ ] 6.3.2  Test `Stop` command clears `is_playing`
- [ ] 6.3.3  Test `SetBpm` updates transport
- [ ] 6.3.4  Test update reaches ZMQ subscriber
- [ ] 6.3.5  Test update reaches `SubscriptionHandle`
- [ ] 6.3.6  Test closed subscription is removed cleanly
- [ ] 6.3.7  Test `LoadMidi` populates timeline
- [ ] 6.3.8  Test `AudioDevices` emits device list
- [ ] 6.3.9  Test `MidiDevices` emits port list
- [ ] 6.3.10 Test multiple subscriptions all receive update

---

## 7. Shell Migration

### 7.1 Formatting

- [ ] 7.1.1  Create `mish-shell/src/format.rs`
- [ ] 7.1.2  Implement `format_update()` — `TransportStarted`
- [ ] 7.1.3  Implement `format_update()` — `TransportStopped`
- [ ] 7.1.4  Implement `format_update()` — `PlayheadMoved` (suppress)
- [ ] 7.1.5  Implement `format_update()` — `DeviceList`
- [ ] 7.1.6  Implement `format_update()` — `PortList`
- [ ] 7.1.7  Implement `format_update()` — errors
- [ ] 7.1.8  Implement `format_update()` — `LoadMudl` acknowledgement
- [ ] 7.1.9  Unit tests for all format functions
- [ ] 7.1.10 Create `mash-shell/src/format.rs` — same pattern
- [ ] 7.1.11 Create `mush-shell/src/format.rs` — combined MIDI + audio

### 7.2 Sink

- [ ] 7.2.1 Create `mish-shell/src/sink.rs`
- [ ] 7.2.2 Define `ReedlineSink` struct
- [ ] 7.2.3 Implement `EngineSink` for `ReedlineSink`
  — `emit()` must be non-blocking
  — use `external_print` not blocking I/O
- [ ] 7.2.4 Repeat for `mash-shell` and `mush-shell`

### 7.3 `mish-shell` Migration

- [ ] 7.3.1 Rename `evaluator.rs` → `evaluator_legacy.rs`
- [ ] 7.3.2 Create `repl.rs` — parse input, send `TransportCommand`
- [ ] 7.3.3 Wire `ZmqClient` into `repl.rs`
  — `connect(addrs::CMD_INPROC, addrs::PUB_INPROC)` for MVP
- [ ] 7.3.4 Rewrite `main.rs` — wire all components
- [ ] 7.3.5 Confirm existing `trycmd` tests pass
- [ ] 7.3.6 Update snapshots if formatting changed
- [ ] 7.3.7 Delete `evaluator_legacy.rs`

### 7.4 `mash-shell` Migration

- [ ] 7.4.1 Follow same pattern as 7.3
- [ ] 7.4.2 Wire `ZmqClient` — inproc transport
- [ ] 7.4.3 Confirm existing tests pass

### 7.5 `mush-shell` Migration

- [ ] 7.5.1 Follow same pattern as 7.3
- [ ] 7.5.2 Full MIDI + audio command set — superset of `mish` and `mash`
- [ ] 7.5.3 Wire `ZmqClient` — inproc transport
- [ ] 7.5.4 Developed after 7.3 and 7.4 are stable
- [ ] 7.5.5 Confirm existing tests pass

---

## 8. Plugin Migration

### 8.1 `nu_plugin_midi` Migration

- [ ] 8.1.1  Wire `ZmqClient` into `main.rs`
   — `connect(addrs::CMD_TCP, addrs::PUB_TCP)`
- [ ] 8.1.2  Rewrite `commands/play.rs`
   — `PipelineData::Empty` → `TransportCommand::Play`
   — `PipelineData::ListStream` → `QueueEvent` per record + `Play`
   — handles: `midi play``open song.mudl | midi play`
              `open song.mudl | where note > 60 | midi play`
- [ ] 8.1.3  Rewrite `commands/stop.rs`
   — `TransportCommand::Stop` via `ZmqClient`
- [ ] 8.1.4  Rewrite `commands/ports.rs`
   — `TransportCommand::MidiDevices` via `ZmqClient`
   — always fresh — no refresh command
- [ ] 8.1.5  Remove `commands/ports_refresh.rs`
   — deprecated, always-fresh enumeration replaces cache
- [ ] 8.1.6  Stub `commands/record.rs`
- [ ] 8.1.7  Add `commands/monitor.rs` — streaming via ZMQ SUB
   — `recv_update()` loop → `engine_update_to_value()` → `ListStream`
- [ ] 8.1.8  Implement `engine_update_to_value()`
- [ ] 8.1.9  Add `commands/audio_devices.rs`
   — `TransportCommand::AudioDevices` via `ZmqClient`
- [ ] 8.1.10 Add `commands/load_mudl.rs`
   — `TransportCommand::LoadMudl(path)` — reserved
- [ ] 8.1.11 Confirm existing plugin tests pass

### 8.2 `nu_plugin_live_audio` Migration

- [ ] 8.2.1 Wire `ZmqClient` into `main.rs`
  — `connect(addrs::CMD_TCP, addrs::PUB_TCP)`
- [ ] 8.2.2 Rewrite `commands/devices.rs`
  — `TransportCommand::AudioDevices` via `ZmqClient`
  — always fresh — no refresh command
- [ ] 8.2.3 Remove `commands/devices_refresh.rs`
  — deprecated, always-fresh enumeration replaces cache
- [ ] 8.2.4 Stub `commands/play.rs`
- [ ] 8.2.5 Stub `commands/stop.rs`
- [ ] 8.2.6 Stub `commands/record.rs`
- [ ] 8.2.7 Confirm existing plugin tests pass

---

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
