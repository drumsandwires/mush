# Slice 0a — Codebase Audit

## Purpose

Establish a complete, accurate picture of the existing implementation
before any architectural changes are made. The output of this audit
drives the migration map used in Phase 2 and confirms or revises the
4-week timeline estimate.

**Time budget:** 2–4 hours.
**Prerequisite:** All existing tests pass on the current branch.

---

## Before You Start

Create an `audit/` directory at the workspace root to collect findings.

```bash
mkdir -p audit
```

Run the full test suite and record the baseline result.

```bash
cargo test --workspace 2>&1 | tee audit/baseline-test-output.txt
cargo clippy --workspace 2>&1 | tee audit/baseline-clippy-output.txt
```

All outputs from this document go into `audit/`.

---

## 1. Dependency Graph

**Goal:** Understand which crates depend on which and identify any
unexpected coupling.

### Task

```bash
cargo tree --workspace 2>&1 | tee audit/cargo-tree.txt
cargo tree --workspace --depth 1 2>&1 | tee audit/deps-depth-1.txt
```

### Record

For each crate, note its direct dependencies on other workspace crates:

| Crate | Depends on (workspace crates) |
|---|---|
| `mush-core` | None |
| `mush-commands` | None |
| `mish-midi-core` | None |
| `mash-audio-core` | None |
| `mish-shell` | mish-midi-core, mush-core, mush-commands |
| `mash-shell` | mash-audio-core, mush-core, mush-commands |
| `mush-shell` | mish-midi-core, mash-audi-core, mush-core, mush-commands|
| `nu_plugin_midi` | mish-midi-core |
| `nu_plugin_live_audio` | mash-audio-core |
| `tests` | mash-shell, mish-shell, mush-shell, nu_plugin_midi |

### Questions to Answer

- Does any shell depend directly on both `mish-midi-core` and
  `mash-audio-core`?
  - Yes, mash-shell, intentionally so. It's a combined audio/midi shell.
- Does `mush-commands` depend on the engine crates, or only on
  `mush-core`?
  - No dependency, only depended-on.
- Do the plugins depend on the engine crates directly, or via
  `mush-commands`?
  - Directly.
- Is `zeromq` already a dependency anywhere?
  - No.
- Is `rmp-serde` already a dependency anywhere?
  - No.
- Is `tokio` already a dependency, and if so with which feature flags?
  - No.
- Is `cpal` already a dependency of `mish-midi-core`?
  - No.
- Is `crossbeam` already a dependency anywhere?
  - No.

---

## 2. `mush-core`

**Goal:** Understand what shared infrastructure already exists and
identify anything that would conflict with the planned additions.

### Task

Read `crates/mush-core/src/lib.rs` and all modules it declares.

### Record

| Item | Present? | Notes |
|---|---|---|
| Shared error type | No | |
| Shared result type | No | |
| Trait definitions | Yes | Shell |
| Shared config struct | No | |
| Logging / tracing setup | No | |
| Any async runtime setup | No | |
| Other | | |

### Questions to Answer

- Is there anything that overlaps with the planned `TransportState`,
  `Shell` trait, or `EngineUpdate` types?
  - Yes, the `Shell` trait exists.
- Is there anything that would conflict with adding those types?
  - Yes, the `Shell` trait exists.
- What is the current public API surface — list all public items.
  - pub fn run<S: Shell>(shell: S) 

---

## 3. `mush-commands`

**Goal:** Understand the role of this crate and decide whether to merge
it into `mush-core` or keep it separate.

### Task

Read `crates/mush-commands/src/lib.rs` and all modules it declares.

### Record

| Item | Present? | Notes |
|---|---|---|
| Command definitions | Yes | Enum only |
| Shared command parsing logic | No | |
| Shared command dispatch logic | No | |
| Display / formatting logic | No | |
| Direct engine crate dependencies | No | |
| Shell-specific logic | No | |

### Questions to Answer

- Does `mush-commands` contain logic that belongs in the shell
  evaluators, or is it genuinely shared across all shells?
  - No
- Does it contain display logic that should move to `format.rs`?
  - No 
- Does it depend on `mish-midi-core` or `mash-audio-core` directly?
  - No

### Decision

Record the decision and rationale:

| Option | Condition | Action |
|---|---|---|
| Merge into `mush-core` | Contains only shared types, no display logic, no engine deps | Merge in Slice 1a |
| Keep separate | Contains display logic or shell-specific concerns | Keep, extend in place |

**Decision:** Merge in Slice 1a.

**Rationale:** Minimise interanl workspace dependencies.

---

## 4. `mish-midi-core`

**Goal:** Understand what MIDI engine functionality already exists and
what needs to be added or restructured.

### Task

Read all files under `crates/mish-midi-core/src/midi/`.

---

### 4.1 `types.rs`

| Item | Present? | Notes |
|---|---|---|
| MIDI event struct | No | |
| Port / device struct | Yes | MidiDeviceInfo |
| Error types | Yes | Uses thiserror |
| `serde` derives | No | |
| `sample_pos` field on any event type | No | |
| Note number → name helper | No | |

---

### 4.2 `engine.rs`

| Item | Present? | Notes |
|---|---|---|
| Playback function | Placeholder | Record signature |
| Stop function | Placeholder | Record signature |
| Direct `midir` calls | No | |
| Thread spawning | No | |
| `crossbeam::ArrayQueue` usage | No | |
| Blocking calls on the hot path | No | |
| RT priority elevation | No | |
| Dispatch loop | No | |

**Playback function signature:**

```
fn play(&self) -> MidiResponse
```

**Stop function signature:**

```
fn stop(&self) -> MidiResponse
```

---

### 4.3 `enumerate.rs`

| Item | Present? | Notes |
|---|---|---|
| Port listing function | Yes |  |
| Return type | Yes | |
| `serde` derives on return type | No | |
| Default port identification | No | |

**List function signature:**

```
// fn list_ports(...) -> ...
pub fn enumerate_devices() -> Result<Vec<MidiPortInfo>, MidiError>
```

---

### 4.4 `tasks.rs`

The tasks.rs file holds deprecated code.

| Item | Present? | Notes |
|---|---|---|
| Async tasks | | |
| `tokio` usage | | |
| Channel usage | | |
| State management | | |
| Overlap with planned `EngineMonitor` | | Describe |

**Assessment:** Does `tasks.rs` implement async coordination that would
be superseded by `EngineMonitor`?

```
Answer:
```

**Decision:** Mark for deletion in Slice 2h 

---

### 4.5 `display.rs`

| Item | Present? | Notes |
|---|---|---|
| Formatting functions | Yes | list_enumerated_devices |
| Direct terminal output calls | No | |
| Coupling to engine types | Yes | |
| Domain helpers (e.g. note → name) | No | |

**Decision:** Move to shell `format.rs`

---

### 4.6 `mish-midi-core` Summary

| Planned component | Exists? | Action |
|---|---|---|
| `TimelineEvent` struct | No | |
| `build_timeline()` | No | |
| `load_midi_bytes()` | No | |
| `MidiEngine` RT thread | No | |
| `DesktopMidiOutput` | No | |
| `Clock` (cpal silence) | No | |
| `MidiPortInfo` with serde | No| |

---

## 5. `mash-audio-core`

**Goal:** Understand what audio engine functionality already exists.

### Task

Read all files under `crates/mash-audio-core/src/audio/`.

---

### 5.1 `types.rs`

| Item | Present? | Notes |
|---|---|---|
| Device info struct | Yes | AudioDeviceInfo |
| Host info struct | Yes | AudioHostInfo |
| `serde` derives | No | |
| `is_default` field | No | |
| `sample_rates` field | No | |
| `channels` field | No | |
| `index` field | No | |

---

### 5.2 `engine.rs`

| Item | Present? | Notes |
|---|---|---|
| `cpal` stream creation | No | |
| Audio callback | No | |
| Silence output (`out.fill(0.0)`) | No | |
| Audio mixing | No | |
| Playhead advancement | No | |
| MIDI dispatch in callback | No | |
| RT priority elevation | No | |

**Assessment:** Is `cpal` already opening an output stream, or only
enumerating devices?

```
Answer: Only enumerating.
```

---

### 5.3 `enumerate.rs`

| Item | Present? | Notes |
|---|---|---|
| Device listing function | Yes  | Record signature |
| Return type | Yes | Record type name |
| Default device identification | Yes | |
| Sample rate enumeration | Yes | |
| `serde` derives on return type | No | |

**List function signature:**

```
// fn list_devices(...) -> ...
pub fn enumerate_hosts_and_devices() -> Result<AudioSystemInfo, AudioDeviceError> 
```

**Gap analysis — fields missing from existing return type vs planned
`AudioHostInfo` / `AudioDeviceInfo`:**

| Field | Present? |
|---|---|
| `index: usize` | No |
| `name: String` | Yes |
| `channels: u16` | Yes |
| `sample_rates: Vec<u32>` | Yes, as AudioSupportedRates |
| `is_default: bool` | Yes |
| `host_name: String` | Yes |
| `default_device_name: Option<String>` | No |

---

### 5.4 `tasks.rs`

The tasks.rs file holds deprecated code.

| Item | Present? | Notes |
|---|---|---|
| Async tasks | | |
| `tokio` usage | | |
| Overlap with planned `EngineMonitor` | | Describe |

**Decision:** Mark for deletion in Slice 2h

---

### 5.5 `display.rs`

| Item | Present? | Notes |
|---|---|---|
| Formatting functions | Yes | list_hosts_and_devices |
| Direct terminal output calls | No | |
| Coupling to engine types | Yes | |

**Decision:** Move to shell `format.rs`

---

### 5.6 `mash-audio-core` Summary

| Planned component | Exists? | Action |
|---|---|---|
| `AudioDeviceInfo` with serde | No  | Add serde|
| `AudioHostInfo` with serde | No | Add serde |
| `list_devices() -> AudioHostInfo` | No | |
| `AudioClip` + `mix_into()` | No | Phase 3 — confirm absent |
| `AudioLoader` (symphonia) | No | Phase 3 — confirm absent |

---

## 6. Shell Evaluators

**Goal:** Produce the migration map — a complete list of existing
commands and their planned `TransportCommand` equivalents.

### Task

Read `evaluator.rs` in each shell crate.

---

### 6.1 `mish-shell` Command Inventory

| Command string | Engine call | `TransportCommand` equivalent |
|---|---|---|
| version |  engine.version() | | 
| devices | engine.devices() | |
| load | engine.load(&path) | |
| save { path } | engine.save(&path) | |
| play | engine.play() | |
| record | engine.record() | |
| monitor | engine.monitor() | |
| stop | engine.stop() | |


---

### 6.2 `mash-shell` Command Inventory

| Command string | Engine call | `TransportCommand` equivalent |
|---|---|---|
| | | |

---

### 6.3 `mush-shell` Command Inventory

| Command string | Engine call | `TransportCommand` equivalent |
|---|---|---|
| | | |

---

### 6.4 Commands With No `TransportCommand` Equivalent

List any commands that do not map to a planned `TransportCommand`
variant. Each needs a decision.

| Shell | Command | Decision |
|---|---|---|
| | | Stub / Defer / Implement |

---

### 6.5 Display Call Inventory

For each shell evaluator, list every place it calls a display or
formatting function:

| Shell | Call site | Output produced | Move to `format.rs`? |
|---|---|---|---|
| | | | |

---

### 6.6 Shared Command Logic in `mush-commands`

Does `mush-commands` provide any command parsing or dispatch that the
evaluators call?

| Function | Called by | Purpose |
|---|---|---|
| | | |

---

### 6.7 Input Parsing

How is user input parsed in the evaluators?

- [ ] Raw string matching (`match line { "play" => ... }`)
- [ ] Split on whitespace
- [ ] Parser combinator (e.g. `nom`, `pest`)
- [ ] Other — describe:

---

## 7. Nushell Plugins

**Goal:** Understand the existing plugin command structure and identify
what changes during migration.

### Task

Read all files under `crates/nu_plugin_midi/src/commands/` and
`crates/nu_plugin_live_audio/src/commands/`.

---

### 7.1 `nu_plugin_midi` Command Inventory

| File | Command name | Engine call | Return type | Streaming? |
|---|---|---|---|---|
| `play.rs` | | | | |
| `stop.rs` | | | | |
| `ports.rs` | | | | |
| `ports_refresh.rs` | | | | |
| `record.rs` | | | | |

---

### 7.2 `nu_plugin_live_audio` Command Inventory

| File | Command name | Engine call | Return type | Streaming? |
|---|---|---|---|---|
| `play.rs` | | | | |
| `stop.rs` | | | | |
| `devices.rs` | | | | |
| `devices_refresh.rs` | | | | |
| `record.rs` | | | | |

---

### 7.3 Plugin State Management

How is plugin state currently managed?

- [ ] Shared struct passed to all commands
- [ ] Stateless — engine calls are pure functions
- [ ] Global / thread-local state
- [ ] Other — describe:

---

### 7.4 `record` Command Assessment

For each plugin's `record` command:

**`nu_plugin_midi` record:**

- What does it currently do?
- Does it have a planned `TransportCommand` equivalent?
- **Decision:** Stub / Defer to Phase 3 / Implement now

**`nu_plugin_live_audio` record:**

- What does it currently do?
- Does it have a planned `TransportCommand` equivalent?
- **Decision:** Stub / Defer to Phase 3 / Implement now

---

### 7.5 Streaming Commands

Do any existing plugin commands return `PipelineData::ListStream`?

| Plugin | Command | Streaming? | Notes |
|---|---|---|---|
| | | | |

---

## 8. Test Coverage

**Goal:** Understand what is tested, how, and what will break during
migration.

### Task

Read `tests/tests/shell.rs`, `tests/tests/plugin.rs`, and all files
under `tests/_src/`.

---

### 8.1 Shell Test Inventory

| Test file | Framework | Commands tested | Snapshot matched exactly? |
|---|---|---|---|
| `mish_tests.rs` | | | |
| `mash_tests.rs` | | | |
| `mush_tests.rs` | | | |
| `repl_test.rs` | | | |

---

### 8.2 Plugin Test Inventory

| Test file | Framework | Commands tested |
|---|---|---|
| `nu_plugin_midi_tests.rs` | | |

---

### 8.3 `trycmd` Snapshot Files

| File | Shell | Commands covered | Output matched exactly? |
|---|---|---|---|
| `mish-banner.toml` | mish | | |
| `mash-banner.toml` | mash | | |
| `mush-banner.toml` | mush | | |

---

### 8.4 Test Coverage Gaps

List any commands or behaviours that are not currently tested:

| Gap | Risk during migration | Recommended action |
|---|---|---|
| | | |

---

### 8.5 Tests That Will Break During Migration

List tests that are likely to break when the architecture changes, and
why:

| Test | Why it will break | Mitigation |
|---|---|---|
| | | |

---

## 9. External Dependencies

**Goal:** Identify any dependencies that affect the migration plan,
particularly version conflicts and missing crates.

### Record

| Dependency | Used by | Version | Relevant to migration? | Notes |
|---|---|---|---|---|
| `cpal` | | | | Already present? |
| `midir` | | | | Already present? |
| `midly` | | | | Already present? |
| `zeromq` | | | | Already present? |
| `rmp-serde` | | | | Already present? |
| `tokio` | | | | Feature flags? |
| `crossbeam` | | | | Already present? |
| `reedline` | | | | Version? |
| `nu-plugin` | | | | Version? |
| `nu-protocol` | | | | Version? |
| `serde` | | | | Feature flags? |
| `trycmd` | | | | Version? |
| `assert_cmd` | | | | Version? |

### Questions to Answer

- Is `zeromq` already a dependency? If not, has a ZMQ spike been done?
- Are there version conflicts between existing dependencies and the
  planned new ones?
- Is `tokio` already used with `features = ["full"]`?
- Are `nu-plugin` and `nu-protocol` on a version that supports
  `PipelineData::ListStream` and `SimplePluginCommand`?

---

## 10. Findings Summary

Complete this section after working through sections 1–9.

---

### 10.1 What Exists That Can Be Reused Directly

| Existing item | Maps to planned component | Effort |
|---|---|---|
| | | Trivial / Minor / Moderate |

---

### 10.2 What Exists That Needs Restructuring

| Existing item | Issue | Planned action |
|---|---|---|
| | | |

---

### 10.3 What Is Missing Entirely

| Planned component | Slice | Notes |
|---|---|---|
| `TransportState` | 1a | |
| `TransportCommand` | 1a | |
| `EngineUpdate` | 1a | |
| `Shell` trait | 1a | |
| `MidiOutput` trait | 1a | |
| `rt_priority` shims | 1a | |
| `ZmqTransport` | 1d | |
| `ZmqClient` | 1d | |
| `EngineMonitor` | 1e | |
| `PluginChannel` | 1e | |
| `SubscriptionHandle` | 1e | |
| `build_timeline()` | 1b | |
| `TimelineEvent` | 1b | |
| `Clock` (cpal silence) | 1b | |
| `MidiEngine` RT thread | 1b | |
| `AudioHostInfo` (serde) | 1c | |
| `AudioDeviceInfo` (serde) | 1c | |
| `format.rs` (shells) | 2b | |
| `ReedlineSink` | 2b | |

---

### 10.4 Decisions Required Before Phase 1

| Decision | Options | Recommendation | Decided |
|---|---|---|---|
| `mush-commands` — merge or keep? | Merge / Keep | | |
| `record` commands — stub or defer? | Stub / Defer | | |
| `display.rs` in core crates — move or keep? | Move / Keep | | |
| `tasks.rs` — delete or keep? | Delete / Keep | | |
| ZMQ crate — `zeromq` or `zmq` (libzmq)? | Pure Rust / libzmq | | |

---

### 10.5 Migration Map

Consolidated from section 6. One row per existing shell command.

| Shell | Command | Engine call | `TransportCommand` | Action |
|---|---|---|---|---|
| mish | | | | Rewrite / Stub / Defer |
| mash | | | | Rewrite / Stub / Defer |
| mush | | | | Rewrite / Stub / Defer |

---

### 10.6 Timeline Impact

Based on the audit findings, assess whether the 4-week estimate holds.

| Finding | Impact |
|---|---|
| | Adds / Saves / Neutral |

**Original estimate:** 48h optimistic, 99h realistic.

**Revised estimate:** _____ hours optimistic, _____ hours realistic.

**Hours per week required for 4 weeks:** _____

**Confidence:** Low / Medium / High

**Recommendation:** Proceed on schedule / Extend timeline / Reduce scope

---

## 11. Outputs

At the end of Slice 0a the following must exist:

```
audit/
    baseline-test-output.txt      — cargo test before any changes
    baseline-clippy-output.txt    — cargo clippy before any changes
    cargo-tree.txt                — full workspace dependency tree
    deps-depth-1.txt              — top-level deps per crate
    slice-0a-findings.md          — completed version of this document
```

`slice-0a-findings.md` is the primary input to:

- Slice 0c — `mush-commands` decision
- Slice 1a — `mush-core` extension plan
- Slice 2a — migration map
- Revised timeline and scope decisions
