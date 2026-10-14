use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Query, State},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::broadcast;
use predix_core::events::DomainEvent;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// Comma-separated market IDs to subscribe to (empty = all)
    pub markets: Option<String>,
}

/// GET /ws?markets=uuid1,uuid2
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<WsQuery>,
) -> impl IntoResponse {
    let rx = state.event_tx.subscribe();
    let market_filter: Vec<String> = params
        .markets
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    ws.on_upgrade(move |socket| handle_socket(socket, rx, market_filter))
}

async fn handle_socket(
    socket: WebSocket,
    mut rx: broadcast::Receiver<DomainEvent>,
    market_filter: Vec<String>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Send welcome frame
    let welcome = json!({
        "type": "CONNECTED",
        "ts": chrono::Utc::now().timestamp_millis(),
        "subscribed_markets": market_filter,
    });
    if sender
        .send(Message::Text(welcome.to_string()))
        .await
        .is_err()
    {
        return;
    }

    // Forward domain events to client
    loop {
        tokio::select! {
            // Incoming events from engine
            event = rx.recv() => {
                match event {
                    Ok(ev) => {
                        // Filter by market if specified
                        let market_id_str = event_market_id(&ev);
                        if !market_filter.is_empty() {
                            if let Some(mid) = &market_id_str {
                                if !market_filter.contains(mid) {
                                    continue;
                                }
                            }
                        }

                        let json = match serde_json::to_string(&ev) {
                            Ok(j) => j,
                            Err(_) => continue,
                        };

                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        // Client is too slow — notify and continue
                        let msg = json!({ "type": "LAG", "skipped": n });
                        let _ = sender.send(Message::Text(msg.to_string())).await;
                    }
                    Err(_) => break,
                }
            }
            // Handle incoming pings / close frames
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Ping(data))) => {
                        let _ = sender.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}

/// Extract market_id string from any event variant
fn event_market_id(ev: &DomainEvent) -> Option<String> {
    match ev {
        DomainEvent::OrderPlaced { order, .. } => Some(order.market_id.to_string()),
        DomainEvent::OrderFilled { trade, .. } => Some(trade.market_id.to_string()),
        DomainEvent::OrderCancelled { market_id, .. } => Some(market_id.to_string()),
        DomainEvent::BookUpdated { market_id, .. } => Some(market_id.to_string()),
        DomainEvent::PriceTick { market_id, .. } => Some(market_id.to_string()),
        DomainEvent::MarketResolved { market_id, .. } => Some(market_id.to_string()),
        DomainEvent::MarketCreated { market_id, .. } => Some(market_id.to_string()),
    }
}
