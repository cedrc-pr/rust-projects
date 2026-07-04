use std::sync::Arc;

use argon2::{Argon2, PasswordVerifier};
use chrono::{DateTime, Utc};
use poem::{
    Route, post,
    web::{
        Data, Json,
        cookie::{Cookie, CookieJar},
    },
};
use tp_sqlx::{
    app_error::AppError, generate_secret, postgres::postgres_adapter::PostgresAdapter,
    structs_and_enums::Creds,
};

pub fn auth_routes() -> Route {
    let route = Route::new();
    route.at("/login", post(login))
}

#[poem::handler]
async fn login(
    cookie_jar: &CookieJar,
    Json(creds): Json<Creds>,
    Data(postgres_adapter): Data<&Arc<PostgresAdapter>>,
    Data(argon2): Data<&Arc<Argon2<'_>>>,
) -> Result<Json<DateTime<Utc>>, AppError> {
    let auth = postgres_adapter.auth_get_by_email(&creds.email).await?;
    let parsed_hash = argon2::PasswordHash::new(&auth.password)
        .map_err(|err| AppError::Argon2(err.to_string()))?;
    argon2
        .verify_password(creds.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("wrong email or password".to_owned()))?;
    let session_id = generate_secret()?;
    let expires_on = postgres_adapter.session_set(&session_id).await?;
    let mut cookie = Cookie::new_with_str("session-id", &session_id);
    cookie.set_path("/");
    cookie_jar.add(cookie);
    Ok(Json(expires_on))
}
