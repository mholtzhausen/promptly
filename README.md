# Promptly

A Rust + GTK4 system-tray application that manages prompt templates with variable placeholders.

## Features

- **System Tray Icon**: Lives in your system tray using the freedesktop/KDE StatusNotifierItem protocol via `ksni`.
:- **Global Hotkey**: Press `Ctrl+Alt+Space` from anywhere to trigger the prompt selector popup. On X11 uses `XGrabKey` directly; on Wayland falls back to `rdev` evdev.
- **Fuzzy Search**: Filter your stored templates quickly by typing.
- **Variable Interpolation**: Templates with placeholders like `{{name|type|default|description}}` open a centered, always-on-top input dialog.
- **Type-Aware Inputs**: Automatically generates GTK fields (Text Entry, SpinButton for numbers, ComboBoxText for options, TextView for multiline text).
- **Editable Final Prompt**: Review and adjust the interpolated prompt in a fixed-height multiline editor before copying.
- **Clipboard Integration**: Copies the final edited prompt to the clipboard.
- **Prompt Metadata and Actions**: Store a title, description, and template content; edit or delete templates from the popup with confirmation.
- **Add Prompts UI**: Add new prompt templates directly from the app interface using the `+` button in the popup.
:- **Auto-background**: Launches silently in the background — the terminal returns to the shell immediately.
- **Cross-Platform/DE Compatibility**: Designed to work on X11 and Wayland (reads inputs directly via `rdev` / `evdev`).

## Requirements

- Rust (2021 edition)
- GTK4 library and development headers
- SQLite3
- `libxdo` library (for rdev evdev fallback on Wayland)

## How to Run

Use the Makefile from the project root:

```bash
make run
```

Useful targets:

```bash
make build
make test
make install     # install to /usr/local/bin
make clean
```

The launcher and Makefile automatically prepare the local `libxdo` linker symlink when the system has `libxdo.so.3` but not the development `libxdo.so` symlink.

## Database Location

Prompts are stored in an SQLite database at:
`~/.config/promptly/prompts.db`
