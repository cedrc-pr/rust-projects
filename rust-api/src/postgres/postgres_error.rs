use strum::Display;

#[derive(Debug, thiserror::Error, Display)]
pub enum PostgresError {
    NotFound,
    Conflict,
    Unexpected,
}

impl From<sqlx::Error> for PostgresError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            sqlx::Error::Database(err) => {
                let code = err.code();
                match code.as_deref() {
                    Some("23505") => PostgresError::Conflict,
                    _ => PostgresError::Unexpected,
                }
            }
            _ => PostgresError::Unexpected,
        }
    }
}

impl From<Box<dyn std::error::Error + 'static + Send + Sync>> for PostgresError {
    fn from(_: Box<dyn std::error::Error + 'static + Send + Sync>) -> Self {
        PostgresError::Unexpected
    }
}
