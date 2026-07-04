pub mod routes;
use std::sync::Arc;

use argon2::{Algorithm, Argon2, Params, Version};
use poem::{EndpointExt, Route, Server, listener::TcpListener};
use tp_sqlx::postgres::postgres_adapter::PostgresAdapter;

use crate::routes::{auth::auth_routes, sessions::session_routes, users::user_routes};

#[tokio::main]
async fn main() -> Result<(), ()> {
    let postgres_adapter = Arc::new(PostgresAdapter::new(None).await.unwrap());
    let params = Params::new(19 * 1024, 2, 1, None).unwrap();
    let argon2 = Arc::new(Argon2::new(Algorithm::Argon2id, Version::V0x13, params));
    let bind = "0.0.0.0:3000";
    let server = Server::new(TcpListener::bind(&bind));

    let routes = Route::new()
        .nest("/auth", auth_routes())
        .nest("/sessions", session_routes())
        .nest("/users", user_routes());

    println!("Listenning on: {}", &bind);
    server
        .run(
            routes
                .data(postgres_adapter)
                .data(argon2)
                .with(poem::middleware::CookieJarManager::new()),
        )
        .await
        .unwrap();
    Ok(())
}
