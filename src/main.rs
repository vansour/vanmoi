use std::net::SocketAddr;

use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{info, warn};

mod api;
mod config;
mod db;
mod error;
mod logs;
mod middleware;
mod notifier;
mod ws;

use config::Config;
use db::Database;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    logs::init();

    info!("Starting Vanmoi server...");

    // Load configuration
    let config = Config::from_env();
    info!("Configuration loaded");

    // Connect to database
    let db = Database::connect(&config.database_url).await?;
    info!("Database connected");

    // Initialize database schema
    db.init_schema().await?;
    info!("Database schema initialized");

    // Initialize admin user if no users exist
    init_admin_user(&db, &config).await?;

    // Create application state
    let state = api::AppState::new(db, config.clone());

    // Build router
    let app = api::create_router(state);

    // Start server
    let addr: SocketAddr = config.listen_addr.parse()?;
    let listener = TcpListener::bind(addr).await?;

    info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Initialize admin user if no users exist in the database.
async fn init_admin_user(db: &Database, config: &Config) -> Result<()> {
    if db.has_users().await? {
        return Ok(());
    }

    info!("No users found, creating initial admin user...");

    let password_hash = api::auth::hash_password(&config.admin_password)?;
    db.create_user(&config.admin_username, &password_hash)
        .await?;

    info!(
        "Admin user '{}' created successfully",
        config.admin_username
    );
    warn!(
        "Default admin password is '{}', please change it after login!",
        config.admin_password
    );

    Ok(())
}
