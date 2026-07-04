use std::sync::Arc;

use chrono::{DateTime, Utc};
use poem::{
    Route,
    http::StatusCode,
    post,
    web::{
        Data, Json,
        cookie::{Cookie, CookieJar},
    },
};
use tp_sqlx::{app_error::AppError, generate_secret, postgres::postgres_adapter::PostgresAdapter};

use crate::routes::Cookies;

pub fn session_routes() -> Route {
    let route = Route::new();
    route.at("/", post(set_session).get(valid_session))
}

#[poem::handler]
async fn set_session(
    cookie_jar: &CookieJar,
    Data(postgres_adapter): Data<&Arc<PostgresAdapter>>,
) -> Result<Json<DateTime<Utc>>, AppError> {
    let session_id = generate_secret()?;
    let expires_on = postgres_adapter.session_set(&session_id).await?;
    let mut cookie = Cookie::new_with_str("session-id", &session_id);
    cookie.set_path("/");
    cookie_jar.add(cookie);
    Ok(Json(expires_on))
}

#[poem::handler]
async fn valid_session(
    cookies: Cookies,
    Data(postgres_adapter): Data<&Arc<PostgresAdapter>>,
) -> Result<poem::http::StatusCode, AppError> {
    let expires_on = postgres_adapter.session_valid(&cookies.session_id).await?;
    if expires_on < Utc::now() {
        return Err(AppError::Unauthorized(
            "expired session-id cookie".to_owned(),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}
