use serde::Serialize;
use std::fmt;
use thiserror::Error;

/// Machine-readable error codes for programmatic error handling
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Invalid input or usage
    UserError,
    /// Worktree, branch, or repository not found
    NotFound,
    /// Git command failed
    GitError,
    /// Configuration issue
    ConfigError,
    /// File system error
    IoError,
}

impl ErrorCode {
    /// Get the exit code for this error category
    pub fn exit_code(&self) -> i32 {
        match self {
            ErrorCode::UserError => 1,
            ErrorCode::NotFound => 2,
            ErrorCode::GitError => 3,
            ErrorCode::ConfigError => 4,
            ErrorCode::IoError => 5,
        }
    }
}

/// Structured error type for wt commands
#[derive(Error, Debug)]
pub enum WtError {
    #[error("{message}")]
    UserError { message: String },

    #[error("{message}")]
    NotFound {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("{message}")]
    GitError {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("{message}")]
    ConfigError {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("{message}")]
    IoError {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },
}

impl WtError {
    /// Get the error code for this error
    pub fn code(&self) -> ErrorCode {
        match self {
            WtError::UserError { .. } => ErrorCode::UserError,
            WtError::NotFound { .. } => ErrorCode::NotFound,
            WtError::GitError { .. } => ErrorCode::GitError,
            WtError::ConfigError { .. } => ErrorCode::ConfigError,
            WtError::IoError { .. } => ErrorCode::IoError,
        }
    }

    /// Get the exit code for this error
    pub fn exit_code(&self) -> i32 {
        self.code().exit_code()
    }

    /// Convert to JSON error output
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "error": true,
            "code": self.code(),
            "message": self.to_string(),
        })
    }

    /// Print error in human-readable structured format
    pub fn print_human(&self) {
        eprintln!(
            "error[{}]: {}",
            format!("{:?}", self.code()).to_lowercase(),
            self
        );
    }
}

/// Result type alias using WtError (used in config.rs)
#[allow(dead_code)]
pub type WtResult<T> = Result<T, WtError>;

/// Helper functions to create errors
impl WtError {
    pub fn user_error(message: impl fmt::Display) -> Self {
        WtError::UserError {
            message: message.to_string(),
        }
    }

    pub fn user_error_with_source(
        message: impl fmt::Display,
        source: impl Into<anyhow::Error>,
    ) -> Self {
        WtError::UserError {
            message: format!("{}: {}", message, source.into()),
        }
    }

    pub fn not_found(message: impl fmt::Display) -> Self {
        WtError::NotFound {
            message: message.to_string(),
            source: None,
        }
    }

    // Not currently used, but kept for API completeness
    #[allow(dead_code)]
    pub fn not_found_with_source(message: impl fmt::Display, source: anyhow::Error) -> Self {
        WtError::NotFound {
            message: message.to_string(),
            source: Some(source),
        }
    }

    // Not currently used, but kept for API completeness
    #[allow(dead_code)]
    pub fn git_error(message: impl fmt::Display) -> Self {
        WtError::GitError {
            message: message.to_string(),
            source: None,
        }
    }

    pub fn git_error_with_source(message: impl fmt::Display, source: anyhow::Error) -> Self {
        WtError::GitError {
            message: message.to_string(),
            source: Some(source),
        }
    }

    // Not currently used, but kept for API completeness
    #[allow(dead_code)]
    pub fn config_error(message: impl fmt::Display) -> Self {
        WtError::ConfigError {
            message: message.to_string(),
            source: None,
        }
    }

    pub fn config_error_with_source(message: impl fmt::Display, source: anyhow::Error) -> Self {
        WtError::ConfigError {
            message: message.to_string(),
            source: Some(source),
        }
    }

    pub fn io_error(message: impl fmt::Display) -> Self {
        WtError::IoError {
            message: message.to_string(),
            source: None,
        }
    }

    pub fn io_error_with_source(message: impl fmt::Display, source: anyhow::Error) -> Self {
        WtError::IoError {
            message: message.to_string(),
            source: Some(source),
        }
    }
}

/// Convert from anyhow::Error to WtError (defaults to UserError)
impl From<anyhow::Error> for WtError {
    fn from(err: anyhow::Error) -> Self {
        WtError::UserError {
            message: err.to_string(),
        }
    }
}
