use crate::languages::Language;
use crate::project_finder::ProjectInfo;
use crossterm::event::KeyModifiers;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::prelude::*;
use ratatui::{
    crossterm::event::{self, KeyCode},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    DefaultTerminal,
};
use std::io;
use std::process::Command;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

fn truncate_str(s: &str, max_width: usize) -> String {
    let width = s.width();
    if width <= max_width {
        format!("{:width$}", s, width = max_width)
    } else {
        // Account for the "..." when truncating
        let mut truncated = String::with_capacity(max_width);
        let mut current_width = 0;

        for c in s.chars() {
            let char_width = UnicodeWidthChar::width(c).unwrap_or(1);
            if current_width + char_width + 3 > max_width {
                break;
            }
            truncated.push(c);
            current_width += char_width;
        }

        format!("{:width$}", truncated + "...", width = max_width)
    }
}

fn prettify_home(s: &str) -> String {
    let home = dirs::home_dir().unwrap();
    let home_str = home.to_str().unwrap();
    s.replace(home_str, "~")
}

pub struct App {
    projects: Vec<ProjectInfo>,
    selected: usize,
    search_active: bool,
    search_query: String,
    filtered_indices: Vec<usize>,
    matcher: SkimMatcherV2,
}

impl App {
    pub fn new(projects: Vec<ProjectInfo>) -> Self {
        let indices: Vec<usize> = (0..projects.len()).collect();
        Self {
            projects,
            selected: 0,
            search_active: false,
            search_query: String::new(),
            filtered_indices: indices,
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn next(&mut self) {
        if !self.filtered_indices.is_empty() {
            let current_pos = self
                .filtered_indices
                .iter()
                .position(|&x| x == self.selected)
                .unwrap_or(0);
            let next_pos = (current_pos + 1) % self.filtered_indices.len();
            self.selected = self.filtered_indices[next_pos];
        }
    }

    pub fn previous(&mut self) {
        if !self.filtered_indices.is_empty() {
            let current_pos = self
                .filtered_indices
                .iter()
                .position(|&x| x == self.selected)
                .unwrap_or(0);
            let prev_pos = if current_pos > 0 {
                current_pos - 1
            } else {
                self.filtered_indices.len() - 1
            };
            self.selected = self.filtered_indices[prev_pos];
        }
    }

    pub fn update_search(&mut self, new_char: char) {
        self.search_query.push(new_char);
        self.filter_projects();
    }

    pub fn backspace_search(&mut self) {
        self.search_query.pop();
        self.filter_projects();
    }

    fn filter_projects(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.projects.len()).collect();
            return;
        }

        let mut scored_indices: Vec<(i64, usize)> = self
            .projects
            .iter()
            .enumerate()
            .filter_map(|(idx, proj)| {
                let search_text = format!("{} {}", proj.name, proj.directory);
                self.matcher
                    .fuzzy_match(&search_text, &self.search_query)
                    .map(|score| (score, idx))
            })
            .collect();

        // Sort by score descending
        scored_indices.sort_by(|a, b| b.0.cmp(&a.0));

        self.filtered_indices = scored_indices.into_iter().map(|(_, idx)| idx).collect();

        // Update selected to first match if we have results
        if let Some(&first_match) = self.filtered_indices.first() {
            self.selected = first_match;
        }
    }

    pub fn open_in_tmux(&self) -> io::Result<()> {
        if let Some(project) = self.projects.get(self.selected) {
            // Create new tmux window in project directory
            Command::new("tmux")
                .args(["new-window", "-c", &project.directory])
                .status()?;

            // Split the window and make it 10% height
            Command::new("tmux")
                .args(["split-window", "-v", "-l", "10%", "-c", &project.directory])
                .status()?;

            // Select the top pane
            Command::new("tmux")
                .args(["select-pane", "-t", "1"])
                .status()?;

            // Launch nvim in the top pane
            Command::new("tmux")
                .args(["send-keys", "nvim .", "C-m"])
                .status()?;

            // Go back to previous window
            Command::new("tmux").args(["last-window"]).status()?;

            // Kill the new window
            Command::new("tmux").args(["kill-window"]).status()?;

            Ok(())
        } else {
            Ok(()) // No project selected
        }
    }
}

fn run(mut terminal: DefaultTerminal, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            // Calculate available width
            let total_width = frame.area().width as usize;
            let min_width = 50;
            let max_width = 120;

            // Calculate the actual width we'll use (bounded between min and max)
            let content_width = total_width.clamp(min_width, max_width);

            // Padding needed to center
            let padding = (total_width.saturating_sub(content_width)) / 2;

            let outer = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Length(padding as u16),
                    Constraint::Length(content_width as u16),
                    Constraint::Length(padding as u16),
                ])
                .split(frame.area());

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(3), Constraint::Min(0)])
                .split(outer[1]);

            // Show different help text based on search state
            let help_text = if app.search_active {
                format!("Search: {} (Esc to cancel)", app.search_query)
            } else {
                "Project Browser (↑/k ↓/j to move, / to search, Enter to select)".to_string()
            };

            frame.render_widget(
                Paragraph::new(help_text)
                    .block(Block::default().borders(Borders::ALL))
                    .alignment(Alignment::Center),
                layout[0],
            );

            let items: Vec<ListItem> = app
                .filtered_indices
                .iter()
                .map(|&idx| {
                    let project = &app.projects[idx];
                    let icon = Language::from_name(&project.language)
                        .map(|l| l.icon)
                        .unwrap_or("󰄛");

                    let style = if idx == app.selected {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };

                    // Format with fixed-width columns for alignment
                    // Calculate dynamic column widths based on available space
                    let available_width = content_width.saturating_sub(3); // 3 for spacing
                    let icon_width = 2;
                    let name_ratio = 0.35; // Name gets 35% of remaining space
                    let name_width = ((available_width - icon_width) as f64 * name_ratio) as usize;
                    let path_width = available_width - icon_width - name_width;

                    ListItem::new(format!(
                        "{:2} {:<width$} {:<path_width$}",
                        icon,
                        truncate_str(&project.name, name_width),
                        truncate_str(&prettify_home(&project.directory), path_width),
                        width = name_width,
                        path_width = path_width
                    ))
                    .style(style)
                })
                .collect();

            let projects_list =
                List::new(items).block(Block::default().borders(Borders::ALL).title("Projects"));

            frame.render_widget(projects_list, layout[1]);
        })?;

        if let event::Event::Key(key) = event::read()? {
            match (key.code, key.modifiers) {
                // If we're in search mode, handle it differently
                (code, _mods) if app.search_active => match code {
                    KeyCode::Esc => {
                        app.search_active = false;
                        app.search_query.clear();
                        app.filter_projects();
                    }
                    KeyCode::Backspace => {
                        app.backspace_search();
                    }
                    KeyCode::Char(c) => {
                        app.update_search(c);
                    }
                    KeyCode::Enter => {
                        app.open_in_tmux()?;
                        return Ok(());
                    }
                    _ => {}
                },
                // Normal navigation mode
                (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => return Ok(()),
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                (KeyCode::Down | KeyCode::Char('j'), _) => app.next(),
                (KeyCode::Up | KeyCode::Char('k'), _) => app.previous(),
                (KeyCode::Char('/'), _) => {
                    app.search_active = true;
                }
                (KeyCode::Enter, _) => {
                    app.open_in_tmux()?;
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

pub fn main(projects: Vec<ProjectInfo>) -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app = App::new(projects);
    let app_result = run(terminal, app);
    ratatui::restore();
    app_result
}
