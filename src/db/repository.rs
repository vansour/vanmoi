//! Database repository operations.
//!
//! CRUD operations for all database models.

use super::Database;
use super::models::*;
use crate::error::AppResult;
use chrono::{Duration, Utc};
use sqlx::Row;
use uuid::Uuid;

impl Database {
    // ==================== User Operations ====================

    /// Create a new user.
    pub async fn create_user(&self, username: &str, password_hash: &str) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, password_hash)
            VALUES ($1, $2)
            RETURNING *
            "#,
        )
        .bind(username)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by username.
    pub async fn find_user_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    /// Find user by ID.
    pub async fn find_user_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    /// Update user password.
    pub async fn update_user_password(&self, id: Uuid, password_hash: &str) -> AppResult<()> {
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(password_hash)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Check if any users exist.
    pub async fn has_users(&self) -> AppResult<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    // ==================== Session Operations ====================

    /// Create a new session.
    pub async fn create_session(
        &self,
        user_id: Uuid,
        token: &str,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
        expires_secs: i64,
    ) -> AppResult<Session> {
        let expires_at = Utc::now() + Duration::seconds(expires_secs);

        let session = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (user_id, token, user_agent, ip_address, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(token)
        .bind(user_agent)
        .bind(ip_address)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    /// Find session by token.
    pub async fn find_session_by_token(&self, token: &str) -> AppResult<Option<Session>> {
        let session = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE token = $1 AND expires_at > NOW()",
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    /// Delete session by token.
    pub async fn delete_session(&self, token: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM sessions WHERE token = $1")
            .bind(token)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Delete all sessions for a user.
    pub async fn delete_user_sessions(&self, user_id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get all sessions for a user.
    pub async fn get_user_sessions(&self, user_id: Uuid) -> AppResult<Vec<Session>> {
        let sessions = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE user_id = $1 AND expires_at > NOW() ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(sessions)
    }

    // ==================== Client Operations ====================

    /// Create a new client.
    pub async fn create_client(&self, name: &str) -> AppResult<Client> {
        let token = format!("vmoi_{}", Uuid::new_v4().to_string().replace("-", ""));

        let client = sqlx::query_as::<_, Client>(
            r#"
            INSERT INTO clients (name, token)
            VALUES ($1, $2)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(&token)
        .fetch_one(&self.pool)
        .await?;

        Ok(client)
    }

    /// Find client by ID.
    pub async fn find_client_by_id(&self, id: Uuid) -> AppResult<Option<Client>> {
        let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(client)
    }

    /// Find client by token.
    pub async fn find_client_by_token(&self, token: &str) -> AppResult<Option<Client>> {
        let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE token = $1")
            .bind(token)
            .fetch_optional(&self.pool)
            .await?;

        Ok(client)
    }

    /// Get all clients.
    pub async fn get_all_clients(&self) -> AppResult<Vec<Client>> {
        let clients =
            sqlx::query_as::<_, Client>("SELECT * FROM clients ORDER BY weight DESC, name")
                .fetch_all(&self.pool)
                .await?;

        Ok(clients)
    }

    /// Get visible clients (not hidden).
    pub async fn get_visible_clients(&self) -> AppResult<Vec<Client>> {
        let clients = sqlx::query_as::<_, Client>(
            "SELECT * FROM clients WHERE hidden = FALSE ORDER BY weight DESC, name",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(clients)
    }

    /// Update client basic info.
    pub async fn update_client_basic_info(
        &self,
        id: Uuid,
        cpu_name: &str,
        arch: &str,
        cpu_cores: i32,
        os: &str,
        kernel_version: &str,
        gpu_name: &str,
        virtualization: &str,
        mem_total: i64,
        swap_total: i64,
        disk_total: i64,
        version: &str,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE clients SET
                cpu_name = $2, arch = $3, cpu_cores = $4, os = $5,
                kernel_version = $6, gpu_name = $7, virtualization = $8,
                mem_total = $9, swap_total = $10, disk_total = $11,
                version = $12, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(cpu_name)
        .bind(arch)
        .bind(cpu_cores)
        .bind(os)
        .bind(kernel_version)
        .bind(gpu_name)
        .bind(virtualization)
        .bind(mem_total)
        .bind(swap_total)
        .bind(disk_total)
        .bind(version)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update client online status.
    pub async fn update_client_online(&self, id: Uuid, online: bool) -> AppResult<()> {
        sqlx::query("UPDATE clients SET online = $2, last_seen_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(online)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Update client IP addresses.
    pub async fn update_client_ips(
        &self,
        id: Uuid,
        ipv4: Option<&str>,
        ipv6: Option<&str>,
    ) -> AppResult<()> {
        sqlx::query("UPDATE clients SET ipv4 = $2, ipv6 = $3, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(ipv4)
            .bind(ipv6)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Delete client.
    pub async fn delete_client(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM clients WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Update client editable fields.
    pub async fn update_client(
        &self,
        id: Uuid,
        name: Option<&str>,
        group_name: Option<&str>,
        remark: Option<&str>,
        public_remark: Option<&str>,
        hidden: Option<bool>,
        weight: Option<i32>,
    ) -> AppResult<()> {
        let mut query = String::from("UPDATE clients SET updated_at = NOW()");
        let mut param_count = 1;

        if name.is_some() {
            param_count += 1;
            query.push_str(&format!(", name = ${}", param_count));
        }
        if group_name.is_some() {
            param_count += 1;
            query.push_str(&format!(", group_name = ${}", param_count));
        }
        if remark.is_some() {
            param_count += 1;
            query.push_str(&format!(", remark = ${}", param_count));
        }
        if public_remark.is_some() {
            param_count += 1;
            query.push_str(&format!(", public_remark = ${}", param_count));
        }
        if hidden.is_some() {
            param_count += 1;
            query.push_str(&format!(", hidden = ${}", param_count));
        }
        if weight.is_some() {
            param_count += 1;
            query.push_str(&format!(", weight = ${}", param_count));
        }

        query.push_str(" WHERE id = $1");

        let mut q = sqlx::query(&query).bind(id);

        if let Some(v) = name {
            q = q.bind(v);
        }
        if let Some(v) = group_name {
            q = q.bind(v);
        }
        if let Some(v) = remark {
            q = q.bind(v);
        }
        if let Some(v) = public_remark {
            q = q.bind(v);
        }
        if let Some(v) = hidden {
            q = q.bind(v);
        }
        if let Some(v) = weight {
            q = q.bind(v);
        }

        q.execute(&self.pool).await?;

        Ok(())
    }

    // ==================== Record Operations ====================

    /// Insert a monitoring record.
    pub async fn insert_record(&self, client_id: Uuid, record: &RecordInput) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO records (
                client_id, cpu, gpu, ram, ram_total, swap, swap_total,
                load, temp, disk, disk_total, net_in, net_out,
                net_total_up, net_total_down, process, connections, connections_udp, uptime
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            "#,
        )
        .bind(client_id)
        .bind(record.cpu)
        .bind(record.gpu)
        .bind(record.ram)
        .bind(record.ram_total)
        .bind(record.swap)
        .bind(record.swap_total)
        .bind(record.load)
        .bind(record.temp)
        .bind(record.disk)
        .bind(record.disk_total)
        .bind(record.net_in)
        .bind(record.net_out)
        .bind(record.net_total_up)
        .bind(record.net_total_down)
        .bind(record.process)
        .bind(record.connections)
        .bind(record.connections_udp)
        .bind(record.uptime)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get recent records for a client.
    pub async fn get_recent_records(&self, client_id: Uuid, limit: i32) -> AppResult<Vec<Record>> {
        let records = sqlx::query_as::<_, Record>(
            "SELECT * FROM records WHERE client_id = $1 ORDER BY time DESC LIMIT $2",
        )
        .bind(client_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Get the latest record for a client.
    pub async fn get_latest_record(&self, client_id: Uuid) -> AppResult<Option<Record>> {
        let record = sqlx::query_as::<_, Record>(
            "SELECT * FROM records WHERE client_id = $1 ORDER BY time DESC LIMIT 1",
        )
        .bind(client_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    /// Delete old records (retention policy).
    pub async fn delete_old_records(&self, days: i32) -> AppResult<u64> {
        let result =
            sqlx::query("DELETE FROM records WHERE time < NOW() - INTERVAL '1 day' * $1::integer")
                .bind(days)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected())
    }

    // ==================== Notification Operations ====================

    /// Create a notification provider.
    pub async fn create_notification(
        &self,
        name: &str,
        provider: &str,
        config: serde_json::Value,
    ) -> AppResult<Notification> {
        let notification = sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notifications (name, provider, config)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(provider)
        .bind(config)
        .fetch_one(&self.pool)
        .await?;

        Ok(notification)
    }

    /// Get all notifications.
    pub async fn get_all_notifications(&self) -> AppResult<Vec<Notification>> {
        let notifications =
            sqlx::query_as::<_, Notification>("SELECT * FROM notifications ORDER BY name")
                .fetch_all(&self.pool)
                .await?;

        Ok(notifications)
    }

    /// Delete notification.
    pub async fn delete_notification(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM notifications WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ==================== Ping Task Operations ====================

    /// Create a ping task.
    pub async fn create_ping_task(
        &self,
        name: &str,
        target: &str,
        interval_seconds: i32,
        timeout_seconds: i32,
    ) -> AppResult<PingTask> {
        let task = sqlx::query_as::<_, PingTask>(
            r#"
            INSERT INTO ping_tasks (name, target, interval_seconds, timeout_seconds)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(target)
        .bind(interval_seconds)
        .bind(timeout_seconds)
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    /// Get all ping tasks.
    pub async fn get_all_ping_tasks(&self) -> AppResult<Vec<PingTask>> {
        let tasks = sqlx::query_as::<_, PingTask>("SELECT * FROM ping_tasks ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        Ok(tasks)
    }

    /// Get enabled ping tasks.
    pub async fn get_enabled_ping_tasks(&self) -> AppResult<Vec<PingTask>> {
        let tasks = sqlx::query_as::<_, PingTask>(
            "SELECT * FROM ping_tasks WHERE enabled = TRUE ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    /// Insert ping record.
    pub async fn insert_ping_record(
        &self,
        task_id: Uuid,
        client_id: Option<Uuid>,
        latency_ms: Option<f32>,
        success: bool,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO ping_records (task_id, client_id, latency_ms, success)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(task_id)
        .bind(client_id)
        .bind(latency_ms)
        .bind(success)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get recent ping records for a task.
    pub async fn get_recent_ping_records(
        &self,
        task_id: Uuid,
        limit: i32,
    ) -> AppResult<Vec<PingRecord>> {
        let records = sqlx::query_as::<_, PingRecord>(
            "SELECT * FROM ping_records WHERE task_id = $1 ORDER BY time DESC LIMIT $2",
        )
        .bind(task_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    // ==================== Settings Operations ====================

    /// Get a setting value.
    pub async fn get_setting(&self, key: &str) -> AppResult<Option<serde_json::Value>> {
        let setting = sqlx::query_as::<_, Setting>("SELECT * FROM settings WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        Ok(setting.map(|s| s.value))
    }

    /// Set a setting value.
    pub async fn set_setting(&self, key: &str, value: serde_json::Value) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO settings (key, value)
            VALUES ($1, $2)
            ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
