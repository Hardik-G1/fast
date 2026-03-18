// fast alias add <name> <cmd...>  → save alias
// fast alias rm  <name>           → remove alias
// fast alias run <name>           → print command
// fast alias list                 → print all aliases
// fast alias --init               → print shell wrapper
//
// f <alias>  →  runs the aliased command

use std::io::stderr;
use std::path::PathBuf;
use std::collections::HashMap;
use ratatui::prelude::CrosstermBackend;
use ratatui::crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode},
    ExecutableCommand,
};
use ratatui::Terminal;
use ratatui::layout::{Layout, Constraint};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::style::{Color, Modifier, Style};
use std::time::Duration;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers, KeyEventKind};


// Format: one alias per line  →  name\tcommand

fn alias_file() -> PathBuf {
    // get the path for the profile 
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();
    //join our folder name
    PathBuf::from(home).join(".fast_aliases")
}

fn load_aliases() -> HashMap<String, String> {
    // get the content from the path 
    let content = std::fs::read_to_string(alias_file()).unwrap_or_default();
    // filter it -> first split it with the separator \t that gives a iterator -> trim the name -> trim the cmd 
    // -> if empty-> return -> else -> gets the name and command and collect 
    content.lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, '\t');
            let name = parts.next()?.trim();
            let cmd  = parts.next()?.trim();
            if name.is_empty() || cmd.is_empty() { return None; }
            Some((name.to_string(), cmd.to_string()))
        })
        .collect()
}

fn save_aliases(map: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    // first get the hashmap and read it and collect , sort it 
    // then write a new entry with endline 
    let mut lines: Vec<String> = map.iter()
        .map(|(n, c)| format!("{}\t{}", n, c))
        .collect();
    lines.sort();
    std::fs::write(alias_file(), lines.join("\n") + "\n")?;
    Ok(())
}


struct Picker {
    // (display, command)
    all:     Vec<(String, String)>,
    filter:  String,
    matches: Vec<usize>,
    cursor:  usize,
}

impl Picker {
    fn new(mut items: Vec<(String, String)>) -> Self {
        items.sort_by(|a, b| a.0.cmp(&b.0));
        let matches = (0..items.len()).collect();
        Picker { all: items, filter: String::new(), matches, cursor: 0 }
    }

    fn refilter(&mut self) {
        let f = self.filter.to_lowercase();
        self.matches = self.all.iter().enumerate()
            .filter(|(_, (name, cmd))| {
                name.to_lowercase().contains(&f) || cmd.to_lowercase().contains(&f)
            })
            .map(|(i, _)| i)
            .collect();
        self.cursor = 0;
    }

    fn selected_cmd(&self) -> Option<&str> {
        self.matches.get(self.cursor).map(|&i| self.all[i].1.as_str())
    }

    fn up(&mut self)   { if self.cursor > 0 { self.cursor -= 1; } }
    fn down(&mut self) { if self.cursor + 1 < self.matches.len() { self.cursor += 1; } }
}

fn run_picker(items: Vec<(String, String)>) -> Result<Option<String>, Box<dyn std::error::Error>> {
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
                            chosen = p.selected_cmd().map(|s| s.to_string());
                            break;
                        }
                        KeyCode::Up   => p.up(),
                        KeyCode::Down => p.down(),
                        //pop because filter term changes
                        KeyCode::Backspace => { p.filter.pop(); p.refilter(); }
                        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL)
                            && (c == 'c' || c == 'q') => break,
                        //push because filter term has new changes
                        KeyCode::Char(c) => { p.filter.push(c); p.refilter(); }
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

            frame.render_widget(
                Paragraph::new(format!(" {}_", p.filter))
                    .block(Block::default().borders(Borders::ALL)
                        .title("Search aliases (↑↓ navigate  Enter run  Esc cancel)")),
                chunks[0],
            );
            //saturating_sub <- used to limit the subtraction result
            let vis = chunks[1].height.saturating_sub(2) as usize;
            let scroll = p.cursor.saturating_sub(vis.saturating_sub(1));
            let items: Vec<ListItem> = p.matches.iter().enumerate()
                .skip(scroll).take(vis)
                .map(|(i, &idx)| {
                    let (name, cmd) = &p.all[idx];
                    let label = format!("{:<20}  {}", name, cmd);
                    let style = if i + scroll == p.cursor {
                        Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(label).style(style)
                })
                .collect();
            frame.render_widget(
                List::new(items)
                    .block(Block::default().borders(Borders::ALL)
                        .title(format!("{}/{} aliases", p.matches.len(), p.all.len()))),
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

        Some("add") => {
            // fast alias add <name> <cmd...>
            if args.len() < 3 {
                eprintln!("Usage: fast alias add <name> <command...>");
                return Ok(());
            }
            let name = &args[1];
            let cmd  = args[2..].join(" ");
            // reject names/commands with control chars or tabs (would corrupt file)
            if name.contains('\t') || name.contains('\n') || name.contains('\0') {
                eprintln!("Alias name cannot contain tabs, newlines, or null bytes");
                return Ok(());
            }
            if cmd.contains('\t') || cmd.contains('\n') || cmd.contains('\0') {
                eprintln!("Alias command cannot contain tabs, newlines, or null bytes");
                return Ok(());
            }
            let mut map = load_aliases();
            map.insert(name.clone(), cmd.clone());
            save_aliases(&map)?;
            eprintln!("Alias '{}' saved → {}", name, cmd);
        }

        Some("rm") => {
            if args.len() < 2 {
                eprintln!("Usage: fast alias rm <name>");
                return Ok(());
            }
            let name = &args[1];
            let mut map = load_aliases();
            if map.remove(name).is_some() {
                save_aliases(&map)?;
                eprintln!("Alias '{}' removed", name);
            } else {
                eprintln!("Alias '{}' not found", name);
            }
        }

        Some("list") => {
            let map = load_aliases();
            if map.is_empty() {
                eprintln!("No aliases. Add one: fast alias add <name> <command>");
            } else {
                let mut pairs: Vec<_> = map.iter().collect();
                pairs.sort_by_key(|(k, _)| k.as_str());
                for (name, cmd) in pairs {
                    println!("{:<20}  {}", name, cmd);
                }
            }
        }

        Some("run") => {
            if args.len() < 2 {
                eprintln!("Usage: fast alias run <name>");
                return Ok(());
            }
            let name = &args[1];
            let map  = load_aliases();
            match map.get(name) {
                Some(cmd) => print!("{}", cmd),
                None      => eprintln!("Alias '{}' not found", name),
            }
        }

        _ => {
            // TUI picker
            let map = load_aliases();
            if map.is_empty() {
                eprintln!("No aliases. Add one: fast alias add <name> <command>");
                return Ok(());
            }
            let items: Vec<(String, String)> = map.into_iter().collect();
            if let Some(cmd) = run_picker(items)? {
                print!("{}", cmd);
            }
        }
    }
    Ok(())
}
