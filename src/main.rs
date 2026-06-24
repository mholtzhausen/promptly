//! Promptly — system-tray app for managing prompt templates.

mod cli;
mod config;
mod db;
mod hotkey;
mod instance;
mod ipc;
mod logging;
mod prompt_parser;
mod tray;
mod webview_app;
mod window_focus;

use anyhow::{Context, Result};

const VERSION: &str = env!("CARGO_PKG_VERSION");

enum CliAction {
    Version,
    Export { output: std::path::PathBuf },
    Import { input: std::path::PathBuf },
    Run { show_on_start: bool },
}

fn parse_cli() -> CliAction {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return CliAction::Run {
            show_on_start: false,
        };
    }

    match args[0].as_str() {
        "--version" | "-V" | "version" => CliAction::Version,
        "export" => {
            let output = args
                .iter()
                .skip(1)
                .find(|a| !a.starts_with('-'))
                .map(std::path::PathBuf::from)
                .unwrap_or_else(cli::default_export_path);
            CliAction::Export { output }
        }
        "import" => {
            let input = args
                .iter()
                .skip(1)
                .find(|a| !a.starts_with('-'))
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| config::config_dir().join("prompts-export.json"));
            CliAction::Import { input }
        }
        _ => {
            let show_on_start = args.iter().any(|a| a == "--show" || a == "-s");
            CliAction::Run { show_on_start }
        }
    }
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

fn run_gui(show_on_start: bool) -> Result<()> {
    daemonize();
    logging::init_logging()?;
    let _lock = instance::InstanceLock::acquire()?;

    config::ensure_config_dir()?;
    log::info!(
        "Starting Promptly v{VERSION} (db: {})",
        config::db_path().display()
    );

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
    webview_app::PromptlyWebviewApp::run(event_loop, app, show_on_start);
}

fn main() -> Result<()> {
    match parse_cli() {
        CliAction::Version => {
            println!("promptly {VERSION}");
            Ok(())
        }
        CliAction::Export { output } => {
            logging::init_logging()?;
            let count = cli::export_prompts(&output).context("export failed")?;
            println!("Exported {count} prompt(s) to {}", output.display());
            Ok(())
        }
        CliAction::Import { input } => {
            logging::init_logging()?;
            let count = cli::import_prompts(&input).context("import failed")?;
            println!("Imported {count} prompt(s) from {}", input.display());
            Ok(())
        }
        CliAction::Run { show_on_start } => run_gui(show_on_start),
    }
}
