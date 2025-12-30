//! Database module.
//!
//! Provides database connection, models, and repository operations.

mod models;
mod repository;
mod schema;

pub use models::*;

use anyhow::Result;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tracing::info;

/// Database connection wrapper.
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Connect to the PostgreSQL database.
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Initialize the database schema.
    pub async fn init_schema(&self) -> Result<()> {
        schema::init_schema(&self.pool).await?;
        info!("Database schema initialized successfully");
        Ok(())
    }

    /// Get a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
