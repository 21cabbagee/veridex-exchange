use std::collections::VecDeque;
use rust_decimal::Decimal;
use predix_core::types::Id;
use serde::{Deserialize, Serialize};

/// A single price level in the book: all orders at the same price, FIFO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// Price (0.01 – 0.99)
    pub price: Decimal,
    /// Total quantity available at this level
    pub quantity: Decimal,
    /// Ordered queue of (order_id, quantity_remaining, user_id)
    pub orders: VecDeque<LevelEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelEntry {
    pub order_id: Id,
    pub user_id: Id,
    pub quantity: Decimal,
}

impl PriceLevel {
    pub fn new(price: Decimal) -> Self {
        Self {
            price,
            quantity: Decimal::ZERO,
            orders: VecDeque::new(),
        }
    }

    pub fn add(&mut self, order_id: Id, user_id: Id, quantity: Decimal) {
        self.quantity += quantity;
        self.orders.push_back(LevelEntry {
            order_id,
            user_id,
            quantity,
        });
    }

    /// Remove `qty` shares from the front of the queue.
    /// Returns list of (order_id, user_id, qty_taken).
    pub fn take(&mut self, mut qty: Decimal) -> Vec<(Id, Id, Decimal)> {
        let mut fills = Vec::new();
        while qty > Decimal::ZERO {
            if let Some(front) = self.orders.front_mut() {
                let take = qty.min(front.quantity);
                front.quantity -= take;
                self.quantity -= take;
                qty -= take;
                fills.push((front.order_id, front.user_id, take));
                if front.quantity == Decimal::ZERO {
                    self.orders.pop_front();
                }
            } else {
                break;
            }
        }
        fills
    }

    /// Cancel a specific order from this level
    pub fn cancel(&mut self, order_id: Id) -> Option<Decimal> {
        if let Some(pos) = self.orders.iter().position(|e| e.order_id == order_id) {
            let removed = self.orders.remove(pos).unwrap();
            self.quantity -= removed.quantity;
            Some(removed.quantity)
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }
}
