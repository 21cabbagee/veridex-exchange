use serde::{Deserialize, Serialize};
use crate::types::*;
use crate::order::{Order, Trade};

/// All domain events that flow through the system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DomainEvent {
    /// New order placed
    OrderPlaced {
        order: Box<Order>,
        ts: TimestampMs,
    },

    /// Order matched — partial or full fill
    OrderFilled {
        trade: Box<Trade>,
        maker_remaining: Option<std::collections::HashMap<String, String>>,
        ts: TimestampMs,
    },

    /// Order cancelled (by user or system)
    OrderCancelled {
        order_id: Id,
        market_id: Id,
        user_id: Id,
        reason: CancelReason,
        ts: TimestampMs,
    },

    /// Order book top-of-book changed
    BookUpdated {
        market_id: Id,
        yes_bid: Option<Price>,
        yes_ask: Option<Price>,
        no_bid: Option<Price>,
        no_ask: Option<Price>,
        ts: TimestampMs,
    },

    /// Market price updated (last trade price)
    PriceTick {
        market_id: Id,
        outcome: Outcome,
        price: Price,
        quantity: rust_decimal::Decimal,
        ts: TimestampMs,
    },

    /// Market resolved
    MarketResolved {
        market_id: Id,
        resolution: Outcome,
        ts: TimestampMs,
    },

    /// New market created
    MarketCreated {
        market_id: Id,
        slug: String,
        title: String,
        ts: TimestampMs,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CancelReason {
    UserRequested,
    MarketHalted,
    MarketResolved,
    Expired,
    SelfTrade,
    SystemError,
}
