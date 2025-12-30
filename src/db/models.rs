//! Database models.
//!
//! Rust structs that map to PostgreSQL tables.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User model.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Session model.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(skip_serializing)]
    pub token: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Client (monitored server) model.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Client {
    pub id: Uuid,
    #[serde(skip_serializing)]
    pub token: String,
    pub name: String,
    pub cpu_name: String,
    pub arch: String,
    pub cpu_cores: i32,
    pub os: String,
    pub kernel_version: String,
    pub gpu_name: String,
    pub virtualization: String,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub region: String,
    pub remark: String,
    pub public_remark: String,
    pub mem_total: i64,
    pub swap_total: i64,
    pub disk_total: i64,
    pub version: String,
    pub weight: i32,
    pub group_name: String,
    pub tags: String,
    pub hidden: bool,
    pub traffic_limit: i64,
    pub traffic_limit_type: String,
    pub online: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Public client info (for non-admin users).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPublic {
    pub id: Uuid,
    pub name: String,
    pub cpu_name: String,
    pub arch: String,
    pub cpu_cores: i32,
    pub os: String,
    pub region: String,
    pub public_remark: String,
    pub mem_total: i64,
    pub disk_total: i64,
    pub group_name: String,
    pub online: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
}

impl From<Client> for ClientPublic {
    fn from(c: Client) -> Self {
        Self {
            id: c.id,
            name: c.name,
            cpu_name: c.cpu_name,
            arch: c.arch,
            cpu_cores: c.cpu_cores,
            os: c.os,
            region: c.region,
            public_remark: c.public_remark,
            mem_total: c.mem_total,
            disk_total: c.disk_total,
            group_name: c.group_name,
            online: c.online,
            last_seen_at: c.last_seen_at,
        }
    }
}

/// Record (monitoring data point) model.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Record {
    pub id: i64,
    pub client_id: Uuid,
    pub time: Option<DateTime<Utc>>,
    pub cpu: f32,
    pub gpu: f32,
    pub ram: i64,
    pub ram_total: i64,
    pub swap: i64,
    pub swap_total: i64,
    pub load: f32,
    pub temp: f32,
    pub disk: i64,
    pub disk_total: i64,
    pub net_in: i64,
    pub net_out: i64,
    pub net_total_up: i64,
    pub net_total_down: i64,
    pub process: i32,
    pub connections: i32,
    pub connections_udp: i32,
    pub uptime: i64,
}

/// Record input from agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordInput {
    pub cpu: f32,
    #[serde(default)]
    pub gpu: f32,
    pub ram: i64,
    pub ram_total: i64,
    #[serde(default)]
    pub swap: i64,
    #[serde(default)]
    pub swap_total: i64,
    #[serde(default)]
    pub load: f32,
    #[serde(default)]
    pub temp: f32,
    pub disk: i64,
    pub disk_total: i64,
    pub net_in: i64,
    pub net_out: i64,
    pub net_total_up: i64,
    pub net_total_down: i64,
    #[serde(default)]
    pub process: i32,
    #[serde(default)]
    pub connections: i32,
    #[serde(default)]
    pub connections_udp: i32,
    #[serde(default)]
    pub uptime: i64,
}

/// Notification provider configuration.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Ping task model.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PingTask {
    pub id: Uuid,
    pub name: String,
    pub target: String,
    pub interval_seconds: i32,
    pub timeout_seconds: i32,
    pub enabled: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Ping record model.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PingRecord {
    pub id: i64,
    pub task_id: Uuid,
    pub client_id: Option<Uuid>,
    pub time: Option<DateTime<Utc>>,
    pub latency_ms: Option<f32>,
    pub success: bool,
}

/// Settings model (key-value).
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: serde_json::Value,
    pub updated_at: Option<DateTime<Utc>>,
}
