//! Notification module.
//!
//! Provides notification sending capabilities for various providers.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// Notification provider types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationProvider {
    Telegram,
    Email,
    Webhook,
}

/// Telegram notification config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
}

/// Email notification config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_addr: String,
    pub to_addr: String,
}

/// Webhook notification config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
}

/// Send a notification.
pub async fn send_notification(
    provider: &str,
    config: &serde_json::Value,
    title: &str,
    message: &str,
) -> Result<()> {
    match provider {
        "telegram" => {
            let cfg: TelegramConfig = serde_json::from_value(config.clone())?;
            send_telegram(&cfg, title, message).await?;
        }
        "email" => {
            let cfg: EmailConfig = serde_json::from_value(config.clone())?;
            send_email(&cfg, title, message).await?;
        }
        "webhook" => {
            let cfg: WebhookConfig = serde_json::from_value(config.clone())?;
            send_webhook(&cfg, title, message).await?;
        }
        _ => {
            error!("Unknown notification provider: {}", provider);
        }
    }
    Ok(())
}

/// Send Telegram notification.
async fn send_telegram(config: &TelegramConfig, title: &str, message: &str) -> Result<()> {
    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        config.bot_token
    );

    let text = format!("*{}*\n\n{}", title, message);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": config.chat_id,
            "text": text,
            "parse_mode": "Markdown"
        }))
        .send()
        .await?;

    if response.status().is_success() {
        info!("Telegram notification sent successfully");
    } else {
        error!(
            "Failed to send Telegram notification: {}",
            response.status()
        );
    }

    Ok(())
}

/// Send email notification (placeholder).
async fn send_email(config: &EmailConfig, title: &str, _message: &str) -> Result<()> {
    // Email sending requires additional dependencies (lettre)
    // For now, just log
    info!(
        "Email notification would be sent to {} with title: {}",
        config.to_addr, title
    );
    Ok(())
}

/// Send webhook notification.
async fn send_webhook(config: &WebhookConfig, title: &str, message: &str) -> Result<()> {
    let client = reqwest::Client::new();

    let mut request = client.post(&config.url).json(&serde_json::json!({
        "title": title,
        "message": message,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    for (key, value) in &config.headers {
        request = request.header(key, value);
    }

    let response = request.send().await?;

    if response.status().is_success() {
        info!("Webhook notification sent successfully to {}", config.url);
    } else {
        error!("Failed to send webhook notification: {}", response.status());
    }

    Ok(())
}
