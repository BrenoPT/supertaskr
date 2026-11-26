# supertaskr

A fast, keyboard-driven task manager for your terminal, written in Rust.

`supertaskr` is a Rust rewrite of my earlier Python CLI to-do app. It keeps everything in a local SQLite database, understands natural-language due dates like "tomorrow 5pm", and makes sure you notice what's due soon or overdue.

<!-- Add a screenshot or GIF here: ![supertaskr screenshot](docs/screenshot.png) -->

## Features

- **Full-screen terminal UI** built with [ratatui](https://ratatui.rs): task list, add form, history and confirmation popups
- **Natural-language due dates**: type `tomorrow 5pm` or `next friday` and it is saved as `YYYY-MM-DD HH:MM`
- **Due soon and overdue awareness**: tasks due within 2 days turn yellow, overdue ones turn red, and a banner at the top keeps them in view
- **Multi-select**: mark several tasks and complete or delete them in one go
- **History**: shows recent entries, press `a` to reveal full scrollable history
- **Local and private**: one SQLite file, no account, no network

## Installation

You need a recent stable [Rust toolchain](https://rustup.rs).

```bash
git clone https://github.com/BrenoPT/supertaskr.git
cd supertaskr
cargo run --release
```

To install it as a command available from anywhere:

```bash
cargo install --path .
```

It is built on [crossterm](https://github.com/crossterm-rs/crossterm), so it should work in Linux, macOS and Windows terminals.

## Usage

Start by running `st`.

### Task list

| Key | Action |
| --- | --- |
| `↑` `↓` or `k` `j` | Move the selection |
| `spacebar` | Mark / unmark the highlighted task |
| `a` | Add a task |
| `c` | Complete the marked tasks (or the highlighted one) |
| `d` | Delete the marked tasks (or the highlighted one) |
| `h` | Open the history |
| `Esc` | Clear the marks if there are any, otherwise quit |
| `q` | Quit |
| `Ctrl+C` | Quit from anywhere |

### Add task

| Key | Action |
| --- | --- |
| `Tab` or `↓` | Next field |
| `Shift+Tab` or `↑` | Previous field |
| `Enter` | Save (asks for confirmation first) |
| `Esc` | Cancel |

### Confirmation popup

| Key | Action |
| --- | --- |
| `y` or `Enter` | Yes |
| `n` or `Esc` | No |

### History

| Key | Action |
| --- | --- |
| `↑` `↓` or `k` `j` | Scroll |
| `a` | Show all history |
| `Esc` or `q` | Back to the list |

## Due dates

Type whatever feels natural in the due date field, or leave it empty for no due date:

```
tomorrow 5pm
next friday
2026-12-31 18:00
```

Dates are understood with the day first (UK style), so `03/04` means 3 April. If a date can't be understood, you'll be told before anything is saved. Whatever you type is normalized to `YYYY-MM-DD HH:MM` before it reaches the database.

## Where your data lives

Tasks are stored in a single SQLite file, `st.db`, inside an `st` folder in your system's data directory:

| OS | Path |
| --- | --- |
| Linux | `~/.local/share/st/st.db` |
| macOS | `~/Library/Application Support/st/st.db` |
| Windows | `%APPDATA%\st\st.db` |

To back up your tasks, copy that file.

## Project structure

```
src/
├── main.rs         # terminal setup and the main event loop
├── app.rs          # application state, screens and key handling
├── ui.rs           # all rendering code, one function per screen
├── db.rs           # SQLite access: schema, queries and tests
├── task.rs         # Task model and due-date logic
└── greetings.rs    # random greetings and goodbyes
```

## Built with

- [ratatui](https://ratatui.rs) and [crossterm](https://github.com/crossterm-rs/crossterm) for the terminal UI
- [rusqlite](https://github.com/rusqlite/rusqlite) for SQLite
- [chrono](https://github.com/chronotope/chrono) and [chrono-english](https://github.com/stevedonovan/chrono-english) for dates
- [dirs](https://github.com/dirs-dev/dirs-rs), [anyhow](https://github.com/dtolnay/anyhow) and [rand](https://github.com/rust-random/rand)

## Roadmap

- [ ] Add text navigation
- [ ] Choose your date format (day-first or month-first)
- [ ] Localization
- [ ] Themes

## License

Copyright (c) Breno Paim Targa <brenotarga@proton.me>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
