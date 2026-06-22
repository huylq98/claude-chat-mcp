//! Connector configuration, loaded entirely from `MONGODB_*` environment
//! variables written by the configurator/wizard.

use connector_core::{env, CoreError};

#[derive(Debug, Clone)]
pub struct MongoConfig {
    /// Connection URI, e.g. `mongodb://user:pass@host:27017`. Carries any TLS
    /// options itself, so there are no separate SSL/proxy fields.
    pub uri: String,
    /// Default database for collection operations when none is passed to a tool.
    pub database: Option<String>,
}

impl MongoConfig {
    pub fn from_env() -> Self {
        Self {
            uri: env::env("MONGODB_URI").unwrap_or_default(),
            database: env::env("MONGODB_DATABASE"),
        }
    }

    pub fn validate(&self) -> Result<(), CoreError> {
        if self.uri.trim().is_empty() {
            return Err(CoreError::Config("MONGODB_URI is required".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_requires_uri() {
        let cfg = MongoConfig {
            uri: String::new(),
            database: None,
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_accepts_uri() {
        let cfg = MongoConfig {
            uri: "mongodb://localhost:27017".into(),
            database: Some("test".into()),
        };
        assert!(cfg.validate().is_ok());
    }
}
