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
    pub description: String,
    pub content: String,
}

pub fn init_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path).context("Failed to open SQLite database")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS prompts (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT UNIQUE NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            content     TEXT NOT NULL
        );",
    )
    .context("Failed to create prompts table")?;
    ensure_description_column(&conn)?;
    Ok(conn)
}

fn ensure_description_column(conn: &Connection) -> Result<()> {
    let has_description = conn
        .prepare("PRAGMA table_info(prompts)")?
        .query_map(params![], |row| row.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<_>, _>>()?
        .iter()
        .any(|column| column == "description");

    if !has_description {
        conn.execute(
            "ALTER TABLE prompts ADD COLUMN description TEXT NOT NULL DEFAULT ''",
            params![],
        )?;
    }

    Ok(())
}

pub fn upsert_prompt(
    conn: &Connection,
    name: &str,
    description: &str,
    content: &str,
) -> Result<i64> {
    let id = conn.query_row(
        "INSERT INTO prompts (name, description, content) VALUES (?1, ?2, ?3)
         ON CONFLICT(name) DO UPDATE SET description=?2, content=?3
         RETURNING id",
        params![name, description, content],
        |row| row.get::<_, i64>(0),
    )?;
    Ok(id)
}

pub fn update_prompt(
    conn: &Connection,
    id: i64,
    name: &str,
    description: &str,
    content: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE prompts SET name = ?1, description = ?2, content = ?3 WHERE id = ?4",
        params![name, description, content, id],
    )?;
    Ok(())
}

pub fn delete_prompt(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM prompts WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn load_prompts(conn: &Connection) -> Result<Vec<Prompt>> {
    let mut stmt =
        conn.prepare("SELECT id, name, description, content FROM prompts ORDER BY name ASC")?;
    let prompts = stmt
        .query_map(params![], |row| {
            Ok(Prompt {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                content: row.get(3)?,
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
        "SELECT id, name, description, content FROM prompts
         WHERE name LIKE ?1 OR description LIKE ?1 OR content LIKE ?1
         ORDER BY name ASC",
    )?;
    let prompts = stmt
        .query_map(params![pattern], |row| {
            Ok(Prompt {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                content: row.get(3)?,
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
        upsert_prompt(
            &conn,
            "greeting",
            "Friendly greeting",
            "Hello {{name|text|world|}}",
        )
        .unwrap();
        let prompts = load_prompts(&conn).unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].name, "greeting");
        assert_eq!(prompts[0].description, "Friendly greeting");
    }

    #[test]
    fn test_update() {
        let (conn, _f) = test_db();
        let id = upsert_prompt(&conn, "draft", "old description", "old content").unwrap();
        update_prompt(&conn, id, "final", "new description", "new content").unwrap();
        let prompts = load_prompts(&conn).unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].name, "final");
        assert_eq!(prompts[0].description, "new description");
        assert_eq!(prompts[0].content, "new content");
    }

    #[test]
    fn test_delete() {
        let (conn, _f) = test_db();
        let id = upsert_prompt(&conn, "temp", "temporary", "content").unwrap();
        delete_prompt(&conn, id).unwrap();
        assert!(load_prompts(&conn).unwrap().is_empty());
    }

    #[test]
    fn test_search() {
        let (conn, _f) = test_db();
        upsert_prompt(&conn, "greet", "welcomes people", "Hello {{name}}").unwrap();
        upsert_prompt(
            &conn,
            "farewell",
            "goodbye flow",
            "Bye {{name|text|friend|}}",
        )
        .unwrap();
        let results = search_prompts(&conn, "welcomes").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "greet");
        let all = search_prompts(&conn, "").unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_init_migrates_existing_database() {
        let f = NamedTempFile::new().unwrap();
        let conn = Connection::open(f.path()).unwrap();
        conn.execute_batch(
            "CREATE TABLE prompts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                content TEXT NOT NULL
            );
            INSERT INTO prompts (name, content) VALUES ('legacy', 'content');",
        )
        .unwrap();
        drop(conn);

        let conn = init_db(f.path()).unwrap();
        let prompts = load_prompts(&conn).unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].description, "");
    }
}
