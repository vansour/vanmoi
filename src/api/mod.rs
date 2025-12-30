//! API module.
//!
//! HTTP API endpoints and router configuration.

mod admin;
pub mod auth;
mod client;
mod public;

use std::sync::Arc;

use axum::{
    Router, middleware,
    routing::{get, post},
};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

use crate::config::Config;
use crate::db::Database;
use crate::middleware::auth_middleware;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(db: Database, config: Config) -> Self {
        Self {
            db,
            config: Arc::new(config),
        }
    }
}

/// Create the application router.
pub fn create_router(state: AppState) -> Router {
    // Public API routes (no auth required)
    let public_routes = Router::new()
        .route("/api/login", post(auth::login))
        .route("/api/logout", get(auth::logout))
        .route("/api/me", get(auth::me))
        .route("/api/clients", get(public::get_clients))
        .route("/api/nodes", get(public::get_nodes))
        .route("/api/recent/{uuid}", get(public::get_recent_records))
        .route("/api/ping", get(public::get_ping_tasks))
        .route("/api/ping/{id}/records", get(public::get_ping_records));

    // Agent API routes (token auth)
    let agent_routes = Router::new()
        .route("/api/agent/register", post(client::register))
        .route("/api/agent/report", post(client::upload_report))
        .route("/api/agent/info", post(client::upload_basic_info))
        .route("/api/agent/ws", get(client::ws_report));

    // Admin API routes (session auth required)
    let admin_routes = Router::new()
        .route("/api/admin/clients", get(admin::list_clients))
        .route("/api/admin/clients", post(admin::add_client))
        .route("/api/admin/clients/{id}", get(admin::get_client))
        .route("/api/admin/clients/{id}", post(admin::edit_client))
        .route(
            "/api/admin/clients/{id}",
            axum::routing::delete(admin::delete_client),
        )
        .route(
            "/api/admin/clients/{id}/token",
            get(admin::get_client_token),
        )
        .route("/api/admin/settings", get(admin::get_settings))
        .route("/api/admin/settings", post(admin::update_settings))
        .route("/api/admin/notifications", get(admin::list_notifications))
        .route("/api/admin/notifications", post(admin::add_notification))
        .route(
            "/api/admin/notifications/{id}",
            axum::routing::delete(admin::delete_notification),
        )
        .route(
            "/api/admin/notifications/test",
            post(admin::test_notification),
        )
        .route("/api/admin/ping", get(admin::list_ping_tasks))
        .route("/api/admin/ping", post(admin::add_ping_task))
        .route(
            "/api/admin/ping/{id}",
            axum::routing::delete(admin::delete_ping_task),
        )
        .route("/api/admin/user/password", post(admin::change_password))
        .route("/api/admin/sessions", get(admin::list_sessions))
        .route(
            "/api/admin/sessions/{id}",
            axum::routing::delete(admin::delete_session),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::require_auth_middleware,
        ));

    // Combine all routes
    let api_routes = Router::new()
        .merge(public_routes)
        .merge(agent_routes)
        .merge(admin_routes)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Static file serving for Vue3 frontend
    let static_service =
        ServeDir::new("public/dist").not_found_service(ServeDir::new("public/dist").fallback(
            tower_http::services::ServeFile::new("public/dist/index.html"),
        ));

    Router::new()
        .merge(api_routes)
        .fallback_service(static_service)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
