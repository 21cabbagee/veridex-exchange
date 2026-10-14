use std::sync::Arc;
use predix_orderbook::BookRegistry;
use tokio::sync::broadcast;
use predix_core::events::DomainEvent;

/// Shared application state injected into every Axum handler
#[derive(Clone)]
pub struct AppState {
    pub books: Arc<BookRegistry>,
    /// Event bus — handlers publish, WS subscribers receive
    pub event_tx: broadcast::Sender<DomainEvent>,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let (event_tx, _) = broadcast::channel(8192);
        Self {
            books: Arc::new(BookRegistry::new()),
            event_tx,
            config: Arc::new(config),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub database_url: Option<String>,
    pub redis_url: Option<String>,
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();
        Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "predix-dev-secret-change-in-prod".into()),
            database_url: std::env::var("DATABASE_URL").ok(),
            redis_url: std::env::var("REDIS_URL").ok(),
            cors_origins: std::env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173,http://localhost:3000".into())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        }
    }
}
