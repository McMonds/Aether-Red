use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;

pub async fn run_tui() -> Result<()> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main Loop
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10), // Header
                        Constraint::Percentage(80), // Body
                        Constraint::Percentage(10), // Footer
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // Header
            let header = Paragraph::new("Project ÆTHER - Secure Terminal Engine")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // Body
            let body = Paragraph::new("System Status: ACTIVE\nWorkers: 5\nProxies: Checking...")
                .block(Block::default().title("Dashboard").borders(Borders::ALL));
            f.render_widget(body, chunks[1]);

            // Footer
            let footer = Paragraph::new("Press 'q' to Quit")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        // Event Handling
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    break;
                }
            }
        }
    }

    // Restore Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}
