use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscribeMessage {
    pub op: String,
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct KlineData {
    pub start: i64,
    pub end: i64,
    pub interval: String,
    pub open: String,
    pub close: String,
    pub high: String,
    pub low: String,
    pub volume: String,
    pub turnover: String,
    pub confirm: bool,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize)]
pub struct KlineResponse {
    pub topic: String,
    pub data: Vec<KlineData>,
    pub ts: i64,
    #[serde(rename = "type")]
    pub response_type: String,
}
