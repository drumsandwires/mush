# Develop

## tokio-console

tokio-console provides a TUI debugger for async Rust tasks. Infrastructure
is in place but task event display is parked pending an upstream release.

### What is wired

- `.cargo/config.toml` sets `--cfg tokio_unstable` workspace-wide
- `mush-tracing` has a `tokio-console` feature that adds `ConsoleLayer`
  to the tracing-subscriber registry on a configurable port
- Each shell binary exposes `--tokio-console` and `--console-port` flags
- Plugins use `MUSH_CONSOLE` / `MUSH_CONSOLE_PORT` env vars
- `mask build-console` and `mask run-console` tasks in README

### How to attach (when working)

    mask run-console mish 6669
    tokio-console http://127.0.0.1:6669

### Parked issue

`tokio-console` 0.1.14 requires `console-api` 0.9. No released version of
`console-subscriber` provides this — the latest release (0.3) ships
`console-api` 0.7. The workspace pins `console-subscriber` to git rev
`b59758e5a6effe3a9df3ec84a2f28d319d178234` which provides `console-api`
0.9 and connects without protocol errors, but emits no task events.

Revisit when `console-subscriber` cuts a crates.io release that ships
`console-api` 0.9. Track at:
https://github.com/tokio-rs/console/releases
