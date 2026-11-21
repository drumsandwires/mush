# Test

This document describes the test strategy, coverage matrix, matrix generation
procedure, and manual test procedures for `mush`.

---

## 1. Test Strategy

### 1.1 Test Tiers

| Tier | Label | Tool | Scope | Speed |
|------|-------|------|-------|-------|
| Unit | **U** | `cargo nextest` | Pure logic — no I/O, no device, no binary | Fast |
| Integration | **I** | `assert_cmd` / `trycmd` | Shell binary stdin/stdout; no real device required | Medium |
| Plugin | **P** | `nu_plugin_test_support` / `assert_cmd` wrapping `nu -c` | Nushell plugin commands | Medium |
| Smoke | **S** | Manual | Requires real or virtual MIDI/audio device | Slow |

### 1.2 Test Naming Convention

Automated tests follow this naming scheme:

    test_<shell>_<command>_<variant>_<scenario>

| Segment | Values |
|---------|--------|
| `shell` | `mi` (mish) · `ma` (mash) · `mu` (mush) · `n_mi` (nu_plugin_midi) · `n_la` (nu_plugin_live_audio) |
| `command` | `ports` · `port` · `load` · `play` · `stop` · `send` · `monitor` · `record` · `save` · `version` · `hosts` · `host` · `devices` · `device` |
| `variant` | Command variant or subcommand — e.g. `get` · `set` · `mid` · `mudl` · omit if only one variant |
| `scenario` | Expected outcome — e.g. `ok` · `err` · `empty` · `invalid` · `no_port` |

Examples:
- `test_mi_port_set_ok` — mish, `port <name>`, valid name, expect success
- `test_n_mi_load_mid_ok` — nu_plugin_midi, `midi load song.mid`, expect success
- `test_mi_send_invalid_err` — mish, `send`, malformed message, expect error

### 1.3 Coverage Symbol Key

| Symbol | Meaning |
|--------|---------|
| ✅ | Test exists and passes |
| ❌ | Test exists and fails |
| 🚧 | Test exists but blocked by open issue |
| ⬜ | No test yet — gap in coverage for an implemented feature |
| 💨 | Smoke only — no automated test possible |
| 🔮 | Planned — feature not yet implemented; test deferred |
| N/A | Command not applicable to this shell |

---

## 2. Virtual Device Setup

Real MIDI and audio devices are required for smoke tests. Where a physical
device is unavailable, use the virtual devices described below.

### 2.1 MIDI Loopback

A virtual MIDI loopback port allows send/monitor/play tests without physical
hardware.

**Linux — snd-virmidi**

    sudo modprobe snd-virmidi
    # Enumerates as "Virtual Raw MIDI" in midir port list
    # Remove after testing:
    sudo modprobe -r snd-virmidi

**macOS — IAC Driver**

1. Open **Audio MIDI Setup** (`/Applications/Utilities/Audio MIDI Setup.app`)
2. Open **MIDI Studio** (Window menu)
3. Double-click **IAC Driver** → check **Device is online**
4. The IAC Bus appears in midir port list immediately

**Windows — loopMIDI**

1. Download and install [loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html) (free)
2. Create a new virtual port — it appears in midir port list immediately
3. Leave loopMIDI running during tests

### 2.2 Virtual Audio Device

A virtual audio sink/source allows audio enumeration and routing tests
without physical hardware.

**Linux — PipeWire / PulseAudio null sink**

    # Create virtual sink (output)
    pactl load-module module-null-sink sink_name=mush-test sink_properties=device.description=mush-test
    # Create virtual source (input) from monitor of sink
    pactl load-module module-virtual-source source_name=mush-test-in master=mush-test.monitor
    # Remove after testing:
    pactl unload-module module-null-sink
    pactl unload-module module-virtual-source

**macOS — BlackHole**

1. Install [BlackHole](https://existential.audio/blackhole/) (free, open source)
2. Select BlackHole as audio device in Audio MIDI Setup
3. Appears in cpal device list immediately

**Windows — VB-Audio Virtual Cable**

1. Download and install [VB-Audio Virtual Cable](https://vb-audio.com/Cable/) (free)
2. CABLE Input/Output appear in cpal device list after reboot

---

## 3. Coverage Matrix

The matrix is generated automatically from `cargo nextest` output.
See [Section 4](#4-matrix-generation) for the generation procedure.
Smoke tests (`💨`) and deferred features (`🔮`) are set manually.

<!-- COVERAGE-MATRIX:BEGIN -->
| Command | Tier | mish | mash | mush | nu_midi | nu_audio | Blocker |
|---------|------|------|------|------|---------|----------|---------|
| `version` | I+P | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | |
| `ports` | I+P | ⬜ | N/A | ⬜ | ⬜ | N/A | |
| `port` (get) | I+P | ⬜ | N/A | ⬜ | ⬜ | N/A | |
| `port <name>` (set) | I+P | ⬜ | N/A | ⬜ | ⬜ | N/A | #79 #80 |
| `load song.mid` | I+P | ⬜ | N/A | ⬜ | ⬜ | N/A | #90 |
| `load song.mudl` | I+P | ⬜ | N/A | ⬜ | ⬜ | N/A | #90 |
| `play` | I+S | ⬜ | N/A | ⬜ | ⬜ | N/A | #82 #83 |
| `stop` | I+P | ⬜ | N/A | ⬜ | ⬜ | N/A | |
| `pipeline play` | P | N/A | N/A | N/A | ⬜ | N/A | |
| `pipeline play filtered` | P | N/A | N/A | N/A | ⬜ | N/A | |
| `send <msg>` | U+I+P | ⬜ | N/A | ⬜ | ⬜ | N/A | |
| `monitor` | S | 🚧 | N/A | 🚧 | 🚧 | N/A | #86 #87 |
| `record` | S | 💨 | N/A | 💨 | 💨 | N/A | |
| `save song.mudl` | I+P | ⬜ | N/A | ⬜ | ⬜ | N/A | |
| `save song.mid` | I+P | ⬜ | N/A | ⬜ | ⬜ | N/A | |
| `audio hosts` | I+P | N/A | ⬜ | ⬜ | N/A | ⬜ | |
| `audio host` (get) | I+P | N/A | ⬜ | ⬜ | N/A | ⬜ | |
| `audio host <name>` (set) | I+P | N/A | ⬜ | ⬜ | N/A | ⬜ | |
| `audio devices` | I+P | N/A | ⬜ | ⬜ | N/A | ⬜ | |
| `audio device` (get) | I+P | N/A | ⬜ | ⬜ | N/A | ⬜ | |
| `audio device <name>` (set) | I+P | N/A | 🔮 | 🔮 | N/A | 🔮 | |
<!-- COVERAGE-MATRIX:END -->

---

## 4. Matrix Generation

The coverage matrix is generated by running the full test suite and parsing
`cargo nextest` JSON output with a Nu script.

### 4.1 Generate

Run the full test suite with JSON output:

    cargo nextest run --workspace --message-format json 2>/dev/null > target/nextest-results.json

Then run the generation script from the workspace root:

    nu scripts/generate-matrix.nu

### 4.2 Generation Script

```nu
# scripts/generate-matrix.nu
# Parses nextest JSON output and splices the updated coverage matrix
# into the COVERAGE-MATRIX sentinel block in TEST.md.

def cell [results: list, shell: string, command: string] {
    let pattern = $"test_($shell)_($command)"
    let matches = ($results | where name =~ $pattern)
    if ($matches | is-empty) { return "⬜" }
    let failed = ($matches | where status == "fail")
    if ($failed | is-empty) { "✅" } else { "❌" }
}

def main [] {
    let results = (
        open target/nextest-results.json
        | lines
        | each { |line| $line | from json }
        | where type == "test"
        | select name status
    )

    let matrix = [
        "| Command                     | Tier  | mish                              | mash                              | mush                              | nu_midi                               | nu_audio                              | Blocker   |",
        "|------------------------------|-------|-----------------------------------|-----------------------------------|-----------------------------------|---------------------------------------|---------------------------------------|-----------|",
        $"| `version`                   | I+P   | (cell $results mi version)        | (cell $results ma version)        | (cell $results mu version)        | (cell $results n_mi version)          | (cell $results n_la version)          |           |",
        $"| `ports`                     | I+P   | (cell $results mi ports)          | N/A                               | (cell $results mu ports)          | (cell $results n_mi ports)            | N/A                                   |           |",
        $"| `port` (get)                | I+P   | (cell $results mi port_get)       | N/A                               | (cell $results mu port_get)       | (cell $results n_mi port_get)         | N/A                                   |           |",
        $"| `port <name>` (set)         | I+P   | (cell $results mi port_set)       | N/A                               | (cell $results mu port_set)       | (cell $results n_mi port_set)         | N/A                                   | #79 #80   |",
        $"| `load song.mid`             | I+P   | (cell $results mi load_mid)       | N/A                               | (cell $results mu load_mid)       | (cell $results n_mi load_mid)         | N/A                                   | #90       |",
        $"| `load song.mudl`            | I+P   | (cell $results mi load_mudl)      | N/A                               | (cell $results mu load_mudl)      | (cell $results n_mi load_mudl)        | N/A                                   | #90       |",
        $"| `play`                      | I+S   | (cell $results mi play)           | N/A                               | (cell $results mu play)           | (cell $results n_mi play)             | N/A                                   | #82 #83   |",
        $"| `stop`                      | I+P   | (cell $results mi stop)           | N/A                               | (cell $results mu stop)           | (cell $results n_mi stop)             | N/A                                   |           |",
        $"| `pipeline play`             | P     | N/A                               | N/A                               | N/A                               | (cell $results n_mi pipeline_play)    | N/A                                   |           |",
        $"| `pipeline play filtered`    | P     | N/A                               | N/A                               | N/A                               | (cell $results n_mi pipeline_filtered)| N/A                                   |           |",
        $"| `send <msg>`                | U+I+P | (cell $results mi send)           | N/A                               | (cell $results mu send)           | (cell $results n_mi send)             | N/A                                   |           |",
        "| `monitor`                   | S     | 🚧                                | N/A                               | 🚧                                | 🚧                                    | N/A                                   | #86 #87   |",
        "| `record`                    | S     | 💨                                | N/A                               | 💨                                | 💨                                    | N/A                                   |           |",
        $"| `save song.mudl`            | I+P   | (cell $results mi save_mudl)      | N/A                               | (cell $results mu save_mudl)      | (cell $results n_mi save_mudl)        | N/A                                   |           |",
        $"| `save song.mid`             | I+P   | (cell $results mi save_mid)       | N/A                               | (cell $results mu save_mid)       | (cell $results n_mi save_mid)         | N/A                                   |           |",
        $"| `audio hosts`               | I+P   | N/A                               | (cell $results ma hosts)          | (cell $results mu hosts)          | N/A                                   | (cell $results n_la hosts)            |           |",
        $"| `audio host` (get)          | I+P   | N/A                               | (cell $results ma host_get)       | (cell $results mu host_get)       | N/A                                   | (cell $results n_la host_get)         |           |",
        $"| `audio host <name>` (set)   | I+P   | N/A                               | (cell $results ma host_set)       | (cell $results mu host_set)       | N/A                                   | (cell $results n_la host_set)         |           |",
        $"| `audio devices`             | I+P   | N/A                               | (cell $results ma devices)        | (cell $results mu devices)        | N/A                                   | (cell $results n_la devices)          |           |",
        $"| `audio device` (get)        | I+P   | N/A                               | (cell $results ma device_get)     | (cell $results mu device_get)     | N/A                                   | (cell $results n_la device_get)       |           |",
        "| `audio device <name>` (set) | I+P   | N/A                               | 🔮                                | 🔮                                | N/A                                   | 🔮                                    |           |",
    ]

    let table = ($matrix | str join "\n")

    let doc = (open TEST.md)
    let before = ($doc | split row "<!-- COVERAGE-MATRIX:BEGIN -->" | first)
    let after  = ($doc | split row "<!-- COVERAGE-MATRIX:END -->"   | last)

    let updated = $"($before)<!-- COVERAGE-MATRIX:BEGIN -->\n($table)\n<!-- COVERAGE-MATRIX:END -->($after)"
    $updated | save -f TEST.md

    print "Coverage matrix updated in TEST.md"
}
```

### 4.3 Mask Task

Add to `README.md`:

    ## coverage
    > Regenerate the TEST.md coverage matrix from the last nextest run

    ```nu
    nu scripts/generate-matrix.nu
    ```
---

## 5. Manual Procedures

### 5.1 Tracing and Diagnostics (8.3.12e)

#### Stage 3 — Conductor, transport instrumentation, and command-level spans

##### ST-3-1 — Clean play/stop cycle produces no RT warnings

**Purpose:** Verify that normal playback produces no `DiagEvent` warnings,
confirming the RT path is operating within capacity under typical conditions.

**Fixture:** `test.mid`

**Applies to:** `mish`

**Prerequisites:**
- A MIDI output port available and selected (see Section 2.1)

**Procedure:**

    mish
    mish> load tests/fixtures/test.mid
    mish> play
    # let run for full duration (~8 seconds at 120 BPM)
    mish> stop
    mish> exit

**Expected:**
- No `warn!` lines containing `RT:` in log output
- `▶ transport started` and `■ transport stopped` displayed
- No interleaved log lines overwriting the prompt (known issue — #96)

**Automatable:** Partially — `assert_cmd` can drive the shell and assert
exit code; log content requires file logging to assert against (#102).

---

##### ST-3-2 — Command-level spans appear at debug log level

**Purpose:** Verify that `debug_span!("command", cmd = "...")` entries
appear when `RUST_LOG=debug` is set.

**Fixture:** `test.mid`

**Applies to:** `mish`

**Procedure:**

    RUST_LOG=debug mish 2>mish-debug.log <<EOF
    load tests/fixtures/test.mid
    play
    stop
    exit
    EOF
    grep "command" mish-debug.log

**Expected:**
- Lines containing `command{cmd="load"}`, `command{cmd="play"}`,
  `command{cmd="stop"}` present in log output

**Automatable:** Yes — `assert_cmd` with piped stdin and stderr capture,
assert log contains expected span names.

---

##### ST-3-3 — nu_plugin_midi command spans at debug level

**Purpose:** Verify plugin command spans appear when invoked via nushell.

**Fixture:** `test.mid`

**Applies to:** `nu_plugin_midi`

**Prerequisites:**
- `nu_plugin_midi` registered in nushell (`mask plug-add midi`)

**Procedure:**

    RUST_LOG=debug midi load tests/fixtures/test.mid 2>plugin-debug.log
    grep "command{cmd=\"load\"}" plugin-debug.log

    RUST_LOG=debug midi play 2>>plugin-debug.log
    grep "command{cmd=\"play\"}" plugin-debug.log

**Expected:**
- `DEBUG command{cmd="load"}: nu_plugin_midi: load: run` present
- `DEBUG command{cmd="play"}: nu_plugin_midi: play: run` present

**Automatable:** Yes — `assert_cmd` wrapping `nu -c "midi load ..."` with
stderr capture.

---

#### Stage 4 — RT diagnostic buffer

##### ST-4-1 — No spurious RT diagnostics during normal playback

**Purpose:** Confirm `DiagEvent` ring buffer produces no warnings under
normal load/play/stop conditions.

**Fixture:** `test.mid`

**Applies to:** `mish`

**Prerequisites:**
- MIDI output port available (see Section 2.1)

**Procedure:**

    mish
    mish> load tests/fixtures/test.mid
    mish> play
    # let run for full duration
    mish> stop
    mish> exit

**Expected:**
- Zero lines matching `RT:` in log output at any log level

**Automatable:** Yes — file logging + `assert_cmd` + assert log does not
contain `RT:`. Requires #102 resolved.

---

##### ST-4-2 — `MidiSendError` on port disconnect mid-playback

**Purpose:** Verify `MidiSendError` is recorded and logged when the MIDI
output port is disconnected during playback, and that the RT thread stops
cleanly without panic.

**Fixture:** `long.mid`

**Applies to:** `mish`

**Prerequisites:**
- Physical or virtual MIDI port that can be disconnected at runtime

**Procedure:**

    mish
    mish> load tests/fixtures/long.mid
    mish> play
    # after ~5 seconds, disconnect MIDI port
    # observe log output
    mish> stop
    mish> exit

**Expected:**
- `WARN RT: MIDI send error` appears in log
- Shell does not panic or hang
- `stop` command completes cleanly

**Automatable:** Unlikely — requires hardware or OS-level port manipulation.

---

##### ST-4-3 — `MidiQueueFull` under queue pressure

**Purpose:** Verify `MidiQueueFull` is recorded when the MIDI queue fills
faster than it is drained.

**Fixture:** `dense.mid`

**Applies to:** `mish`

**Procedure:**

    mish
    mish> load tests/fixtures/dense.mid
    mish> play
    mish> stop
    mish> exit

**Expected:**
- One or more `WARN RT: midi_queue full — MIDI event dropped` lines in log

**Notes:** If `MIDI_QUEUE_CAPACITY` is large enough to absorb `dense.mid`
without filling, temporarily reduce it to 4 in `conductor.rs` and rebuild.
Restore after testing.

**Automatable:** Yes with reduced capacity build — `assert_cmd` + file log
assertion. Requires #102 resolved.

---

##### ST-4-4 — `TimelineLockMissed` under write contention

**Purpose:** Verify `TimelineLockMissed` is recorded when a timeline write
lock is held during an audio callback.

**Fixture:** `long.mid`

**Applies to:** `mish`

**Prerequisites:**
- Test injection point in the timeline write path (not yet implemented)

**Procedure:**
1. Add `std::thread::sleep(Duration::from_millis(50))` inside a timeline
   write lock in `conductor.rs`
2. Rebuild
3. Run load/play/stop cycle with `long.mid`

**Expected:**
- One or more `WARN RT: timeline lock missed — buffer period skipped`
  in log

**Automatable:** Unlikely without a `#[cfg(test)]` injection hook in the
timeline write path.

---

#### Stage 5 — CLI flags

##### ST-5-1 — Default log level is warn

**Purpose:** Verify that without any flags or env vars, only `warn` and
above appears in log output.

**Applies to:** `mish`, `mash`, `mush`

**Procedure:**

    mish 2>out.log <<EOF
    exit
    EOF
    grep -E "^(DEBUG|INFO|TRACE)" out.log

**Expected:**
- No `DEBUG`, `INFO`, or `TRACE` lines in output

**Automatable:** Yes — `assert_cmd` with stderr capture.

---

##### ST-5-2 — `RUST_LOG` env var controls log level

**Purpose:** Verify `RUST_LOG=debug` surfaces debug-level output.

**Applies to:** `mish`, `mash`, `mush`, `nu_plugin_midi`, `nu_plugin_live_audio`

**Procedure:**

    RUST_LOG=debug mish 2>out.log <<EOF
    exit
    EOF
    grep "DEBUG" out.log

**Expected:**
- One or more `DEBUG` lines present

**Automatable:** Yes — `assert_cmd` with env and stderr capture.

---

##### ST-5-3 — `--log-level` flag overrides default

**Purpose:** Verify `--log-level debug` produces debug output equivalent
to `RUST_LOG=debug`.

**Applies to:** `mish`, `mash`, `mush`

**Procedure:**

    mish --log-level debug 2>out.log <<EOF
    exit
    EOF
    grep "DEBUG" out.log

**Expected:**
- One or more `DEBUG` lines present

**Automatable:** Yes — `assert_cmd`.

---

##### ST-5-4 — `--log-file` writes output to file

**Purpose:** Verify `--log-file mush.log` routes tracing output to the
specified file rather than stderr.

**Applies to:** `mish`, `mash`, `mush`

**Status:** Blocked — #102 (`--log-file` flag not yet implemented)

**Procedure:**

    mish --log-level debug --log-file mush.log <<EOF
    exit
    EOF
    cat mush.log

**Expected:**
- `mush.log` exists and contains tracing output
- stderr is clean

**Automatable:** Yes — `assert_cmd` + file existence + content assertion.

---

##### ST-5-5 — `--tokio-console` flag accepted without error

**Purpose:** Verify the flag is accepted and the process starts cleanly.
Full console functionality is deferred (Stage 6).

**Applies to:** `mish`, `mash`, `mush`

**Procedure:**

    mish --tokio-console <<EOF
    exit
    EOF

**Expected:**
- Process starts and exits cleanly
- No `unrecognized argument` error

**Automatable:** Yes — `assert_cmd` exit code assertion.

---

#### Stage 6 — Tokio Console

Deferred — tokio-console TUI connects but shows no task events.
See mask task `tokio-console` and open investigation notes in `main.rs`.

---

#### Automation Assessment (8.3.12e)

| Test   | Automatable | Tool         | Fixture      | Blocker                          |
|--------|-------------|--------------|--------------|----------------------------------|
| ST-3-1 | Partially   | `assert_cmd` | `test.mid`   | Needs file logging (#102)        |
| ST-3-2 | Yes         | `assert_cmd` | `test.mid`   | None                             |
| ST-3-3 | Yes         | `assert_cmd` | `test.mid`   | nushell must be in PATH          |
| ST-4-1 | Yes         | `assert_cmd` | `test.mid`   | Needs file logging (#102)        |
| ST-4-2 | No          | —            | `long.mid`   | Hardware/OS dependency           |
| ST-4-3 | Yes         | `assert_cmd` | `dense.mid`  | Reduced capacity build required  |
| ST-4-4 | No          | —            | `long.mid`   | Needs test injection point       |
| ST-5-1 | Yes         | `assert_cmd` | —            | None                             |
| ST-5-2 | Yes         | `assert_cmd` | —            | None                             |
| ST-5-3 | Yes         | `assert_cmd` | —            | None                             |
| ST-5-4 | Yes         | `assert_cmd` | —            | #102                             |
| ST-5-5 | Yes         | `assert_cmd` | —            | None                             |

---

### 5.2 MVP Command Surface (9.0)

The following procedures cover the full MVP command surface defined in
section 9.0. Each procedure notes which shells it applies to and whether
a virtual device is required.

#### Prerequisites

- Fixtures generated: `cargo run -p mush-core --bin gen_fixtures -- all`
- For smoke tests: virtual MIDI port configured (see Section 2.1)
- For audio smoke tests: virtual audio device configured (see Section 2.2)
- For plugin procedures: `nu_plugin_midi` and `nu_plugin_live_audio`
  registered in nushell (`mask plug-add midi`, `mask plug-add live_audio`)

---

##### SP-9-1 — `version`

**Applies to:** `mish`, `mash`, `mush`, `nu_plugin_midi`, `nu_plugin_live_audio`

**Tier:** I + P

**Procedure:**

    # Shells
    mish --version
    mash --version
    mush --version

    # Plugins
    nu -c "midi version"
    nu -c "audio version"

**Expected:**
- Version string printed, exit 0

**Automatable:** Yes — covered by existing `trycmd` banner tests.

---

##### SP-9-2 — `ports` / `midi ports`

**Applies to:** `mish`, `mush`, `nu_plugin_midi`

**Tier:** I + P

**Procedure:**

    # Shell
    mish <<EOF
    ports
    exit
    EOF

    # Plugin
    nu -c "midi ports"

**Expected:**
- Structured list of available MIDI ports printed, exit 0
- May be empty in environments without a virtual device; assert command
  succeeds and output is a valid list (including empty)

**Automatable:** Yes — `assert_cmd` exit 0 + output contains table header
or empty list marker.

---

##### SP-9-3 — `port` / `midi port` (get)

**Applies to:** `mish`, `mush`, `nu_plugin_midi`

**Tier:** I + P

**Procedure:**

    # Shell
    mish <<EOF
    port
    exit
    EOF

    # Plugin
    nu -c "midi port"

**Expected:**
- Returns `(none selected)` or active port name, exit 0

**Automatable:** Yes.

---

##### SP-9-4 — `port <name>` / `midi port <name>` (set)

**Applies to:** `mish`, `mush`, `nu_plugin_midi`

**Tier:** I + P

**Requires:** Virtual MIDI port (Section 2.1)

**Blockers:** #79, #80

**Procedure:**

    # Set valid port — confirm persisted
    mish <<EOF
    port "Virtual Raw MIDI"
    port
    exit
    EOF

**Expected:**
- Port set confirmed, subsequent `port` returns the set name

**Error path:**

    mish <<EOF
    port "nonexistent port"
    port
    exit
    EOF

**Expected:**
- Error message printed on invalid name
- Subsequent `port` returns previous value, not cleared (#79)

**Automatable:** Yes once #79 and #80 are resolved.

---

##### SP-9-5 — `load song.mid` / `midi load song.mid`

**Applies to:** `mish`, `mush`, `nu_plugin_midi`

**Tier:** I + P

**Blocker:** #90 (relative path resolution)

**Procedure:**

    # Shell
    mish <<EOF
    load tests/fixtures/test.mid
    exit
    EOF

    # Plugin
    nu -c "midi load tests/fixtures/test.mid"

**Expected:**
- File loads cleanly, summary printed (#81), exit 0
- Clock position reset to 0 (#82)

**Automatable:** Yes once #90 resolved.

---

##### SP-9-6 — `load song.mudl` / `midi load song.mudl`

**Applies to:** `mish`, `mush`, `nu_plugin_midi`

**Tier:** I + P

**Blocker:** #90 (relative path resolution)

**Procedure:**

    # Shell
    mish <<EOF
    load tests/fixtures/test.mudl
    exit
    EOF

    # Plugin
    nu -c "midi load tests/fixtures/test.mudl"

**Expected:**
- MUDL file loads cleanly, summary printed, exit 0
- Clock position reset to 0 (#82)

**Automatable:** Yes once #90 resolved.

---

##### SP-9-7 — `play` / `midi play`

**Applies to:** `mish`, `mush`, `nu_plugin_midi`

**Tier:** I + S

**Requires:** Virtual MIDI port + FluidSynth for audible verification

**Blockers:** #82 (clock not reset on load), #83 (play after record)

**Procedure:**

    mish <<EOF
    load tests/fixtures/test.mid
    play
    stop
    exit
    EOF

**Expected:**
- Audible C major scale on connected synth
- `▶ transport started` displayed on play
- `■ transport stopped` displayed on stop
- Exit 0

**Automatable:** Partially — `assert_cmd` can assert exit 0 and output
markers; audible verification is manual.

---

##### SP-9-8 — `stop` / `midi stop`

**Applies to:** `mish`, `mush`, `nu_plugin_midi`

**Tier:** I + P

**Procedure:**

    # Stop without prior play — must be a no-op, not an error
    mish <<EOF
    stop
    exit
    EOF

    # Stop after play
    mish <<EOF
    load tests/fixtures/test.mid
    play
    stop
    exit
    EOF

**Expected:**
- Both cases exit 0
- No panic or hang

**Automatable:** Yes.

---

##### SP-9-9 — Pipeline play (`open | midi play`)

**Applies to:** `nu_plugin_midi` only

**Tier:** P

**Procedure:**

    nu -c "open tests/fixtures/test.mudl | midi play"

**Expected:**
- Playback begins, exit 0

**Automatable:** Yes — `assert_cmd` wrapping `nu -c`.

---

##### SP-9-10 — Filtered pipeline play (`open | where | midi play`)

**Applies to:** `nu_plugin_midi` only

**Tier:** P

**Procedure:**

    nu -c "open tests/fixtures/test.mudl | where note > 60 | midi play"

**Expected:**
- Only notes above 60 play, exit 0

**Automatable:** Yes — `assert_cmd` wrapping `nu -c`.

---

##### SP-9-11 — `send <msg>` / `midi send <msg>`

**Applies to:** `mish`, `mush`, `nu_plugin_midi`

**Tier:** U + I + P

**Requires:** Virtual MIDI port for delivery verification

**Procedure:**

    # Symbolic format
    mish <<EOF
    send note-on 1 60 100
    exit
    EOF

    # Numeric format
    mish <<EOF
    send 0x90 0x3C 0x64
    exit
    EOF

    # Plugin
    nu -c "midi send note-on 1 60 100"

    # Invalid input — expect error, not panic
    mish <<EOF
    send garbage
    exit
    EOF

**Expected:**
- Valid messages: delivered to port, exit 0
- Invalid message: error printed, non-zero exit, no panic

**Automatable:** Yes — unit tests for codec parse; `assert_cmd` for
delivery and error path.

---

##### SP-9-12 — `monitor` / `midi monitor`

**Applies to:** `mish`, `mush`, `nu_plugin_midi`

**Tier:** S

**Blockers:** #86 (wrong port / replays sent events), #87 (exits immediately)

**Status:** Deferred until #86 and #87 resolved.

**Procedure (post-fix):**

    mish <<EOF
    monitor
    # send MIDI events from external source
    # observe streamed output
    # Ctrl-C to exit
    exit
    EOF

**Expected:**
- Stays open, streams incoming events
- Pre-roll autosaved on exit
- No previously sent events replayed

---

##### SP-9-13 — `record` / `midi record`

**Applies to:** `mish`, `mush`, `nu_plugin_midi`

**Tier:** S

**Requires:** Virtual MIDI input port with live input source (Section 2.1)

**Procedure:**

    mish <<EOF
    record
    # play notes on input device
    stop
    exit
    EOF

**Expected:**
- Events captured during recording window
- Bounded recording stops cleanly on `stop`
- No panic or hang

**Automatable:** No — requires live input.

---

##### SP-9-14 — `save song.mudl` / `midi save song.mudl`

**Applies to:** `mish`, `mush`, `nu_plugin_midi`

**Tier:** I + P

**Procedure:**

    mish <<EOF
    load tests/fixtures/test.mid
    save /tmp/out.mudl
    exit
    EOF

    # Verify file exists and is non-empty
    ls -lh /tmp/out.mudl

    # Plugin
    nu -c "midi load tests/fixtures/test.mid; midi save /tmp/out.mudl"

**Expected:**
- File written, exit 0
- Round-trip: `load /tmp/out.mudl` produces equivalent timeline

**Automatable:** Yes — `assert_cmd` + file existence + optional round-trip
load assertion.

---

##### SP-9-15 — `save song.mid` / `midi save song.mid`

**Applies to:** `mish`, `mush`, `nu_plugin_midi`

**Tier:** I + P

**Procedure:**

    mish <<EOF
    load tests/fixtures/test.mid
    save /tmp/out.mid
    exit
    EOF

    # Verify file exists and is non-empty
    ls -lh /tmp/out.mid

**Expected:**
- Valid SMF file written, exit 0
- File readable by external MIDI tools

**Automatable:** Yes — `assert_cmd` + file existence assertion.

---

##### SP-9-16 — `audio hosts` / `audio hosts`

**Applies to:** `mash`, `mush`, `nu_plugin_live_audio`

**Tier:** I + P

**Procedure:**

    mash <<EOF
    hosts
    exit
    EOF

    nu -c "audio hosts"

**Expected:**
- Non-empty list of audio hosts, exit 0
- At least one host present on any supported platform

**Automatable:** Yes — `assert_cmd` exit 0 + non-empty output.

---

##### SP-9-17 — `audio host` (get)

**Applies to:** `mash`, `mush`, `nu_plugin_live_audio`

**Tier:** I + P

**Procedure:**

    mash <<EOF
    host
    exit
    EOF

    nu -c "audio host"

**Expected:**
- Returns default host name, exit 0

**Automatable:** Yes.

---

##### SP-9-18 — `audio host <name>` (set)

**Applies to:** `mash`, `mush`, `nu_plugin_live_audio`

**Tier:** I + P

**Procedure:**

    mash <<EOF
    host "default"
    host
    exit
    EOF

**Expected:**
- Host set confirmed, subsequent `host` returns set name, exit 0

**Automatable:** Yes.

---

##### SP-9-19 — `audio devices`

**Applies to:** `mash`, `mush`, `nu_plugin_live_audio`

**Tier:** I + P

**Procedure:**

    mash <<EOF
    devices
    exit
    EOF

    nu -c "audio devices"

**Expected:**
- Non-empty list of audio devices, exit 0
- Covered by existing `assert_cmd` proof-of-concept

**Automatable:** Yes — PoC exists, needs promotion to final test.

---

##### SP-9-20 — `audio device` (get)

**Applies to:** `mash`, `mush`, `nu_plugin_live_audio`

**Tier:** I + P

**Procedure:**

    mash <<EOF
    device
    exit
    EOF

    nu -c "audio device"

**Expected:**
- Returns default device name, exit 0

**Automatable:** Yes.

---

##### SP-9-21 — `audio device <name>` (set)

**Applies to:** `mash`, `mush`, `nu_plugin_live_audio`

**Tier:** I + P

**Status:** 🔮 Planned — audio device selection not yet implemented at MVP.

---

#### Automation Assessment (9.0)

| Test    | Automatable | Tool                  | Fixture       | Blocker                     |
|---------|-------------|-----------------------|---------------|-----------------------------|
| SP-9-1  | Yes         | `trycmd`              | —             | None (existing)             |
| SP-9-2  | Yes         | `assert_cmd`          | —             | None                        |
| SP-9-3  | Yes         | `assert_cmd`          | —             | None                        |
| SP-9-4  | Yes         | `assert_cmd`          | —             | #79 #80                     |
| SP-9-5  | Yes         | `assert_cmd`          | `test.mid`    | #90|
| SP-9-6  | Yes         | `assert_cmd`          | `test.mudl`   | #90                         |
| SP-9-7  | Partially   | `assert_cmd`          | `test.mid`    | #82 #83                     |
| SP-9-8  | Yes         | `assert_cmd`          | `test.mid`    | None                        |
| SP-9-9  | Yes         | `assert_cmd` / `nu`   | `test.mudl`   | None                        |
| SP-9-10 | Yes         | `assert_cmd` / `nu`   | `test.mudl`   | None                        |
| SP-9-11 | Yes         | `assert_cmd` + unit   | —             | None                        |
| SP-9-12 | No          | —                     | —             | #86 #87                     |
| SP-9-13 | No          | —                     | —             | Live input required         |
| SP-9-14 | Yes         | `assert_cmd`          | `test.mid`    | None                        |
| SP-9-15 | Yes         | `assert_cmd`          | `test.mid`    | None                        |
| SP-9-16 | Yes         | `assert_cmd`          | —             | None                        |
| SP-9-17 | Yes         | `assert_cmd`          | —             | None                        |
| SP-9-18 | Yes         | `assert_cmd`          | —             | None                        |
| SP-9-19 | Yes         | `assert_cmd`          | —             | None (PoC exists)           |
| SP-9-20 | Yes         | `assert_cmd`          | —             | None                        |
| SP-9-21 | Yes         | `assert_cmd`          | —             | 🔮 Not yet implemented      |
