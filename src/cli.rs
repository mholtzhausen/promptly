//! CLI export/import and seeding without starting the GUI.

use std::path::{Path, PathBuf};

use crate::config;
use crate::db::{self, normalize_category};
use crate::seed;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ExportBundle {
    pub version: u32,
    pub prompts: Vec<db::Prompt>,
}

#[derive(Debug, serde::Deserialize)]
struct ImportPrompt {
    name: String,
    description: String,
    content: String,
    #[serde(default = "default_import_category")]
    category: String,
}

fn default_import_category() -> String {
    db::DEFAULT_PROMPT_CATEGORY.to_string()
}

#[derive(Debug, serde::Deserialize)]
struct ImportBundle {
    version: u32,
    prompts: Vec<ImportPrompt>,
}

const EXPORT_VERSION: u32 = 2;

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
    let bundle: ImportBundle = serde_json::from_str(&raw)?;
    if bundle.version != 1 && bundle.version != EXPORT_VERSION {
        anyhow::bail!(
            "Unsupported export version {} (expected 1 or {EXPORT_VERSION})",
            bundle.version
        );
    }
    let conn = db::init_db(&config::db_path())?;
    let mut imported = 0usize;
    for prompt in bundle.prompts {
        let category = normalize_category(&prompt.category);
        db::upsert_prompt(
            &conn,
            &prompt.name,
            &prompt.description,
            &prompt.content,
            &category,
        )?;
        imported += 1;
    }
    Ok(imported)
}

pub fn seed_prompts() -> anyhow::Result<usize> {
    config::ensure_config_dir()?;
    let conn = db::init_db(&config::db_path())?;
    seed::upsert_seed_prompts(&conn)
}

pub fn default_export_path() -> PathBuf {
    config::config_dir().join("prompts-export.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::NamedTempFile;

    static CLI_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn export_import_roundtrip() {
        let _guard = CLI_TEST_LOCK.lock().unwrap();
        let db_file = NamedTempFile::new().unwrap();
        std::env::set_var("PROMPTLY_DB_PATH", db_file.path());

        let conn = db::init_db(db_file.path()).unwrap();
        db::upsert_prompt(
            &conn,
            "demo",
            "desc",
            r#"content <var name="x" type="text" />"#,
            "development",
        )
        .unwrap();

        let export_path = db_file.path().with_extension("json");
        let count = export_prompts(&export_path).unwrap();
        assert_eq!(count, 1);

        db::delete_prompt(&conn, 1).unwrap();
        assert!(db::load_prompts(&conn).unwrap().is_empty());

        let imported = import_prompts(&export_path).unwrap();
        assert_eq!(imported, 1);
        let conn = db::init_db(db_file.path()).unwrap();
        let prompts = db::load_prompts(&conn).unwrap();
        assert_eq!(prompts[0].name, "demo");
        assert_eq!(prompts[0].category, "development");

        std::env::remove_var("PROMPTLY_DB_PATH");
    }

    #[test]
    fn import_v1_defaults_category() {
        let _guard = CLI_TEST_LOCK.lock().unwrap();
        let db_file = NamedTempFile::new().unwrap();
        std::env::set_var("PROMPTLY_DB_PATH", db_file.path());
        db::init_db(db_file.path()).unwrap();

        let import_path = db_file.path().with_extension("v1.json");
        std::fs::write(
            &import_path,
            r#"{
  "version": 1,
  "prompts": [
    {
      "id": 1,
      "name": "legacy",
      "description": "old export",
      "content": "body"
    }
  ]
}"#,
        )
        .unwrap();

        import_prompts(&import_path).unwrap();
        let conn = db::init_db(db_file.path()).unwrap();
        let prompts = db::load_prompts(&conn).unwrap();
        assert_eq!(prompts[0].category, "general");

        std::env::remove_var("PROMPTLY_DB_PATH");
    }
}
