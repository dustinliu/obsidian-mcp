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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_display_format() {
        let err = AppError::Api {
            status: 404,
            body: "Not Found".to_string(),
        };
        assert_eq!(err.to_string(), "Obsidian API error (404): Not Found");
    }

    #[test]
    fn json_error_from_conversion() {
        let json_err: serde_json::Error =
            serde_json::from_str::<String>("not valid json").unwrap_err();
        let expected_msg = format!("JSON error: {}", json_err);
        let app_err: AppError = json_err.into();
        assert_eq!(app_err.to_string(), expected_msg);
        assert!(matches!(app_err, AppError::Json(_)));
    }
}
