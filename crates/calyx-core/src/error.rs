use thiserror::Error;

#[derive(Debug, Error)]
pub enum CalyxError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("JSON validation error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Widevine L3 DRM error: {0}")]
    Drm(String),
    #[error("Audio playback error: {0}")]
    Audio(String),
    #[error("Authentication required: {0}")]
    Auth(String),
    #[error("Invalid input boundary: {0}")]
    Validation(String),
}

pub type CalyxResult<T> = Result<T, CalyxError>;