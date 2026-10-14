use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// A price in the range [0.01, 0.99] — represents probability × 100
pub type Price = Decimal;

/// Dollar amount with 2 decimal precision
pub type Amount = Decimal;

/// Unique identifier for all domain objects
pub type Id = Uuid;

/// Unix timestamp in milliseconds (for WebSocket efficiency)
pub type TimestampMs = i64;

#[inline]
pub fn now_ms() -> TimestampMs {
    Utc::now().timestamp_millis()
}

#[inline]
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

/// Outcome of a binary market
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Outcome {
    Yes,
    No,
}

impl Outcome {
    pub fn opposite(self) -> Self {
        match self {
            Outcome::Yes => Outcome::No,
            Outcome::No => Outcome::Yes,
        }
    }
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Outcome::Yes => write!(f, "YES"),
            Outcome::No => write!(f, "NO"),
        }
    }
}

/// Order side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    Buy,
    Sell,
}

/// Market status lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketStatus {
    Draft,
    Active,
    Halted,
    Resolved,
    Voided,
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Expired,
    Rejected,
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    Market,
    FillOrKill,
    ImmediateOrCancel,
}

/// Trade execution type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecType {
    New,
    Trade,
    Cancelled,
    Rejected,
}

/// Category tags for markets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Politics,
    Sports,
    Crypto,
    Finance,
    Science,
    Entertainment,
    Weather,
    Tech,
    Other,
}
