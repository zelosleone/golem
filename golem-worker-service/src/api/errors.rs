use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Route validation error: {0}")]
    Route(String),
    #[error("Type validation error: {0}")]
    Type(String),
    #[error("Multiple validation errors: {0:?}")]
    Multiple(Vec<String>)
}

pub type ValidationResult<T> = Result<T, ValidationError>;
