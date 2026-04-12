use crate::config::Config;
use crate::technitium;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;
use tokio::sync::RwLock;

pub struct AppState {
    pub config: Config,
    pub is_ready: RwLock<bool>,
    pub client: RwLock<technitium::TechnitiumClient>,
    pub use_static_token: bool,
}

impl AppState {
    pub async fn ensure_ready(&self) -> Result<(), AppError> {
        if !*self.is_ready.read().await {
            return Err(AppError::NotReady);
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Service not ready yet. Try again later.")]
    NotReady,
    #[error("Failed to serialize JSON: {0}")]
    JsonSerializeError(#[from] serde_json::Error),
    #[error("Failed to communicate with Technitium server: {0}")]
    TechnitiumError(#[from] technitium::TechnitiumError),
}

// Implement IntoResponse for our custom error to control the HTTP response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotReady => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AppError::JsonSerializeError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::TechnitiumError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (
            status,
            json!({
                "error": error_message,
            })
            .to_string(),
        )
            .into_response()
    }
}
