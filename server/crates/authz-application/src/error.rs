use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("validation failed: {message}")]
    Validation { message: String },
    #[error("unexpected: {message}")]
    Unexpected {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    },
}

impl ApplicationError {
    pub fn unexpected(message: impl Into<String>) -> Self {
        Self::Unexpected {
            message: message.into(),
            source: None,
        }
    }

    pub fn unexpected_with_source<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Unexpected {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}
