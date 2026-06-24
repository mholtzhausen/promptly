# Contributing to Promptly

## Prerequisites

- Rust stable (see [rust-toolchain.toml](rust-toolchain.toml))
- Node.js 20+ (see [frontend/package.json](frontend/package.json))
- Linux with GTK4 and WebKitGTK dev packages

## Required checks before opening a PR

```bash
make lint
make test
```

This runs:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- Frontend typecheck and Vitest

## Development workflow

```bash
make run          # build and launch in foreground with --show
make build        # release build
PROMPTLY_FOREGROUND=1 ./target/release/promptly --show
```

Logs are written to `~/.local/state/promptly/promptly.log`. Set `RUST_LOG=debug` for verbose output.

## Project layout

- `src/` — Rust host (IPC, DB, tray, hotkey, webview)
- `frontend/` — React UI embedded in the binary
- `packaging/` — `.desktop` and systemd unit files

See [AGENTS.md](AGENTS.md) for architecture details.

## Commit guidelines

- Keep changes focused; match existing code style
- Update [CHANGELOG.md](CHANGELOG.md) for user-visible changes
- Add tests for new behavior where practical
