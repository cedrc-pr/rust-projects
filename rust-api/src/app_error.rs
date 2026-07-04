use poem::{IntoResponse, http::StatusCode};

use crate::postgres::postgres_error::PostgresError;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("postgres error: {0}")]
    Postgres(#[from] PostgresError),
    #[error("getrandom error: {0}")]
    GetRandom(#[from] getrandom::Error),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("argon2 hash error: {0}")]
    Argon2(String),
}

impl poem::error::ResponseError for AppError {
    fn status(&self) -> poem::http::StatusCode {
        match self {
            Self::Postgres(err) => match err {
                PostgresError::NotFound => StatusCode::NOT_FOUND,
                PostgresError::Conflict => StatusCode::CONFLICT,
                PostgresError::Unexpected => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::GetRandom(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Argon2(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn as_response(&self) -> poem::Response
    where
        Self: std::error::Error + Send + Sync + 'static,
    {
        poem::Response::builder()
            .status(self.status())
            .body(self.to_string())
            .into_response()
    }
}
