use std::sync::Arc;

use argon2::{Argon2, PasswordHasher};
use poem::{
    Route, get,
    http::StatusCode,
    web::{Data, Json, Path, Query},
};
use tp_sqlx::{
    app_error::AppError,
    postgres::postgres_adapter::PostgresAdapter,
    structs_and_enums::{Auth, Creds, User, UserOptions, UserQuery},
};

pub fn user_routes() -> Route {
    let route = Route::new();
    route.at("/", get(user_get_many).post(user_add)).at(
        "/:id_or_login",
        get(user_get).put(user_edit).delete(user_delete),
    )
}

#[poem::handler]
async fn user_get_many(
    Data(postgres_adapter): Data<&Arc<PostgresAdapter>>,
    Query(user_query): Query<UserQuery>,
) -> Result<Json<Vec<User>>, AppError> {
    let partial_user = user_query.to_partial_user();

    if let Some(field) = user_query.field.clone() {
        partial_user.has_id_if_field(field, user_query.id)?;
    }

    Ok(Json(
        postgres_adapter
            .user_get_many(
                user_query.field,
                user_query.order,
                (user_query.id, Some(partial_user)),
                user_query.limit,
            )
            .await?,
    ))
}

#[poem::handler]
async fn user_get(
    Path(id): Path<String>,
    Data(postgres_adapter): Data<&Arc<PostgresAdapter>>,
) -> Result<Json<User>, AppError> {
    if let Ok(id) = id.parse::<i32>() {
        return Ok(Json(postgres_adapter.user_get(id).await?));
    }
    Ok(Json(postgres_adapter.user_get_by_login(&id).await?))
}

#[poem::handler]
async fn user_add(
    Json(user): Json<UserOptions>,
    Data(postgres_adapter): Data<&Arc<PostgresAdapter>>,
    Data(argon2): Data<&Arc<Argon2<'_>>>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let creds = Creds {
        email: user.email.clone(),
        password: user.password.clone(),
    };
    let user_added = postgres_adapter.user_add(user).await?;
    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let password_hash = argon2
        .hash_password(creds.password.as_bytes(), &salt)
        .map_err(|e| AppError::Argon2(e.to_string()))?
        .to_string();
    let auth: Auth = Auth {
        user_id: user_added.id,
        email: creds.email,
        password: password_hash,
    };
    postgres_adapter.auth_add(auth).await?;
    Ok((StatusCode::CREATED, Json(user_added)))
}

#[poem::handler]
async fn user_edit(
    Path(id): Path<i32>,
    Json(user): Json<UserOptions>,
    Data(postgres_adapter): Data<&Arc<PostgresAdapter>>,
) -> Result<Json<User>, AppError> {
    Ok(Json(postgres_adapter.user_edit(id, user).await?))
}

#[poem::handler]
async fn user_delete(
    Path(id): Path<i32>,
    Data(postgres_adapter): Data<&Arc<PostgresAdapter>>,
) -> Result<Json<User>, AppError> {
    Ok(Json(postgres_adapter.user_delete(id).await?))
}
