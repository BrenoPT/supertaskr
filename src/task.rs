use chrono::{DateTime, Local, NaiveDateTime, TimeDelta};
use chrono_english::{Dialect, parse_date_string};

pub const STORED_FORMAT: &str = "%Y-%m-%d %H:%M";

#[derive(Debug)]
pub struct Task {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub completed: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum DueStatus {
    NoDate,
    Later,
    Soon,
    Overdue,
}

pub fn parse_user_date(input: &str) -> Option<DateTime<Local>> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    parse_date_string(input, Local::now(), Dialect::Uk).ok()
}

impl Task {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Task {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            due_date: row.get(3)?,
            completed: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }

    pub fn due(&self) -> Option<DateTime<Local>> {
        let s = self.due_date.as_deref()?;
        NaiveDateTime::parse_from_str(s, STORED_FORMAT)
            .ok()?
            .and_local_timezone(Local)
            .single()
    }

    pub fn due_status(&self) -> DueStatus {
        let Some(due) = self.due() else {
            return DueStatus::NoDate;
        };
        let delta = due - Local::now();
        if delta < TimeDelta::zero() {
            DueStatus::Overdue
        } else if delta <= TimeDelta::days(2) {
            DueStatus::Soon
        } else {
            DueStatus::Later
        }
    }

    pub fn due_text(&self) -> String {
        self.due_date.clone().unwrap_or("(No due date)".into())
    }
}
