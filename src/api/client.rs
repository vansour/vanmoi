//! Agent (client) API endpoints.
//!
//! These endpoints are used by monitoring agents to register and report data.

use axum::{
    Json,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, header},
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::api::AppState;
use crate::db::RecordInput;
use crate::error::{AppError, AppResult};

/// Register request.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    #[serde(default)]
    pub name: String,
}

/// Register response.
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub uuid: String,
    pub token: String,
}

/// POST /api/agent/register - Register a new agent.
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<RegisterResponse>> {
    let name = if req.name.is_empty() {
        "New Server".to_string()
    } else {
        req.name
    };

    let client = state.db.create_client(&name).await?;

    info!("New agent registered: {} ({})", client.name, client.id);

    Ok(Json(RegisterResponse {
        uuid: client.id.to_string(),
        token: client.token,
    }))
}

/// Basic info upload request.
#[derive(Debug, Deserialize)]
pub struct BasicInfoRequest {
    #[serde(default)]
    pub cpu_name: String,
    #[serde(default)]
    pub arch: String,
    #[serde(default)]
    pub cpu_cores: i32,
    #[serde(default)]
    pub os: String,
    #[serde(default)]
    pub kernel_version: String,
    #[serde(default)]
    pub gpu_name: String,
    #[serde(default)]
    pub virtualization: String,
    #[serde(default)]
    pub mem_total: i64,
    #[serde(default)]
    pub swap_total: i64,
    #[serde(default)]
    pub disk_total: i64,
    #[serde(default)]
    pub version: String,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
}

/// POST /api/agent/info - Upload basic system information.
pub async fn upload_basic_info(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<BasicInfoRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let token = extract_agent_token(&headers)?;
    let client = state
        .db
        .find_client_by_token(&token)
        .await?
        .ok_or(AppError::Unauthorized)?;

    state
        .db
        .update_client_basic_info(
            client.id,
            &req.cpu_name,
            &req.arch,
            req.cpu_cores,
            &req.os,
            &req.kernel_version,
            &req.gpu_name,
            &req.virtualization,
            req.mem_total,
            req.swap_total,
            req.disk_total,
            &req.version,
        )
        .await?;

    if req.ipv4.is_some() || req.ipv6.is_some() {
        state
            .db
            .update_client_ips(client.id, req.ipv4.as_deref(), req.ipv6.as_deref())
            .await?;
    }

    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// POST /api/agent/report - Upload monitoring data.
pub async fn upload_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RecordInput>,
) -> AppResult<Json<serde_json::Value>> {
    let token = extract_agent_token(&headers)?;
    let client = state
        .db
        .find_client_by_token(&token)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Update online status
    state.db.update_client_online(client.id, true).await?;

    // Insert record
    state.db.insert_record(client.id, &req).await?;

    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// GET /api/agent/ws - WebSocket connection for real-time reporting.
pub async fn ws_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, AppError> {
    let token = extract_agent_token(&headers)?;
    let client = state
        .db
        .find_client_by_token(&token)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let client_id = client.id;
    let client_name = client.name.clone();

    Ok(ws.on_upgrade(move |socket| handle_agent_ws(state, client_id, client_name, socket)))
}

/// Handle WebSocket connection from agent.
async fn handle_agent_ws(
    state: AppState,
    client_id: uuid::Uuid,
    client_name: String,
    socket: WebSocket,
) {
    let (mut sender, mut receiver) = socket.split();

    info!(
        "Agent connected via WebSocket: {} ({})",
        client_name, client_id
    );

    // Mark as online
    if let Err(e) = state.db.update_client_online(client_id, true).await {
        error!("Failed to update client online status: {}", e);
    }

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Parse and store record
                match serde_json::from_str::<RecordInput>(&text) {
                    Ok(record) => {
                        if let Err(e) = state.db.insert_record(client_id, &record).await {
                            error!("Failed to insert record: {}", e);
                        }
                        // Update last seen
                        let _ = state.db.update_client_online(client_id, true).await;
                    }
                    Err(e) => {
                        warn!("Invalid record data from {}: {}", client_name, e);
                    }
                }
            }
            Ok(Message::Ping(data)) => {
                if sender.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) => break,
            Err(e) => {
                error!("WebSocket error from {}: {}", client_name, e);
                break;
            }
            _ => {}
        }
    }

    info!("Agent disconnected: {} ({})", client_name, client_id);

    // Mark as offline
    if let Err(e) = state.db.update_client_online(client_id, false).await {
        error!("Failed to update client offline status: {}", e);
    }
}

/// Extract agent token from headers.
fn extract_agent_token(headers: &HeaderMap) -> AppResult<String> {
    if let Some(auth) = headers.get(header::AUTHORIZATION)
        && let Ok(auth_str) = auth.to_str()
            && let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Ok(token.to_string());
            }
    Err(AppError::Unauthorized)
}
