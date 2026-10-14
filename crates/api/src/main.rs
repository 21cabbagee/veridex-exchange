mod routes;
mod state;

use axum::{
    routing::{delete, get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{fmt, EnvFilter};

use state::{AppState, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "predix_api=debug,tower_http=info".into()),
        )
        .json()
        .init();

    let config = Config::from_env();
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;

    let state = AppState::new(config);

    // Seed books for demo markets
    let demo_ids = [
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000002",
        "00000000-0000-0000-0000-000000000003",
        "00000000-0000-0000-0000-000000000004",
    ];
    for id_str in &demo_ids {
        let id = uuid::Uuid::parse_str(id_str)?;
        state.books.register(id);
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Health
        .route("/health", get(routes::health::health))
        // Markets
        .route("/api/v1/markets", get(routes::markets::list_markets))
        .route("/api/v1/markets/:id", get(routes::markets::get_market))
        .route("/api/v1/markets/:id/orderbook", get(routes::markets::get_orderbook))
        .route("/api/v1/markets/:id/trades", get(routes::markets::get_trades))
        // Orders
        .route("/api/v1/orders", get(routes::orders::list_orders))
        .route("/api/v1/orders", post(routes::orders::place_order))
        .route("/api/v1/orders/:id", delete(routes::orders::cancel_order))
        // Auth
        .route("/api/v1/auth/register", post(routes::auth::register))
        .route("/api/v1/auth/login", post(routes::auth::login))
        .route("/api/v1/auth/me", get(routes::auth::me))
        // WebSocket
        .route("/ws", get(routes::ws::ws_handler))
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    tracing::info!("🚀 PREDIX server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
