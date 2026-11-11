# `mush-midi-codec` — Design Spec

## Overview

A symmetric MIDI message codec crate.

```
&str ──parse──► Vec<u8> ──format──► String
```
  
Shared by all shells (`mish`, `mash`, `mush`) and Nu plugins
(`nu_plugin_midi`, `nu_plugin_live_audio`) for `midi send` input
and `midi monitor` display.

---

## Crate Layout

```text
crates/mush-midi-codec/
├── Cargo.toml
└── src/
├── lib.rs — public API, re-exports
├── status.rs — StatusByte enum + parse/format
├── note.rs — note name ↔ MIDI number
├── velocity.rs — dynamic marking ↔ u8
├── parse.rs — &str → Vec<u8>
├── format.rs — Vec<u8> → String
└── error.rs — ParseError, FormatError
```
  
---

## Public API

```rust
/// Parse a symbolic or numeric MIDI message string into raw wire bytes.
pub fn parse(input: &str) -> Result<Vec<u8>, ParseError>;

/// Format raw MIDI wire bytes into a human-readable string.
pub fn format(bytes: &[u8], style: Style) -> Result<String, FormatError>;

/// Output style for `format`.
pub enum Style {
    /// Full symbolic names — e.g. `"NoteOn C4 mf"`.
    /// Default for file output and save previews.
    Long,
    /// Abbreviated names — e.g. `"Non C4 mf"`.
    /// Default for live monitor display.
    Short,
}
```

---

## Status Table

| Long              | Short   | Status Byte | Data1         | Data2    |
| ----------------- | ------- | ----------- | ------------- | -------- |
| `NoteOff`         | `Noff`  | `0x8n`      | note          | velocity |
| `NoteOn`          | `Non`   | `0x9n`      | note          | velocity |
| `PolyPressure`    | `PP`    | `0xAn`      | note          | pressure |
| `ControlChange`   | `CC`    | `0xBn`      | controller    | value    |
| `ProgramChange`   | `PC`    | `0xCn`      | program       | —        |
| `ChannelPressure` | `CP`    | `0xDn`      | pressure      | —        |
| `PitchBend`       | `PB`    | `0xEn`      | lsb           | msb      |
| `SysEx`           | `SysEx` | `0xF0`      | bytes… `0xF7` | —        |

- `n` in the status byte = channel nibble (0–15, channel 1–16).
- `SysEx` has no short form — the name is the same in both styles.
- SysEx is the only variable-length message; all others are fixed width.

---

## Note Names

- Scientific pitch notation: `C-1` = 0 … `G9` = 127.

- Middle C = `C4` = 60.

- Sharps: `C#4`. Flats: `Bb4`.

- Enharmonic equivalents are valid on parse:

  - `B#3` = `C4`, `Cb4` = `B3`, `E#4` = `F4`, `Fb4` = `E4`

- Harmonic correctness is out of scope for MVP.

- Parse is case-insensitive. Format always emits canonical uppercase
  (`C#4`, `Bb4`).

- Flat is preferred over sharp on format (e.g. `Bb4` not `A#4`).

---

## Velocity Symbols

| Symbol | Value |
| ------ | ----- |
| `pppp` | 8     |
| `ppp`  | 16    |
| `pp`   | 32    |
| `p`    | 48    |
| `mp`   | 64    |
| `mf`   | 80    |
| `pf`   | 88    |
| `f`    | 96    |
| `ff`   | 112   |
| `fff`  | 120   |
| `ffff` | 127   |

- Parse is case-insensitive.
- Format picks the nearest symbol; ties go to the louder value.
- Numeric values are also valid in velocity position (e.g. `NoteOn C4 100`).

---

## Numeric Literals

Valid anywhere a byte value is expected (status byte, data bytes,
channel suffix, SysEx payload bytes).

| Prefix | Base    | Example    |
| ------ | ------- | ---------- |
| `0x`   | hex     | `0x3C`     |
| `0o`   | octal   | `0o74`     |
| `0b`   | binary  | `0b111100` |
| none   | decimal | `60`       |

Range constraints:

- Note, velocity, data bytes: 0–127.
- Channel suffix: 1–16.
- SysEx payload bytes: 0–255 (full byte range).

---

## Channel Suffix

- Appended to the status symbol with `/`: `NoteOn/2 C4 mf`.
- 1-indexed (1–16). Omitted = channel 1.
- Valid on all channel-voice messages (`NoteOn` … `PitchBend`).
- Not valid on `SysEx` (system message, no channel).

---

## Parse Grammar

```text
message      = sysex_msg | symbolic_msg | numeric_msg

numeric_msg  = byte byte byte

symbolic_msg = status_sym channel_suffix? SP data1 (SP data2)?

sysex_msg    = sysex_kw SP byte+ SP "0xF7"
sysex_kw     = "SysEx" | "sysex" | "SYSEX"

status_sym   = "NoteOn"  | "Non"
             | "NoteOff" | "Noff"
             | "PolyPressure"    | "PP"
             | "ControlChange"   | "CC"
             | "ProgramChange"   | "PC"
             | "ChannelPressure" | "CP"
             | "PitchBend"       | "PB"

channel_suffix = "/" [1-16]

data1        = note_name | velocity_sym | numeric
data2        = velocity_sym | numeric

note_name    = [A-Ga-g] accidental? octave
accidental   = "#" | "b"
octave       = "-1" | [0-9]

velocity_sym = "pppp" | "ppp" | "pp" | "p"
             | "mp" | "mf" | "pf"
             | "f" | "ff" | "fff" | "ffff"

numeric      = ("0x" [0-9A-Fa-f]+)
             | ("0o" [0-7]+)
             | ("0b" [01]+)
             | [0-9]+

SP           = one or more whitespace characters
```

### Mixed forms

Symbolic status with numeric data is valid:

```text
NoteOn C4 100       — symbolic note, numeric velocity
NoteOn 60 mf        — numeric note, symbolic velocity
NoteOn/3 0x3C 0x64  — hex data bytes
CC 7 0x40           — symbolic status, mixed numeric data
```

---

## Format Direction

### Channel voice messages

```text
[0x90, 60, 100]  →  "NoteOn C4 f"      (Long)
[0x90, 60, 100]  →  "Non C4 f"         (Short)
[0x91, 60, 100]  →  "NoteOn/2 C4 f"    (Long, channel 2)
[0xB0, 7, 64]    →  "ControlChange 7 64"  (Long)
[0xB0, 7, 64]    →  "CC 7 64"          (Short)
[0xC0, 40, 0]    →  "ProgramChange 40" (Long, no data2)
[0xE0, 0, 64]    →  "PitchBend 0 64"   (Long)
```

- Channel suffix `/n` is omitted when channel = 1.
- `ProgramChange` and `ChannelPressure` omit the unused third byte.
- Controller and program numbers are always formatted as decimal
  integers (no symbolic names at MVP).
- Velocity in `NoteOn`/`NoteOff`/`PolyPressure` is formatted as the
  nearest dynamic symbol.
- All other data bytes are formatted as decimal integers.

### SysEx

`[0xF0, 0x41, 0x10, 0xF7] → "SysEx 0x41 0x10 0xF7"`

- `0xF0` (SOX) is stripped from output; `0xF7` (EOX) is retained.
- Payload bytes always formatted as `0x`-prefixed hex in both styles.
- Style parameter has no effect on SysEx output.

---

## Error Types

### `ParseError`

```rust
pub enum ParseError {
    /// Input string was empty or all whitespace.
    EmptyInput,
    /// Status symbol not recognised.
    UnknownStatus(String),
    /// Note name could not be parsed.
    UnknownNote(String),
    /// Velocity symbol not recognised.
    UnknownVelocity(String),
    /// Channel suffix out of range (must be 1–16).
    InvalidChannel(u8),
    /// Numeric literal out of range for its position.
    ByteOutOfRange(i64),
    /// Numeric literal could not be parsed.
    InvalidNumericLiteral(String),
    /// Wrong number of data arguments for this status.
    WrongArgumentCount {
        status:   String,
        expected: usize,
        got:      usize,
    },
    /// SysEx message missing 0xF7 terminator.
    MissingSysExTerminator,
    /// Unexpected tokens after a complete message.
    TrailingInput(String),
}
```

### `FormatError`

```rust
pub enum FormatError {
    /// Byte slice was empty.
    EmptyInput,
    /// Status byte not recognised.
    UnknownStatus(u8),
    /// Byte slice too short for the message implied by the status byte.
    TruncatedMessage {
        status:   u8,
        expected: usize,
        got:      usize,
    },
}
```

---

## Default Style by Context

| Context                        | Default Style |
| ------------------------------ | ------------- |
| `midi monitor` live display    | `Short`       |
| `midi send` echo / confirm     | `Long`        |
| `MidiUpdate::EventDispatched`  | `Short`       |
| File save preview / record log | `Long`        |
| Round-trip tests               | `Long`        |

---

## Round-Trip Guarantee

For all channel-voice messages in the supported set:

`parse(format(bytes, Style::Long)?) == Ok(bytes)`

SysEx round-trips for well-formed messages (F0 … F7).
Numeric-only input does not guarantee symbolic round-trip
(e.g. `parse("144 60 100")` may format as `"NoteOn C4 f"`).
