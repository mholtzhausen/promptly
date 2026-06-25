//! Built-in starter prompt templates.

use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::Deserialize;

use crate::db::{self, normalize_category, validate_category};

const SEED_JSON: &str = include_str!("../assets/prompts-seed.json");

#[derive(Debug, Deserialize)]
struct SeedBundle {
    version: u32,
    prompts: Vec<SeedPrompt>,
}

#[derive(Debug, Deserialize)]
struct SeedPrompt {
    name: String,
    description: String,
    category: String,
    content: String,
}

const SEED_VERSION: u32 = 1;

/// Upsert all built-in seed prompts by name. Returns the number of prompts processed.
pub fn upsert_seed_prompts(conn: &Connection) -> Result<usize> {
    let bundle: SeedBundle =
        serde_json::from_str(SEED_JSON).context("Failed to parse built-in seed prompts")?;
    if bundle.version != SEED_VERSION {
        anyhow::bail!(
            "Unsupported seed version {} (expected {SEED_VERSION})",
            bundle.version
        );
    }

    let mut count = 0usize;
    for prompt in bundle.prompts {
        let name = prompt.name.trim();
        let description = prompt.description.trim();
        let content = prompt.content.trim();
        let category = normalize_category(&prompt.category);
        validate_category(&category).map_err(|e| anyhow::anyhow!(e))?;

        if name.is_empty() || description.is_empty() || content.is_empty() {
            anyhow::bail!("Seed prompt has empty name, description, or content");
        }

        db::upsert_prompt(conn, name, description, content, &category)?;
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn upsert_seed_prompts_loads_all_templates() {
        let f = NamedTempFile::new().unwrap();
        let conn = db::init_db(f.path()).unwrap();
        let count = upsert_seed_prompts(&conn).unwrap();
        assert_eq!(count, 22);
        assert_eq!(db::prompt_count(&conn).unwrap(), 22);
    }

    #[test]
    fn upsert_seed_prompts_is_idempotent_by_name() {
        let f = NamedTempFile::new().unwrap();
        let conn = db::init_db(f.path()).unwrap();
        upsert_seed_prompts(&conn).unwrap();
        upsert_seed_prompts(&conn).unwrap();
        assert_eq!(db::prompt_count(&conn).unwrap(), 22);
    }

    #[test]
    fn seed_prompts_have_expected_categories() {
        let f = NamedTempFile::new().unwrap();
        let conn = db::init_db(f.path()).unwrap();
        upsert_seed_prompts(&conn).unwrap();
        let prompts = db::load_prompts(&conn).unwrap();
        let code_review = prompts
            .iter()
            .find(|p| p.name == "Code Review")
            .expect("Code Review seed");
        assert_eq!(code_review.category, "development");
        let image = prompts
            .iter()
            .find(|p| p.name == "Image Prompt Builder")
            .expect("Image Prompt Builder seed");
        assert_eq!(image.category, "image");
    }
}
