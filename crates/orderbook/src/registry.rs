use dashmap::DashMap;
use predix_core::types::{Id, Outcome};
use crate::book::OrderBook;
use parking_lot::Mutex;
use std::sync::Arc;

/// Thread-safe registry of all active order books
/// Each market has two books: YES and NO
pub struct BookRegistry {
    // market_id → (yes_book, no_book)
    books: DashMap<Id, (Arc<Mutex<OrderBook>>, Arc<Mutex<OrderBook>>)>,
}

impl BookRegistry {
    pub fn new() -> Self {
        Self {
            books: DashMap::new(),
        }
    }

    /// Register a new market (creates empty YES+NO books)
    pub fn register(&self, market_id: Id) {
        self.books.insert(
            market_id,
            (
                Arc::new(Mutex::new(OrderBook::new(market_id, Outcome::Yes))),
                Arc::new(Mutex::new(OrderBook::new(market_id, Outcome::No))),
            ),
        );
    }

    /// Get the YES book for a market
    pub fn yes_book(&self, market_id: &Id) -> Option<Arc<Mutex<OrderBook>>> {
        self.books.get(market_id).map(|v| v.0.clone())
    }

    /// Get the NO book for a market
    pub fn no_book(&self, market_id: &Id) -> Option<Arc<Mutex<OrderBook>>> {
        self.books.get(market_id).map(|v| v.1.clone())
    }

    /// Get the book for a specific outcome
    pub fn book(&self, market_id: &Id, outcome: Outcome) -> Option<Arc<Mutex<OrderBook>>> {
        match outcome {
            Outcome::Yes => self.yes_book(market_id),
            Outcome::No => self.no_book(market_id),
        }
    }

    /// Unregister market (on resolution/void)
    pub fn deregister(&self, market_id: &Id) {
        self.books.remove(market_id);
    }

    pub fn market_count(&self) -> usize {
        self.books.len()
    }
}

impl Default for BookRegistry {
    fn default() -> Self {
        Self::new()
    }
}
