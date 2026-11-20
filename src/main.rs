mod app;
mod db;
mod greetings;
mod task;
mod ui;

use crate::app::App;
use anyhow::Result;
use crossterm::event;
use crossterm::event::{Event, KeyEventKind};
use ratatui::DefaultTerminal;
use std::time::Duration;

fn main() -> Result<()> {
    ratatui::run(run)?;
    println!("{}", greetings::goodbye());
    Ok(())
}

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut app = App::new()?;
    while !app.should_quit {
        terminal.draw(|frame| ui::render(&mut app, frame))?;
        if event::poll(Duration::from_secs(30))? {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                App::handle_key(&mut app, key)?;
            }
        }
    }
    Ok(())
}
