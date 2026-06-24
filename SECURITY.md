# Security Policy

## Supported versions

| Version | Supported |
| ------- | --------- |
| 0.1.x   | Yes       |

## Reporting a vulnerability

If you discover a security issue, please **do not** open a public GitHub issue.

Email the maintainers with:

- A description of the vulnerability
- Steps to reproduce
- Impact assessment (if known)

We aim to acknowledge reports within 5 business days.

## Scope

Promptly is a **local single-user desktop application**. In scope:

- IPC handling between the embedded webview and Rust backend
- SQLite database access and migrations
- Clipboard and notification integrations

Out of scope:

- Network-facing services (none exist)
- Third-party desktop environment bugs (GTK, WebKitGTK, compositors)

## Security model

- The UI runs in an embedded WebKit webview with navigation restricted to `about:blank` and `data:` URLs.
- IPC is unauthenticated but only reachable from the embedded frontend bundled in the binary.
- Prompt data is stored unencrypted at `~/.config/promptly/prompts.db` with default filesystem permissions.
