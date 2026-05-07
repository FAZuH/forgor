use crate::config::ConfigError;
use crate::ui::UiError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("invalid argument: {0}")]
    CliError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error(transparent)]
    UiError(#[from] UiError),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

impl From<ConfigError> for AppError {
    fn from(value: ConfigError) -> Self {
        Self::ConfigError(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
