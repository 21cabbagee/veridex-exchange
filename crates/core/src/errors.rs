use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Market not found: {0}")]
    MarketNotFound(String),

    #[error("Order not found: {0}")]
    OrderNotFound(String),

    #[error("Insufficient funds: need {need}, have {have}")]
    InsufficientFunds { need: String, have: String },

    #[error("Invalid price {price}: must be between 0.01 and 0.99")]
    InvalidPrice { price: String },

    #[error("Invalid quantity {qty}: must be > 0")]
    InvalidQuantity { qty: String },

    #[error("Market is not active: status={status}")]
    MarketNotActive { status: String },

    #[error("Market already resolved")]
    MarketAlreadyResolved,

    #[error("Self-trading not allowed")]
    SelfTrade,

    #[error("Order already cancelled")]
    AlreadyCancelled,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
