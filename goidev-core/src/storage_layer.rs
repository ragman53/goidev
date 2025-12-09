use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordEntry {
    pub id: Option<i64>,
    pub word: String,
    pub base_form: String,
    pub sentence: String,
    pub source_doc: Option<String>,
    pub page_num: Option<u32>,
    pub created_at: i64,
    pub review_count: u32,
    pub next_review: Option<i64>,
    pub ease_factor: f32,
}

pub fn init_db(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    create_tables(&conn)?;
    Ok(conn)
}

pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "BEGIN;
        CREATE TABLE IF NOT EXISTS words (
            id INTEGER PRIMARY KEY,
            word TEXT NOT NULL,
            base_form TEXT NOT NULL,
            sentence TEXT NOT NULL,
            source_doc TEXT,
            page_num INTEGER,
            created_at INTEGER NOT NULL,
            review_count INTEGER NOT NULL DEFAULT 0,
            next_review INTEGER,
            ease_factor REAL NOT NULL DEFAULT 2.5
        );
        CREATE INDEX IF NOT EXISTS idx_words_base_form ON words(base_form);
        CREATE INDEX IF NOT EXISTS idx_words_next_review ON words(next_review);
        COMMIT;",
    )?;
    Ok(())
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn save_word(conn: &Connection, mut entry: WordEntry) -> Result<WordEntry> {
    let created_at = if entry.created_at == 0 {
        now_ts()
    } else {
        entry.created_at
    };
    conn.execute(
        "INSERT INTO words (word, base_form, sentence, source_doc, page_num, created_at, review_count, next_review, ease_factor)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            entry.word,
            entry.base_form,
            entry.sentence,
            entry.source_doc,
            entry.page_num.map(|p| p as i64),
            created_at,
            entry.review_count as i64,
            entry.next_review,
            entry.ease_factor
        ],
    )?;
    entry.id = Some(conn.last_insert_rowid());
    entry.created_at = created_at;
    Ok(entry)
}

pub fn get_vocabulary(conn: &Connection) -> Result<Vec<WordEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, word, base_form, sentence, source_doc, page_num, created_at, review_count, next_review, ease_factor
         FROM words ORDER BY created_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(WordEntry {
            id: Some(row.get(0)?),
            word: row.get(1)?,
            base_form: row.get(2)?,
            sentence: row.get(3)?,
            source_doc: row.get(4)?,
            page_num: row.get::<_, Option<i64>>(5)?.map(|v| v as u32),
            created_at: row.get(6)?,
            review_count: row.get::<_, i64>(7)? as u32,
            next_review: row.get(8)?,
            ease_factor: row.get(9)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get_by_base_form(conn: &Connection, base: &str) -> Result<Vec<WordEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, word, base_form, sentence, source_doc, page_num, created_at, review_count, next_review, ease_factor
         FROM words WHERE base_form = ?1 ORDER BY created_at DESC",
    )?;

    let rows = stmt.query_map([base], |row| {
        Ok(WordEntry {
            id: Some(row.get(0)?),
            word: row.get(1)?,
            base_form: row.get(2)?,
            sentence: row.get(3)?,
            source_doc: row.get(4)?,
            page_num: row.get::<_, Option<i64>>(5)?.map(|v| v as u32),
            created_at: row.get(6)?,
            review_count: row.get::<_, i64>(7)? as u32,
            next_review: row.get(8)?,
            ease_factor: row.get(9)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn update_review(
    conn: &Connection,
    id: i64,
    next_review: Option<i64>,
    review_count: u32,
    ease_factor: f32,
) -> Result<()> {
    conn.execute(
        "UPDATE words SET next_review = ?1, review_count = ?2, ease_factor = ?3 WHERE id = ?4",
        params![next_review, review_count as i64, ease_factor, id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_query_word() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        let entry = WordEntry {
            id: None,
            word: "running".to_string(),
            base_form: "run".to_string(),
            sentence: "I am running fast.".to_string(),
            source_doc: Some("test-doc".to_string()),
            page_num: Some(3),
            created_at: 0,
            review_count: 0,
            next_review: None,
            ease_factor: 2.5,
        };

        let saved = save_word(&conn, entry).unwrap();
        assert!(saved.id.is_some());

        let all = get_vocabulary(&conn).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].base_form, "run");

        let by_base = get_by_base_form(&conn, "run").unwrap();
        assert_eq!(by_base.len(), 1);

        // Update review
        let id = all[0].id.unwrap();
        update_review(&conn, id, Some(1_700_000_000), 1, 2.6).unwrap();

        let updated = get_vocabulary(&conn).unwrap();
        assert_eq!(updated[0].review_count, 1);
        assert!((updated[0].ease_factor - 2.6).abs() < 1e-6);
    }
}
