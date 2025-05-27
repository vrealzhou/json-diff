//! Terminal-based display for JSON diff results
//!
//! This module provides a TUI (Text User Interface) for displaying JSON diff results
//! with vim-like keybindings for navigation using ratatui. The interface is designed
//! for keyboard-only operation and does not support mouse interactions.

use std::io;
use anyhow::{Result, Context};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap, Clear},
    Frame, Terminal,
};
use json_diff_core::{DiffResult, DiffType};

/// Display mode for the application
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    /// List view showing diff entries
    List,
    /// Split-screen view showing both JSON files with highlighted differences
    SplitScreen,
}

/// App holds the state of the application
pub struct App {
    diff_result: DiffResult,
    current_index: usize,
    quit: bool,
    help_visible: bool,
    use_readable_format: bool,
    view_mode: ViewMode,
    // Split-screen specific state
    left_content: Vec<String>,
    right_content: Vec<String>,
    left_scroll: usize,
    right_scroll: usize,
    current_diff_index: usize,
    // Sorted diff indices for proper navigation order
    sorted_diff_indices: Vec<usize>,
    // Current position in the sorted diff list
    current_sorted_position: usize,
}

impl App {
    pub fn new(diff_result: DiffResult) -> Self {
        // Load file contents for split-screen view
        let left_content = Self::load_file_content(&diff_result.left_file);
        let right_content = Self::load_file_content(&diff_result.right_file);

        // Create sorted indices for proper diff navigation order
        let sorted_diff_indices = Self::create_sorted_diff_indices(&diff_result);

        Self {
            diff_result,
            current_index: 0,
            quit: false,
            help_visible: false,
            use_readable_format: false,
            view_mode: ViewMode::List,
            left_content,
            right_content,
            left_scroll: 0,
            right_scroll: 0,
            current_diff_index: 0,
            sorted_diff_indices,
            current_sorted_position: 0,
        }
    }

    fn create_sorted_diff_indices(diff_result: &DiffResult) -> Vec<usize> {
        let mut indices_with_lines: Vec<(usize, usize)> = diff_result.entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                // Use the minimum line number (left or right) for sorting
                // This ensures we navigate in the order they appear in the files
                let line_num = match (entry.left_line, entry.right_line) {
                    (Some(left), Some(right)) => left.min(right),
                    (Some(left), None) => left,
                    (None, Some(right)) => right,
                    (None, None) => usize::MAX, // Put entries without line numbers at the end
                };
                (index, line_num)
            })
            .collect();

        // Sort by line number
        indices_with_lines.sort_by_key(|(_, line_num)| *line_num);

        // Extract just the indices
        indices_with_lines.into_iter().map(|(index, _)| index).collect()
    }

    fn load_file_content(file_path: &Option<std::path::PathBuf>) -> Vec<String> {
        if let Some(path) = file_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                return content.lines().map(|s| s.to_string()).collect();
            }
        }
        vec!["File not found or could not be read".to_string()]
    }

    pub fn next(&mut self) {
        match self.view_mode {
            ViewMode::List => {
                if !self.diff_result.entries.is_empty() {
                    self.current_index = (self.current_index + 1) % self.diff_result.entries.len();
                }
            }
            ViewMode::SplitScreen => {
                self.scroll_down();
            }
        }
    }

    pub fn previous(&mut self) {
        match self.view_mode {
            ViewMode::List => {
                if !self.diff_result.entries.is_empty() {
                    self.current_index = if self.current_index > 0 {
                        self.current_index - 1
                    } else {
                        self.diff_result.entries.len() - 1
                    };
                }
            }
            ViewMode::SplitScreen => {
                self.scroll_up();
            }
        }
    }

    pub fn toggle_help(&mut self) {
        self.help_visible = !self.help_visible;
    }

    pub fn toggle_format(&mut self) {
        self.use_readable_format = !self.use_readable_format;
    }

    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::List => ViewMode::SplitScreen,
            ViewMode::SplitScreen => ViewMode::List,
        };
    }

    pub fn scroll_up(&mut self) {
        if self.left_scroll > 0 {
            self.left_scroll -= 1;
        }
        if self.right_scroll > 0 {
            self.right_scroll -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        let max_left = self.left_content.len().saturating_sub(1);
        let max_right = self.right_content.len().saturating_sub(1);

        if self.left_scroll < max_left {
            self.left_scroll += 1;
        }
        if self.right_scroll < max_right {
            self.right_scroll += 1;
        }
    }

    pub fn next_diff(&mut self) {
        if !self.sorted_diff_indices.is_empty() {
            self.current_sorted_position = (self.current_sorted_position + 1) % self.sorted_diff_indices.len();
            self.current_diff_index = self.sorted_diff_indices[self.current_sorted_position];
            self.jump_to_current_diff();
        }
    }

    pub fn previous_diff(&mut self) {
        if !self.sorted_diff_indices.is_empty() {
            self.current_sorted_position = if self.current_sorted_position > 0 {
                self.current_sorted_position - 1
            } else {
                self.sorted_diff_indices.len() - 1
            };
            self.current_diff_index = self.sorted_diff_indices[self.current_sorted_position];
            self.jump_to_current_diff();
        }
    }

    fn jump_to_current_diff(&mut self) {
        if let Some(entry) = self.diff_result.entries.get(self.current_diff_index) {
            // Jump to the line number of the current diff
            if let Some(left_line) = entry.left_line {
                self.left_scroll = left_line.saturating_sub(1);
            }
            if let Some(right_line) = entry.right_line {
                self.right_scroll = right_line.saturating_sub(1);
            }
        }
    }
}

/// Runs the terminal UI for displaying diff results
pub fn run_display(diff_result: DiffResult) -> Result<()> {
    run_display_with_options(diff_result, false)
}

/// Runs the terminal UI for displaying diff results with options
pub fn run_display_with_options(diff_result: DiffResult, use_readable_format: bool) -> Result<()> {
    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(diff_result);
    app.use_readable_format = use_readable_format;

    // Main loop
    let result = run_main_loop(&mut terminal, &mut app);

    // Restore terminal (always do this, even if there was an error)
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    result
}

fn run_main_loop<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    while !app.quit {
        // Draw UI
        terminal.draw(|f| ui(f, app))?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => app.quit = true,
                KeyCode::Char('j') | KeyCode::Down => app.next(),
                KeyCode::Char('k') | KeyCode::Up => app.previous(),
                KeyCode::Char('n') => app.next_diff(),
                KeyCode::Char('N') => app.previous_diff(),
                KeyCode::Char('v') => app.toggle_view_mode(),
                KeyCode::Char('h') | KeyCode::Char('?') => app.toggle_help(),
                KeyCode::Char('r') => app.toggle_format(),
                KeyCode::Esc => {
                    if app.help_visible {
                        app.help_visible = false;
                    } else {
                        app.quit = true;
                    }
                },
                _ => {}
            }
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    match app.view_mode {
        ViewMode::List => render_list_view(f, app, size),
        ViewMode::SplitScreen => render_split_screen_view(f, app, size),
    }

    // Help overlay (common to both views)
    if app.help_visible {
        let help = create_help_popup();
        let popup_area = centered_rect(80, 60, size);
        f.render_widget(Clear, popup_area);
        f.render_widget(help, popup_area);
    }
}

fn render_list_view(f: &mut Frame, app: &App, size: Rect) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),  // Header
                Constraint::Min(5),     // Diff content
                Constraint::Length(3),  // Footer
            ]
            .as_ref(),
        )
        .split(size);

    // Header
    let header = create_header(&app.diff_result);
    f.render_widget(header, chunks[0]);

    // Diff content
    let diff_content = create_diff_content(app);
    let mut list_state = ListState::default();
    list_state.select(Some(app.current_index));
    f.render_stateful_widget(diff_content, chunks[1], &mut list_state);

    // Footer
    let footer = create_footer(app);
    f.render_widget(footer, chunks[2]);
}

fn render_split_screen_view(f: &mut Frame, app: &App, size: Rect) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),  // Header
                Constraint::Min(5),     // Split content
                Constraint::Length(3),  // Footer
            ]
            .as_ref(),
        )
        .split(size);

    // Header
    let header = create_split_header(&app.diff_result, app.current_diff_index);
    f.render_widget(header, chunks[0]);

    // Split the main area horizontally
    let split_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    // Left file content
    let left_content = create_file_content(&app.left_content, app.left_scroll, "Left File", &app.diff_result, true, app.current_diff_index);
    f.render_widget(left_content, split_chunks[0]);

    // Right file content
    let right_content = create_file_content(&app.right_content, app.right_scroll, "Right File", &app.diff_result, false, app.current_diff_index);
    f.render_widget(right_content, split_chunks[1]);

    // Footer
    let footer = create_split_footer(app);
    f.render_widget(footer, chunks[2]);
}

fn create_split_header(diff_result: &DiffResult, current_diff_index: usize) -> Paragraph<'static> {
    let left_file = diff_result.left_file.as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let right_file = diff_result.right_file.as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let current_diff = if !diff_result.entries.is_empty() {
        format!("Diff {}/{}", current_diff_index + 1, diff_result.entries.len())
    } else {
        "No differences".to_string()
    };

    let header_text = vec![
        Line::from(vec![
            Span::styled("JSON Diff Viewer - Split Screen", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw(format!("Left: {} | Right: {} | {}", left_file, right_file, current_diff)),
        ]),
    ];

    Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Split View"))
        .wrap(Wrap { trim: true })
}

fn create_file_content(content: &[String], scroll: usize, title: &'static str, diff_result: &DiffResult, is_left: bool, current_diff_index: usize) -> Paragraph<'static> {
    let visible_lines = 20; // Adjust based on terminal size
    let start = scroll;
    let end = (start + visible_lines).min(content.len());

    let mut lines = Vec::new();

    for (i, line) in content.iter().enumerate().skip(start).take(end - start) {
        let line_number = i + 1;
        let mut spans = vec![
            Span::styled(format!("{:4} ", line_number), Style::default().fg(Color::DarkGray)),
        ];

        // Check if this line has a diff and if it's the current diff
        let (has_diff, is_current_diff) = check_diff_status(diff_result, line_number, is_left, current_diff_index);

        if has_diff {
            if is_current_diff {
                // Highlight the current diff line with a bright, distinct color
                spans.push(Span::styled(line.clone(), Style::default().bg(Color::Magenta).fg(Color::White).add_modifier(Modifier::BOLD)));
            } else {
                // Highlight other diff lines with gray
                spans.push(Span::styled(line.clone(), Style::default().bg(Color::DarkGray).fg(Color::White)));
            }
        } else {
            // Apply basic JSON syntax highlighting
            spans.extend(highlight_json_line(line));
        }

        lines.push(Line::from(spans));
    }

    Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false })
}

fn highlight_json_line(line: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if !current.is_empty() {
                    spans.push(Span::raw(current.clone()));
                    current.clear();
                }
                current.push(ch);

                // Look for the closing quote
                while let Some(next_ch) = chars.next() {
                    current.push(next_ch);
                    if next_ch == '"' && chars.peek() == Some(&':') {
                        // This is a key
                        spans.push(Span::styled(current.clone(), Style::default().fg(Color::Cyan)));
                        current.clear();
                        break;
                    } else if next_ch == '"' {
                        // This is a string value
                        spans.push(Span::styled(current.clone(), Style::default().fg(Color::Green)));
                        current.clear();
                        break;
                    }
                }
            }
            ':' | ',' | '{' | '}' | '[' | ']' => {
                if !current.is_empty() {
                    spans.push(Span::raw(current.clone()));
                    current.clear();
                }
                spans.push(Span::styled(ch.to_string(), Style::default().fg(Color::Yellow)));
            }
            _ if ch.is_numeric() => {
                current.push(ch);
                // Continue collecting digits
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_numeric() || next_ch == '.' {
                        current.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                spans.push(Span::styled(current.clone(), Style::default().fg(Color::Magenta)));
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        spans.push(Span::raw(current));
    }

    spans
}

fn check_diff_status(diff_result: &DiffResult, line_number: usize, is_left: bool, current_diff_index: usize) -> (bool, bool) {
    let mut has_diff = false;
    let mut is_current_diff = false;

    for (index, entry) in diff_result.entries.iter().enumerate() {
        let line_matches = if is_left {
            entry.left_line == Some(line_number)
        } else {
            entry.right_line == Some(line_number)
        };

        if line_matches {
            has_diff = true;
            if index == current_diff_index {
                is_current_diff = true;
            }
            break;
        }
    }

    (has_diff, is_current_diff)
}

fn create_split_footer(app: &App) -> Paragraph<'static> {
    let format_mode = if app.use_readable_format { "Readable (default)" } else { "Symbols" };

    let current_diff_info = if !app.diff_result.entries.is_empty() {
        let entry = &app.diff_result.entries[app.current_diff_index];
        let diff_type = if app.use_readable_format {
            entry.diff_type.readable_text()
        } else {
            entry.diff_type.symbol()
        };
        format!("Current: {} {}", diff_type, entry.path)
    } else {
        "No differences".to_string()
    };

    let nav_info = format!(
        "Diff {}/{} | {} | Format: {} | j/k: scroll, n/N: next/prev diff, v: view, r: format, h/?: help, q: quit",
        if app.diff_result.entries.is_empty() { 0 } else { app.current_diff_index + 1 },
        app.diff_result.entries.len(),
        current_diff_info,
        format_mode
    );

    let text = vec![
        Line::from(vec![
            Span::raw(nav_info),
        ]),
    ];

    Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .wrap(Wrap { trim: true })
}

fn create_header(diff_result: &DiffResult) -> Paragraph<'static> {
    let left_file = diff_result.left_file.as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let right_file = diff_result.right_file.as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let header_text = vec![
        Line::from(vec![
            Span::styled("JSON Diff Viewer", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw(format!("Left: {}, Right: {}", left_file, right_file)),
        ]),
    ];

    Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Header"))
        .wrap(Wrap { trim: true })
}

fn create_diff_content(app: &App) -> List<'static> {
    let mut list_items = Vec::new();

    for entry in app.diff_result.entries.iter() {
        let color = match entry.diff_type {
            DiffType::Added => Color::Green,
            DiffType::Removed => Color::Red,
            DiffType::Modified => Color::Yellow,
            DiffType::ArrayItemChanged => Color::Cyan,
            DiffType::ArrayReordered => Color::Magenta,
            DiffType::Ignored => Color::DarkGray,
        };

        let entry_text = if app.use_readable_format {
            entry.format_readable()
        } else {
            format!("{}", entry)
        };
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled(entry_text, Style::default().fg(color)),
        ])));
    }

    List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Diff Result"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
}

fn create_footer(app: &App) -> Paragraph<'static> {
    let format_mode = if app.use_readable_format { "Readable (default)" } else { "Symbols" };
    let view_mode = match app.view_mode {
        ViewMode::List => "List",
        ViewMode::SplitScreen => "Split",
    };
    let nav_info = format!(
        "Entry {}/{} | View: {} | Format: {} | j/k: navigate, v: view, r: format, h/?: help, q: quit",
        if app.diff_result.entries.is_empty() { 0 } else { app.current_index + 1 },
        app.diff_result.entries.len(),
        view_mode,
        format_mode
    );

    let text = vec![
        Line::from(vec![
            Span::raw(nav_info),
        ]),
    ];

    Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .wrap(Wrap { trim: true })
}

fn create_help_popup() -> Paragraph<'static> {
    let text = vec![
        Line::from(Span::styled("JSON Diff Viewer Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("View Modes:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  v: Toggle between List and Split-Screen view"),
        Line::from(""),
        Line::from(Span::styled("List View Navigation:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j, Down Arrow: Move to next diff entry"),
        Line::from("  k, Up Arrow: Move to previous diff entry"),
        Line::from(""),
        Line::from(Span::styled("Split-Screen View Navigation:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  j, Down Arrow: Scroll down both files"),
        Line::from("  k, Up Arrow: Scroll up both files"),
        Line::from("  n: Jump to next diff location (in line number order)"),
        Line::from("  N: Jump to previous diff location (in line number order)"),
        Line::from(""),
        Line::from(Span::styled("Display Controls:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  r: Toggle between readable (default) and symbols format"),
        Line::from("  h, ?: Toggle help"),
        Line::from("  q, Esc: Quit"),
        Line::from(""),
        Line::from(Span::styled("Diff Type Symbols:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  + (ADDED): Property exists in target but not in source"),
        Line::from("  - (REMOVED): Property exists in source but not in target"),
        Line::from("  ~ (MODIFIED): Property exists in both but with different values"),
        Line::from("  ! (ARRAY_ITEM_CHANGED): Array item has changed"),
        Line::from("  * (ARRAY_REORDERED): Array elements are reordered"),
        Line::from("  ? (IGNORED): Property was ignored based on configuration"),
        Line::from(""),
        Line::from(Span::styled("Split-Screen Features:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  • JSON syntax highlighting"),
        Line::from("  • Line numbers"),
        Line::from("  • Highlighted diff lines"),
        Line::from("  • Side-by-side comparison"),
        Line::from(""),
        Line::from("Note: Mouse operations are not supported"),
        Line::from("Press any key to close this help"),
    ];

    Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: true })
}

/// Helper function to create a centered rect using a percentage of the available rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
