//! A terminal-based candlestick chart implementation with MA50 indicator
//! using ratatui and crossterm.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::Color,
    widgets::{
        canvas::{Canvas, Context, Line, Points},
        Block, Borders,
    },
    Frame, Terminal,
};
use std::{collections::VecDeque, io, time::Duration};

// Constants
const UPDATE_INTERVAL_MS: u64 = 500;
const VISIBLE_RANGE: usize = 50;
const MA_WINDOW_SIZE: usize = 50;
const PRICE_VOLATILITY_FACTOR: f64 = 0.01;
const PRICE_CHANGE_RANGE: (f64, f64) = (-2.0, 2.0);
const INITIAL_PRICE: f64 = 100.0;

/// Represents a single candlestick with OHLC data
#[derive(Debug, Clone)]
struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

impl Candle {
    fn new(open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            open,
            high,
            low,
            close,
        }
    }

    fn is_bullish(&self) -> bool {
        self.close >= self.open
    }
}

/// Main chart structure containing candlesticks and MA50 data
struct CandlestickChart {
    candles: Vec<Candle>,
    visible_range: usize,
    last_price: f64,
    ma50_values: VecDeque<f64>,
}

impl CandlestickChart {
    fn new(candles: Vec<Candle>, visible_range: usize) -> Self {
        let last_price = candles.last().map_or(INITIAL_PRICE, |c| c.close);
        Self {
            candles,
            visible_range,
            last_price,
            ma50_values: VecDeque::new(),
        }
    }

    fn calculate_ma50(&mut self) {
        let start_idx = self.candles.len().saturating_sub(MA_WINDOW_SIZE);
        let sum: f64 = self.candles[start_idx..].iter().map(|c| c.close).sum();
        let count = self.candles.len() - start_idx;
        let ma50 = sum / count as f64;

        self.ma50_values.push_back(ma50);
        while self.ma50_values.len() > self.visible_range {
            self.ma50_values.pop_front();
        }
    }

    fn generate_new_candle(&mut self) {
        let mut rng = rand::thread_rng();
        let price_change_percent = rng.gen_range(PRICE_CHANGE_RANGE.0..PRICE_CHANGE_RANGE.1);
        let movement = self.last_price * (price_change_percent / 100.0);
        let new_price = self.last_price + movement;

        let volatility = self.last_price * PRICE_VOLATILITY_FACTOR;
        let high = new_price + rng.gen_range(0.0..volatility);
        let low = new_price - rng.gen_range(0.0..volatility);

        let new_candle = Candle::new(self.last_price, high, low, new_price);

        self.last_price = new_price;
        self.candles.push(new_candle);

        if self.candles.len() > self.visible_range {
            self.candles.remove(0);
        }

        self.calculate_ma50();
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let chart_block = Block::default()
            .borders(Borders::ALL)
            .title("Live Candlestick Chart with MA50 (Press 'q' to quit)");

        let visible_candles =
            &self.candles[self.candles.len().saturating_sub(self.visible_range)..];
        let (min_price, max_price) = self.calculate_price_range(visible_candles);
        let (y_min, y_max) = self.calculate_y_bounds(min_price, max_price);

        let canvas = Canvas::default()
            .block(chart_block)
            .paint(|ctx| {
                self.draw_price_labels(ctx, visible_candles.len() as f64, y_min, y_max);
                self.draw_candlesticks(ctx, visible_candles);
                self.draw_ma50_line(ctx);
                self.draw_indicators(ctx, visible_candles, y_max);
            })
            .x_bounds([0.0, (visible_candles.len() + 2) as f64])
            .y_bounds([y_min, y_max]);

        frame.render_widget(canvas, area);
    }

    fn calculate_price_range(&self, candles: &[Candle]) -> (f64, f64) {
        let min_price = candles
            .iter()
            .map(|c| c.low)
            .fold(f64::INFINITY, |a, b| a.min(b));
        let max_price = candles
            .iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, |a, b| a.max(b));
        (min_price, max_price)
    }

    fn calculate_y_bounds(&self, min_price: f64, max_price: f64) -> (f64, f64) {
        let price_range = max_price - min_price;
        let padding = price_range * 0.1;
        (min_price - padding, max_price + padding)
    }

    fn draw_price_labels(&self, ctx: &mut Context, x: f64, y_min: f64, y_max: f64) {
        let num_labels = 5;
        for i in 0..=num_labels {
            let price = y_min + (y_max - y_min) * (i as f64 / num_labels as f64);
            ctx.print(x + 0.5, price, format!("{:.2}", price));
        }
    }

    fn draw_candlesticks(&self, ctx: &mut Context, candles: &[Candle]) {
        let candle_width = 0.8;
        for (i, candle) in candles.iter().enumerate() {
            let x = i as f64;
            let color = if candle.is_bullish() {
                Color::Green
            } else {
                Color::Red
            };

            self.draw_candle_wick(ctx, x, candle_width, candle, color);
            self.draw_candle_body(ctx, x, candle_width, candle, color);
        }
    }

    fn draw_candle_wick(
        &self,
        ctx: &mut Context,
        x: f64,
        width: f64,
        candle: &Candle,
        color: Color,
    ) {
        ctx.draw(&Line {
            x1: x + width / 2.0,
            y1: candle.low,
            x2: x + width / 2.0,
            y2: candle.high,
            color,
        });
    }

    fn draw_candle_body(
        &self,
        ctx: &mut Context,
        x: f64,
        width: f64,
        candle: &Candle,
        color: Color,
    ) {
        let (body_top, body_bottom) = if candle.is_bullish() {
            (candle.close, candle.open)
        } else {
            (candle.open, candle.close)
        };

        if (body_top - body_bottom).abs() < 0.001 {
            self.draw_flat_candle(ctx, x, width, body_top, color);
        } else {
            self.draw_filled_candle(ctx, x, width, body_top, body_bottom, color);
        }
    }

    fn draw_flat_candle(&self, ctx: &mut Context, x: f64, width: f64, price: f64, color: Color) {
        ctx.draw(&Line {
            x1: x,
            y1: price,
            x2: x + width,
            y2: price,
            color,
        });
    }

    fn draw_filled_candle(
        &self,
        ctx: &mut Context,
        x: f64,
        width: f64,
        top: f64,
        bottom: f64,
        color: Color,
    ) {
        let points = vec![(x, bottom), (x + width, bottom), (x + width, top), (x, top)];
        ctx.draw(&Points {
            coords: &points,
            color,
        });

        for y in (((bottom * 100.0) as i32)..=((top * 100.0) as i32)).step_by(1) {
            let y = y as f64 / 100.0;
            ctx.draw(&Line {
                x1: x,
                y1: y,
                x2: x + width,
                y2: y,
                color,
            });
        }
    }

    fn draw_ma50_line(&self, ctx: &mut Context) {
        let ma50_color = Color::Yellow;
        for i in 1..self.ma50_values.len() {
            if let (Some(prev_ma), Some(curr_ma)) =
                (self.ma50_values.get(i - 1), self.ma50_values.get(i))
            {
                ctx.draw(&Line {
                    x1: (i - 1) as f64,
                    y1: *prev_ma,
                    x2: i as f64,
                    y2: *curr_ma,
                    color: ma50_color,
                });
            }
        }
    }

    fn draw_indicators(&self, ctx: &mut Context, candles: &[Candle], y_max: f64) {
        if let Some(last_candle) = candles.last() {
            ctx.print(
                0.0,
                y_max * 0.95,
                format!("Current: {:.2}", last_candle.close),
            );
            if let Some(last_ma) = self.ma50_values.back() {
                ctx.print(0.0, y_max * 0.90, format!("MA50: {:.2}", last_ma));
            }
        }
    }
}

fn main() -> io::Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize chart
    let initial_candle = Candle::new(
        INITIAL_PRICE,
        INITIAL_PRICE * 1.05,
        INITIAL_PRICE * 0.98,
        INITIAL_PRICE * 1.03,
    );
    let mut chart = CandlestickChart::new(vec![initial_candle], VISIBLE_RANGE);

    // Main loop
    let mut last_update = std::time::Instant::now();
    let update_interval = Duration::from_millis(UPDATE_INTERVAL_MS);

    loop {
        if last_update.elapsed() >= update_interval {
            chart.generate_new_candle();
            last_update = std::time::Instant::now();
        }

        terminal.draw(|f| {
            chart.draw(f, f.size());
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
