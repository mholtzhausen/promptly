# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

### Changed

- Restructured IPC into `src/ipc/` modules
- Decomposed React frontend into components, hooks, and typed API client
- `make run` uses foreground mode for easier local debugging

## [0.1.0] - 2026-06-24

### Added

- Initial Tao/Wry + React system-tray prompt manager
- Fuzzy search, variable interpolation, copy history
- Global hotkey (Ctrl+Alt+Space) and ksni tray icon

[Unreleased]: https://github.com/example/promptly/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/example/promptly/releases/tag/v0.1.0
