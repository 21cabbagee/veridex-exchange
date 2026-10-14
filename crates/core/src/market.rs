use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use crate::types::*;

/// A prediction market (binary: YES/NO outcome)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub id: Id,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub category: Category,
    pub status: MarketStatus,

    /// Current best YES ask price (probability implied)
    pub yes_price: Price,
    /// Current best NO ask price = 1 - yes_price (approx)
    pub no_price: Price,

    pub volume_24h: Amount,
    pub volume_total: Amount,
    pub liquidity: Amount,
    pub open_interest: Amount,

    /// Number of traders
    pub num_traders: u64,
    /// Number of open orders
    pub num_orders: u64,

    pub created_at: DateTime<Utc>,
    pub closes_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<Outcome>,

    /// Tags for searching / filtering
    pub tags: Vec<String>,
    /// Thumbnail image URL
    pub image_url: Option<String>,
    /// Source/resolver (e.g., "Polymarket resolution team")
    pub resolver: String,
}

impl Market {
    pub fn is_active(&self) -> bool {
        self.status == MarketStatus::Active
    }

    /// Implied probability of YES in percent
    pub fn yes_probability_pct(&self) -> Decimal {
        self.yes_price * Decimal::from(100)
    }
}
