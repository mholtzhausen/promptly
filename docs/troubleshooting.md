# Troubleshooting

## Hotkey does not work (Ctrl+Alt+Space)

**X11:** Promptly registers a global grab via XGrabKey. Another app may have grabbed the same shortcut. Check logs at `~/.local/state/promptly/promptly.log`.

**Wayland:** Promptly falls back to `rdev` (evdev). Your user may need membership in the `input` group:

```bash
sudo usermod -aG input $USER
# log out and back in
```

## Tray icon missing

Promptly uses the freedesktop StatusNotifierItem protocol (`ksni`). Some desktop environments hide legacy tray icons. Ensure your panel supports StatusNotifierItem / AppIndicator.

## Window does not appear

1. Run in foreground to see errors:

   ```bash
   PROMPTLY_FOREGROUND=1 promptly --show
   ```

2. Check logs: `~/.local/state/promptly/promptly.log`

3. Verify WebKitGTK is installed:

   ```bash
   ldconfig -p | grep webkit
   ```

## Build failures (missing GTK/WebKit)

On Debian/Ubuntu:

```bash
sudo apt-get install libgtk-4-dev libwebkit2gtk-4.1-dev libappindicator3-dev pkg-config build-essential
```

## "Another Promptly instance is already running"

Only one instance is allowed. Stop the existing process or remove a stale lock if no process is running:

```bash
pkill promptly
rm -f ~/.config/promptly/promptly.lock
```

## Database issues

- Location: `~/.config/promptly/prompts.db`
- Backup: copy the file while Promptly is not running
- Export prompts: `promptly export ~/prompts-backup.json`
- Import prompts: `promptly import ~/prompts-backup.json`

## Environment variables

| Variable | Purpose |
|----------|---------|
| `PROMPTLY_FOREGROUND=1` | Skip daemonize; keep attached to terminal |
| `PROMPTLY_DB_PATH` | Override SQLite database path |
| `RUST_LOG` | Log level (`debug`, `info`, `warn`, `error`) |
