use std::io;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal,
};

use ratatui::prelude::*;

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let outer = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Percentage(25),
                    Constraint::Percentage(50),
                    Constraint::Percentage(25),
                ])
                .split(frame.area());
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ])
                .split(outer[1]);

            frame.render_widget(
                Paragraph::new("Top").block(Block::new().borders(Borders::ALL)),
                layout[0],
            );

            frame.render_widget(
                Paragraph::new("Mid").block(Block::new().borders(Borders::ALL)),
                layout[1],
            );
            frame.render_widget(
                Paragraph::new("Bottom").block(Block::new().borders(Borders::ALL)),
                layout[2],
            );
        })?;

        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
                return Ok(());
            }
        }
    }
}

pub fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal);
    ratatui::restore();
    app_result
}
