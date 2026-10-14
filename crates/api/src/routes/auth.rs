use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use uuid::Uuid;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,     // user_id
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

/// POST /api/v1/auth/register
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: persist to DB + hash password with argon2
    let user_id = Uuid::new_v4();
    let token = make_token(&state.config.jwt_secret, user_id, &req.username)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "user": {
            "id": user_id,
            "username": req.username,
            "email": req.email,
            "balance": "1000.00",
        },
        "token": token,
    })))
}

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: look up user from DB, verify argon2 hash
    let user_id = Uuid::new_v4();
    let username = req.email.split('@').next().unwrap_or("trader").to_string();
    let token = make_token(&state.config.jwt_secret, user_id, &username)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "user": {
            "id": user_id,
            "username": username,
            "email": req.email,
            "balance": "1000.00",
            "lifetime_pnl": "0.00",
        },
        "token": token,
    })))
}

/// GET /api/v1/auth/me
pub async fn me() -> Json<Value> {
    Json(json!({
        "id": Uuid::new_v4(),
        "username": "trader",
        "balance": "1000.00",
        "portfolio_value": "1247.50",
        "lifetime_pnl": "+247.50",
    }))
}

fn make_token(secret: &str, user_id: Uuid, username: &str) -> anyhow::Result<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::days(7)).timestamp(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}
