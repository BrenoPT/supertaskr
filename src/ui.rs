use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, HighlightSpacing, List, ListItem, Paragraph};

use crate::app::{Action, App, Field, Screen};
use crate::task::DueStatus;

pub fn render(app: &mut App, frame: &mut Frame) {
    let show_banner = matches!(
        app.screen,
        Screen::List | Screen::Confirm(Action::Complete | Action::Delete)
    );

    let banner = if show_banner {
        banner_lines(app)
    } else {
        vec![]
    };
    let banner_height = if banner.is_empty() {
        0
    } else {
        banner.len() as u16 + 2
    };

    let [header, banner_area, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(banner_height),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    draw_header(app, frame, header);

    if !banner.is_empty() {
        let overdue = app
            .tasks
            .iter()
            .any(|t| t.due_status() == DueStatus::Overdue);
        let border = if overdue { Color::Red } else { Color::Yellow };
        frame.render_widget(
            Paragraph::new(banner).block(
                Block::bordered()
                    .title("Due soon / overdue")
                    .border_style(Style::new().fg(border)),
            ),
            banner_area,
        );
    }

    match app.screen {
        Screen::List => draw_list(app, frame, body),
        Screen::Add => draw_add(app, frame, body),
        Screen::History => draw_history(app, frame, body),
        Screen::Confirm(action) => {
            match action {
                Action::Add => draw_add(app, frame, body), // form stays visible underneath
                _ => draw_list(app, frame, body),
            }
            draw_popup(app, frame, body, action); // popup drawn last = on top
        }
    }

    draw_footer(app, frame, footer);
}

fn draw_header(app: &App, frame: &mut Frame, area: Rect) {
    let text = match &app.greeting {
        Some(greeting) => greeting.clone(),
        None => format!("Supertaskr ({} pending)", app.tasks.len()),
    };
    frame.render_widget(Paragraph::new(text), area);
}
fn draw_footer(app: &App, frame: &mut Frame, area: Rect) {
    // match on a pair: (status message, current screen)
    let text = match (&app.status, app.screen) {
        (Some(status), _) => status.as_str(),
        (None, Screen::List) => {
            "a add · Spacebar select · c complete · d delete · h history · q quit"
        }
        (None, Screen::Add) => "↑/↓ change field · Enter save · Esc cancel",
        (None, Screen::Confirm(_)) => "y / Enter yes · n / Esc no",
        (None, Screen::History) => "q / Esc back",
    };
    frame.render_widget(
        Paragraph::new(text).style(Style::new().add_modifier(Modifier::DIM)),
        area,
    );
}

fn draw_history(app: &mut App, frame: &mut Frame, area: Rect) {
    let counts = &app.history_counts;
    let hidden = counts.total().saturating_sub(app.history.len());

    let [list_area, more_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(if hidden > 0 { 1 } else { 0 }),
    ])
    .areas(area);

    let items: Vec<ListItem> = app
        .history
        .iter()
        .map(|task| {
            let style = if task.completed {
                Style::new().fg(Color::Green)
            } else {
                Style::new()
            };
            let mark = if task.completed { "✓" } else { "✗" };
            ListItem::new(format!("[{mark}] {}", task.name)).style(style)
        })
        .collect();

    let title = format!(
        "History: {} done, {} not done",
        counts.done, counts.not_done
    );

    let list = List::new(items)
        .block(Block::bordered().title(title))
        .highlight_style(
            Style::new()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_stateful_widget(list, list_area, &mut app.history_state);

    if hidden > 0 {
        frame.render_widget(
            Paragraph::new(format!("+{hidden} more · press a to show all"))
                .style(Style::new().fg(Color::Blue)),
            more_area,
        );
    }
}

fn draw_popup(app: &App, frame: &mut Frame, area: Rect, action: Action) {
    let question = match action {
        Action::Add => "Create this task? (Y/n)".to_string(),
        Action::Complete | Action::Delete => {
            let verb = if action == Action::Complete {
                "Complete"
            } else {
                "Delete"
            };
            let targets = app.target_tasks();
            let what = if targets.len() == 1 {
                format!("\"{}\"", targets[0].name)
            } else {
                format!("{} tasks", targets.len())
            };
            format!("{verb} {what}? (Y/n)")
        }
    };
    let popup = centered(area, 46, 3);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(question).block(Block::bordered().title("Confirm")),
        popup,
    );
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    area
}

fn draw_add(app: &App, frame: &mut Frame, area: Rect) {
    let form = &app.add_form;
    let [name_area, desc_area, due_area] = Layout::vertical([Constraint::Length(3); 3]).areas(area);
    let boxes = [
        ("Name", &form.name, Field::Name, name_area),
        (
            "Description",
            &form.description,
            Field::Description,
            desc_area,
        ),
        (
            "Due date (empty for none)",
            &form.due_date,
            Field::DueDate,
            due_area,
        ),
    ];
    for (title, text, field, rect) in boxes {
        let focused = form.focus == field;
        let border = if focused {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new()
        };
        let widget = Paragraph::new(text.as_str())
            .block(Block::bordered().title(title).border_style(border));
        frame.render_widget(widget, rect);
        if focused {
            frame.set_cursor_position((rect.x + 1 + text.chars().count() as u16, rect.y + 1));
        }
    }
}
fn draw_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::bordered().title("Tasks");

    if app.tasks.is_empty() {
        frame.render_widget(
            Paragraph::new("✓ All clear! Nothing left to do.").block(block),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .map(|task| {
            let style = match task.due_status() {
                DueStatus::Overdue => Style::new().fg(Color::Red),
                DueStatus::Soon => Style::new().fg(Color::Yellow),
                _ => Style::new(),
            };
            let mark = if app.selected.contains(&task.id) {
                "● "
            } else {
                "  "
            };
            ListItem::new(vec![
                Line::styled(format!("{mark}{}", task.name), style),
                Line::styled(
                    format!(
                        "   {}",
                        task.description.as_deref().unwrap_or("(No description)")
                    ),
                    style,
                ),
                Line::styled(format!("   {}", task.due_text()), style),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_symbol("> ")
        .highlight_style(
            Style::new()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(list, area, &mut app.list_state);
}
fn banner_lines(app: &App) -> Vec<Line<'static>> {
    const MAX_BANNER_LINES: usize = 5;

    let mut lines: Vec<Line> = Vec::new();

    for status in [DueStatus::Overdue, DueStatus::Soon] {
        let color = if status == DueStatus::Overdue {
            Color::Red
        } else {
            Color::Yellow
        };
        for task in app.tasks.iter().filter(|t| t.due_status() == status) {
            lines.push(Line::styled(
                format!("{}  ·  {}", task.name, task.due_text()),
                Style::new().fg(color),
            ));
        }
    }

    if lines.len() > MAX_BANNER_LINES {
        let hidden = lines.len() - (MAX_BANNER_LINES - 1);
        lines.truncate(MAX_BANNER_LINES - 1);
        lines.push(Line::from(format!("+{hidden} more")));
    }
    lines
}
