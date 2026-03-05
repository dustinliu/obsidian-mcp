use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Obsidian API error ({status}): {body}")]
    Api { status: u16, body: String },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
