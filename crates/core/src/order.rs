use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use crate::types::*;

/// A limit order submitted to the order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Id,
    pub market_id: Id,
    pub user_id: Id,

    pub outcome: Outcome,
    pub side: Side,
    pub order_type: OrderType,
    pub status: OrderStatus,

    /// Limit price per share (0.01 – 0.99)
    pub price: Price,
    /// Original quantity in shares (1 share = $1 max payout)
    pub quantity: Decimal,
    /// How many shares have been filled
    pub quantity_filled: Decimal,
    /// Remaining unfilled shares
    pub quantity_remaining: Decimal,

    /// Total cost basis (price × quantity for BUYs)
    pub cost_basis: Amount,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Order {
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Expired | OrderStatus::Rejected
        )
    }
}

/// A matched trade between two orders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Id,
    pub market_id: Id,

    /// The resting (maker) order
    pub maker_order_id: Id,
    pub maker_user_id: Id,

    /// The aggressive (taker) order
    pub taker_order_id: Id,
    pub taker_user_id: Id,

    pub outcome: Outcome,
    pub price: Price,
    pub quantity: Decimal,

    /// Taker side
    pub aggressor_side: Side,

    pub executed_at: DateTime<Utc>,
}

/// A user's position in a market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Id,
    pub user_id: Id,
    pub market_id: Id,
    pub outcome: Outcome,

    /// Net shares held
    pub shares: Decimal,
    /// Average cost per share
    pub avg_price: Price,
    /// Current mark-to-market value
    pub mark_value: Amount,
    /// Unrealized P&L
    pub unrealized_pnl: Amount,
    /// Realized P&L from closed trades
    pub realized_pnl: Amount,

    pub updated_at: DateTime<Utc>,
}
