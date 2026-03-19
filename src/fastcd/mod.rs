// 3-column file browser no file previews, always 3 dir-listing columns
// col0 = current dir  col1 = children of col0-selected  col2 = children of col1-selected
// ↑↓ move within focused col  → move focus right (at col2: navigate in)  ← move focus left (at col0: go to parent)
// Tab = file preview (col0 stays, col1+col2 become preview)  ← exits preview
// Enter = cd to highlighted dir and exit   r = reset   q = quit
use std::io::stderr;
use std::path::{Path, PathBuf};
use ratatui::prelude::CrosstermBackend;
use ratatui::crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode},
    event::{EnableMouseCapture, DisableMouseCapture, MouseEventKind},
    ExecutableCommand,
};
use ratatui::Terminal;
use std::time::Duration;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers, KeyEventKind};
use ratatui::layout::{Layout, Constraint};
use ratatui::widgets::{List, ListItem, Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::style::{Style, Color, Modifier};


pub struct DirEntry {
    pub name:       String,
    pub is_dir:     bool,
    pub size:       String,
    pub permission: String,
}

fn fmt_size(bytes: u64) -> String {
    if bytes < 1024                { format!("{} B",     bytes) }
    else if bytes < 1024*1024      { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else if bytes < 1024*1024*1024 { format!("{:.1} MB", bytes as f64 / (1024.0*1024.0)) }
    else                           { format!("{:.1} GB", bytes as f64 / (1024.0*1024.0*1024.0)) }
}

#[cfg(unix)]
fn fmt_permissions(meta: &std::fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = meta.permissions().mode();
    [(0o400,'r'),(0o200,'w'),(0o100,'x'),(0o040,'r'),(0o020,'w'),
     (0o010,'x'),(0o004,'r'),(0o002,'w'),(0o001,'x')]
        .iter().map(|(b,c)| if mode & b != 0 { *c } else { '-' }).collect()
}
#[cfg(not(unix))]
fn fmt_permissions(meta: &std::fs::Metadata) -> String {
    match meta.permissions().readonly() {
        true => "r--------".into(),
        false => "rw-rw-rw-".into(),
    }
}

fn dir_size(path: &Path) -> u64 {
    dir_size_inner(path, 10)
}

fn dir_size_inner(path: &Path, depth: u8) -> u64 {
    if depth == 0 { return 0; }
    let Ok(entries) = std::fs::read_dir(path) else { return 0; };
    entries.filter_map(|e| e.ok()).map(|e| {
        // use symlink_metadata to detect symlinks without following them
        let Ok(m) = e.path().symlink_metadata() else { return 0; };
        if m.file_type().is_symlink() {
            0 // skip symlinks entirely
        } else if m.is_dir() {
            dir_size_inner(&e.path(), depth - 1)
        } else {
            m.len()
        }
    }).sum()
}

fn format_modified(meta: &std::fs::Metadata) -> String {
    use std::time::UNIX_EPOCH;
    meta.modified().ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| {
            let s = d.as_secs();
            let (mins, hours, days) = ((s%3600)/60, (s%86400)/3600, s/86400);
            let (year, doy) = (1970 + days/365, days%365);
            format!("{}-{:02}-{:02} {:02}:{:02}", year, doy/30+1, doy%30+1, hours, mins)
        })
        .unwrap_or_else(|| "?".to_string())
}

pub fn read_path(path: &Path) -> Vec<DirEntry> {
    let mut result = Vec::new();
    let Ok(entries) = std::fs::read_dir(path) else { return result; };
    for item in entries {
        let Ok(item) = item else { continue; };
        let Ok(meta)  = item.metadata() else { continue; };
        let name       = item.file_name().to_string_lossy().to_string();
        let is_dir     = meta.is_dir();
        let permission = fmt_permissions(&meta);
        let size       = if is_dir { 0 } else { fmt_size(meta.len()) };
        result.push(DirEntry { name, is_dir, size, permission });
    }
    result.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    result
}


pub struct AppState {
    pub current_dir:    String,
    pub start_dir:      String,
    pub entries:        [Vec<DirEntry>; 3],
    pub selected:       [usize; 3],
    pub focus_col:      usize,
    pub status:         String,
    // preview
    pub preview_mode:   bool,
    pub preview_file:   String,
    pub preview_lines:  Vec<String>,
    pub preview_scroll: usize,
}

impl AppState {
    pub fn new() -> Self {
        let dir     = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let dir_str = dir.to_string_lossy().to_string();
        let mut s   = AppState {
            current_dir:    dir_str.clone(),
            start_dir:      dir_str,
            entries:        [vec![], vec![], vec![]],
            selected:       [0, 0, 0],
            focus_col:      0,
            status:         "↑↓ move  → focus/nav  ← back  Tab preview  Enter cd+exit  r reset  q quit".to_string(),
            preview_mode:   false,
            preview_file:   String::new(),
            preview_lines:  vec![],
            preview_scroll: 0,
        };
        s.entries[0] = read_path(&dir);
        s.repopulate_from(0);
        s
    }

    // Rebuild columns to the right of changed_col
    pub fn repopulate_from(&mut self, changed_col: usize) {
        let dir = PathBuf::from(&self.current_dir);
        match changed_col {
            0 => {
                let sel = self.entries[0].get(self.selected[0]).map(|e| (e.is_dir, e.name.clone()));
                if let Some((true, name)) = sel {
                    self.entries[1]  = read_path(&dir.join(&name));
                    self.selected[1] = 0;
                    self.repopulate_col1(&dir.join(&name));
                } else {
                    self.entries[1] = vec![];
                    self.entries[2] = vec![];
                    self.selected[1] = 0;
                    self.selected[2] = 0;
                }
            }
            1 => {
                let col0_name = self.entries[0].get(self.selected[0])
                    .map(|e| e.name.clone()).unwrap_or_default();
                self.repopulate_col1(&dir.join(&col0_name));
            }
            _ => {}
        }
    }

    fn repopulate_col1(&mut self, col1_base: &Path) {
        if let Some(name) = self.entries[1].get(self.selected[1]).map(|e| e.name.clone()) {
            let path = col1_base.join(&name);
            self.entries[2] = if path.is_dir() { read_path(&path) } else { vec![] };
            self.selected[2] = 0;
        } else {
            self.entries[2]  = vec![];
            self.selected[2] = 0;
        }
    }

    // → key: move focus right; at col2 navigate into selected dir
    pub fn focus_right(&mut self) {
        match self.focus_col {
            0 => { if !self.entries[1].is_empty() { self.focus_col = 1; } }
            1 => { if !self.entries[2].is_empty() { self.focus_col = 2; } }
            2 => {
                let n0 = self.entries[0].get(self.selected[0]).map(|e| e.name.clone());
                let n1 = self.entries[1].get(self.selected[1]).map(|e| e.name.clone());
                let n2 = self.entries[2].get(self.selected[2])
                    .and_then(|e| if e.is_dir { Some(e.name.clone()) } else { None });
                if let (Some(a), Some(b), Some(c)) = (n0, n1, n2) {
                    let path = PathBuf::from(&self.current_dir).join(a).join(b).join(c);
                    self.current_dir = path.to_string_lossy().to_string();
                    self.entries[0]  = read_path(&path);
                    self.entries[1]  = vec![];
                    self.entries[2]  = vec![];
                    self.selected    = [0, 0, 0];
                    self.focus_col   = 0;
                    self.status      = format!("→ {}", self.current_dir);
                    self.repopulate_from(0);
                }
            }
            _ => {}
        }
    }

    // ← key at col0: go to parent directory
    pub fn navigate_left(&mut self) {
        let current = PathBuf::from(&self.current_dir);
        let Some(parent) = current.parent() else { return; };
        if parent.as_os_str().is_empty() { return; }
        let current_name = current.file_name()
            .map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let parent_entries = read_path(parent);
        let cursor = parent_entries.iter().position(|e| e.name == current_name).unwrap_or(0);
        self.current_dir = parent.to_string_lossy().to_string();
        self.entries[0]  = parent_entries;
        self.entries[1]  = vec![];
        self.entries[2]  = vec![];
        self.selected    = [cursor, 0, 0];
        self.status      = format!("← {}", self.current_dir);
        self.repopulate_from(0);
    }

    // p/Tab: preview the selected entry (file → content, dir → listing)
    pub fn enter_preview(&mut self) {
        // Compute the full path of the selected entry based on focus_col
        let target_path = match self.focus_col {
            0 => {
                let Some(e) = self.entries[0].get(self.selected[0]) else { return; };
                PathBuf::from(&self.current_dir).join(&e.name)
            }
            1 => {
                let n0 = self.entries[0].get(self.selected[0]).map(|e| e.name.clone());
                let Some(e) = self.entries[1].get(self.selected[1]) else { return; };
                let Some(a) = n0 else { return; };
                PathBuf::from(&self.current_dir).join(a).join(&e.name)
            }
            2 => {
                let n0 = self.entries[0].get(self.selected[0]).map(|e| e.name.clone());
                let n1 = self.entries[1].get(self.selected[1]).map(|e| e.name.clone());
                let Some(e) = self.entries[2].get(self.selected[2]) else { return; };
                let (Some(a), Some(b)) = (n0, n1) else { return; };
                PathBuf::from(&self.current_dir).join(a).join(b).join(&e.name)
            }
            _ => return,
        };

        let dir_path = target_path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(&self.current_dir));

        // Load content: file → text lines, dir → entry list
        if target_path.is_dir() {
            let listing = read_path(&target_path);
            self.preview_lines = listing.iter()
                .map(|e| format!("{} {}", if e.is_dir { "▶" } else { " " }, e.name))
                .collect();
        } else {
            // limit preview to 1MB to avoid OOM on large files
            let size = std::fs::metadata(&target_path).map(|m| m.len()).unwrap_or(0);
            if size > 1_048_576 {
                self.preview_lines = vec![format!("[file too large: {} bytes]", size)];
            } else {
                let content = std::fs::read_to_string(&target_path)
                    .unwrap_or_else(|_| "[binary or unreadable file]".to_string());
                self.preview_lines = content.lines().map(|l| l.to_string()).collect();
            }
        }

        self.preview_file   = target_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        self.preview_scroll = 0;

        // Set col0 to the parent dir with the entry highlighted
        self.current_dir = dir_path.to_string_lossy().to_string();
        let entries  = read_path(&dir_path);
        let sel_idx  = entries.iter().position(|e| e.name == self.preview_file).unwrap_or(0);
        self.entries  = [entries, vec![], vec![]];
        self.selected = [sel_idx, 0, 0];
        self.focus_col    = 0;
        self.preview_mode = true;
        self.status = "↑↓/scroll preview  ← exit preview".to_string();
    }

    pub fn exit_preview(&mut self) {
        self.preview_mode = false;
        self.repopulate_from(0);
        self.status = "↑↓ move  → focus/nav  ← back  Tab preview  Enter cd+exit  r reset  q quit".to_string();
    }

    pub fn reset(&mut self) {
        let dir = PathBuf::from(&self.start_dir);
        self.current_dir  = self.start_dir.clone();
        self.entries[0]   = read_path(&dir);
        self.entries[1]   = vec![];
        self.entries[2]   = vec![];
        self.selected     = [0, 0, 0];
        self.focus_col    = 0;
        self.preview_mode = false;
        self.status       = format!("reset → {}", self.start_dir);
        self.repopulate_from(0);
    }

    // Compute the path to cd to on Enter
    pub fn exit_path(&self) -> String {
        let base = PathBuf::from(&self.current_dir);
        let path = match self.focus_col {
            0 => self.entries[0].get(self.selected[0])
                    .filter(|e| e.is_dir)
                    .map(|e| base.join(&e.name))
                    .unwrap_or(base),
            1 => {
                let n0 = self.entries[0].get(self.selected[0]).map(|e| e.name.clone());
                let n1 = self.entries[1].get(self.selected[1]).filter(|e| e.is_dir).map(|e| e.name.clone());
                match n0 {
                    Some(a) => n1.map(|b| base.join(&a).join(b)).unwrap_or_else(|| base.join(a)),
                    None    => base,
                }
            }
            2 => {
                let n0 = self.entries[0].get(self.selected[0]).map(|e| e.name.clone());
                let n1 = self.entries[1].get(self.selected[1]).map(|e| e.name.clone());
                let n2 = self.entries[2].get(self.selected[2]).filter(|e| e.is_dir).map(|e| e.name.clone());
                match (n0, n1) {
                    (Some(a), Some(b)) => n2.map(|c| base.join(&a).join(&b).join(c))
                                            .unwrap_or_else(|| base.join(a).join(b)),
                    _ => base,
                }
            }
            _ => base,
        };
        path.to_string_lossy().to_string()
    }

    fn meta_line(&self) -> String {
        let Some(e) = self.entries[0].get(self.selected[0]) else { return String::new(); };
        let path     = PathBuf::from(&self.current_dir).join(&e.name);
        let modified = std::fs::metadata(&path).ok()
            .map(|m| format_modified(&m)).unwrap_or_else(|| "?".to_string());
        format!("  {}  │  {}  │  {}  │  {}", e.permission, e.size, modified, e.name)
    }
}



fn border_style(focused: bool) -> Style {
    if focused { Style::default().fg(Color::White) } else { Style::default().fg(Color::DarkGray) }
}

fn make_list(entries: &[DirEntry], selected: usize, focused: bool, scroll: usize, vis: usize) -> Vec<ListItem<'_>> {
    entries.iter().enumerate().skip(scroll).take(vis).map(|(i, e)| {
        let label = format!("{} {}", if e.is_dir { "▶" } else { " " }, e.name);
        let style = if i == selected && focused {
            Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
        } else if i == selected {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else if e.is_dir {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        ListItem::new(label).style(style)
    }).collect()
}

fn vscroll(selected: usize, vis: usize) -> usize {
    selected.saturating_sub(vis.saturating_sub(1))
}



pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    stderr().execute(EnterAlternateScreen)?;
    stderr().execute(EnableMouseCapture)?;
    let mut terminal  = Terminal::new(CrosstermBackend::new(stderr()))?;
    let mut app       = AppState::new();
    let mut exit_path: Option<String> = None;

    loop {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {

                    KeyCode::Up => {
                        if app.preview_mode {
                            if app.preview_scroll > 0 { app.preview_scroll -= 1; }
                        } else {
                            match app.focus_col {
                                0 => { if app.selected[0] > 0 { app.selected[0] -= 1; app.repopulate_from(0); } }
                                1 => { if app.selected[1] > 0 { app.selected[1] -= 1; app.repopulate_from(1); } }
                                2 => { if app.selected[2] > 0 { app.selected[2] -= 1; } }
                                _ => {}
                            }
                        }
                    }

                    KeyCode::Down => {
                        if app.preview_mode {
                            let max = app.preview_lines.len().saturating_sub(1);
                            if app.preview_scroll < max { app.preview_scroll += 1; }
                        } else {
                            match app.focus_col {
                                0 => { if app.selected[0]+1 < app.entries[0].len() { app.selected[0] += 1; app.repopulate_from(0); } }
                                1 => { if app.selected[1]+1 < app.entries[1].len() { app.selected[1] += 1; app.repopulate_from(1); } }
                                2 => { if app.selected[2]+1 < app.entries[2].len() { app.selected[2] += 1; } }
                                _ => {}
                            }
                        }
                    }

                    KeyCode::Right => { if !app.preview_mode { app.focus_right(); } }

                    KeyCode::Left => {
                        if app.preview_mode {
                            app.exit_preview();
                        } else if app.focus_col > 0 {
                            app.focus_col -= 1;
                        } else {
                            app.navigate_left();
                        }
                    }

                    KeyCode::Tab => app.enter_preview(),

                    KeyCode::Enter => {
                        exit_path = Some(app.exit_path());
                        break;
                    }

                    KeyCode::Char('r') => app.reset(),
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Esc       => break,
                    _ => {}
                }

                Event::Mouse(mouse) => {
                    if app.preview_mode {
                        let max = app.preview_lines.len().saturating_sub(1);
                        match mouse.kind {
                            MouseEventKind::ScrollUp   => { app.preview_scroll = app.preview_scroll.saturating_sub(3); }
                            MouseEventKind::ScrollDown => { app.preview_scroll = (app.preview_scroll + 3).min(max); }
                            _ => {}
                        }
                    }
                }

                _ => {}
            }
        }

        terminal.draw(|frame| {
            let root = Layout::vertical([Constraint::Min(0), Constraint::Length(4)]).split(frame.area());
            let (main_area, status_area) = (root[0], root[1]);

            if app.preview_mode {
                // ── preview layout: 33% dir | 67% file content ──
                let columns = Layout::horizontal([
                    Constraint::Percentage(33),
                    Constraint::Percentage(67),
                ]).split(main_area);

                let vis0 = columns[0].height.saturating_sub(2) as usize;
                frame.render_widget(
                    List::new(make_list(&app.entries[0], app.selected[0], false,
                        vscroll(app.selected[0], vis0), vis0))
                        .block(Block::default().borders(Borders::ALL)
                            .title(app.current_dir.as_str())
                            .border_style(border_style(false))),
                    columns[0],
                );

                let vis_p = columns[1].height.saturating_sub(2) as usize;
                let start = app.preview_scroll;
                let total = app.preview_lines.len();
                let preview_text: String = app.preview_lines.iter()
                    .skip(start)
                    .take(vis_p)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n");
                frame.render_widget(
                    Paragraph::new(preview_text)
                        .block(Block::default().borders(Borders::ALL)
                            .title(app.preview_file.as_str())
                            .border_style(border_style(true))),
                    columns[1],
                );

                // Scrollbar on the right edge of the preview panel
                if total > vis_p {
                    let mut sb_state = ScrollbarState::new(total.saturating_sub(vis_p))
                        .position(start);
                    frame.render_stateful_widget(
                        Scrollbar::new(ScrollbarOrientation::VerticalRight)
                            .style(Style::default().fg(Color::DarkGray)),
                        columns[1],
                        &mut sb_state,
                    );
                }

            } else {
                // ── normal 3-column layout ──
                let columns = Layout::horizontal([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ]).split(main_area);

                let vis0 = columns[0].height.saturating_sub(2) as usize;
                frame.render_widget(
                    List::new(make_list(&app.entries[0], app.selected[0], app.focus_col==0,
                        vscroll(app.selected[0], vis0), vis0))
                        .block(Block::default().borders(Borders::ALL)
                            .title(app.current_dir.as_str())
                            .border_style(border_style(app.focus_col == 0))),
                    columns[0],
                );

                let vis1   = columns[1].height.saturating_sub(2) as usize;
                let title1 = app.entries[0].get(app.selected[0]).map(|e| e.name.as_str()).unwrap_or("");
                frame.render_widget(
                    List::new(make_list(&app.entries[1], app.selected[1], app.focus_col==1,
                        vscroll(app.selected[1], vis1), vis1))
                        .block(Block::default().borders(Borders::ALL).title(title1)
                            .border_style(border_style(app.focus_col == 1))),
                    columns[1],
                );

                let vis2   = columns[2].height.saturating_sub(2) as usize;
                let title2 = app.entries[1].get(app.selected[1]).map(|e| e.name.as_str()).unwrap_or("");
                frame.render_widget(
                    List::new(make_list(&app.entries[2], app.selected[2], app.focus_col==2,
                        vscroll(app.selected[2], vis2), vis2))
                        .block(Block::default().borders(Borders::ALL).title(title2)
                            .border_style(border_style(app.focus_col == 2))),
                    columns[2],
                );
            }

            // ── status bar ──
            frame.render_widget(
                Paragraph::new(format!("{}\n{}  │  {}", app.current_dir, app.meta_line(), app.status))
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().fg(Color::DarkGray)),
                status_area,
            );
        })?;
    }

    disable_raw_mode()?;
    stderr().execute(DisableMouseCapture)?;
    stderr().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Some(path) = exit_path {
        print!("{}", path);
    }
    Ok(())
}