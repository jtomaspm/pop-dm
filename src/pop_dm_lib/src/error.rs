use thiserror::Error;

pub type Result<T> = std::result::Result<T, PopDMLibError>;

#[derive(Debug, Error)]
pub enum PopDMLibError {
    #[error("user not found: {0}")]
    UserNotFound(String),
}
