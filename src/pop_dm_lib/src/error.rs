use thiserror::Error;

pub type Result<T> = std::result::Result<T, PopDMLibError>;

#[derive(Debug, Error)]
pub enum PopDMLibError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("user not found: {0}")]
    UserNotFound(String),

    #[error("auth failed: {0}")]
    AuthFailed(String),

    #[error("pam auth failed: {0}")]
    PamAuth(#[from] pam_client::Error),

    #[error("invalid session: {0}")]
    InvalidSession(String),
}
