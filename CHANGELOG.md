# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Startup and tray **Check for Updates** with desktop notifications and in-app changelog update dialog
- `promptly update` CLI command to install the latest release via `scripts/install.sh`
- Install script systemd user-service management (`PROMPTLY_MANAGE_SERVICE`, interactive TTY prompt)
- Update checks resolve the latest release via GitHub redirect/Atom feed (no API token required); GitHub API is a fallback only

## [0.8.0+1] - 2026-06-25

### Added

- Template editor with CodeMirror variable chips, inline editing popover, and variable picker
- `scripts/install.sh` for one-line user-local install from GitHub releases
- Schema v2 migration: legacy `{{name|type|default|description}}` placeholders converted to `<var />` on database open

### Changed

- Restored production config path (`~/.config/promptly`) after POC branch isolation
- README installation instructions and requirements
- GitHub Actions release workflow and packaging gitignore

## [0.8.0] - 2026-06-25

### Added

- GitHub Actions CI (fmt, clippy, tests, cargo/npm audit)
- Persistent file logging to `~/.local/state/promptly/promptly.log`
- Single-instance lock via `~/.config/promptly/promptly.lock`
- IPC payload size limits
- SQLite WAL mode and versioned schema migrations
- CLI: `promptly --version`, `promptly export`, `promptly import`
- User-local install: `make install-user` with `.desktop` and systemd user unit
- Frontend Vitest unit tests
- Accessibility improvements (ARIA labels, live regions)
- LICENSE (MIT), SECURITY.md, CONTRIBUTING.md, troubleshooting docs
- History management with search, edit, and prune
- Dynamic window title updates and window reveal/geometry handling
- Keyboard shortcuts (Escape to close, Ctrl+Escape to quit, Ctrl/Meta prompt selection)
- YAML configuration support for window preferences

### Changed

- Restructured IPC into `src/ipc/` modules
- Decomposed React frontend into components, hooks, and typed API client
- `make run` uses foreground mode for easier local debugging
- Refactored CSS to use variables for improved maintainability
- Enhanced build and installation processes with improved error handling
- Dependency updates (GTK/glib, rusqlite, notify-rust, CI actions)

## [0.1.0] - 2026-06-24

### Added

- Initial Tao/Wry + React system-tray prompt manager
- Fuzzy search, variable interpolation, copy history
- Global hotkey (Ctrl+Alt+Space) and ksni tray icon

[Unreleased]: https://github.com/mholtzhausen/promptly/compare/v0.8.0+1...HEAD
[0.8.0+1]: https://github.com/mholtzhausen/promptly/releases/tag/v0.8.0%2B1
[0.8.0]: https://github.com/mholtzhausen/promptly/releases/tag/v0.8.0
[0.1.0]: https://github.com/mholtzhausen/promptly/releases/tag/v0.1.0
