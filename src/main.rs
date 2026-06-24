//! Promptly — system-tray app for managing prompt templates.
//!
//! Global shortcut: Ctrl+Alt+Space toggles the prompt-manager webview window.
//! Variables syntax: {{name|type|default|description}}

mod config;
mod db;
mod hotkey;
mod ipc;
mod prompt_parser;
mod tray;
mod webview_app;
mod window_focus;

use anyhow::Result;

struct CliOptions {
    show_on_start: bool,
}

fn parse_cli() -> CliOptions {
    let mut show_on_start = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--show" | "-s" => show_on_start = true,
            _ => {}
        }
    }
    CliOptions { show_on_start }
}

/// Detach from the terminal by re-execing with `PROMPTLY_FOREGROUND=1`.
fn daemonize() {
    if std::env::var("PROMPTLY_FOREGROUND").is_ok() {
        return;
    }
    let exe = std::env::current_exe().expect("failed to get current exe path");
    let _child = std::process::Command::new(&exe)
        .env("PROMPTLY_FOREGROUND", "1")
        .args(std::env::args().skip(1))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to daemonize");
    std::process::exit(0);
}

fn main() -> Result<()> {
    let cli = parse_cli();
    daemonize();
    env_logger::init();

    config::ensure_config_dir()?;

    let event_loop =
        tao::event_loop::EventLoopBuilder::<webview_app::AppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let (toggle_tx, toggle_rx) = std::sync::mpsc::channel::<()>();

    let tray_state = tray::TrayState::build(toggle_tx.clone())?;

    hotkey::register_global_hotkey(toggle_tx.clone());

    let toggle_proxy = proxy.clone();
    std::thread::spawn(move || {
        while toggle_rx.recv().is_ok() {
            let _ = toggle_proxy.send_event(webview_app::AppEvent::ToggleWindow);
        }
    });

    let app = webview_app::PromptlyWebviewApp::new(
        &event_loop,
        proxy.clone(),
        tray_state,
        config::db_path(),
    )?;
    webview_app::PromptlyWebviewApp::run(event_loop, app, cli.show_on_start);
}
