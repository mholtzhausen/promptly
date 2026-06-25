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

**To install a release** (curl script below): Linux x86_64 and WebKitGTK runtime libraries (`libwebkit2gtk-4.1-0`, GTK4).

**To build from source**:

- Rust (2021 edition)
- Node.js ^20.19 or >=22.12
- WebKitGTK (via Wry on Linux; `libwebkit2gtk-4.1-dev` and GTK4 dev packages)
- SQLite3 (bundled via `rusqlite`)

## Install

On Linux x86_64, install or update the latest release to `~/.local` (same layout as `make install-user`):

```bash
curl -fsSL https://raw.githubusercontent.com/mholtzhausen/promptly/main/scripts/install.sh | bash
```

This installs:

- `~/.local/bin/promptly`
- `~/.local/share/applications/promptly.desktop`
- `~/.config/autostart/promptly.desktop`
- `~/.config/systemd/user/promptly.service`

Enable autostart after install:

```bash
systemctl --user enable --now promptly.service
```

Pin a specific release:

```bash
PROMPTLY_VERSION=v0.8.0 curl -fsSL https://raw.githubusercontent.com/mholtzhausen/promptly/main/scripts/install.sh | bash
```

To remove a user-local install: `make uninstall` (from a source checkout) or delete the files listed above manually.

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
make test             # frontend build + cargo + vitest
make lint             # fmt, clippy, tsc
make install          # install to /usr/local/bin (sudo)
make install-user     # install to ~/.local/bin + desktop + systemd user unit
make uninstall        # remove user-local install
make clean
```

Or use `./run.sh` (equivalent to `make run`).

## CLI

```bash
promptly --version              # print version
promptly --show                 # show the prompt window (used by tray/hotkey path)
promptly export [path]          # export prompts (+ history) to JSON
promptly import <path>          # import prompts from JSON
```

Set `PROMPTLY_DB_PATH` to override the default SQLite location. Set `RUST_LOG=promptly=debug` for verbose logging.

## Data Locations

| Path | Purpose |
|---|---|
| `~/.config/promptly/prompts.db` | SQLite prompts + copy history |
| `~/.config/promptly/config.yml` | Window size preferences |
| `~/.config/promptly/promptly.lock` | Single-instance lock file |
| `~/.local/state/promptly/promptly.log` | Daemon log file (when backgrounded) |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). CI runs `make lint`, `cargo test`, `cargo audit`, and frontend tests on every PR.

## Troubleshooting

See [docs/troubleshooting.md](docs/troubleshooting.md) for hotkey, tray, and WebKitGTK issues.
