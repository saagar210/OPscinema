use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppErrorCode {
    PermissionDenied,
    ValidationFailed,
    NotFound,
    Conflict,
    PolicyBlocked,
    NetworkBlocked,
    ExportGateFailed,
    ProviderSchemaInvalid,
    Io,
    Db,
    JobCancelled,
    Unsupported,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Error)]
#[error("{code:?}: {message}")]
pub struct AppError {
    pub code: AppErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub recoverable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_hint: Option<String>,
}

pub type AppResult<T> = Result<T, AppError>;
