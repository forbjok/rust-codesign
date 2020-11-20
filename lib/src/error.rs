use std::io;

use bitness::BitnessError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodeSignError {
    #[error("I/O error")]
    Io(#[from] io::Error),

    #[error("SignTool exited with code {exit_code}: {stderr}")]
    SignToolError { exit_code: i32, stderr: String },

    #[error("{0}")]
    Other(String),
}

impl From<BitnessError> for CodeSignError {
    fn from(err: BitnessError) -> Self {
        CodeSignError::Other(err.to_string())
    }
}

impl From<String> for CodeSignError {
    fn from(err: String) -> Self {
        CodeSignError::Other(err)
    }
}
