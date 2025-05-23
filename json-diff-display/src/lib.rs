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

/// App holds the state of the application
pub struct App {
    diff_result: DiffResult,
    current_index: usize,
    quit: bool,
    help_visible: bool,
}

impl App {
    pub fn new(diff_result: DiffResult) -> Self {
        Self {
            diff_result,
            current_index: 0,
            quit: false,
            help_visible: false,
        }
    }

    pub fn next(&mut self) {
        if !self.diff_result.entries.is_empty() {
            self.current_index = (self.current_index + 1) % self.diff_result.entries.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.diff_result.entries.is_empty() {
            self.current_index = if self.current_index > 0 {
                self.current_index - 1
            } else {
                self.diff_result.entries.len() - 1
            };
        }
    }

    pub fn toggle_help(&mut self) {
        self.help_visible = !self.help_visible;
    }
}

/// Runs the terminal UI for displaying diff results
pub fn run_display(diff_result: DiffResult) -> Result<()> {
    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(diff_result);

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
                KeyCode::Char('h') | KeyCode::Char('?') => app.toggle_help(),
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

    // Help overlay
    if app.help_visible {
        let help = create_help_popup();
        let popup_area = centered_rect(60, 40, size);
        f.render_widget(Clear, popup_area);
        f.render_widget(help, popup_area);
    }
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

        let entry_text = format!("{}", entry);
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled(entry_text, Style::default().fg(color)),
        ])));
    }

    List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Diff Result"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
}

fn create_footer(app: &App) -> Paragraph<'static> {
    let nav_info = format!(
        "Entry {}/{} | Press j/k to navigate, h/? for help, q to quit",
        if app.diff_result.entries.is_empty() { 0 } else { app.current_index + 1 },
        app.diff_result.entries.len()
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
        Line::from("Navigation (Keyboard Only):"),
        Line::from("  j, Down Arrow: Move to next diff"),
        Line::from("  k, Up Arrow: Move to previous diff"),
        Line::from(""),
        Line::from("Other Controls:"),
        Line::from("  h, ?: Toggle help"),
        Line::from("  q, Esc: Quit"),
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
