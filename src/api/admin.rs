//! Admin API endpoints.
//!
//! These endpoints require authentication and are used for server management.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::{Extension, Path, State},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::AppState;
use crate::db::{Client, Notification, PingTask, Session, User};
use crate::error::{AppError, AppResult};

// ==================== Client Management ====================

/// GET /api/admin/clients - List all clients.
pub async fn list_clients(State(state): State<AppState>) -> AppResult<Json<Vec<Client>>> {
    let clients = state.db.get_all_clients().await?;
    Ok(Json(clients))
}

/// Add client request.
#[derive(Debug, Deserialize)]
pub struct AddClientRequest {
    pub name: String,
}

/// POST /api/admin/clients - Add a new client.
pub async fn add_client(
    State(state): State<AppState>,
    Json(req): Json<AddClientRequest>,
) -> AppResult<Json<Client>> {
    let client = state.db.create_client(&req.name).await?;
    Ok(Json(client))
}

/// GET /api/admin/clients/:id - Get client details.
pub async fn get_client(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Client>> {
    let client = state
        .db
        .find_client_by_id(id)
        .await?
        .ok_or(AppError::NotFound("Client not found".into()))?;
    Ok(Json(client))
}

/// Edit client request.
#[derive(Debug, Deserialize)]
pub struct EditClientRequest {
    pub name: Option<String>,
    pub group_name: Option<String>,
    pub remark: Option<String>,
    pub public_remark: Option<String>,
    pub hidden: Option<bool>,
    pub weight: Option<i32>,
}

/// POST /api/admin/clients/:id - Edit client.
pub async fn edit_client(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<EditClientRequest>,
) -> AppResult<Json<serde_json::Value>> {
    state
        .db
        .update_client(
            id,
            req.name.as_deref(),
            req.group_name.as_deref(),
            req.remark.as_deref(),
            req.public_remark.as_deref(),
            req.hidden,
            req.weight,
        )
        .await?;

    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// DELETE /api/admin/clients/:id - Delete client.
pub async fn delete_client(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.db.delete_client(id).await?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// GET /api/admin/clients/:id/token - Get client token.
pub async fn get_client_token(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let client = state
        .db
        .find_client_by_id(id)
        .await?
        .ok_or(AppError::NotFound("Client not found".into()))?;

    Ok(Json(serde_json::json!({
        "uuid": client.id.to_string(),
        "token": client.token
    })))
}

// ==================== Settings ====================

/// GET /api/admin/settings - Get all settings.
pub async fn get_settings(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let site_name = state
        .db
        .get_setting("site_name")
        .await?
        .unwrap_or(serde_json::json!("Vanmoi"));
    let site_description = state
        .db
        .get_setting("site_description")
        .await?
        .unwrap_or(serde_json::json!("Server Monitoring"));

    Ok(Json(serde_json::json!({
        "site_name": site_name,
        "site_description": site_description
    })))
}

/// Update settings request.
#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub site_name: Option<String>,
    pub site_description: Option<String>,
}

/// POST /api/admin/settings - Update settings.
pub async fn update_settings(
    State(state): State<AppState>,
    Json(req): Json<UpdateSettingsRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if let Some(name) = req.site_name {
        state
            .db
            .set_setting("site_name", serde_json::json!(name))
            .await?;
    }
    if let Some(desc) = req.site_description {
        state
            .db
            .set_setting("site_description", serde_json::json!(desc))
            .await?;
    }

    Ok(Json(serde_json::json!({"status": "ok"})))
}

// ==================== Notifications ====================

/// GET /api/admin/notifications - List all notifications.
pub async fn list_notifications(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<Notification>>> {
    let notifications = state.db.get_all_notifications().await?;
    Ok(Json(notifications))
}

/// Add notification request.
#[derive(Debug, Deserialize)]
pub struct AddNotificationRequest {
    pub name: String,
    pub provider: String,
    pub config: serde_json::Value,
}

/// POST /api/admin/notifications - Add notification.
pub async fn add_notification(
    State(state): State<AppState>,
    Json(req): Json<AddNotificationRequest>,
) -> AppResult<Json<Notification>> {
    let notification = state
        .db
        .create_notification(&req.name, &req.provider, req.config)
        .await?;
    Ok(Json(notification))
}

/// DELETE /api/admin/notifications/:id - Delete notification.
pub async fn delete_notification(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.db.delete_notification(id).await?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

/// Test notification request.
#[derive(Debug, Deserialize)]
pub struct TestNotificationRequest {
    pub provider: String,
    pub config: serde_json::Value,
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_message")]
    pub message: String,
}

fn default_title() -> String {
    "Vanmoi Test".to_string()
}

fn default_message() -> String {
    "This is a test notification from Vanmoi.".to_string()
}

/// POST /api/admin/notifications/test - Test notification.
pub async fn test_notification(
    Json(req): Json<TestNotificationRequest>,
) -> AppResult<Json<serde_json::Value>> {
    crate::notifier::send_notification(&req.provider, &req.config, &req.title, &req.message)
        .await
        .map_err(|e| AppError::Internal(format!("Notification failed: {}", e)))?;

    Ok(Json(
        serde_json::json!({"status": "ok", "message": "Notification sent"}),
    ))
}

// ==================== Ping Tasks ====================

/// GET /api/admin/ping - List all ping tasks.
pub async fn list_ping_tasks(State(state): State<AppState>) -> AppResult<Json<Vec<PingTask>>> {
    let tasks = state.db.get_all_ping_tasks().await?;
    Ok(Json(tasks))
}

/// Add ping task request.
#[derive(Debug, Deserialize)]
pub struct AddPingTaskRequest {
    pub name: String,
    pub target: String,
    #[serde(default = "default_interval")]
    pub interval_seconds: i32,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: i32,
}

fn default_interval() -> i32 {
    60
}

fn default_timeout() -> i32 {
    5
}

/// POST /api/admin/ping - Add ping task.
pub async fn add_ping_task(
    State(state): State<AppState>,
    Json(req): Json<AddPingTaskRequest>,
) -> AppResult<Json<PingTask>> {
    let task = state
        .db
        .create_ping_task(
            &req.name,
            &req.target,
            req.interval_seconds,
            req.timeout_seconds,
        )
        .await?;
    Ok(Json(task))
}

/// DELETE /api/admin/ping/:id - Delete ping task.
pub async fn delete_ping_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM ping_tasks WHERE id = $1")
        .bind(id)
        .execute(state.db.pool())
        .await?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

// ==================== User Management ====================

/// Change password request.
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// POST /api/admin/user/password - Change password.
pub async fn change_password(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify old password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::Internal("Invalid password hash".into()))?;

    let valid = Argon2::default()
        .verify_password(req.old_password.as_bytes(), &parsed_hash)
        .is_ok();

    if !valid {
        return Err(AppError::BadRequest("Invalid old password".into()));
    }

    // Hash new password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let new_hash = argon2
        .hash_password(req.new_password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
        .to_string();

    // Update password
    state.db.update_user_password(user.id, &new_hash).await?;

    Ok(Json(serde_json::json!({"status": "ok"})))
}

// ==================== Session Management ====================

/// GET /api/admin/sessions - List user sessions.
pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> AppResult<Json<Vec<Session>>> {
    let sessions = state.db.get_user_sessions(user.id).await?;
    Ok(Json(sessions))
}

/// DELETE /api/admin/sessions/:id - Delete a session.
pub async fn delete_session(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify session belongs to user
    let sessions = state.db.get_user_sessions(user.id).await?;
    let session = sessions
        .iter()
        .find(|s| s.id == id)
        .ok_or(AppError::NotFound("Session not found".into()))?;

    state.db.delete_session(&session.token).await?;

    Ok(Json(serde_json::json!({"status": "ok"})))
}
