//! CLI export/import without starting the GUI.

use std::path::{Path, PathBuf};

use crate::config;
use crate::db;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ExportBundle {
    pub version: u32,
    pub prompts: Vec<db::Prompt>,
}

const EXPORT_VERSION: u32 = 1;

pub fn export_prompts(output: &Path) -> anyhow::Result<usize> {
    config::ensure_config_dir()?;
    let conn = db::init_db(&config::db_path())?;
    let prompts = db::load_prompts(&conn)?;
    let count = prompts.len();
    let bundle = ExportBundle {
        version: EXPORT_VERSION,
        prompts,
    };
    let json = serde_json::to_string_pretty(&bundle)?;
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(output, json)?;
    Ok(count)
}

pub fn import_prompts(input: &Path) -> anyhow::Result<usize> {
    config::ensure_config_dir()?;
    let raw = std::fs::read_to_string(input)?;
    let bundle: ExportBundle = serde_json::from_str(&raw)?;
    if bundle.version != EXPORT_VERSION {
        anyhow::bail!(
            "Unsupported export version {} (expected {EXPORT_VERSION})",
            bundle.version
        );
    }
    let conn = db::init_db(&config::db_path())?;
    let mut imported = 0usize;
    for prompt in bundle.prompts {
        db::upsert_prompt(&conn, &prompt.name, &prompt.description, &prompt.content)?;
        imported += 1;
    }
    Ok(imported)
}

pub fn default_export_path() -> PathBuf {
    config::config_dir().join("prompts-export.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn export_import_roundtrip() {
        let db_file = NamedTempFile::new().unwrap();
        std::env::set_var("PROMPTLY_DB_PATH", db_file.path());

        let conn = db::init_db(db_file.path()).unwrap();
        db::upsert_prompt(
            &conn,
            "demo",
            "desc",
            r#"content <var name="x" type="text" />"#,
        )
        .unwrap();

        let export_path = db_file.path().with_extension("json");
        let count = export_prompts(&export_path).unwrap();
        assert_eq!(count, 1);

        db::delete_prompt(&conn, 1).unwrap();
        assert!(db::load_prompts(&conn).unwrap().is_empty());

        let imported = import_prompts(&export_path).unwrap();
        assert_eq!(imported, 1);
        let prompts = db::load_prompts(&conn).unwrap();
        assert_eq!(prompts[0].name, "demo");

        std::env::remove_var("PROMPTLY_DB_PATH");
    }
}
