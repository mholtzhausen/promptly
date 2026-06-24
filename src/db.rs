//! SQLite database layer for prompt templates and copy history.

use anyhow::{Context, Result};
use rusqlite::params;
pub use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const TITLE_VALUE_MAX_LEN: usize = 40;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryVariable {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryListItem {
    pub id: i64,
    pub title: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub variables: Vec<HistoryVariable>,
    pub prompt_id: Option<i64>,
    pub prompt_name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryListResult {
    pub entries: Vec<HistoryListItem>,
    pub total_count: i64,
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
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS prompt_history (
            id             INTEGER PRIMARY KEY AUTOINCREMENT,
            content_hash   TEXT NOT NULL UNIQUE,
            title          TEXT NOT NULL,
            content        TEXT NOT NULL,
            variables_json TEXT NOT NULL,
            prompt_id      INTEGER,
            prompt_name    TEXT NOT NULL,
            created_at     INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_prompt_history_created_at
            ON prompt_history (created_at DESC);",
    )
    .context("Failed to create prompt_history table")?;
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

pub fn content_hash(content: &str) -> String {
    let digest = Sha256::digest(content.as_bytes());
    format!("{digest:x}")
}

fn truncate_title_value(value: &str) -> String {
    if value.chars().count() <= TITLE_VALUE_MAX_LEN {
        value.to_string()
    } else {
        let truncated: String = value.chars().take(TITLE_VALUE_MAX_LEN).collect();
        format!("{truncated}…")
    }
}

pub fn build_history_title(prompt_name: &str, values: &[HistoryVariable]) -> String {
    if values.is_empty() {
        return format!("[{prompt_name}]()");
    }
    let pairs: Vec<String> = values
        .iter()
        .map(|v| format!("{}:{}", v.name, truncate_title_value(&v.value)))
        .collect();
    format!("[{prompt_name}]({})", pairs.join(", "))
}

fn unix_now_secs() -> Result<i64> {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System clock is before Unix epoch")?
        .as_secs();
    i64::try_from(secs).context("Timestamp overflow")
}

/// Insert a history row when the content hash is new.
/// Returns `(inserted, total_count)`.
pub fn insert_history_if_new(
    conn: &Connection,
    content: &str,
    prompt_id: Option<i64>,
    prompt_name: &str,
    values: &[HistoryVariable],
) -> Result<(bool, i64)> {
    let hash = content_hash(content);
    let title = build_history_title(prompt_name, values);
    let variables_json = serde_json::to_string(values).context("Failed to serialize variables")?;
    let created_at = unix_now_secs()?;

    let inserted = conn
        .execute(
            "INSERT OR IGNORE INTO prompt_history
             (content_hash, title, content, variables_json, prompt_id, prompt_name, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                hash,
                title,
                content,
                variables_json,
                prompt_id,
                prompt_name,
                created_at
            ],
        )?
        > 0;

    let total_count: i64 = conn.query_row("SELECT COUNT(*) FROM prompt_history", [], |row| {
        row.get(0)
    })?;

    Ok((inserted, total_count))
}

pub fn list_history(conn: &Connection) -> Result<HistoryListResult> {
    let total_count = history_count(conn)?;

    let mut stmt = conn.prepare(
        "SELECT id, title, created_at FROM prompt_history ORDER BY created_at DESC",
    )?;
    let entries = stmt
        .query_map([], |row| {
            Ok(HistoryListItem {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .context("Failed to query history")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to map history rows")?;

    Ok(HistoryListResult {
        entries,
        total_count,
    })
}

pub fn get_history_entry(conn: &Connection, id: i64) -> Result<Option<HistoryEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, content, variables_json, prompt_id, prompt_name, created_at
         FROM prompt_history WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![id])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };

    let variables_json: String = row.get(3)?;
    let variables: Vec<HistoryVariable> =
        serde_json::from_str(&variables_json).context("Failed to parse history variables")?;

    Ok(Some(HistoryEntry {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        variables,
        prompt_id: row.get(4)?,
        prompt_name: row.get(5)?,
        created_at: row.get(6)?,
    }))
}

pub fn update_history_entry(conn: &Connection, id: i64, content: &str) -> Result<()> {
    let hash = content_hash(content);
    let updated = conn.execute(
        "UPDATE prompt_history SET content = ?1, content_hash = ?2 WHERE id = ?3",
        params![content, hash, id],
    )?;
    if updated == 0 {
        anyhow::bail!("History entry {id} not found");
    }
    Ok(())
}

pub fn delete_history_entry(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM prompt_history WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn prune_history_keep_last(conn: &Connection, keep: i64) -> Result<()> {
    if keep <= 0 {
        conn.execute("DELETE FROM prompt_history", [])?;
        return Ok(());
    }
    conn.execute(
        "DELETE FROM prompt_history
         WHERE id NOT IN (
             SELECT id FROM prompt_history
             ORDER BY created_at DESC
             LIMIT ?1
         )",
        params![keep],
    )?;
    Ok(())
}

pub fn history_count(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM prompt_history", [], |row| row.get(0))
        .context("Failed to count history rows")
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

    #[test]
    fn test_build_history_title_no_variables() {
        assert_eq!(build_history_title("Code Prompt", &[]), "[Code Prompt]()");
    }

    #[test]
    fn test_build_history_title_with_variables() {
        let values = vec![
            HistoryVariable {
                name: "branch".to_string(),
                value: "main".to_string(),
            },
            HistoryVariable {
                name: "message".to_string(),
                value: "fix bug".to_string(),
            },
        ];
        assert_eq!(
            build_history_title("git-commit", &values),
            "[git-commit](branch:main, message:fix bug)"
        );
    }

    #[test]
    fn test_build_history_title_truncates_long_values() {
        let long = "a".repeat(50);
        let values = vec![HistoryVariable {
            name: "body".to_string(),
            value: long,
        }];
        let title = build_history_title("tpl", &values);
        assert!(title.contains("…"));
        assert!(title.len() < 60);
    }

    #[test]
    fn test_insert_history_dedup_by_hash() {
        let (conn, _f) = test_db();
        let values = vec![HistoryVariable {
            name: "name".to_string(),
            value: "Alice".to_string(),
        }];

        let (inserted1, count1) =
            insert_history_if_new(&conn, "Hello Alice", Some(1), "greeting", &values).unwrap();
        assert!(inserted1);
        assert_eq!(count1, 1);

        let (inserted2, count2) =
            insert_history_if_new(&conn, "Hello Alice", Some(1), "greeting", &values).unwrap();
        assert!(!inserted2);
        assert_eq!(count2, 1);

        let list = list_history(&conn).unwrap();
        assert_eq!(list.entries.len(), 1);
        assert_eq!(list.entries[0].title, "[greeting](name:Alice)");
    }

    #[test]
    fn test_list_history_order_newest_first() {
        let (conn, _f) = test_db();
        conn.execute(
            "INSERT INTO prompt_history
             (content_hash, title, content, variables_json, prompt_id, prompt_name, created_at)
             VALUES ('hash1', 'older', 'c1', '[]', NULL, 'a', 100)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO prompt_history
             (content_hash, title, content, variables_json, prompt_id, prompt_name, created_at)
             VALUES ('hash2', 'newer', 'c2', '[]', NULL, 'b', 200)",
            [],
        )
        .unwrap();

        let list = list_history(&conn).unwrap();
        assert_eq!(list.entries[0].title, "newer");
        assert_eq!(list.entries[1].title, "older");
    }

    #[test]
    fn test_get_delete_and_prune_history() {
        let (conn, _f) = test_db();
        let (inserted, _) =
            insert_history_if_new(&conn, "content", None, "tpl", &[]).unwrap();
        assert!(inserted);

        let list = list_history(&conn).unwrap();
        let id = list.entries[0].id;

        let entry = get_history_entry(&conn, id).unwrap().unwrap();
        assert_eq!(entry.content, "content");
        assert_eq!(entry.prompt_name, "tpl");

        update_history_entry(&conn, id, "updated content").unwrap();
        let updated = get_history_entry(&conn, id).unwrap().unwrap();
        assert_eq!(updated.content, "updated content");

        delete_history_entry(&conn, id).unwrap();
        assert!(get_history_entry(&conn, id).unwrap().is_none());

        insert_history_if_new(&conn, "a", None, "t1", &[]).unwrap();
        insert_history_if_new(&conn, "b", None, "t2", &[]).unwrap();
        prune_history_keep_last(&conn, 0).unwrap();
        assert_eq!(history_count(&conn).unwrap(), 0);
    }

    #[test]
    fn test_prune_history_keep_last() {
        let (conn, _f) = test_db();
        for i in 0..5 {
            conn.execute(
                "INSERT INTO prompt_history
                 (content_hash, title, content, variables_json, prompt_id, prompt_name, created_at)
                 VALUES (?1, ?2, ?3, '[]', NULL, 'tpl', ?4)",
                params![
                    format!("hash{i}"),
                    format!("title{i}"),
                    format!("content{i}"),
                    i
                ],
            )
            .unwrap();
        }
        prune_history_keep_last(&conn, 2).unwrap();
        assert_eq!(history_count(&conn).unwrap(), 2);
        let list = list_history(&conn).unwrap();
        assert_eq!(list.entries.len(), 2);
        assert_eq!(list.entries[0].title, "title4");
        assert_eq!(list.entries[1].title, "title3");
    }
}
