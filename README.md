# Prompt Tray (`prompt_tray`)

A Rust + GTK4 system-tray application that manages prompt templates with variable placeholders.

## Features

- **System Tray Icon**: Lives in your system tray using the freedesktop/KDE StatusNotifierItem protocol via `ksni`.
- **Global Hotkey**: Press `Ctrl+Alt+Space` from anywhere to trigger the prompt selector popup.
- **Fuzzy Search**: Filter your stored templates quickly by typing.
- **Variable Interpolation**: Templates with placeholders like `{{name|type|default|description}}` will open a type-aware input dialog.
- **Type-Aware Inputs**: Automatically generates GTK fields (Text Entry, SpinButton for numbers, ComboBoxText for options, TextView for multiline text).
- **Clipboard Integration**: Automatically interpolates the inputs and copies the final prompt to the clipboard.
- **Add Prompts UI**: Add new prompt templates directly from the app interface using the `+` button in the popup.
- **Cross-Platform/DE Compatibility**: Designed to work on X11 and Wayland (reads inputs directly via `rdev` / `evdev`).

## Requirements

- Rust (2021 edition)
- GTK4 library and development headers
- SQLite3
- `libxdo` library (for keyboard input simulation / hotkey listening)

## How to Run

Use the Makefile from the project root:

```bash
make run
```

Useful targets:

```bash
make build
make test
make clean
```

The launcher and Makefile automatically prepare the local `libxdo` linker symlink when the system has `libxdo.so.3` but not the development `libxdo.so` symlink.

## Database Location

Prompts are stored in an SQLite database at:
`~/.config/prompt_tray/prompts.db`
