# Promptly

A Linux desktop system-tray application that manages prompt templates with variable placeholders. The UI is a React app embedded in a native webview (Tao/Wry).

## Features

- **System Tray Icon**: freedesktop StatusNotifierItem via `ksni`.
- **Global Hotkey**: `Ctrl+Alt+Space` toggles the prompt window. X11 uses `XGrabKey`; Wayland falls back to `rdev`.
- **Fuzzy Search**: Filter templates by name, description, or content (client-side).
- **Variable Interpolation**: Templates with `{{name|type|default|description}}` open a type-aware input screen.
- **Type-Aware Inputs**: Text, number, option (dropdown), and multiline fields.
- **Live Preview**: Interpolated prompt updates as you fill variables; editable before copy.
- **Clipboard Integration**: Copies the final prompt; desktop notifications on success.
- **Copy History**: Deduplicated history of copied prompts with search, edit, and prune.
- **Prompt CRUD**: Create, edit, and delete templates from the UI.
- **Window Config**: Window size persisted in `~/.config/promptly/config.yml`.
- **Auto-background**: Daemonizes on launch unless `PROMPTLY_FOREGROUND=1`.

## Requirements

- Rust (2021 edition)
- Node.js ^20.19 or >=22.12
- WebKitGTK (via Wry on Linux; `libwebkit2gtk-4.1-dev` and GTK4 dev packages)
- SQLite3 (bundled via `rusqlite`)

## How to Run

From the project root:

```bash
make run
```

This builds the frontend (`npm ci` + `vite build`), then compiles the Rust binary, and launches it.

Other targets:

```bash
make build            # release build
make frontend-build   # frontend only
make test             # frontend build + cargo test
make install          # install to /usr/local/bin
make clean
```

Or use `./run.sh` (equivalent to `make run`).

## Data Locations

| Path | Purpose |
|---|---|
| `~/.config/promptly/prompts.db` | SQLite prompts + copy history |
| `~/.config/promptly/config.yml` | Window size preferences |
