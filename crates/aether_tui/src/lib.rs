use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline, Cell, Row, Table},
    Terminal,
};
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use aether_traits::SharedState;
use std::sync::atomic::Ordering;

pub async fn run_tui(shared_state: Arc<SharedState>) -> Result<()> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut last_tick = Instant::now();
    let _tick_rate = Duration::from_millis(33); // ~30 FPS for smooth UI without CPU waste
    
    let mut rps_history: Vec<u64> = vec![0; 100];
    let mut last_requests = 0;

    // Main Loop
    loop {
        // [Directive: Atomic Snapshots]
        // Perform a lock-free Snapshot Read once per frame.
        let total_requests = shared_state.total_requests.load(Ordering::Relaxed);
        let total_bytes = shared_state.total_bytes.load(Ordering::Relaxed);
        let error_count = shared_state.error_count.load(Ordering::Relaxed);
        let target_rps = shared_state.target_rps.load(Ordering::Relaxed);
        let jitter = shared_state.jitter_factor.load(Ordering::Relaxed);
        
        // Calculate RPS
        let now = Instant::now();
        if now.duration_since(last_tick) >= Duration::from_secs(1) {
            let diff = total_requests - last_requests;
            rps_history.remove(0);
            rps_history.push(diff);
            last_requests = total_requests;
            last_tick = now;
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3), // Header
                        Constraint::Length(10), // Metrics & Sparkline
                        Constraint::Min(10),   // Worker Hive
                        Constraint::Length(3), // Footer / Controls
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // 1. Header: Branding & Uptime
            let header = Paragraph::new(Line::from(vec![
                Span::styled(" AETHER-RED ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" | Adversarial C2 [STAGE: TEST_PHASE]"),
            ]))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // 2. Metrics Panel (Sparkline + Stats)
            let metrics_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(chunks[1]);

            let rps_spark = Sparkline::default()
                .block(Block::default().title("RPS Velocity").borders(Borders::ALL))
                .data(&rps_history)
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(rps_spark, metrics_chunks[0]);

            let stats_text = format!(
                "Requests: {}\nTraffic:  {:.2} MB\nErrors:   {}\nTarget:   {} RPS (Jitter: {}%)",
                total_requests,
                total_bytes as f64 / 1_048_576.0,
                error_count,
                target_rps,
                jitter
            );
            let stats = Paragraph::new(stats_text)
                .block(Block::default().title("Telemetry Snapshot").borders(Borders::ALL))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(stats, metrics_chunks[1]);

            // 3. Worker Hive Grid (Heatmap with Liveness Decay)
            render_worker_hive(f, chunks[2], &shared_state);

            // 4. Footer & Control Knobs
            let footer = Paragraph::new(" [Q]uit | [↑/↓] Throttle RPS | [←/→] Jitter Adjust | [C]lear Logs")
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(footer, chunks[3]);
        })?;

        // [Directive: Interactive C2 Knobs]
        if event::poll(Duration::from_millis(30))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => {
                        shared_state.target_rps.fetch_add(100, Ordering::Relaxed);
                    }
                    KeyCode::Down => {
                        let current = shared_state.target_rps.load(Ordering::Relaxed);
                        if current > 100 {
                            shared_state.target_rps.fetch_sub(100, Ordering::Relaxed);
                        }
                    }
                    KeyCode::Right => {
                        let current = shared_state.jitter_factor.load(Ordering::Relaxed);
                        if current < 100 {
                            shared_state.jitter_factor.fetch_add(5, Ordering::Relaxed);
                        }
                    }
                    KeyCode::Left => {
                        let current = shared_state.jitter_factor.load(Ordering::Relaxed);
                        if current >= 5 {
                            shared_state.jitter_factor.fetch_sub(5, Ordering::Relaxed);
                        }
                    }
                    _ => {}
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

/// [Directive: Worker Hive Grid]
/// Renders a 2D color-coded heatmap of thousands of workers.
/// [Flaw 2 Fix: Heartbeat-Driven Liveness Decay] Detects zombie threads via timestamp delta.
/// [Fix 3] Swarm Sampling: Optimize rendering for massive worker pools.
fn render_worker_hive(f: &mut ratatui::Frame, area: Rect, shared_state: &Arc<SharedState>) {
    let num_workers = shared_state.worker_statuses.len();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    // [Fix 3] Swarm Sampling Logic
    // If we have thousands of workers, rendering every cell is a CPU killer.
    // We sample the population to maintain responsiveness.
    let (display_count, sample_step) = if num_workers > 400 {
        (400, num_workers / 400)
    } else {
        (num_workers, 1)
    };

    let mut rows = Vec::new();
    let mut current_row = Vec::new();
    let cols = 20; 
    
    for i in (0..num_workers).step_by(sample_step).take(display_count) {
        let status = shared_state.worker_statuses[i].load(Ordering::Relaxed);
        let last_seen = shared_state.worker_heartbeats[i].load(Ordering::Relaxed);
        
        let (symbol, color) = if now > last_seen + 5 {
            (" 💀 ", Color::Gray) 
        } else {
            match status {
                0 => (" ● ", Color::DarkGray), // Idle
                1 => (" ⚡ ", Color::Blue),     // Handshaking
                2 => (" ☄ ", Color::Green),    // Sending
                3 => (" ⏳ ", Color::Yellow),   // Blocked
                _ => (" ✖ ", Color::Red),     // Dead
            }
        };

        current_row.push(Cell::from(Span::styled(symbol, Style::default().fg(color))));
        
        if current_row.len() == cols || i >= num_workers - sample_step {
            rows.push(Row::new(current_row));
            current_row = Vec::new();
        }
    }

    let title = if sample_step > 1 {
        format!("Swarm Intelligence Hive (Sampled 1:{})", sample_step)
    } else {
        "Swarm Intelligence Intelligence Hive".to_string()
    };

    let table = Table::new(rows, vec![Constraint::Length(4); cols])
        .block(Block::default().title(title).borders(Borders::ALL));
    f.render_widget(table, area);
}
