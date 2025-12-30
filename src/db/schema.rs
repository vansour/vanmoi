//! Database schema initialization.

use anyhow::Result;
use sqlx::PgPool;

/// Initialize the database schema.
pub async fn init_schema(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        -- Users table
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            username VARCHAR(50) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        -- Sessions table
        CREATE TABLE IF NOT EXISTS sessions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            token VARCHAR(255) UNIQUE NOT NULL,
            user_agent TEXT,
            ip_address VARCHAR(100),
            expires_at TIMESTAMPTZ NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        -- Clients (monitored servers) table
        CREATE TABLE IF NOT EXISTS clients (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            token VARCHAR(255) UNIQUE NOT NULL,
            name VARCHAR(100) NOT NULL DEFAULT '',
            cpu_name VARCHAR(100) DEFAULT '',
            arch VARCHAR(50) DEFAULT '',
            cpu_cores INTEGER DEFAULT 0,
            os VARCHAR(100) DEFAULT '',
            kernel_version VARCHAR(100) DEFAULT '',
            gpu_name VARCHAR(100) DEFAULT '',
            virtualization VARCHAR(50) DEFAULT '',
            ipv4 VARCHAR(100),
            ipv6 VARCHAR(100),
            region VARCHAR(100) DEFAULT '',
            remark TEXT DEFAULT '',
            public_remark TEXT DEFAULT '',
            mem_total BIGINT DEFAULT 0,
            swap_total BIGINT DEFAULT 0,
            disk_total BIGINT DEFAULT 0,
            version VARCHAR(50) DEFAULT '',
            weight INTEGER DEFAULT 0,
            group_name VARCHAR(100) DEFAULT '',
            tags TEXT DEFAULT '',
            hidden BOOLEAN DEFAULT FALSE,
            traffic_limit BIGINT DEFAULT 0,
            traffic_limit_type VARCHAR(10) DEFAULT 'max',
            online BOOLEAN DEFAULT FALSE,
            last_seen_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        -- Records (monitoring data) table
        CREATE TABLE IF NOT EXISTS records (
            id BIGSERIAL PRIMARY KEY,
            client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
            time TIMESTAMPTZ DEFAULT NOW(),
            cpu REAL DEFAULT 0,
            gpu REAL DEFAULT 0,
            ram BIGINT DEFAULT 0,
            ram_total BIGINT DEFAULT 0,
            swap BIGINT DEFAULT 0,
            swap_total BIGINT DEFAULT 0,
            load REAL DEFAULT 0,
            temp REAL DEFAULT 0,
            disk BIGINT DEFAULT 0,
            disk_total BIGINT DEFAULT 0,
            net_in BIGINT DEFAULT 0,
            net_out BIGINT DEFAULT 0,
            net_total_up BIGINT DEFAULT 0,
            net_total_down BIGINT DEFAULT 0,
            process INTEGER DEFAULT 0,
            connections INTEGER DEFAULT 0,
            connections_udp INTEGER DEFAULT 0,
            uptime BIGINT DEFAULT 0
        );

        -- Index for faster record queries
        CREATE INDEX IF NOT EXISTS idx_records_client_time ON records(client_id, time DESC);

        -- Notifications table
        CREATE TABLE IF NOT EXISTS notifications (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL,
            provider VARCHAR(50) NOT NULL,
            config JSONB NOT NULL DEFAULT '{}',
            enabled BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        -- Offline notifications (per client)
        CREATE TABLE IF NOT EXISTS offline_notifications (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
            notification_id UUID REFERENCES notifications(id) ON DELETE SET NULL,
            enabled BOOLEAN DEFAULT FALSE,
            threshold_seconds INTEGER DEFAULT 60,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );

        -- Ping tasks table
        CREATE TABLE IF NOT EXISTS ping_tasks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL,
            target VARCHAR(255) NOT NULL,
            interval_seconds INTEGER DEFAULT 60,
            timeout_seconds INTEGER DEFAULT 5,
            enabled BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );

        -- Ping records table
        CREATE TABLE IF NOT EXISTS ping_records (
            id BIGSERIAL PRIMARY KEY,
            task_id UUID NOT NULL REFERENCES ping_tasks(id) ON DELETE CASCADE,
            client_id UUID REFERENCES clients(id) ON DELETE CASCADE,
            time TIMESTAMPTZ DEFAULT NOW(),
            latency_ms REAL,
            success BOOLEAN DEFAULT FALSE
        );

        -- Index for ping records
        CREATE INDEX IF NOT EXISTS idx_ping_records_task_time ON ping_records(task_id, time DESC);

        -- Settings table (key-value store)
        CREATE TABLE IF NOT EXISTS settings (
            key VARCHAR(100) PRIMARY KEY,
            value JSONB NOT NULL DEFAULT '{}',
            updated_at TIMESTAMPTZ DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
