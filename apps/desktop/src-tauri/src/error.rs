use std::sync::{MutexGuard, PoisonError};
use rusqlite::Connection;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Sidecar error: {0}")]
    Sidecar(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Workflow error: {0}")]
    Workflow(String),
    #[error("Budget exhausted: {0}")]
    BudgetExhausted(String),
    #[error("{0}")]
    Internal(String),
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<AppError> for String {
    fn from(e: AppError) -> String {
        e.to_string()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Db(e.to_string())
    }
}

impl From<PoisonError<MutexGuard<'_, Connection>>> for AppError {
    fn from(e: PoisonError<MutexGuard<'_, Connection>>) -> Self {
        AppError::Db(format!("Lock poisoned: {e}"))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
