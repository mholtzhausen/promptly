//! Persistent file logging for daemonized runs.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crate::config;

static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();
static MAX_LEVEL: OnceLock<log::LevelFilter> = OnceLock::new();

struct DualLogger;

impl log::Log for DualLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= max_level()
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!(
            "{} [{}] {}: {}\n",
            wall_timestamp_secs(),
            record.level(),
            record.target(),
            record.args()
        );
        let _ = std::io::stderr().write_all(line.as_bytes());
        if let Some(file) = LOG_FILE.get() {
            if let Ok(mut f) = file.lock() {
                let _ = f.write_all(line.as_bytes());
            }
        }
    }

    fn flush(&self) {
        let _ = std::io::stderr().flush();
        if let Some(file) = LOG_FILE.get() {
            if let Ok(mut f) = file.lock() {
                let _ = f.flush();
            }
        }
    }
}

fn max_level() -> log::LevelFilter {
    *MAX_LEVEL.get_or_init(parse_rust_log)
}

fn parse_rust_log() -> log::LevelFilter {
    match std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".into())
        .to_ascii_lowercase()
        .as_str()
    {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "warn" | "warning" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        "off" => log::LevelFilter::Off,
        _ => log::LevelFilter::Info,
    }
}

fn wall_timestamp_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn open_log_file(path: &Path) -> anyhow::Result<std::fs::File> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(OpenOptions::new().create(true).append(true).open(path)?)
}

/// Initialize logging to stderr and `~/.local/state/promptly/promptly.log`.
pub fn init_logging() -> anyhow::Result<()> {
    let path = config::log_file_path();
    let file = open_log_file(&path)?;
    LOG_FILE.set(Mutex::new(file)).ok();
    let _ = max_level();
    log::set_logger(&DualLogger)?;
    log::set_max_level(max_level());
    log::info!(
        "Logging initialized (file: {}, RUST_LOG={:?})",
        path.display(),
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_log_file_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/promptly.log");
        let _f = open_log_file(&path).unwrap();
        assert!(path.exists());
    }
}
