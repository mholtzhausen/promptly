//! SQLite database layer for prompt templates.

use anyhow::{Context, Result};
use rusqlite::params;
pub use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: i64,
    pub name: String,
    pub content: String,
}

pub fn init_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path).context("Failed to open SQLite database")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS prompts (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            content TEXT NOT NULL
        );",
    )
    .context("Failed to create prompts table")?;
    Ok(conn)
}

pub fn upsert_prompt(conn: &Connection, name: &str, content: &str) -> Result<i64> {
    let id = conn.query_row(
        "INSERT INTO prompts (name, content) VALUES (?1, ?2)
         ON CONFLICT(name) DO UPDATE SET content=?2
         RETURNING id",
        params![name, content],
        |row| row.get::<_, i64>(0),
    )?;
    Ok(id)
}

#[allow(dead_code)]
pub fn delete_prompt(conn: &Connection, name: &str) -> Result<()> {
    conn.execute("DELETE FROM prompts WHERE name = ?1", params![name])?;
    Ok(())
}

pub fn load_prompts(conn: &Connection) -> Result<Vec<Prompt>> {
    let mut stmt = conn.prepare("SELECT id, name, content FROM prompts ORDER BY name ASC")?;
    let prompts = stmt
        .query_map(params![], |row| {
            Ok(Prompt {
                id: row.get(0)?,
                name: row.get(1)?,
                content: row.get(2)?,
            })
        })
        .context("Failed to query prompts")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to map prompt rows")?;
    Ok(prompts)
}

#[allow(dead_code)]
pub fn search_prompts(conn: &Connection, query: &str) -> Result<Vec<Prompt>> {
    if query.is_empty() {
        return load_prompts(conn);
    }
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, name, content FROM prompts
         WHERE name LIKE ?1 OR content LIKE ?1
         ORDER BY name ASC",
    )?;
    let prompts = stmt
        .query_map(params![pattern], |row| {
            Ok(Prompt {
                id: row.get(0)?,
                name: row.get(1)?,
                content: row.get(2)?,
            })
        })
        .context("Failed to query search results")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to map search rows")?;
    Ok(prompts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn test_db() -> (Connection, NamedTempFile) {
        let f = NamedTempFile::new().unwrap();
        let path = f.path().to_path_buf();
        let conn = init_db(&path).unwrap();
        (conn, f)
    }

    #[test]
    fn test_upsert_and_load() {
        let (conn, _f) = test_db();
        upsert_prompt(&conn, "greeting", "Hello {{name|text|world|}}").unwrap();
        let prompts = load_prompts(&conn).unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].name, "greeting");
    }

    #[test]
    fn test_delete() {
        let (conn, _f) = test_db();
        upsert_prompt(&conn, "temp", "content").unwrap();
        delete_prompt(&conn, "temp").unwrap();
        assert!(load_prompts(&conn).unwrap().is_empty());
    }

    #[test]
    fn test_search() {
        let (conn, _f) = test_db();
        upsert_prompt(&conn, "greet", "Hello {{name}}").unwrap();
        upsert_prompt(&conn, "farewell", "Bye {{name|text|friend|}}").unwrap();
        let results = search_prompts(&conn, "gre").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "greet");
        let all = search_prompts(&conn, "").unwrap();
        assert_eq!(all.len(), 2);
    }
}
