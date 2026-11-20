use crate::task::Task;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;

#[derive(Default)]
pub struct HistoryCounts {
    pub done: usize,
    pub not_done: usize,
}

impl HistoryCounts {
    pub fn total(&self) -> usize {
        self.done + self.not_done
    }
}

pub fn history_counts(conn: &Connection) -> Result<HistoryCounts> {
    let (done, total): (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(completed), 0), COUNT(*) FROM tasks",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    Ok(HistoryCounts {
        done: done as usize,
        not_done: (total - done) as usize,
    })
}

const COLUMNS: &str = "id, name, description, due_date, completed, created_at, updated_at";

fn get_db_path() -> Result<PathBuf> {
    let mut path = dirs::data_dir().context("Couldn't get data directory")?;
    path.push("st");
    std::fs::create_dir_all(&path)?;
    path.push("st.db");
    Ok(path)
}

pub fn open_db() -> Result<Connection> {
    let db = get_db_path()?;
    let conn = Connection::open(db)?;
    init_schema(&conn)?;
    Ok(conn)
}

pub fn list_pending(conn: &Connection) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM tasks WHERE completed = 0 ORDER BY due_date IS NULL, due_date, id"
    ))?;
    let tasks = stmt
        .query_map([], Task::from_row)?
        .collect::<rusqlite::Result<Vec<Task>>>()?;
    Ok(tasks)
}
pub fn insert_task(
    conn: &Connection,
    name: &str,
    description: Option<&str>,
    due_date: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO tasks (name, description, due_date) VALUES (?1, ?2, ?3)",
        (name, description, due_date),
    )?;
    Ok(())
}
pub fn complete_task(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE tasks SET completed = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        [id],
    )?;
    Ok(())
}

pub fn delete_task(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
    Ok(())
}
pub fn list_history(conn: &Connection, limit: Option<usize>) -> Result<Vec<Task>> {
    let limit: i64 = limit.map(|n| n as i64).unwrap_or(-1);
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM tasks ORDER BY id DESC LIMIT ?1"
    ))?;
    let tasks = stmt
        .query_map([limit], Task::from_row)?
        .collect::<rusqlite::Result<Vec<Task>>>()?;
    Ok(tasks)
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (\
        id INTEGER PRIMARY KEY, \
        name TEXT NOT NULL, \
        description TEXT DEFAULT NULL, \
        due_date TEXT DEFAULT NULL, \
        completed BOOLEAN DEFAULT 0, \
        created_at TEXT DEFAULT CURRENT_TIMESTAMP, \
        updated_at TEXT DEFAULT CURRENT_TIMESTAMP \
        )",
        (),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_limit_and_counts() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        init_schema(&conn)?;
        for i in 0..15 {
            insert_task(&conn, &format!("task {i}"), None, None)?;
        }
        assert_eq!(list_history(&conn, Some(10))?.len(), 10);
        assert_eq!(list_history(&conn, None)?.len(), 15);

        let counts = history_counts(&conn)?;
        assert_eq!((counts.done, counts.not_done), (0, 15));

        let first = list_pending(&conn)?[0].id;
        complete_task(&conn, first)?;
        let counts = history_counts(&conn)?;
        assert_eq!((counts.done, counts.not_done), (1, 14));
        assert_eq!(counts.total(), 15);
        Ok(())
    }
}
