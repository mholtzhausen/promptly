//! Promptly — system-tray app for managing prompt templates.

mod about;
mod cli;
mod config;
mod db;
mod hotkey;
mod instance;
mod ipc;
mod logging;
mod prompt_parser;
mod seed;
mod tray;
mod update;
mod webview_app;
mod window_focus;

use anyhow::{Context, Result};

const VERSION: &str = env!("CARGO_PKG_VERSION");

enum CliAction {
    Version,
    Export { output: std::path::PathBuf },
    Import { input: std::path::PathBuf },
    Update,
    Seed,
    Run {
        show_on_start: bool,
        seed_first: bool,
    },
}

fn parse_cli() -> CliAction {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return CliAction::Run {
            show_on_start: false,
            seed_first: false,
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
        "update" => CliAction::Update,
        "seed" => CliAction::Seed,
        _ => {
            let show_on_start = args.iter().any(|a| a == "--show" || a == "-s");
            let seed_first = args.iter().any(|a| a == "--seed");
            let only_flags = args.iter().all(|a| a.starts_with('-'));
            if seed_first && only_flags && !show_on_start {
                return CliAction::Seed;
            }
            CliAction::Run {
                show_on_start,
                seed_first,
            }
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

fn run_gui(show_on_start: bool, seed_first: bool) -> Result<()> {
    if seed_first {
        logging::init_logging()?;
        config::ensure_config_dir()?;
        let count = cli::seed_prompts().context("seed failed")?;
        println!("Seeded {count} prompt(s)");
    }

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

    let (toggle_tx, toggle_rx) = std::sync::mpsc::channel::<tray::TrayAction>();
    let tray_state = tray::TrayState::build(toggle_tx.clone())?;
    hotkey::register_global_hotkey(toggle_tx.clone());

    let toggle_proxy = proxy.clone();
    std::thread::spawn(move || {
        while let Ok(action) = toggle_rx.recv() {
            let event = match action {
                tray::TrayAction::ToggleWindow => webview_app::AppEvent::ToggleWindow,
                tray::TrayAction::CheckForUpdates => {
                    webview_app::AppEvent::CheckForUpdates {
                        user_initiated: true,
                    }
                }
                tray::TrayAction::ShowAbout => webview_app::AppEvent::ShowAbout,
            };
            let _ = toggle_proxy.send_event(event);
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
        CliAction::Update => {
            logging::init_logging()?;
            update::run_update().context("update failed")?;
            Ok(())
        }
        CliAction::Seed => {
            logging::init_logging()?;
            let count = cli::seed_prompts().context("seed failed")?;
            println!("Seeded {count} prompt(s)");
            Ok(())
        }
        CliAction::Run {
            show_on_start,
            seed_first,
        } => run_gui(show_on_start, seed_first),
    }
}
