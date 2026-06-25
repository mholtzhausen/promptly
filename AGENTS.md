# Repository Guidelines

## Project Overview

**Promptly** (`promptly`) — a Linux desktop system-tray application written in Rust with a React UI embedded in a Tao/Wry webview. It manages prompt templates with variable placeholders in SQLite, provides fuzzy-search selection in the web UI, type-aware variable input dialogs, copy history, and clipboard integration.

## Architecture & Data Flow

```
┌──────────┐    mpsc       ┌──────────────────────┐
│ hotkey   │ ────────────▶ │ Tao Event Loop       │
│ thread   │  UserEvent    │ (webview_app.rs)     │
└──────────┘               │ ┌──────────────────┐ │
┌──────────┐               │ │ Wry WebView      │ │
│ ksni     │ ──toggle────▶ │ │ (React frontend) │ │
│ Tray     │               │ └────────┬─────────┘ │
└──────────┘               │          │ IPC JSON   │
                           │   ┌──────▼──────┐    │
                           │   │ ipc.rs      │    │
                           │   │ db + parser │    │
                           │   └─────────────┘    │
                           └──────────────────────┘
```

- **No async/await** — synchronous Tao event loop + blocking I/O. Hotkey and tray run in background threads; IPC and DB access run on the main thread.
- **Hotkey detection**: On X11 uses `XGrabKey` (via `x11` crate) in a spawned `std::thread`. Falls back to `rdev::listen()` on Wayland (needs `input` group). Sends `()` through `mpsc` to the Tao event loop as `AppEvent::ToggleWindow`.
- **System tray**: `ksni` crate (freedesktop StatusNotifierItem protocol). Lives in a background thread managed by the crate.
- **Frontend**: React SPA built with Vite (`vite-plugin-singlefile`) → single `frontend/dist/index.html` embedded via `include_str!` in the binary.
- **IPC**: JSON request/response over Wry's `window.ipc.postMessage` / `evaluate_script` (`window.__promptlyReceive`).
- **DB connection**: Single `rusqlite::Connection` held in `IpcBackend` (all IPC on main thread).
- **Window focus**: X11 EWMH hints via `window_focus.rs` (graceful no-op on Wayland for some paths).
- **Daemonization**: `daemonize()` re-execs with `PROMPTLY_FOREGROUND=1` and exits the parent.

## Key Directories

| Directory | Purpose |
|---|---|
| `src/` | Rust host: event loop, IPC, DB, tray, hotkey |
| `src/ipc/` | IPC types, command handlers, response helpers |
| `frontend/` | React UI (Vite build → embedded in binary) |
| `target/` | Build artifacts (gitignored) |

## Key Modules

| File | Role |
|---|---|
| `src/main.rs` | Entry: CLI, daemonize, hotkey thread, Tao bootstrap |
| `src/hotkey.rs` | X11 XGrabKey + rdev fallback |
| `src/webview_app.rs` | Tao window + Wry webview + show/hide lifecycle |
| `src/ipc/` | JSON IPC dispatch, prompts/history commands |
| `src/db.rs` | SQLite prompts + copy history |
| `src/prompt_parser.rs` | `{{var\|type\|default\|desc}}` parse + interpolate |
| `src/config.rs` | Paths, YAML window size config |
| `src/window_focus.rs` | X11 centering, always-on-top, opacity |
| `src/tray.rs` | ksni tray menu (Show / Quit) |
| `frontend/src/App.tsx` | React UI (view router + screens) |

## Development Commands

```bash
make run              # frontend build + cargo release + launch
make build            # frontend build + cargo build --release
make test             # frontend build + cargo test + vitest
make lint             # cargo fmt --check, clippy, tsc
make frontend-build   # npm ci + vite build only
make clean            # rm frontend/dist + cargo clean
make install          # copy to /usr/local/bin
make install-user     # ~/.local/bin + .desktop + systemd user unit
make uninstall        # remove user-local install
```

CLI: `promptly --version`, `promptly export [path]`, `promptly import <path>`.
Logs: `~/.local/state/promptly/promptly.log` when daemonized; `RUST_LOG=promptly=debug`.
Single instance: flock on `~/.config/promptly-poc/promptly.lock` (POC branch).

Convenience script: `./run.sh` (same as `make run`, bash-only).

## Important Files

| File | Description |
|---|---|
| `Cargo.toml` | Rust package and dependencies |
| `Makefile` | Build orchestration (frontend then cargo) |
| `build.rs` | Requires `frontend/dist/index.html`; watches frontend sources |
| `frontend/vite.config.ts` | Single-file build + WebKitGTK script attr fix |

## Runtime / Tooling Preferences

- **Linux only** — X11 preferred for window hints; Wayland supported with reduced focus behavior.
- **Rust edition 2021**, stable toolchain.
- **Node.js** ^20.19 or >=22.12 for frontend build.
- **GTK4/WebKitGTK** — pulled in by Wry's `build_gtk` on Linux (not a standalone GTK app UI).
- **Release profile**: `opt-level=2`, LTO enabled.

## Code Conventions

### Rust
- `anyhow::Result` for error propagation.
- `snake_case` functions, `PascalCase` types.
- IPC DTOs use `serde` with `camelCase` JSON.

### Frontend
- React 19, no router — manual `view` state union.
- Fuzzy search is client-side only (`frontend/src/lib/fuzzy.ts`).
- Typed IPC via `frontend/src/api/commands.ts`.

### Template Syntax (POC branch)
```xml
<var name="variable_name" type="text" value="default" label="Field label" placeholder="hint" />
<var name="color" type="option" options="red,green,blue" value="red" label="Pick one" />
```
Required attributes: `name`, `type`. Types: `text`, `number`, `option`, `multiline`.

Template editor: CodeMirror with inline variable chips; click chip to edit attributes via popover.

POC data path: `~/.config/promptly-poc/` (DB, config, lock). Logs remain at `~/.local/state/promptly/`.

### History Title Format (Rust ↔ frontend contract)
```
[PromptName](var1:value1, var2:value2)
```
Built by `db::build_history_title`; parsed in `frontend/src/lib/historyTitle.ts`.

## Testing

```bash
make test
```

- Inline `#[cfg(test)]` modules in `db.rs`, `prompt_parser.rs`, `config.rs`, `ipc/`.
- `tempfile` dev-dependency for isolated SQLite fixtures.
- No integration test directory; IPC contract test in `src/ipc/contract.rs`.
