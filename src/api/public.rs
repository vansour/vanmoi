//! Public API endpoints (no auth required).

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AppState;
use crate::db::{ClientPublic, PingRecord, PingTask, Record};
use crate::error::AppResult;

/// Get clients response.
#[derive(Debug, Serialize)]
pub struct ClientsResponse {
    pub clients: Vec<ClientWithStatus>,
}

/// Client with current status.
#[derive(Debug, Serialize)]
pub struct ClientWithStatus {
    #[serde(flatten)]
    pub client: ClientPublic,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ClientStatus>,
}

/// Client current status.
#[derive(Debug, Serialize)]
pub struct ClientStatus {
    pub cpu: f32,
    pub ram: i64,
    pub ram_total: i64,
    pub disk: i64,
    pub disk_total: i64,
    pub net_in: i64,
    pub net_out: i64,
    pub load: f32,
    pub uptime: i64,
}

/// GET /api/clients - Get all visible clients with their current status.
pub async fn get_clients(State(state): State<AppState>) -> AppResult<Json<ClientsResponse>> {
    let clients = state.db.get_visible_clients().await?;

    let mut result = Vec::new();
    for client in clients {
        let status = if client.online {
            state
                .db
                .get_latest_record(client.id)
                .await?
                .map(|r| ClientStatus {
                    cpu: r.cpu,
                    ram: r.ram,
                    ram_total: r.ram_total,
                    disk: r.disk,
                    disk_total: r.disk_total,
                    net_in: r.net_in,
                    net_out: r.net_out,
                    load: r.load,
                    uptime: r.uptime,
                })
        } else {
            None
        };

        result.push(ClientWithStatus {
            client: client.into(),
            status,
        });
    }

    Ok(Json(ClientsResponse { clients: result }))
}

/// Node information for API compatibility.
#[derive(Debug, Serialize)]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    pub group: String,
    pub online: bool,
}

/// GET /api/nodes - Get node list (simplified).
pub async fn get_nodes(State(state): State<AppState>) -> AppResult<Json<Vec<NodeInfo>>> {
    let clients = state.db.get_visible_clients().await?;

    let nodes: Vec<NodeInfo> = clients
        .into_iter()
        .map(|c| NodeInfo {
            id: c.id.to_string(),
            name: c.name,
            group: c.group_name,
            online: c.online,
        })
        .collect();

    Ok(Json(nodes))
}

/// Query params for records.
#[derive(Debug, Deserialize)]
pub struct RecordsQuery {
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_limit() -> i32 {
    60
}

/// GET /api/recent/:uuid - Get recent records for a client.
pub async fn get_recent_records(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
    Query(query): Query<RecordsQuery>,
) -> AppResult<Json<Vec<Record>>> {
    let records = state.db.get_recent_records(uuid, query.limit).await?;
    Ok(Json(records))
}

/// GET /api/ping - Get all ping tasks.
pub async fn get_ping_tasks(State(state): State<AppState>) -> AppResult<Json<Vec<PingTask>>> {
    let tasks = state.db.get_all_ping_tasks().await?;
    Ok(Json(tasks))
}

/// GET /api/ping/:id/records - Get ping records for a task.
pub async fn get_ping_records(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<RecordsQuery>,
) -> AppResult<Json<Vec<PingRecord>>> {
    let records = state.db.get_recent_ping_records(id, query.limit).await?;
    Ok(Json(records))
}
