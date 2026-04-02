// backend/crates/shared-kernel/src/errors.rs
use serde::{Deserialize, Serialize};

// TODO(typeshare): add #[derive(Typeshare)] once typeshare dep is wired in workspace (ADR-009)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub details: Vec<ErrorDetail>,
}

// TODO(typeshare): add #[derive(Typeshare)] once typeshare dep is wired in workspace (ADR-009)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub field: String,
    pub issue: String,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: vec![],
        }
    }

    pub fn with_detail(mut self, field: impl Into<String>, issue: impl Into<String>) -> Self {
        self.details.push(ErrorDetail {
            field: field.into(),
            issue: issue.into(),
        });
        self
    }
}
