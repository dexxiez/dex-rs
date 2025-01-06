use anyhow::{Context, Result};
use crossterm::event::{self, KeyCode};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    DefaultTerminal,
};
use serde_json::json;
use std::{fs, io, path::Path};

pub struct CreateApp {
    project_name: String,
    selected_language: String,
    edit_mode: EditMode,
    languages: Vec<String>,
    filtered_languages: Vec<String>, // New: for showing filtered language options
}

#[derive(PartialEq, Clone)]
pub enum EditMode {
    Name,
    Language,
}

impl CreateApp {
    fn new() -> Self {
        let current_dir = std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_default();

        // Get available languages plus "UNKNOWN"
        let mut languages = crate::languages::LANGUAGES
            .iter()
            .flat_map(|l| l.names.first().cloned())
            .map(String::from)
            .collect::<Vec<_>>();
        languages.push("UNKNOWN".to_string());

        Self {
            project_name: current_dir,
            selected_language: String::new(),
            edit_mode: EditMode::Name,
            filtered_languages: languages.clone(),
            languages,
        }
    }

    fn filter_languages(&mut self) {
        let query = self.selected_language.to_lowercase();
        self.filtered_languages = self
            .languages
            .iter()
            .filter(|lang| lang.to_lowercase().contains(&query))
            .cloned()
            .collect();
    }

    fn handle_input(&mut self, key: KeyCode) -> bool {
        match (self.edit_mode.clone(), key) {
            (_, KeyCode::Tab) => {
                self.edit_mode = match self.edit_mode {
                    EditMode::Name => EditMode::Language,
                    EditMode::Language => EditMode::Name,
                };
            }
            (EditMode::Name, KeyCode::Char(c)) => {
                self.project_name.push(c);
            }
            (EditMode::Name, KeyCode::Backspace) => {
                self.project_name.pop();
            }
            (EditMode::Language, KeyCode::Char(c)) => {
                self.selected_language.push(c);
                self.filter_languages();
            }
            (EditMode::Language, KeyCode::Backspace) => {
                self.selected_language.pop();
                self.filter_languages();
            }
            (EditMode::Language, KeyCode::Enter) => {
                // Only allow Enter if the language exists in our filtered list
                // or if it exactly matches "UNKNOWN"
                if self.filtered_languages.len() == 1 {
                    self.selected_language = self.filtered_languages[0].clone();
                    return true;
                } else if self.selected_language == "UNKNOWN" {
                    return true;
                }
                return false;
            }
            (EditMode::Name, KeyCode::Enter) => {
                self.edit_mode = EditMode::Language;
            }
            (_, KeyCode::Esc) => std::process::exit(0),
            _ => {}
        }
        false
    }
}

fn run_ui(mut terminal: DefaultTerminal) -> io::Result<(String, String)> {
    let mut app = CreateApp::new();

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let box_width = 60u16;
            let box_height = 16u16; // Increased height for language suggestions
            let box_x = (area.width.saturating_sub(box_width)) / 2;
            let box_y = (area.height.saturating_sub(box_height)) / 2;

            let centered_rect = Rect::new(box_x, box_y, box_width, box_height);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3), // Project name
                    Constraint::Length(3), // Language input
                    Constraint::Length(5), // Language suggestions
                    Constraint::Min(1),    // Help text
                ])
                .split(centered_rect);

            // Project name input
            let name_style = if app.edit_mode == EditMode::Name {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let name_block = Block::default()
                .title("Project Name")
                .borders(Borders::ALL)
                .style(name_style);

            frame.render_widget(
                Paragraph::new(app.project_name.as_str()).block(name_block),
                chunks[0],
            );

            // Language input
            let lang_style = if app.edit_mode == EditMode::Language {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let lang_block = Block::default()
                .title("Language")
                .borders(Borders::ALL)
                .style(lang_style);

            frame.render_widget(
                Paragraph::new(app.selected_language.as_str()).block(lang_block),
                chunks[1],
            );

            // Language suggestions
            if app.edit_mode == EditMode::Language {
                let suggestions: Vec<ListItem> = app
                    .filtered_languages
                    .iter()
                    .map(|lang| ListItem::new(lang.as_str()))
                    .collect();

                let suggestions_list = List::new(suggestions).block(
                    Block::default()
                        .title("Available Languages")
                        .borders(Borders::ALL),
                );

                frame.render_widget(suggestions_list, chunks[2]);
            }

            // Help text
            let help_text = match app.edit_mode {
                EditMode::Name => "Tab/Enter to proceed to language | Esc to quit",
                EditMode::Language => {
                    if app.filtered_languages.len() == 1 {
                        "Enter to select highlighted language | Esc to quit"
                    } else {
                        "Type to filter languages | Enter when one remains | Esc to quit"
                    }
                }
            };
            frame.render_widget(
                Paragraph::new(help_text).alignment(Alignment::Center),
                chunks[3],
            );
        })?;

        if let event::Event::Key(key) = event::read()? {
            if app.handle_input(key.code) {
                return Ok((app.project_name, app.selected_language));
            }
        }
    }
}

pub fn create_project() -> Result<()> {
    let project_file = Path::new(".dexproject");

    if project_file.exists() {
        // Check if user wants to overwrite
        let mut overwrite = String::new();
        println!("Project file already exists. Overwrite? (y/n)");
        io::stdin().read_line(&mut overwrite)?;
        if overwrite.trim() != "y" {
            anyhow::bail!("Project file already exists");
        }
    }

    let mut terminal = ratatui::init();
    terminal.clear()?;

    let (name, language) = run_ui(terminal)?;

    ratatui::restore();

    let project_json = json!({
        "name": name,
        "language": language
    });

    fs::write(project_file, serde_json::to_string_pretty(&project_json)?)
        .context("Failed to write project file")?;

    Ok(())
}
