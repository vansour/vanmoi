//! Authentication API endpoints.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::{Extension, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::db::User;
use crate::error::{AppError, AppResult};

/// Login request body.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response body.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

/// User info response.
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
}

impl From<&User> for UserInfo {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username.clone(),
        }
    }
}

/// POST /api/login - User login.
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    // Find user
    let user = state
        .db
        .find_user_by_username(&req.username)
        .await?
        .ok_or(AppError::BadRequest("Invalid username or password".into()))?;

    // Verify password using argon2
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::Internal("Invalid password hash".into()))?;

    let valid = Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !valid {
        return Err(AppError::BadRequest("Invalid username or password".into()));
    }

    // Generate session token
    let token = format!(
        "vmses_{}",
        uuid::Uuid::new_v4().to_string().replace("-", "")
    );

    // Create session
    state
        .db
        .create_session(user.id, &token, None, None, state.config.jwt_expires_secs)
        .await?;

    let response = LoginResponse {
        token: token.clone(),
        user: UserInfo::from(&user),
    };

    // Set cookie
    let cookie = format!(
        "token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        token, state.config.jwt_expires_secs
    );

    Ok(([(header::SET_COOKIE, cookie)], Json(response)))
}

/// GET /api/logout - User logout.
pub async fn logout(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> AppResult<impl IntoResponse> {
    // Extract token
    if let Some(cookie_header) = headers.get(header::COOKIE)
        && let Ok(cookies) = cookie_header.to_str() {
            for cookie in cookies.split(';') {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if parts.len() == 2 && parts[0] == "token" {
                    let _ = state.db.delete_session(parts[1]).await;
                }
            }
        }

    // Clear cookie
    let cookie = "token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";

    Ok((
        [(header::SET_COOKIE, cookie)],
        Json(serde_json::json!({"message": "Logged out successfully"})),
    ))
}

/// GET /api/me - Get current user info.
pub async fn me(Extension(user): Extension<Option<User>>) -> impl IntoResponse {
    match user {
        Some(user) => (StatusCode::OK, Json(Some(UserInfo::from(&user)))),
        None => (StatusCode::OK, Json(None)),
    }
}

/// Hash a password using argon2.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;
    Ok(hash.to_string())
}
