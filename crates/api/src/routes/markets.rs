use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use chrono::Utc;
use predix_core::{
    types::*,
    market::Market,
};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListMarketsQuery {
    pub category: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// GET /api/v1/markets
pub async fn list_markets(
    State(_state): State<AppState>,
    Query(params): Query<ListMarketsQuery>,
) -> Json<Value> {
    // In production: query from DB. Here we return in-memory seed data.
    let markets = seed_markets();
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let filtered: Vec<&Market> = markets
        .iter()
        .filter(|m| {
            if let Some(ref cat) = params.category {
                let cat_lower = cat.to_lowercase();
                if !format!("{:?}", m.category).to_lowercase().contains(&cat_lower) {
                    return false;
                }
            }
            if let Some(ref q) = params.search {
                let q_lower = q.to_lowercase();
                if !m.title.to_lowercase().contains(&q_lower) {
                    return false;
                }
            }
            true
        })
        .collect();

    let total = filtered.len();
    let page: Vec<&Market> = filtered.into_iter().skip(offset).take(limit).collect();

    Json(json!({
        "markets": page,
        "total": total,
        "limit": limit,
        "offset": offset,
    }))
}

/// GET /api/v1/markets/:id
pub async fn get_market(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let markets = seed_markets();
    let market = markets
        .into_iter()
        .find(|m| m.id.to_string() == id || m.slug == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(json!({ "market": market })))
}

/// GET /api/v1/markets/:id/orderbook
pub async fn get_orderbook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Value> {
    // Try to parse UUID, fall back to seed data lookup
    let market_id = Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::nil());

    let (yes_bids, yes_asks, no_bids, no_asks) = if let Some(book) = state.books.yes_book(&market_id) {
        let b = book.lock();
        let yb = b.bid_depth(10);
        let ya = b.ask_depth(10);
        drop(b);
        let nb = state.books.no_book(&market_id).map(|nb| {
            let b = nb.lock();
            b.bid_depth(10)
        }).unwrap_or_default();
        let na = state.books.no_book(&market_id).map(|nb| {
            let b = nb.lock();
            b.ask_depth(10)
        }).unwrap_or_default();
        (yb, ya, nb, na)
    } else {
        // Return synthetic book for demo
        (
            vec![(dec!(0.72), dec!(1250)), (dec!(0.71), dec!(800)), (dec!(0.70), dec!(2000))],
            vec![(dec!(0.74), dec!(950)), (dec!(0.75), dec!(1100)), (dec!(0.76), dec!(600))],
            vec![(dec!(0.27), dec!(1250)), (dec!(0.26), dec!(800))],
            vec![(dec!(0.28), dec!(950)), (dec!(0.29), dec!(600))],
        )
    };

    Json(json!({
        "market_id": id,
        "yes": {
            "bids": yes_bids.iter().map(|(p,q)| json!({"price": p, "quantity": q})).collect::<Vec<_>>(),
            "asks": yes_asks.iter().map(|(p,q)| json!({"price": p, "quantity": q})).collect::<Vec<_>>(),
        },
        "no": {
            "bids": no_bids.iter().map(|(p,q)| json!({"price": p, "quantity": q})).collect::<Vec<_>>(),
            "asks": no_asks.iter().map(|(p,q)| json!({"price": p, "quantity": q})).collect::<Vec<_>>(),
        },
        "ts": Utc::now().timestamp_millis(),
    }))
}

/// GET /api/v1/markets/:id/trades
pub async fn get_trades(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Value> {
    // Seed recent trades
    let now = Utc::now().timestamp_millis();
    let trades: Vec<Value> = (0..20).map(|i| {
        let price = dec!(0.70) + Decimal::from(i % 5) * dec!(0.01);
        json!({
            "id": Uuid::new_v4(),
            "price": price,
            "quantity": 100 + (i * 37) % 400,
            "side": if i % 2 == 0 { "BUY" } else { "SELL" },
            "outcome": if i % 3 == 0 { "NO" } else { "YES" },
            "ts": now - (i as i64 * 8000),
        })
    }).collect();

    Json(json!({ "trades": trades, "market_id": id }))
}

// ---------------------------------------------------------------------------
// Seed data (replaces DB in the stateless demo build)
// ---------------------------------------------------------------------------
pub fn seed_markets() -> Vec<Market> {
    vec![
        Market {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            slug: "trump-wins-2024".into(),
            title: "Will Trump win the 2024 US Presidential Election?".into(),
            description: "Resolves YES if Donald Trump wins the 2024 US Presidential Election.".into(),
            category: Category::Politics,
            status: MarketStatus::Active,
            yes_price: dec!(0.73),
            no_price: dec!(0.27),
            volume_24h: dec!(4250000),
            volume_total: dec!(28500000),
            liquidity: dec!(1200000),
            open_interest: dec!(3100000),
            num_traders: 48291,
            num_orders: 12044,
            created_at: Utc::now(),
            closes_at: None,
            resolved_at: None,
            resolution: None,
            tags: vec!["politics".into(), "usa".into(), "election".into()],
            image_url: Some("https://polymarket-upload.s3.us-east-2.amazonaws.com/trump.png".into()),
            resolver: "PREDIX Resolution Committee".into(),
        },
        Market {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            slug: "btc-100k-2024".into(),
            title: "Will Bitcoin reach $100,000 before December 31, 2024?".into(),
            description: "Resolves YES if BTC/USD spot price reaches $100,000 on any major exchange.".into(),
            category: Category::Crypto,
            status: MarketStatus::Active,
            yes_price: dec!(0.82),
            no_price: dec!(0.18),
            volume_24h: dec!(8100000),
            volume_total: dec!(41200000),
            liquidity: dec!(3400000),
            open_interest: dec!(7200000),
            num_traders: 92847,
            num_orders: 31022,
            created_at: Utc::now(),
            closes_at: None,
            resolved_at: None,
            resolution: None,
            tags: vec!["crypto".into(), "bitcoin".into()],
            image_url: Some("https://upload.wikimedia.org/wikipedia/commons/4/46/Bitcoin.svg".into()),
            resolver: "Chainlink Oracle".into(),
        },
        Market {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
            slug: "fed-rate-cut-dec-2024".into(),
            title: "Will the Fed cut rates in December 2024?".into(),
            description: "Resolves YES if the FOMC cuts the federal funds rate target range in December 2024.".into(),
            category: Category::Finance,
            status: MarketStatus::Active,
            yes_price: dec!(0.61),
            no_price: dec!(0.39),
            volume_24h: dec!(2700000),
            volume_total: dec!(9800000),
            liquidity: dec!(880000),
            open_interest: dec!(1900000),
            num_traders: 21440,
            num_orders: 8811,
            created_at: Utc::now(),
            closes_at: None,
            resolved_at: None,
            resolution: None,
            tags: vec!["finance".into(), "fed".into(), "rates".into()],
            image_url: None,
            resolver: "Federal Reserve press release".into(),
        },
        Market {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
            slug: "chiefs-sb-2025".into(),
            title: "Will the Kansas City Chiefs win Super Bowl LIX?".into(),
            description: "Resolves YES if KC Chiefs win Super Bowl LIX in February 2025.".into(),
            category: Category::Sports,
            status: MarketStatus::Active,
            yes_price: dec!(0.32),
            no_price: dec!(0.68),
            volume_24h: dec!(1900000),
            volume_total: dec!(6600000),
            liquidity: dec!(540000),
            open_interest: dec!(1200000),
            num_traders: 15621,
            num_orders: 4920,
            created_at: Utc::now(),
            closes_at: None,
            resolved_at: None,
            resolution: None,
            tags: vec!["sports".into(), "nfl".into(), "superbowl".into()],
            image_url: None,
            resolver: "NFL official result".into(),
        },
        Market {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000005").unwrap(),
            slug: "eth-etf-approved-2024".into(),
            title: "Will the SEC approve an Ethereum spot ETF in 2024?".into(),
            description: "Resolves YES if SEC approves at least one spot ETH ETF by Dec 31, 2024.".into(),
            category: Category::Crypto,
            status: MarketStatus::Resolved,
            yes_price: dec!(1.0),
            no_price: dec!(0.0),
            volume_24h: dec!(0),
            volume_total: dec!(14500000),
            liquidity: dec!(0),
            open_interest: dec!(0),
            num_traders: 67210,
            num_orders: 0,
            created_at: Utc::now(),
            closes_at: None,
            resolved_at: Some(Utc::now()),
            resolution: Some(Outcome::Yes),
            tags: vec!["crypto".into(), "ethereum".into(), "etf".into()],
            image_url: None,
            resolver: "SEC official announcement".into(),
        },
    ]
}
