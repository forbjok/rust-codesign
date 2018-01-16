use std::io;
use std::result;

use bitness::BitnessError;

#[derive(Debug, Fail)]
pub enum CodeSignError {
    #[fail(display = "I/O error: {}", description)]
    IoError {
        description: String,
    },

    #[fail(display = "{}", description)]
    Error {
        description: String,
    },

    #[fail(display = "SignTool exited with code {}: {}", exit_code, stderr)]
    SignToolError {
        exit_code: i32,
        stderr: String,
    },
}

impl From<String> for CodeSignError {
    fn from(err: String) -> Self {
        CodeSignError::Error {
            description: err,
        }
    }
}

impl From<io::Error> for CodeSignError {
    fn from(err: io::Error) -> Self {
        CodeSignError::IoError {
            description: err.to_string(),
        }
    }
}

impl From<BitnessError> for CodeSignError {
    fn from(err: BitnessError) -> Self {
        CodeSignError::Error {
            description: err.to_string(),
        }
    }
}

pub type CodeSignResult<T> = result::Result<T, CodeSignError>;
