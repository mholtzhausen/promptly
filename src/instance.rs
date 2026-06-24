//! Single-instance lock via flock on a pid file.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use fs2::FileExt;

use crate::config;

pub struct InstanceLock {
    _file: std::fs::File,
}

impl InstanceLock {
    /// Acquire an exclusive lock, or bail if another instance holds it.
    pub fn acquire() -> anyhow::Result<Self> {
        let path = config::lock_file_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;
        file.try_lock_exclusive().map_err(|e| {
            anyhow::anyhow!(
                "Another Promptly instance is already running (lock: {}). {e}",
                path.display()
            )
        })?;
        rewrite_lock_metadata(&mut file, &path)?;
        Ok(Self { _file: file })
    }
}

fn rewrite_lock_metadata(file: &mut std::fs::File, path: &Path) -> anyhow::Result<()> {
    file.set_len(0)?;
    writeln!(
        file,
        "pid={}\nexe={}\n",
        std::process::id(),
        std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "?".into())
    )?;
    file.sync_all()?;
    log::debug!("Instance lock acquired at {}", path.display());
    Ok(())
}
