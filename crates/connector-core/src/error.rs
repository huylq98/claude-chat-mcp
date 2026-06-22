use thiserror::Error;

/// Errors shared by every connector's HTTP-based client.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("API error (HTTP {status}): {message}")]
    Http { status: u16, message: String },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl CoreError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Http { status, .. } => *status,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_error_formats_status_and_message() {
        let err = CoreError::Http {
            status: 404,
            message: "Not found".into(),
        };
        assert_eq!(err.to_string(), "API error (HTTP 404): Not found");
        assert_eq!(err.status_code(), 404);
    }

    #[test]
    fn non_http_error_status_is_zero() {
        let err = CoreError::Config("no url".into());
        assert_eq!(err.status_code(), 0);
    }
}
