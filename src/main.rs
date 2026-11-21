mod app;
mod config;
mod hardware;
mod market;
mod ui;

use anyhow::Result;
use app::App;
use config::Config;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

fn main() -> Result<()> {
    let config = Config::load();
    let mut app = App::new(config);
    let mut terminal = TerminalGuard::new()?;
    let mut last_sample = Instant::now() - Duration::from_secs(5);
    let mut last_draw = Instant::now();

    loop {
        if last_sample.elapsed() >= Duration::from_millis(app.sample_interval_ms) {
            app.sample_hardware();
            if !app.paused {
                app.update_market();
            }
            last_sample = Instant::now();
        }

        if last_draw.elapsed() >= Duration::from_millis(80) {
            terminal.terminal.draw(|frame| ui::draw(frame, &app))?;
            last_draw = Instant::now();
        }

        if event::poll(Duration::from_millis(30))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char(' ') | KeyCode::Char('p') => app.paused = !app.paused,
                        KeyCode::Char('r') => app.reset_market(),
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            app.sample_interval_ms =
                                app.sample_interval_ms.saturating_sub(250).max(500);
                        }
                        KeyCode::Char('-') => {
                            app.sample_interval_ms = (app.sample_interval_ms + 250).min(10_000);
                        }
                        KeyCode::Char('1') => app.sample_interval_ms = 1_000,
                        KeyCode::Char('2') => app.sample_interval_ms = 2_500,
                        KeyCode::Char('3') => app.sample_interval_ms = 5_000,
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
