// UI module - terminal rendering with ratatui

pub mod chart;
mod dashboard;
mod widgets;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap, Clear},
};

// Color constants for consistent styling
pub const GREEN: &str = "\x1b[32m";
pub const RED: &str = "\x1b[31m";
pub const YELLOW: &str = "\x1b[33m";
pub const GRAY: &str = "\x1b[90m";
pub const RESET: &str = "\x1b[0m";

// Main application state
#[derive(Debug, Default)]
pub struct App {
    /// Market engine (holds candle history)
    market: crate::market::engine::MarketEngine,
    
    /// Hardware collector
    hardware: crate::hardware::collector::HardwareCollector,
    
    /// Configuration
    config_file_path: String,
    
    /// Running state
    is_paused: bool,
    needs_reset: bool,
    
    /// Update frequency control
    update_interval_ms: u64,
}

impl App {
    /// Create new application instance
    pub fn new() -> Result<Self> {
        let collector = crate::hardware::collector::HardwareCollector::init_hardware()?;
        
        // Try to load config file
        let config_path = "candlestick.toml";
        
        Ok(App {
            market: crate::market::engine::MarketEngine::default(),
            hardware: collector,
            config_file_path: config_path.to_string(),
            is_paused: false,
            needs_reset: true, // Start fresh with initial price
            update_interval_ms: 2500,
        })
    }

    /// Initialize market with starting price
    fn initialize_market(&mut self) {
        let mut rng = rand::thread_rng();
        let start_price = 48.0 + rng.gen_range(-2.0..=2.0);
        
        self.market.current_price = start_price.clamp(
            self.market.config.min_price,
            self.market.config.max_price
        );
    }

    /// Main application loop
    pub fn run(&mut self) -> Result<()> {
        // Setup terminal for alternate screen
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        
        let mut terminal = Terminal::new(CustomBackend::new())?;
        
        // Initialize market
        self.initialize_market();
        
        // If needs reset (start of run), process initial state
        if self.needs_reset {
            let _state = self.hardware.collect().unwrap_or_default();
            self.market.process_state(&_state).ok();
            self.needs_reset = false;
        }

        // Main event loop using tokio runtime for async operations
        #[tokio::select]
        {
            // Keyboard input channel
            let keyboard = async {
                loop {
                    if !self.is_paused {
                        // Collect hardware and update market
                        let state = match self.hardware.collect() {
                            Ok(s) => s,
                            Err(_) => Default::default(),
                        };
                        
                        if let Err(e) = self.market.process_state(&state) {
                            eprintln!("Market update error: {}", e);
                        }
                    }
                    
                    // Check for keyboard input
                    while let Ok(ev) = event::poll(std::time::Duration::from_millis(10)) {
                        if ev {
                            match event::read() {
                                Ok(Event::Key(key_event)) if key_event.kind == KeyEventKind => {
                                    let key = key_event.code;
                                    
                                    // Handle quit
                                    if matches!(key, KeyCode::Char('q') | KeyCode::Esc) {
                                        break 'keyboard None;
                                    }
                                    
                                    // Handle other keys
                                    match key {
                                        KeyCode::Char('p') => self.is_paused = !self.is_paused,
                                        KeyCode::Char('r') => {
                                            self.needs_reset = true;
                                            self.initialize_market();
                                        }
                                        KeyCode::Char('+') => {
                                            self.update_interval_ms = 
                                                (self.update_interval_ms - 500).max(100);
                                        }
                                        KeyCode::Char('-') => {
                                            self.update_interval_ms = 
                                                (self.update_interval_ms + 500).min(10000);
                                        }
                                        _ => {}
                                    }
                                }
                                _ => continue,
                            }
                        }
                    }
                }
            };

            // Drawing loop
            let drawing = async {
                loop {
                    if !self.is_paused {
                        terminal.draw(|frame| self.render(frame))?;
                    }
                    
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            };

            // Wait for quit signal
            if let Some(_) = keyboard.await {
                break;
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

        Ok(())
    }

    /// Render the application UI
    fn render(&mut self, frame: &mut Frame) -> Result<()> {
        let size = frame.area();
        
        // Title bar
        let title = Paragraph::new("HARDWARE EXCHANGE")
            .style(Style::default().add_modifier(Modifier::BOLD))
            .block(Block::default()
                .borders(Borders::ALL)
                .title("🔴 MARKET OPEN"));
        
        frame.render_widget(title, size);

        // Main chart and dashboard area
        let sub_area = Rect {
            x: 0,
            y: 1,
            width: size.width,
            height: size.height - 3,
        };

        self.draw_main_content(frame, &sub_area)?;

        Ok(())
    }

    /// Draw main content (chart + dashboard)
    fn draw_main_content(&self, frame: &mut Frame, area: &Rect) -> Result<()> {
        // Split vertically: chart on top, hardware metrics below
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Chart area
                Constraint::Min(8),         // Dashboard area
            ])
            .split(*area);

        // Draw candlestick chart
        self.create_chart(chunks[0], &self.market)?;
        
        // Draw hardware dashboard
        if let Ok(state) = self.hardware.collect() {
            self.create_dashboard(chunks[1], &state, &self.market.candles())?;
        }

        // Footer with controls
        let footer = Paragraph::new("Press: Q=Quit | P=Pause/Resume | R=Reset | +/-=Speed")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        
        let footer_area = Rect {
            x: 0,
            y: chunks[1].bottom() + 1,
            width: size.width,
            height: 2,
        };
        
        frame.render_widget(footer, footer_area);

        Ok(())
    }

    /// Create candlestick chart widget
    fn create_chart(&self, area: &Rect, market: &crate::market::engine::MarketEngine) -> Result<()> {
        let price_min = market.config.min_price;
        let price_max = market.config.max_price;
        
        // Draw chart with candles
        match CandlestickChart::new()
            .with_area(*area)
            .with_candles(market.candles())
            .with_current_price(market.get_current_price())
            .with_price_range(price_min, price_max)
            .build_widget()
        {
            Widget::Paragraph(ref p) => frame.render_widget(p.clone(), *area),
            _ => {}
        }

        Ok(())
    }

    /// Create dashboard widget with hardware info
    fn create_dashboard(
        &self, 
        area: &Rect, 
        hardware: &crate::hardware::metrics::HardwareState,
        candles: &[crate::market::candle::Candle]
    ) -> Result<()> {
        // Check if we have data
        let metrics_text = hardware.format_metrics();
        
        let dashboard_widget = Paragraph::new(&metrics_text)
            .block(Block::default()
                .title("Hardware Dashboard")
                .borders(Borders::ALL));
        
        frame.render_widget(dashboard_widget, *area);

        Ok(())
    }
}

/// Custom terminal backend for ratatui
struct CustomBackend;

impl CustomBackend {
    fn new() -> Self {
        CustomBackend
    }
}
