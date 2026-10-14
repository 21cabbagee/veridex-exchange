use std::collections::BTreeMap;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::{debug, info};
use predix_core::{
    types::{Id, Outcome, Side, OrderType, OrderStatus, Price, Amount},
    order::{Order, Trade},
    events::{DomainEvent, CancelReason},
};
use crate::level::PriceLevel;

/// Result of attempting to match an incoming order
#[derive(Debug)]
pub struct MatchResult {
    /// Trades executed (may be empty for a pure resting order)
    pub trades: Vec<Trade>,
    /// Updated/created order after matching
    pub order: Order,
    /// Domain events to publish
    pub events: Vec<DomainEvent>,
}

/// A single binary-outcome order book (YES side only).
/// The NO book is derived: NO_bid = 1 - YES_ask, NO_ask = 1 - YES_bid.
///
/// Price-time priority:
///   - BUY  orders: highest price first (BTreeMap descending key)
///   - SELL orders: lowest  price first (BTreeMap ascending key)
pub struct OrderBook {
    pub market_id: Id,
    pub outcome: Outcome,

    /// Buy orders keyed by price DESC (highest bid first)
    bids: BTreeMap<OrdPrice, PriceLevel>,
    /// Sell orders keyed by price ASC (lowest ask first)
    asks: BTreeMap<OrdPrice, PriceLevel>,
}

/// Wrapper so we can reverse BTreeMap ordering for bids
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct OrdPrice(i64); // price * 10000 as integer

impl From<Decimal> for OrdPrice {
    fn from(d: Decimal) -> Self {
        OrdPrice((d * dec!(10000)).to_i64_digit().unwrap_or(0))
    }
}

/// Bid key: we store as negative so BTreeMap gives us descending order
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BidKey(i64);

impl From<Decimal> for BidKey {
    fn from(d: Decimal) -> Self {
        let raw = (d * dec!(10000)).to_i64_digit().unwrap_or(0);
        BidKey(-raw) // negate → highest first
    }
}

impl OrderBook {
    pub fn new(market_id: Id, outcome: Outcome) -> Self {
        Self {
            market_id,
            outcome,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Best bid (highest buy price)
    pub fn best_bid(&self) -> Option<Price> {
        self.bids
            .values()
            .next()
            .map(|l| l.price)
    }

    /// Best ask (lowest sell price)
    pub fn best_ask(&self) -> Option<Price> {
        self.asks
            .values()
            .next()
            .map(|l| l.price)
    }

    /// Total quantity available at bid
    pub fn bid_depth(&self, levels: usize) -> Vec<(Price, Decimal)> {
        self.bids
            .values()
            .take(levels)
            .map(|l| (l.price, l.quantity))
            .collect()
    }

    /// Total quantity available at ask
    pub fn ask_depth(&self, levels: usize) -> Vec<(Price, Decimal)> {
        self.asks
            .values()
            .take(levels)
            .map(|l| (l.price, l.quantity))
            .collect()
    }

    /// Submit an order. Returns trades + updated order + events.
    pub fn submit(&mut self, mut order: Order) -> MatchResult {
        let mut trades = Vec::new();
        let mut events = Vec::new();

        match order.side {
            Side::Buy => {
                // Match against asks (lowest ask first)
                let mut keys_to_remove = Vec::new();
                for (key, level) in self.asks.iter_mut() {
                    if order.quantity_remaining <= Decimal::ZERO {
                        break;
                    }
                    // For a buy, we fill if ask_price <= our limit price
                    if order.order_type == OrderType::Limit && level.price > order.price {
                        break;
                    }

                    let fill_qty = order.quantity_remaining.min(level.quantity);
                    let fills = level.take(fill_qty);

                    for (maker_id, maker_user_id, qty) in fills {
                        let trade = Trade {
                            id: Id::new_v4(),
                            market_id: self.market_id,
                            maker_order_id: maker_id,
                            maker_user_id,
                            taker_order_id: order.id,
                            taker_user_id: order.user_id,
                            outcome: self.outcome,
                            price: level.price,
                            quantity: qty,
                            aggressor_side: Side::Buy,
                            executed_at: Utc::now(),
                        };

                        let ts = trade.executed_at.timestamp_millis();
                        order.quantity_filled += qty;
                        order.quantity_remaining -= qty;

                        events.push(DomainEvent::PriceTick {
                            market_id: self.market_id,
                            outcome: self.outcome,
                            price: level.price,
                            quantity: qty,
                            ts,
                        });

                        trades.push(trade);
                    }

                    if level.is_empty() {
                        keys_to_remove.push(*key);
                    }
                }
                for k in keys_to_remove {
                    self.asks.remove(&k);
                }
            }
            Side::Sell => {
                // Match against bids (highest bid first)
                let mut keys_to_remove = Vec::new();
                for (key, level) in self.bids.iter_mut() {
                    if order.quantity_remaining <= Decimal::ZERO {
                        break;
                    }
                    if order.order_type == OrderType::Limit && level.price < order.price {
                        break;
                    }

                    let fill_qty = order.quantity_remaining.min(level.quantity);
                    let fills = level.take(fill_qty);

                    for (maker_id, maker_user_id, qty) in fills {
                        let trade = Trade {
                            id: Id::new_v4(),
                            market_id: self.market_id,
                            maker_order_id: maker_id,
                            maker_user_id,
                            taker_order_id: order.id,
                            taker_user_id: order.user_id,
                            outcome: self.outcome,
                            price: level.price,
                            quantity: qty,
                            aggressor_side: Side::Sell,
                            executed_at: Utc::now(),
                        };

                        let ts = trade.executed_at.timestamp_millis();
                        order.quantity_filled += qty;
                        order.quantity_remaining -= qty;

                        events.push(DomainEvent::PriceTick {
                            market_id: self.market_id,
                            outcome: self.outcome,
                            price: level.price,
                            quantity: qty,
                            ts,
                        });

                        trades.push(trade);
                    }

                    if level.is_empty() {
                        keys_to_remove.push(*key);
                    }
                }
                for k in keys_to_remove {
                    self.bids.remove(&k);
                }
            }
        }

        // Determine final order status
        if order.quantity_remaining == Decimal::ZERO {
            order.status = OrderStatus::Filled;
        } else if order.quantity_filled > Decimal::ZERO {
            // Partial fill
            match order.order_type {
                OrderType::FillOrKill | OrderType::ImmediateOrCancel => {
                    order.status = OrderStatus::Cancelled;
                    events.push(DomainEvent::OrderCancelled {
                        order_id: order.id,
                        market_id: order.market_id,
                        user_id: order.user_id,
                        reason: CancelReason::UserRequested,
                        ts: Utc::now().timestamp_millis(),
                    });
                }
                _ => {
                    // Rest the unfilled portion
                    order.status = OrderStatus::PartiallyFilled;
                    self.rest(&order);
                }
            }
        } else {
            // No fill at all
            match order.order_type {
                OrderType::FillOrKill | OrderType::Market => {
                    order.status = OrderStatus::Cancelled;
                }
                _ => {
                    order.status = OrderStatus::Open;
                    self.rest(&order);
                }
            }
        }

        order.updated_at = Utc::now();

        events.push(DomainEvent::BookUpdated {
            market_id: self.market_id,
            yes_bid: self.best_bid(),
            yes_ask: self.best_ask(),
            no_bid: self.best_ask().map(|p| dec!(1) - p),
            no_ask: self.best_bid().map(|p| dec!(1) - p),
            ts: Utc::now().timestamp_millis(),
        });

        MatchResult { trades, order, events }
    }

    /// Place the unfilled remainder of an order on the book (rest it)
    fn rest(&mut self, order: &Order) {
        match order.side {
            Side::Buy => {
                let key = BidKey::from(order.price);
                // BidKey map is sorted by ascending BidKey (which is negated price)
                // We need a separate map for bids. For simplicity reuse OrdPrice with negation trick.
                self.bids
                    .entry(OrdPrice(-(order.price * dec!(10000)).to_i64_digit().unwrap_or(0)))
                    .or_insert_with(|| PriceLevel::new(order.price))
                    .add(order.id, order.user_id, order.quantity_remaining);
            }
            Side::Sell => {
                self.asks
                    .entry(OrdPrice::from(order.price))
                    .or_insert_with(|| PriceLevel::new(order.price))
                    .add(order.id, order.user_id, order.quantity_remaining);
            }
        }
    }

    /// Cancel a resting order, returning quantity freed
    pub fn cancel(&mut self, order_id: Id, side: Side, price: Price) -> Option<Decimal> {
        match side {
            Side::Buy => {
                let key = OrdPrice(-(price * dec!(10000)).to_i64_digit().unwrap_or(0));
                if let Some(level) = self.bids.get_mut(&key) {
                    let qty = level.cancel(order_id);
                    if level.is_empty() {
                        self.bids.remove(&key);
                    }
                    qty
                } else {
                    None
                }
            }
            Side::Sell => {
                let key = OrdPrice::from(price);
                if let Some(level) = self.asks.get_mut(&key) {
                    let qty = level.cancel(order_id);
                    if level.is_empty() {
                        self.asks.remove(&key);
                    }
                    qty
                } else {
                    None
                }
            }
        }
    }
}

// Helper: Decimal → i64
trait ToI64Digit {
    fn to_i64_digit(self) -> Option<i64>;
}
impl ToI64Digit for Decimal {
    fn to_i64_digit(self) -> Option<i64> {
        use std::str::FromStr;
        i64::from_str(&self.round_dp(0).to_string()).ok()
    }
}
