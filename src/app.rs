use crate::task::Task;
use crate::{db, greetings, task};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;
use rusqlite::Connection;
use std::collections::HashSet;

const HISTORY_PREVIEW: usize = 10;

#[derive(Clone, Copy, PartialEq)]
pub enum Action {
    Add,
    Complete,
    Delete,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    List,
    Add,
    Confirm(Action),
    History,
}

#[derive(Default)]
pub struct AddForm {
    pub name: String,
    pub description: String,
    pub due_date: String,
    pub focus: Field,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum Field {
    #[default]
    Name,
    Description,
    DueDate,
}

pub struct App {
    pub conn: Connection,
    pub screen: Screen,
    pub tasks: Vec<Task>,
    pub list_state: ListState,
    pub add_form: AddForm,
    pub status: Option<String>, // "✓ Task added" style messages
    pub greeting: Option<String>,
    pub should_quit: bool,
    pub history: Vec<Task>,
    pub selected: HashSet<i64>,
    pub history_counts: db::HistoryCounts,
    pub history_state: ListState,
}
impl App {
    pub fn new() -> Result<Self> {
        let conn = db::open_db()?;
        let mut app = Self {
            conn,
            screen: Screen::List,
            tasks: vec![],
            history: vec![],
            list_state: Default::default(),
            add_form: AddForm::default(),
            status: None,
            greeting: Some(greetings::greeting().to_string()),
            should_quit: false,
            selected: HashSet::new(),
            history_counts: Default::default(),
            history_state: Default::default(),
        };
        app.refresh_tasks()?;
        Ok(app)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        self.greeting = None;
        self.status = None;

        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return Ok(());
        }
        match self.screen {
            Screen::List => self.handle_list_key(key),
            Screen::Add => self.handle_add_key(key),
            Screen::Confirm(action) => self.handle_confirm_key(key, action),
            Screen::History => self.handle_history_key(key),
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc if !self.selected.is_empty() => self.selected.clear(),
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char(' ') => self.toggle_selected(),
            KeyCode::Char('a') | KeyCode::Char('A') => self.screen = Screen::Add,
            KeyCode::Char('c') => self.ask(Action::Complete),
            KeyCode::Char('d') => self.ask(Action::Delete),
            KeyCode::Char('h') => self.open_history()?,
            KeyCode::Down | KeyCode::Char('j') => self.list_state.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.select_previous(),
            _ => {}
        }
        Ok(())
    }

    fn handle_add_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char(c) => self.add_form.active_field().push(c),
            KeyCode::Backspace => {
                self.add_form.active_field().pop();
            }
            KeyCode::Esc => {
                self.add_form = AddForm::default();
                self.screen = Screen::List;
            }
            KeyCode::Down | KeyCode::Tab => {
                self.add_form.focus = match self.add_form.focus {
                    Field::Name => Field::Description,
                    Field::Description => Field::DueDate,
                    Field::DueDate => Field::Name,
                }
            }
            KeyCode::Up | KeyCode::BackTab => {
                self.add_form.focus = match self.add_form.focus {
                    Field::Description => Field::Name,
                    Field::DueDate => Field::Description,
                    Field::Name => Field::DueDate,
                }
            }
            KeyCode::Enter => {
                let due = self.add_form.due_date.trim();
                if self.add_form.name.trim().is_empty() {
                    self.status = Some("Task name cannot be empty".into());
                } else if !due.is_empty() && task::parse_user_date(due).is_none() {
                    self.status =
                        Some("Date not recognized, try 'tomorrow 5pm' or '25/12 18:00'".into());
                } else {
                    self.screen = Screen::Confirm(Action::Add);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_confirm_key(&mut self, key: KeyEvent, action: Action) -> Result<()> {
        match key.code {
            KeyCode::Char('y' | 'Y') | KeyCode::Enter => self.run_action(action)?,
            KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                self.status = Some("Cancelled, nothing changed".into());
                self.screen = if action == Action::Add {
                    Screen::Add
                } else {
                    Screen::List
                };
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_history_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::List,
            KeyCode::Char('a' | 'A') => self.load_history(true)?,
            KeyCode::Down | KeyCode::Char('j') => self.history_state.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.history_state.select_previous(),
            _ => {}
        }
        Ok(())
    }

    fn load_history(&mut self, all: bool) -> Result<()> {
        let limit = if all { None } else { Some(HISTORY_PREVIEW) };
        self.history = db::list_history(&self.conn, limit)?;
        self.history_counts = db::history_counts(&self.conn)?;
        Ok(())
    }

    fn open_history(&mut self) -> Result<()> {
        self.load_history(false)?;
        self.history_state = ListState::default();
        self.history_state.select_first(); // start at the newest entry
        self.screen = Screen::History;
        Ok(())
    }

    pub fn refresh_tasks(&mut self) -> Result<()> {
        self.tasks = db::list_pending(&self.conn)?;
        let tasks = &self.tasks;
        self.selected.retain(|id| tasks.iter().any(|t| t.id == *id));
        match (self.tasks.len(), self.list_state.selected()) {
            (0, _) => self.list_state.select(None),
            (n, Some(i)) if i >= n => self.list_state.select(Some(n - 1)),
            (_, None) => self.list_state.select(Some(0)),
            _ => {}
        }
        Ok(())
    }

    pub fn selected_task(&self) -> Option<&Task> {
        let index = self.list_state.selected()?;
        self.tasks.get(index)
    }

    fn toggle_selected(&mut self) {
        let Some(id) = self.selected_task().map(|t| t.id) else {
            return;
        };
        if !self.selected.insert(id) {
            self.selected.remove(&id);
        }
    }

    pub fn target_tasks(&self) -> Vec<&Task> {
        if self.selected.is_empty() {
            self.selected_task().into_iter().collect()
        } else {
            self.tasks
                .iter()
                .filter(|t| self.selected.contains(&t.id))
                .collect()
        }
    }

    fn ask(&mut self, action: Action) {
        if self.target_tasks().is_empty() {
            self.status = Some("Nothing selected".into());
        } else {
            self.screen = Screen::Confirm(action);
        }
    }

    fn run_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Add => {
                let form = &self.add_form;
                let due = task::parse_user_date(&form.due_date)
                    .map(|d| d.format(task::STORED_FORMAT).to_string());
                db::insert_task(
                    &self.conn,
                    form.name.trim(),
                    Self::none_if_empty(&form.description),
                    due.as_deref(),
                )?;
                self.add_form = AddForm::default();
                self.status = Some("✓ Task added".into());
            }
            Action::Complete | Action::Delete => {
                let targets: Vec<(i64, String)> = self
                    .target_tasks()
                    .into_iter()
                    .map(|t| (t.id, t.name.clone()))
                    .collect();
                for (id, _) in &targets {
                    if action == Action::Complete {
                        db::complete_task(&self.conn, *id)?;
                    } else {
                        db::delete_task(&self.conn, *id)?;
                    }
                }
                let names: Vec<&str> = targets.iter().map(|(_, n)| n.as_str()).collect();
                let verb = if action == Action::Complete {
                    "Completed"
                } else {
                    "Deleted"
                };
                self.status = Some(format!("✓ {verb}: {}", names.join(", ")));
                self.selected.clear();
            }
        }
        self.screen = Screen::List;
        self.refresh_tasks()
    }

    fn none_if_empty(s: &str) -> Option<&str> {
        let s = s.trim();
        if s.is_empty() { None } else { Some(s) }
    }
}

impl AddForm {
    pub fn active_field(&mut self) -> &mut String {
        match self.focus {
            Field::Name => &mut self.name,
            Field::Description => &mut self.description,
            Field::DueDate => &mut self.due_date,
        }
    }
}
