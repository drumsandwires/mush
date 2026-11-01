# Implementation Log

commit 0878f523f273eb73ca6349a26787c80cffcf0860
Merge: 0e2a695 2cc3dd8
Author: arthurtd <atetzlaffdeas@gmail.com>
Date:   Sat Oct 31 14:56:18 2026 +0000

merge: feat/add_conductor to develop (#31)

## What

Implements `Conductor` — the central coordinator for the mush engine —
along with the `Subscription`/`SubscriptionHandle` pair that supports
in-process update fan-out.

### New files
- `mush-core/src/conductor.rs` — `Conductor` struct and `select!` loop
- `mush-core/src/subscription.rs` — `Subscription` /
`SubscriptionHandle`
  pair built on `tokio::sync::mpsc`

### Modified files
- `mush-core/src/transport.rs` — `MidiEvent` struct, `QueueEvent` and
  `LoadMudl` variants added to `TransportCommand`
- `mush-core/src/lib.rs` — `conductor` and `subscription` modules wired
in
- `mish-midi-core/src/midi/clock.rs` — `status_tx` migrated from
  `std::sync::mpsc` to `tokio::sync::mpsc`
- `mish-midi-core/src/midi/engine.rs` — `status_tx` migrated from
  `std::sync::mpsc` to `tokio::sync::mpsc`

## Why

### `Conductor`

The engine subsystems — `Clock`, `MidiEngine`, `MidiEngineRt` — were
built in tasks 2–4 but had no coordinator. Commands from shells had
nowhere to go. Updates from the engine had no fan-out path.

`Conductor` closes that gap. It owns the ZMQ transport, receives status
updates from the clock and MIDI engine via `tokio::sync::mpsc`, and
fans out `EngineUpdate` to all registered subscribers on every
iteration of its `tokio::select!` loop.

### `tokio::select!` loop

Three arms race simultaneously:

```
cmd  = transport.recv_command()  → handle_command()
upd  = audio_rx.recv()          → emit_all(EngineUpdate::Audio)
upd  = midi_rx.recv()           → emit_all(EngineUpdate::Midi)
```

No polling, no busy-waiting. The tokio runtime suspends the task
until one arm fires. This is why `tokio::sync::mpsc` was chosen over
`std::sync::mpsc` — receivers integrate directly as futures in
`select!`.

### `Subscription` / `SubscriptionHandle`

ZMQ PUB handles out-of-process subscribers. In-process subscribers
(shells embedding the conductor at MVP) need something lighter.

`Subscription`/`SubscriptionHandle` is a thin `tokio::sync::mpsc`
wrapper. The conductor holds `Vec<Subscription>` and calls
`try_send()` on each. The shell holds `SubscriptionHandle` and calls
`recv().await`. When a handle is dropped, `is_closed()` returns `true`
and `retain()` removes the dead subscription silently on the next
`emit_all()` call.

### `tokio::sync::mpsc` migration

`Clock` and `MidiEngineRt` previously used `std::sync::mpsc::Sender`.
These were migrated to `tokio::sync::mpsc::Sender` so the conductor
can select on the receivers directly.

The RT constraint is preserved throughout — the cpal audio callback
cannot block and cannot `.await`. All call sites use `try_send()`
which returns immediately whether or not the channel is full. A full
channel drops the update — acceptable for a periodic playhead report
at 100ms intervals.

### `TransportCommand` extensions

Three additions reserved to ensure the architecture supports the
primary Nushell pipeline use case:

```nushell
open song.mudl | where note > 60 | midi play
```

- `MidiEvent` — structured representation of one pipeline record
- `QueueEvent(MidiEvent)` — streams a single event from `midi play`
  into the engine
- `LoadMudl(PathBuf)` — loads a `.mudl` file and builds the timeline

`LoadMudl` returns a `NotImplemented` error at runtime — parsing is
implemented in task 8. `QueueEvent` is stubbed — timeline append is
implemented in task 7. Both variants are present now so no
architectural changes are needed later.

## MVP Deployment Model

Each binary embeds its own `Conductor` at MVP:

```
mish-shell binary
  ├── Conductor (embedded)
  ├── ZmqTransport  inproc://mush-cmd · inproc://mush-pub
  └── ZmqClient     inproc://mush-cmd · inproc://mush-pub

nu_plugin_midi binary
  ├── Conductor (embedded)
  ├── ZmqTransport  inproc://mush-cmd · inproc://mush-pub
  └── ZmqClient     inproc://mush-cmd · inproc://mush-pub
```

Post-MVP migration to a standalone conductor daemon requires only:
- Extract `Conductor` into a `mush-conductor` binary crate
- Change client address strings from `inproc://` to `tcp://`
- No other code changes

## Stubbed command handlers

Several `handle_command()` arms are intentional stubs:

| Command | Stub reason |
|---|---|
| `SetBpm` / `Seek` | `TransportState` not yet wired into conductor |
| `LoadMidi` | Timeline parsing deferred to task 7 |
| `LoadMudl` | MUDL parsing deferred to task 8 |
| `QueueEvent` | Timeline append deferred to task 7 |
| `AudioDevices` | Enumeration call deferred to task 7 |
| `MidiDevices` | Enumeration call deferred to task 7 |

## Testing

6 new tests — all pass:

```
Summary [2.862s] 52 tests run: 52 passed, 0 skipped
```

| Test | Covers |
|---|---|
| `play_emits_transport_started` | 6.3.1 |
| `stop_emits_transport_stopped` | 6.3.2 |
| `set_bpm_does_not_panic` | 6.3.3 |
| `update_reaches_subscription_handle` | 6.3.5 |
| `closed_subscription_removed_cleanly` | 6.3.6 |
| `multiple_subscriptions_all_receive_update` | 6.3.10 |

### Deferred tests

`update_reaches_zmq_subscriber` (6.3.4) is deferred to task 9.
ZMQ `inproc://` requires a shared context between server and client.
Two independent `Context::new()` calls cannot communicate over
`inproc://` — this test requires integration test infrastructure
with a managed shared context.

## Hmm...

1. `Conductor::new()` returns `tmq::TmqError` on ZMQ bind failure.
   Should this be wrapped in a domain-specific `ConductorError` type
   for cleaner error handling at call sites?

2. The `select!` loop breaks on `None` from any arm — a dropped
   sender shuts down the conductor. Should shutdown be more explicit,
   for example a dedicated `TransportCommand::Shutdown` variant?

3. `emit_all()` silently drops updates to full subscription channels.
   Should a full channel be logged at debug level or surfaced as an
   `EngineUpdate::Error`?

commit 354f2b9e0a775d35367c0cb21d86771c52ee1c58
Merge: b6b57ee d439036
Author: arthurtd <atetzlaffdeas@gmail.com>
Date:   Fri Oct 30 12:08:08 2026 +0000

merge: feat/add_zmq_transport into develop (#30)

## What

Adds the ZMQ transport layer connecting shell processes to the
`EngineMonitor`, with transparent support for both intra-process
and inter-process communication.

- `zmq_transport.rs` — server-side `ZmqTransport` struct binding
  ROUTER and PUB sockets
- `zmq_client.rs` — client-side `ZmqClient` struct connecting
  DEALER and SUB sockets
- `addrs.rs` — constants for TCP and inproc addresses
- Adds `PartialEq` to `TransportCommand`, `AudioUpdate`,
  `MidiUpdate`, and `EngineUpdate`
- Adds `rmp-serde`, `futures`, and `tmq` to `mush-core` dependencies
- 6 msgpack roundtrip tests covering all `TransportCommand` variants
  and key `EngineUpdate` variants

## Why

Shells (`mish-shell`, `mash-shell`) may run in the same process as
the `EngineMonitor` or as separate processes. ZMQ handles both cases
transparently — only the address string changes, the socket API and
shell code are identical in both cases:

```
Same process:      ZmqClient::connect(addrs::CMD_INPROC, addrs::PUB_INPROC)
Separate process:  ZmqClient::connect(addrs::CMD_TCP,    addrs::PUB_TCP)
```

The topology is:

```
EngineMonitor
  ROUTER  tcp://*:5252  or  inproc://mush-cmd  ← TransportCommand
  PUB     tcp://*:5253  or  inproc://mush-pub  → EngineUpdate

Shell
  DEALER  → sends TransportCommand
  SUB     ← receives EngineUpdate
```

Ports 5252/5253 are used rather than the ZMQ convention of 5555/5556
to avoid conflicts with other ZMQ applications.

PUB/SUB is used for updates rather than routing replies back through
DEALER/ROUTER because PUB/SUB delivers to all subscribers with a
single send, regardless of client count. DEALER/ROUTER alone would
require the server to maintain a routing table and loop over client
identities — reimplementing what PUB/SUB does natively.

Serialisation uses msgpack via `rmp-serde` — more compact and faster
to parse than JSON, with no schema overhead. Appropriate for a local
IPC channel where human readability is not needed. msgpack is used
even for `inproc://` at MVP — skipping serialisation for same-process
communication is a post-MVP optimisation deferred until profiling
shows it matters.

Nushell plugins use `mpsc` channels directly and do not participate
in the ZMQ transport. Plugins share process space with the
`EngineMonitor` — adding ZMQ to the plugin path would be two IPC
mechanisms doing the same job.

### ZMQ `inproc://` Constraint

For `inproc://` the server must bind before any client connects.
This is already the natural startup order — the `EngineMonitor`
binds its sockets during initialisation before any shell is started.
No special handling is needed.

## How

- **`async`/`await`** — `recv_command` and `recv_update` suspend
  without blocking the thread, allowing the `EngineMonitor` to
  participate in a `tokio::select!` loop.

- **`PartialEq` vs `Eq`** — `TransportCommand::SetBpm(f64)` contains
  an `f64`. `f64` implements `PartialEq` but not `Eq` because
  `NaN != NaN` — floating-point equality is not total. The compiler
  rejects `#[derive(Eq)]` on any type containing `f64`.

- **Type state pattern** — `tmq` 0.4 uses distinct types
  `SubscribeWithoutTopic` and `Subscribe` to enforce that a topic
  filter is set before the socket is used. The compiler prevents
  misuse at zero runtime cost. The correct construction order is
  `.connect()` then `.subscribe()`.

- **`tmq::Multipart` frame access** — `Multipart` is a newtype
  wrapper, not a plain `Vec`. Frames are accessed via
  `into_iter().collect()` then indexed by position. ROUTER frames
  are `[identity, empty delimiter, payload]` — payload is at index
  2. SUB frames are `[topic, payload]` — payload is at index 1.

- **Address parameterisation** — socket addresses are `&str`
  parameters rather than hardcoded constants. `addrs.rs` is the
  single source of truth for all address strings across the
  workspace.

## Testing

6 new msgpack roundtrip tests — all pass:

```
Summary [2.758s] 41 tests run: 41 passed, 0 skipped
```

ZMQ socket roundtrip tests (bind + connect + send + recv) for both
`tcp://` and `inproc://` transports are deferred to
integration tests — they require port management and cannot run
reliably in a unit test environment.

## Hmm...

1. Should the address selection (inproc vs TCP) be driven by a
   config file, environment variable, or a CLI flag at startup?
   Currently the caller passes the address string directly.

2. Should `addrs.rs` expose a helper that returns the correct
   address pair based on a `RunMode` enum (`InProcess` /
   `Standalone`) rather than exposing the raw string constants?

3. `recv_command` and `recv_update` silently drop messages that
   fail to deserialise by returning `None`. Should deserialisation
   failures be surfaced as `EngineUpdate::Error` events or logged
   at debug level?

## Nope

- **JSON via `serde_json`** — rejected. More verbose on the wire,
slower to parse. No benefit for a local IPC channel. Plus, I can't stand
JSON.

- **Single port DEALER/ROUTER** — rejected. Requires the server to
  maintain a client identity table and loop to send updates.
  PUB/SUB handles fan-out natively and more efficiently.

- **ZMQ for Nushell plugins** — rejected. Plugins share process
  space with the `EngineMonitor`. `mpsc` is faster, simpler, and
  already the right tool for that boundary.

- **Hardcoded TCP addresses** — rejected. Prevents same-process
  deployment. Parameterised addresses with `inproc://` support
  added at no extra complexity cost.

commit b6b57ee00338e49020d5a4891e2a58d71882d8b4
Merge: 90fed0c cdda677
Author: arthurtd <atetzlaffdeas@gmail.com>
Date:   Thu Oct 29 23:17:27 2026 +0000

Rework audio device enumeration (#29)
    
## What

Prepares `mash-audio-core` audio device types for the `EngineMonitor`
integration in task 6.

- Adds `Serialize`/`Deserialize` derives to `AudioSupportedRates`,
  `AudioDeviceInfo`, `AudioHostInfo`, and `AudioSystemInfo`
- Adds `Clone` derives to all four types
- Adds `list_devices()` to `enumerate.rs` — always-fresh enumeration,
  no cache
- Adds `serde` to `mash-audio-core/Cargo.toml`

## Why

The `EngineMonitor` (task 6) will call `list_devices()` directly in
response to `TransportCommand::AudioDevices`. The structured types
need `Serialize`/`Deserialize` for ZMQ transport (task 5) and `Clone`
because `EngineUpdate` derives `Clone` and will contain these types
in the fan-out path.

`list_devices()` always enumerates fresh — no cache. Unlike the old
shell architecture where `devices()` could be called repeatedly,
the `EngineMonitor` only enumerates on explicit user request. The
cost of re-enumerating is negligible and correctness is guaranteed —
hardware changes are never missed.

## `AudioUpdate::DeviceList`

`DeviceList` remains `DeviceList(String)` in `mush-core/src/update.rs`.
Changing it to `DeviceList(AudioSystemInfo)` would require `mush-core`
to depend on `mash-audio-core`, creating a dependency cycle:

```
mush-core → mash-audio-core → mush-core
```

The `EngineMonitor` will format `AudioSystemInfo` into a string at
the boundary before sending the update. This will be revisited if a
structured wire format is needed.

## Existing Enumeration

The  `enumerate_hosts_and_devices()` fn is already
implemented and required no changes:

- ALSA wildcard rate filtering (`1–4_294_967_295 Hz`) removes
  virtual/plugin devices with no real hardware constraint
- Deduplication by description — default-flagged entries displace
  non-default entries with the same description
- Stable sort order — defaults first, then alphabetical
- `AudioDeviceError` unchanged — errors are not serialised, they
  are converted to `String` at the boundary

## How

- **`Clone` required transitively** — `EngineUpdate` derives `Clone`
  and will contain audio types in the fan-out path. Every type in
  the chain must implement `Clone`.

- **Dependency cycle prevention** — `mush-core` is the foundation
  crate. Structured types that would create a cycle stay as `String`
  placeholders until a better resolution is available post-MVP.

- **`thiserror` errors are not serialised** — `AudioDeviceError`
  contains `cpal::Error` which does not implement `Serialize`.
  Errors are converted to `String` at the boundary before entering
  `EngineUpdate`.

## Testing

All 35 existing tests pass:

```
Summary [2.728s] 35 tests run: 35 passed, 0 skipped
```

No new tests in this PR — `list_devices()` delegates directly to
`enumerate_hosts_and_devices()` which is already exercised by the
existing device enumeration tests.

## Hmm...

1. Should `list_devices()` be the canonical public API and
   `enumerate_hosts_and_devices()` made private? The longer name
   was appropriate when it was the only entry point but
   `list_devices()` is cleaner for `EngineMonitor` callers.

2. The ALSA wildcard filter uses `u32::MAX` as a sentinel. Is this
   robust across all ALSA versions or should it be a range check
   with a more conservative upper bound?

commit 0ff1706f23a42fcf8101ff23fc2f6701519c2df6
Merge: 79c8357 2699776
Author: arthurtd <atetzlaffdeas@gmail.com>
Date:   Thu Oct 29 22:30:00 2026 +0000

Adds midi device enumeration (3.4). (#28)

## What

Extends `mish-midi-core` to prepare MIDI device enumeration for
the `EngineMonitor` integration.

- Adds `Serialize`/`Deserialize` derives to `MidiPortInfo` and
  `MidiPortDirection` in `types.rs`
- Adds `list_midi_ports()` to `enumerate.rs` — always-fresh
  enumeration, no cache
- Marks `devices_refresh()` and `invalidate_devices_cache()` on
  `MidiEngine` as deprecated
- Adds `#![allow(deprecated)]` to the test module to preserve the
  existing 8 tests without modification

## Why

The `EngineMonitor` (task 6) will call `list_midi_ports()` directly
in response to `TransportCommand::MidiDevices`. It does not need a
cache — enumeration only happens on explicit user request, so the
cost of always enumerating fresh is negligible and correctness is
guaranteed. Hardware changes (port connect/disconnect) are never
missed.

The cache in `MidiEngine::devices()` was appropriate for the old
direct-call architecture where the shell could call `devices()`
repeatedly. That architecture is being replaced. Deprecating the
cache methods now makes the intent visible to reviewers before the
cache is removed entirely in task 7 when the shells migrate to the
`EngineMonitor`.

`MidiPortInfo` is serialised because it will travel over the ZMQ
transport in task 5. `MidiError` is not serialised — errors are
converted to `String` at the boundary before being placed in
`MidiUpdate::Error(String)`.

## Note on `MidiUpdate::PortList`

`PortList` remains `PortList(String)` in `mush-core/src/update.rs`.
Changing it to `PortList(Vec<MidiPortInfo>)` would require
`mush-core` to depend on `mish-midi-core`, creating a dependency
cycle:

```
mush-core → mish-midi-core → mush-core
```

The `EngineMonitor` will format `Vec<MidiPortInfo>` into a string
at the boundary before sending the update. This will be revisited
if a structured wire format is needed.

## How

- **`#[deprecated]`** — marks items scheduled for removal. The
compiler warns at every call site independently. `#[allow(deprecated)]`
  is scoped — it suppresses the lint only within the item it is
  attached to, not in callers.

- **`#![allow(...)]`** — inner attribute (note the `!`). Applied to
  the enclosing item rather than the next item. Used here to suppress
  deprecation warnings across the entire test module.

- **Dependency graph discipline** — `mush-core` is the foundation
  crate. Nothing it depends on may depend on it in return. This
  constraint drives several design decisions including the
  `PortList(String)` placeholder.

## Testing

All 35 existing tests pass:

```
Summary [2.675s] 35 tests run: 35 passed, 0 skipped
```

No new tests in this PR — `list_midi_ports()` delegates directly
to `enumerate_devices()` which is already covered by
`test_devices_returns_output`.

## Hmm....

1. Should `list_midi_ports()` be the canonical public API and
   `enumerate_devices()` made private? Or is there value in keeping
   both public for callers that want the legacy name?

2. The deprecation attributes will become dead code warnings once
   the shell migration in task 7 removes the last callers. Should
   the cache be removed now rather than deprecated, accepting that
   the shell tests will need updating immediately?

