use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::auth;
use crate::config::Config;
use crate::error::AppError;
use crate::models::EventEnvelope;
use crate::ws;

pub mod auth_routes;
pub mod events;
pub mod holders;
pub mod nfts;
pub mod similar;
pub mod stats;

pub struct AppState {
    pub pool: PgPool,
    pub cfg: Config,
    pub event_tx: broadcast::Sender<EventEnvelope>,
}

pub fn router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/api/events", get(events::list))
        .route("/api/nfts/:contract/:token_id", get(nfts::get))
        .route("/api/nfts/similar/:contract/:token_id", get(similar::get))
        .route("/api/holders/:address", get(holders::get))
        .route("/api/stats", get(stats::get))
        .route("/api/me", get(me))
        .route("/auth/nonce", post(auth_routes::nonce))
        .route("/auth/verify", post(auth_routes::verify))
        .route("/ws/events", get(ws::handle))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

pub struct AuthUser {
    pub address: String,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Arc<AppState>: axum::extract::FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = <Arc<AppState> as axum::extract::FromRef<S>>::from_ref(state);
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;
        let token = header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;
        let claims = auth::verify(&app_state.cfg.jwt_secret, token)?;
        Ok(AuthUser { address: claims.sub })
    }
}

async fn me(user: AuthUser) -> impl IntoResponse {
    (
        StatusCode::OK,
        axum::Json(serde_json::json!({"address": user.address})),
    )
}

#[allow(dead_code)]
fn header_value_static(s: &'static str) -> HeaderValue {
    HeaderValue::from_static(s)
}
