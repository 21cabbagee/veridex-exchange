use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use crate::types::*;

/// Registered user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Id,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,

    pub display_name: String,
    pub avatar_url: Option<String>,

    /// USDC balance (deposited collateral)
    pub balance: Amount,
    /// Balance locked in open orders
    pub locked_balance: Amount,

    /// Lifetime P&L
    pub lifetime_pnl: Amount,
    /// Number of trades ever
    pub trade_count: u64,
    /// Portfolio value (positions mark-to-market)
    pub portfolio_value: Amount,

    /// KYC / compliance level
    pub kyc_level: KycLevel,
    pub is_banned: bool,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

impl User {
    pub fn available_balance(&self) -> Amount {
        self.balance - self.locked_balance
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KycLevel {
    None,
    Basic,
    Full,
}

/// Redacted user profile safe to expose in API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicProfile {
    pub id: Id,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub trade_count: u64,
    pub lifetime_pnl: Decimal,
    pub joined_at: DateTime<Utc>,
}

impl From<&User> for PublicProfile {
    fn from(u: &User) -> Self {
        PublicProfile {
            id: u.id,
            username: u.username.clone(),
            display_name: u.display_name.clone(),
            avatar_url: u.avatar_url.clone(),
            trade_count: u.trade_count,
            lifetime_pnl: u.lifetime_pnl,
            joined_at: u.created_at,
        }
    }
}
