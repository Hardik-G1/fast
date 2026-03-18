// fast hist              → TUI picker of folder-specific history; Enter runs command
// fast hist --add <cmd>  → record a command (called by shell hook)
// fast hist --init       → print setup instructions
//
// Storage: ~/.fast_hist — one line per unique command per directory
// Format:  directory\tcount\tcommand
// Sorted by count desc, max 50 entries per directory.

use std::io::stderr;
use std::path::PathBuf;
use ratatui::prelude::CrosstermBackend;
use ratatui::{Terminal, crossterm::{
    ExecutableCommand, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}
}};
use ratatui::layout::{Layout, Constraint};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::style::{Color, Modifier, Style};
use std::time::Duration;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers, KeyEventKind};

fn hist_file() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();
    PathBuf::from(home).join(".fast_hist")
}

struct Entry {
    dir:   String,
    count: usize,
    cmd:   String,
}

fn load_all() -> Vec<Entry> {
    let content = std::fs::read_to_string(hist_file()).unwrap_or_default();
    content.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| {
            let mut parts = line.splitn(3, '\t'); // this gives 3 next which is below
            let dir   = parts.next()?.trim().to_string();
            let count = parts.next()?.trim().parse::<usize>().ok()?;
            let cmd   = parts.next()?.trim().to_string();
            if cmd.is_empty() { return None; }
            Some(Entry { dir, count, cmd })
        })
        .collect()
}

fn save_all(entries: &[Entry]) {
    let content: String = entries.iter()
        .map(|e| format!("{}\t{}\t{}", e.dir, e.count, e.cmd))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = std::fs::write(hist_file(), content + "\n");
}

fn dirs_match(a: &str, b: &str) -> bool {
    if cfg!(windows) { a.eq_ignore_ascii_case(b) } else { a == b }
}

/// Load commands for the current directory, sorted by count (most used first).
fn load_hist() -> Vec<String> {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut entries: Vec<Entry> = load_all().into_iter()
        .filter(|e| dirs_match(&e.dir, &cwd))
        .collect();
    entries.sort_by(|a, b| b.count.cmp(&a.count).then(a.cmd.cmp(&b.cmd)));
    entries.into_iter().map(|e| e.cmd).collect()
}

/// Record a command: increment count if exists, else add with count=1.
/// Keeps max 50 per directory, drops the least-used when full.
fn add_command(cmd: &str) {
    let cmd = cmd.trim();
    if cmd.is_empty() { return; }
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut all = load_all();

    // find existing entry for this dir+cmd
    if let Some(entry) = all.iter_mut().find(|e| dirs_match(&e.dir, &cwd) && e.cmd == cmd) {
        entry.count += 1;
    } else {
        // count how many entries this dir already has
        let mut dir_entries: Vec<usize> = all.iter().enumerate()
            .filter(|(_, e)| dirs_match(&e.dir, &cwd))
            .map(|(i, _)| i)
            .collect();

        if dir_entries.len() >= 50 {
            // find the least-used entry for this dir and remove it
            dir_entries.sort_by(|&a, &b| all[a].count.cmp(&all[b].count));
            all.remove(dir_entries[0]);
        }
        all.push(Entry { dir: cwd, count: 1, cmd: cmd.to_string() });
    }

    save_all(&all);
}


struct Picker {
    all:      Vec<String>,
    filter:   String,
    matches:  Vec<usize>,
    cursor:   usize,
}

impl Picker {
    fn new(items: Vec<String>) -> Self {
        let matches = (0..items.len()).collect();
        Picker { all: items, filter: String::new(), matches, cursor: 0 }
    }

    fn refilter(&mut self) {
        let f = self.filter.to_lowercase();
        self.matches = self.all.iter().enumerate()
            .filter(|(_, s)| s.to_lowercase().contains(&f))
            .map(|(i, _)| i)
            .collect();
        self.cursor = 0;
    }

    fn selected(&self) -> Option<&str> {
        self.matches.get(self.cursor).map(|&i| self.all[i].as_str())
    }

    fn up(&mut self)   { if self.cursor > 0 { self.cursor -= 1; } }
    fn down(&mut self) { if self.cursor + 1 < self.matches.len() { self.cursor += 1; } }
}

fn run_picker(items: Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if items.is_empty() {
        return Ok(None);
    }
    let mut p = Picker::new(items);
    enable_raw_mode()?;
    stderr().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stderr()))?;
    let mut chosen: Option<String> = None;

    loop {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter => {
                            chosen = p.selected().map(|s| s.to_string());
                            break;
                        }
                        KeyCode::Up   => p.up(),
                        KeyCode::Down => p.down(),
                        KeyCode::Backspace => {
                            p.filter.pop();
                            p.refilter();
                        }
                        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL)
                            && (c == 'c' || c == 'q') => break,
                        KeyCode::Char(c) => {
                            p.filter.push(c);
                            p.refilter();
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }

        terminal.draw(|frame| {
            let area = frame.area();
            let chunks = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(0),
            ]).split(area);

            // filter box
            frame.render_widget(
                Paragraph::new(format!(" {}_", p.filter))
                    .block(Block::default().borders(Borders::ALL)
                        .title("Search history (↑↓ navigate  Enter run  Esc cancel)")),
                chunks[0],
            );

            // list
            let vis = chunks[1].height.saturating_sub(2) as usize;
            let scroll = p.cursor.saturating_sub(vis.saturating_sub(1));
            let items: Vec<ListItem> = p.matches.iter().enumerate()
                .skip(scroll).take(vis)
                .map(|(i, &idx)| {
                    let style = if i + scroll == p.cursor {
                        Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(p.all[idx].as_str()).style(style)
                })
                .collect();
            frame.render_widget(
                List::new(items)
                    .block(Block::default().borders(Borders::ALL)
                        .title(format!("{}/{} matches", p.matches.len(), p.all.len()))),
                chunks[1],
            );
        })?;
    }

    disable_raw_mode()?;
    stderr().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(chosen)
}


pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    match args.first().map(|s| s.as_str()) {
        Some("--add") => {
            if args.len() > 1 {
                let cmd = args[1..].join(" ");
                add_command(&cmd);
            }
            return Ok(());
        }
        _ => {}
    }

    let items = load_hist();
    if items.is_empty() {
        eprintln!("No history yet. Run some commands and they'll appear here.");
        return Ok(());
    }

    if let Some(cmd) = run_picker(items)? {
        #[cfg(windows)]
        std::process::Command::new("cmd").args(["/C", &cmd]).status()?;
        #[cfg(not(windows))]
        std::process::Command::new("sh").args(["-c", &cmd]).status()?;
    }
    Ok(())
}
