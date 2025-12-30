//! Authentication middleware.

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};

use crate::api::AppState;
use crate::db::User;

/// Extract session from request and add user to extensions.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Try to get token from Authorization header or cookie
    let token = extract_token(&request);

    if let Some(token) = token {
        // Find session
        if let Ok(Some(session)) = state.db.find_session_by_token(&token).await {
            // Find user
            if let Ok(Some(user)) = state.db.find_user_by_id(session.user_id).await {
                request.extensions_mut().insert(user);
            }
        }
    }

    Ok(next.run(request).await)
}

/// Require authentication - return 401 if not authenticated.
pub async fn require_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = extract_token(&request);

    let token = token.ok_or(StatusCode::UNAUTHORIZED)?;

    let session = state
        .db
        .find_session_by_token(&token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let user = state
        .db
        .find_user_by_id(session.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

/// Extract token from Authorization header or cookie.
fn extract_token(request: &Request) -> Option<String> {
    // Try Authorization header first
    if let Some(auth_header) = request.headers().get(header::AUTHORIZATION)
        && let Ok(auth_str) = auth_header.to_str()
            && let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }

    // Try cookie
    if let Some(cookie_header) = request.headers().get(header::COOKIE)
        && let Ok(cookies) = cookie_header.to_str() {
            for cookie in cookies.split(';') {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if parts.len() == 2 && parts[0] == "token" {
                    return Some(parts[1].to_string());
                }
            }
        }

    None
}

/// Extract current user from request extensions.
pub fn get_current_user(request: &Request) -> Option<&User> {
    request.extensions().get::<User>()
}
