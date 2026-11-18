#!/usr/bin/env python3
"""
Inserts tracing span entry and exit debug lines into nu_plugin run() methods.

For each file:
  - Finds the opening brace of `fn run(`
  - Inserts `let _span = ...` and `tracing::debug!` after it
  - Inserts a debug line before the final `Ok(` or `Err(` return

Usage:
    python3 tools/instrument_plugin_commands.py
"""

import re
from pathlib import Path

FILES = [
    "crates/nu_plugin_midi/src/commands/save.rs",
    "crates/nu_plugin_midi/src/commands/record.rs",
    "crates/nu_plugin_midi/src/commands/ports.rs",
    "crates/nu_plugin_midi/src/commands/load.rs",
    "crates/nu_plugin_midi/src/commands/stop.rs",
    "crates/nu_plugin_midi/src/commands/send.rs",
    "crates/nu_plugin_midi/src/commands/port.rs",
    "crates/nu_plugin_midi/src/commands/monitor.rs",
    "crates/nu_plugin_midi/src/commands/play.rs",
    "crates/nu_plugin_live_audio/src/commands/devices.rs",
    "crates/nu_plugin_live_audio/src/commands/record.rs",
    "crates/nu_plugin_live_audio/src/commands/hosts.rs",
    "crates/nu_plugin_live_audio/src/commands/stop.rs",
    "crates/nu_plugin_live_audio/src/commands/device.rs",
    "crates/nu_plugin_live_audio/src/commands/play.rs",
    "crates/nu_plugin_live_audio/src/commands/host.rs",
]

SPAN_ENTRY = '''\
        let _span = tracing::debug_span!("command", cmd = "{cmd}").entered();
        tracing::debug!("{cmd}: run");'''

SPAN_EXIT      = '        tracing::debug!(result = %s, "{cmd}: done");'
SPAN_EXIT_BARE = '        tracing::debug!("{cmd}: done");'
SPAN_EXIT_ERR  = '        tracing::debug!("{cmd}: not implemented");'


def find_run_open_brace(lines: list[str], run_line: int) -> int:
    """Return the index of the line containing the opening brace of fn run()."""
    for i in range(run_line, min(run_line + 10, len(lines))):
        if "{" in lines[i]:
            return i
    raise ValueError(f"Could not find opening brace after line {run_line}")


def find_last_ok_or_err(lines: list[str]) -> tuple[int, str]:
    """Return the index and kind ('ok' or 'err') of the last Ok( or Err( line."""
    for i in range(len(lines) - 1, -1, -1):
        if re.search(r"^\s+Ok\(", lines[i]):
            return i, "ok"
        if re.search(r"^\s+Err\(", lines[i]):
            return i, "err"
    raise ValueError("Could not find final Ok( or Err( line")


def has_s_binding(lines: list[str], from_line: int, to_line: int) -> bool:
    """Return True if `s` is assigned anywhere between from_line and to_line."""
    for line in lines[from_line:to_line]:
        if re.search(r"\blet\s+s\b", line):
            return True
    return False


def already_instrumented(lines: list[str]) -> bool:
    return any("debug_span" in line for line in lines)


def instrument(path: Path) -> None:
    cmd = path.stem  # filename without .rs = command name
    lines = path.read_text().splitlines(keepends=True)

    if already_instrumented(lines):
        print(f"  skip (already instrumented): {path}")
        return

    # Find fn run( line
    run_line = next(
        (i for i, l in enumerate(lines) if re.search(r"fn run\(", l)), None
    )
    if run_line is None:
        print(f"  skip (no fn run): {path}")
        return

    # Find opening brace
    brace_line = find_run_open_brace(lines, run_line)

    # Find last Ok( or Err(
    ok_line, kind = find_last_ok_or_err(lines)

    # Build insertion strings
    entry = SPAN_ENTRY.format(cmd=cmd) + "\n"

    if kind == "err":
        exit_ = SPAN_EXIT_ERR.format(cmd=cmd) + "\n"
    elif has_s_binding(lines, brace_line, ok_line):
        exit_ = SPAN_EXIT.format(cmd=cmd) + "\n"
    else:
        exit_ = SPAN_EXIT_BARE.format(cmd=cmd) + "\n"

    # Insert in reverse order to preserve line indices
    lines.insert(ok_line, exit_)
    lines.insert(brace_line + 1, entry)

    path.write_text("".join(lines))
    print(f"  instrumented: {path}")


def main() -> None:
    root = Path(__file__).parent.parent
    for rel in FILES:
        path = root / rel
        if not path.exists():
            print(f"  missing: {path}")
            continue
        instrument(path)
    print("done — run `cargo check` to verify")


if __name__ == "__main__":
    main()
