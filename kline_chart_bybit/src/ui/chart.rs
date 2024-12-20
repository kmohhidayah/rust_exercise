use crate::{
    constants::MA_WINDOW_SIZE,
    models::{Candle, KlineData},
};
use ratatui::{
    layout::Rect,
    style::Color,
    widgets::{
        canvas::{Canvas, Context, Line, Points},
        Block, Borders,
    },
    Frame,
};
use std::collections::VecDeque;

pub struct CandlestickChart {
    candles: Vec<Candle>,
    visible_range: usize,
    ma50_values: VecDeque<f64>,
}

impl CandlestickChart {
    pub fn new(visible_range: usize) -> Self {
        Self {
            candles: Vec::new(),
            visible_range,
            ma50_values: VecDeque::new(),
        }
    }

    pub fn update_from_kline(&mut self, kline_data: &KlineData) {
        if let Some(candle) = Candle::from_kline_data(kline_data) {
            if kline_data.confirm {
                if self.candles.len() >= self.visible_range {
                    self.candles.remove(0);
                }
                self.candles.push(candle);
            } else {
                // Update last candle if it's still open
                if let Some(last) = self.candles.last_mut() {
                    *last = candle;
                } else {
                    self.candles.push(candle);
                }
            }
            self.calculate_ma50();
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

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let chart_block = Block::default()
            .borders(Borders::ALL)
            .title("Live Candlestick Chart with MA50 (Press 'q' to quit)");

        let visible_candles = if !self.candles.is_empty() {
            &self.candles[self.candles.len().saturating_sub(self.visible_range)..]
        } else {
            &[]
        };

        if visible_candles.is_empty() {
            let canvas = Canvas::default()
                .block(chart_block)
                .x_bounds([0.0, 1.0])
                .y_bounds([0.0, 1.0])
                .paint(|ctx| {
                    ctx.print(0.0, 0.5, "Waiting for data...");
                });
            frame.render_widget(canvas, area);
            return;
        }

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
