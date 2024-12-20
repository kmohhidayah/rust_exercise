use super::websocket::KlineData;

#[derive(Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl Candle {
    pub fn from_kline_data(data: &KlineData) -> Option<Self> {
        Some(Self {
            open: data.open.parse().ok()?,
            high: data.high.parse().ok()?,
            low: data.low.parse().ok()?,
            close: data.close.parse().ok()?,
        })
    }

    pub fn is_bullish(&self) -> bool {
        self.close >= self.open
    }
}
