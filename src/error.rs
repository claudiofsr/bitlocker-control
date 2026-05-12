use thiserror::Error;

/// Standardized Result alias for the application
pub type BitResult<T> = std::result::Result<T, BitError>;

/// Defines all possible errors that can occur within the application.
/// Using thiserror provides automatic std::error::Error implementations.
#[derive(Error, Debug)]
pub enum BitError {
    #[error("Command Execution Failed: {0}")]
    Command(String),

    #[error("Missing System Dependencies: {0}")]
    Dependency(String),

    #[error("Decryption Failed: {0}")]
    Decryption(String),

    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON Parsing Error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Mount/Unmount Failed: {0}")]
    Mount(String),

    #[error("Privilege Elevation Failed: {0}")]
    Privilege(std::io::Error),
}
