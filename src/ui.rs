use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

// Colour Scheme 1 - Deuteranopia
const BG: Color = Color::Rgb(10, 14, 22);
const PANEL: Color = Color::Rgb(17, 24, 36);
const MUTED: Color = Color::Rgb(125, 143, 164);

const BLUE: Color = Color::Rgb(0, 137, 232);
const RED: Color = Color::Rgb(209, 74, 81);
const ORANGE: Color = Color::Rgb(242, 162, 52);

const WHITE: Color = Color::Rgb(225, 235, 245);
const GRID: Color = Color::Rgb(34, 47, 63);
const BORDER: Color = Color::Rgb(42, 57, 77);

// Use ASCII characters for every panel border.
const ASCII_BORDERS: symbols::border::Set = symbols::border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();

    frame.render_widget(Block::default().style(Style::default().bg(BG)), size);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(8),
            Constraint::Length(2),
        ])
        .margin(1)
        .split(size);

    draw_header(frame, app, outer[0]);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(72), Constraint::Percentage(28)])
        .split(outer[1]);

    draw_chart(frame, app, middle[0]);
    draw_market_panel(frame, app, middle[1]);
    draw_hardware(frame, app, outer[2]);
    draw_footer(frame, app, outer[3]);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let title = Line::from(vec![
        Span::styled(
            "  HARDWARE ",
            Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "EXCHANGE",
            Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
        ),
        Span::styled("   //   LIVE SYSTEM MARKET", Style::default().fg(MUTED)),
        Span::raw("                                      "),
        Span::styled(
            if app.paused {
                "[ PAUSED ] "
            } else {
                "[ MARKET OPEN ] "
            },
            Style::default()
                .fg(if app.paused { RED } else { BLUE })
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            Local::now().format("%H:%M:%S").to_string(),
            Style::default().fg(WHITE),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(title).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_set(ASCII_BORDERS)
                .border_style(Style::default().fg(BORDER)),
        ),
        area,
    );
}

fn draw_chart(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("  BTC / SYSTEM INDEX  -  LIVE CANDLESTICKS  ")
        .borders(Borders::ALL)
        .border_set(ASCII_BORDERS)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(PANEL));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 18 || inner.height < 5 {
        return;
    }

    let label_width = 9usize;
    let plot_width = inner.width.saturating_sub(label_width as u16) as usize;
    let height = inner.height as usize;
    let candles = &app.market.candles;

    if candles.is_empty() {
        frame.render_widget(
            Paragraph::new("  Collecting first hardware sample...")
                .style(Style::default().fg(MUTED)),
            Rect::new(inner.x + 1, inner.y + 1, inner.width.saturating_sub(2), 1),
        );
        return;
    }

    let visible = (plot_width / 2).max(1).min(candles.len());
    let shown = &candles[candles.len() - visible..];

    let mut min_price = shown.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);

    let mut max_price = shown
        .iter()
        .map(|c| c.high)
        .fold(f64::NEG_INFINITY, f64::max);

    let padding = ((max_price - min_price) * 0.08).max(0.15);
    min_price -= padding;
    max_price += padding;

    let range = (max_price - min_price).max(0.01);

    let mut cells = vec![vec![None::<(char, Style)>; plot_width]; height];

    for (index, candle) in shown.iter().enumerate() {
        let x = index * 2;

        if x >= plot_width {
            break;
        }

        let color = if candle.bullish() { BLUE } else { RED };
        let style = Style::default().fg(color);

        let y_of = |price: f64| -> usize {
            (((max_price - price) / range) * (height.saturating_sub(1) as f64))
                .round()
                .clamp(0.0, height.saturating_sub(1) as f64) as usize
        };

        let high_y = y_of(candle.high);
        let low_y = y_of(candle.low);
        let open_y = y_of(candle.open);
        let close_y = y_of(candle.close);

        // ASCII candle wick.
        for y in high_y.min(low_y)..=high_y.max(low_y) {
            cells[y][x] = Some(('|', style));
        }

        // ASCII candle body.
        let body_top = open_y.min(close_y);
        let body_bottom = open_y.max(close_y);

        for y in body_top..=body_bottom {
            cells[y][x] = Some(('#', style.add_modifier(Modifier::BOLD)));
        }
    }

    let mut lines = Vec::with_capacity(height);

    for row in 0..height {
        let price = max_price - (row as f64 / height.saturating_sub(1).max(1) as f64) * range;

        let mut spans = vec![Span::styled(
            format!("{:>7.2} ", price),
            Style::default().fg(MUTED),
        )];

        for col in 0..plot_width {
            if let Some((ch, style)) = cells[row][col] {
                spans.push(Span::styled(ch.to_string(), style));
            } else {
                spans.push(Span::styled(".", Style::default().fg(GRID)));
            }
        }

        lines.push(Line::from(spans));
    }

    frame.render_widget(
        Paragraph::new(Text::from(lines)).style(Style::default().bg(PANEL)),
        inner,
    );
}

fn draw_market_panel(frame: &mut Frame, app: &App, area: Rect) {
    let price_color = if app.market.change_percent >= 0.0 {
        BLUE
    } else {
        RED
    };

    let change_sign = if app.market.change_percent >= 0.0 {
        "+"
    } else {
        ""
    };

    let lines = vec![
        Line::from(Span::styled(
            "  INDEX PRICE",
            Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  ${:.2}", app.market.price),
            Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!(
                "  {}{:.3}% this tick",
                change_sign, app.market.change_percent
            ),
            Style::default().fg(price_color),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  SESSION RANGE",
            Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("  HIGH  ${:.2}", app.market.high_24)),
        Line::from(format!("  LOW   ${:.2}", app.market.low_24)),
        Line::from(""),
        Line::from(Span::styled(
            format!("  CANDLES  {}", app.market.candles.len()),
            Style::default().fg(BLUE),
        )),
        Line::from(Span::styled(
            format!("  SAMPLES  {}", app.samples),
            Style::default().fg(MUTED),
        )),
    ];

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title("  MARKET DATA  ")
                    .borders(Borders::ALL)
                    .border_set(ASCII_BORDERS)
                    .border_style(Style::default().fg(BORDER))
                    .style(Style::default().bg(PANEL)),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_hardware(frame: &mut Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    draw_metric(
        frame,
        columns[0],
        "CPU LOAD",
        app.metrics.cpu_percent,
        format!("{:.1}%", app.metrics.cpu_percent),
        BLUE,
    );

    draw_metric(
        frame,
        columns[1],
        "MEMORY",
        app.metrics.memory_percent,
        format!(
            "{:.1}% ({:.1}/{:.1} GB)",
            app.metrics.memory_percent, app.metrics.memory_used_gb, app.metrics.memory_total_gb
        ),
        ORANGE,
    );

    let network_value = (app.metrics.network_rx_kbps + app.metrics.network_tx_kbps).min(100.0);

    let network_label = format!(
        "RX {:.0} KB/s TX {:.0} KB/s",
        app.metrics.network_rx_kbps, app.metrics.network_tx_kbps
    );

    draw_metric(
        frame,
        columns[2],
        "NETWORK I/O",
        network_value,
        network_label,
        ORANGE,
    );

    let uptime = app.metrics.uptime_seconds;

    let uptime_label = format!(
        "{}d {:02}h {:02}m",
        uptime / 86_400,
        (uptime / 3_600) % 24,
        (uptime / 60) % 60
    );

    let info = vec![
        Line::from(Span::styled(
            " SYSTEM",
            Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            truncate(&app.metrics.hostname, 24),
            Style::default().fg(WHITE),
        )),
        Line::from(Span::styled(
            format!("UPTIME  {}", uptime_label),
            Style::default().fg(BLUE),
        )),
    ];

    frame.render_widget(
        Paragraph::new(info).block(
            Block::default()
                .title("  HOST  ")
                .borders(Borders::ALL)
                .border_set(ASCII_BORDERS)
                .border_style(Style::default().fg(BORDER))
                .style(Style::default().bg(PANEL)),
        ),
        columns[3],
    );
}

fn draw_metric(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    value: f64,
    label: String,
    color: Color,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .margin(1)
        .split(area);

    frame.render_widget(
        Paragraph::new(title).style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        chunks[0],
    );

    // Ten-character ASCII usage bar.
    let bar_width = 10usize;
    let ratio = (value / 100.0).clamp(0.0, 1.0);
    let filled = (ratio * bar_width as f64).round() as usize;
    let empty = bar_width.saturating_sub(filled);

    let bar = format!("[{}{}] {}", "#".repeat(filled), ".".repeat(empty), label);

    frame.render_widget(
        Paragraph::new(bar).style(Style::default().fg(color)),
        chunks[1],
    );

    frame.render_widget(
        Paragraph::new("LIVE TELEMETRY").style(Style::default().fg(MUTED)),
        chunks[2],
    );
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let elapsed = app.started_at.elapsed().as_secs();

    let key_style = Style::default()
        .fg(BG)
        .bg(BLUE)
        .add_modifier(Modifier::BOLD);

    let line = Line::from(vec![
        Span::styled(" Q ", key_style),
        Span::styled(" quit  ", Style::default().fg(MUTED)),
        Span::styled(" SPACE ", key_style),
        Span::styled(" pause  ", Style::default().fg(MUTED)),
        Span::styled(" R ", key_style),
        Span::styled(" reset  ", Style::default().fg(MUTED)),
        Span::styled(" +/- ", key_style),
        Span::styled(" sample speed  ", Style::default().fg(MUTED)),
        Span::styled(
            format!(
                "{}s session  |  {}ms interval",
                elapsed, app.sample_interval_ms
            ),
            Style::default().fg(MUTED),
        ),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        value.to_string()
    } else {
        value
            .chars()
            .take(max.saturating_sub(3))
            .collect::<String>()
            + "..."
    }
}
