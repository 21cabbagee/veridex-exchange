use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use predix_core::{
    types::*,
    order::{Order, Trade},
    events::DomainEvent,
};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PlaceOrderRequest {
    pub market_id: String,
    pub outcome: String,       // "YES" | "NO"
    pub side: String,          // "BUY" | "SELL"
    pub order_type: String,    // "LIMIT" | "MARKET"
    pub price: Option<Decimal>,
    pub quantity: Decimal,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub status: String,
    pub filled: Decimal,
    pub remaining: Decimal,
    pub trades: Vec<Value>,
}

/// POST /api/v1/orders
pub async fn place_order(
    State(state): State<AppState>,
    Json(req): Json<PlaceOrderRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Parse inputs
    let market_id = Uuid::parse_str(&req.market_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let outcome = match req.outcome.to_uppercase().as_str() {
        "YES" => Outcome::Yes,
        "NO" => Outcome::No,
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    let side = match req.side.to_uppercase().as_str() {
        "BUY" => Side::Buy,
        "SELL" => Side::Sell,
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    let order_type = match req.order_type.to_uppercase().as_str() {
        "LIMIT" => OrderType::Limit,
        "MARKET" => OrderType::Market,
        "FOK" => OrderType::FillOrKill,
        "IOC" => OrderType::ImmediateOrCancel,
        _ => OrderType::Limit,
    };

    let price = req.price.unwrap_or(dec!(0.5));

    // Build order object
    let now = Utc::now();
    let order = Order {
        id: Uuid::new_v4(),
        market_id,
        user_id: Uuid::new_v4(), // TODO: extract from JWT
        outcome,
        side,
        order_type,
        status: OrderStatus::Open,
        price,
        quantity: req.quantity,
        quantity_filled: Decimal::ZERO,
        quantity_remaining: req.quantity,
        cost_basis: price * req.quantity,
        created_at: now,
        updated_at: now,
        expires_at: None,
    };

    // Submit to order book
    let result = if let Some(book) = state.books.book(&market_id, outcome) {
        let mut b = book.lock();
        Some(b.submit(order.clone()))
    } else {
        // Market not in memory — auto-register and submit
        state.books.register(market_id);
        let book = state.books.book(&market_id, outcome).unwrap();
        let mut b = book.lock();
        Some(b.submit(order.clone()))
    };

    let result = result.unwrap();

    // Publish events to WebSocket subscribers
    for event in &result.events {
        let _ = state.event_tx.send(event.clone());
    }

    let trades: Vec<Value> = result.trades.iter().map(|t| json!({
        "id": t.id,
        "price": t.price,
        "quantity": t.quantity,
        "executed_at": t.executed_at,
    })).collect();

    Ok(Json(json!({
        "order": {
            "id": result.order.id,
            "status": result.order.status,
            "price": result.order.price,
            "quantity": result.order.quantity,
            "filled": result.order.quantity_filled,
            "remaining": result.order.quantity_remaining,
            "created_at": result.order.created_at,
        },
        "trades": trades,
        "trade_count": trades.len(),
    })))
}

/// DELETE /api/v1/orders/:id  (cancel)
pub async fn cancel_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Json<Value> {
    // In production: look up order from DB to get market_id/side/price, then cancel
    Json(json!({
        "cancelled": true,
        "order_id": order_id,
        "ts": Utc::now().timestamp_millis(),
    }))
}

/// GET /api/v1/orders?market_id=...
pub async fn list_orders(State(_state): State<AppState>) -> Json<Value> {
    Json(json!({ "orders": [], "total": 0 }))
}
