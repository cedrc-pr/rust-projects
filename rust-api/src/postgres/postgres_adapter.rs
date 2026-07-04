use chrono::{DateTime, Duration, Utc};
use sqlx::{Error, query_as, query_as_with};

use crate::{
    postgres::{
        postgres_error::PostgresError,
        postgres_request_builder::{CompOp, Order, PsqlRqBuilder},
    },
    structs_and_enums::{
        Auth, PartialUser, Task, TaskOptions, TaskWithAuthorIdField, TaskWithAuthorLogin, User,
        UserField, UserOptions,
    },
};

pub struct PostgresAdapter {
    pub pool: sqlx::PgPool,
}

impl PostgresAdapter {
    pub async fn new(url: Option<&str>) -> Result<Self, Error> {
        Ok(Self {
            pool: sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(url.unwrap_or("postgres://sqlx_user:abc@localhost/sqlx"))
                .await?,
        })
    }

    pub async fn user_get(&self, id: i32) -> Result<User, PostgresError> {
        let mut builder = PsqlRqBuilder::default();
        builder.r#where("id", CompOp::Equal, id)?;
        let sql = format!("SELECT * FROM \"user\" {};", builder.build());
        Ok(query_as_with(&sql, builder.args)
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn user_get_by_login(&self, login: &str) -> Result<User, PostgresError> {
        let mut builder = PsqlRqBuilder::default();
        builder.r#where("login", CompOp::Equal, login)?;
        let sql = format!("SELECT * FROM \"user\" {};", builder.build());
        Ok(query_as_with(&sql, builder.args)
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn user_add(&self, user: UserOptions) -> Result<User, PostgresError> {
        Ok(
            query_as("insert into \"user\" (name, login) values ($1, $2) returning *;")
                .bind(user.name)
                .bind(user.login)
                .fetch_one(&self.pool)
                .await?,
        )
    }

    pub async fn user_edit(&self, id: i32, user: UserOptions) -> Result<User, PostgresError> {
        let mut builder = PsqlRqBuilder::default();
        builder.assignment("name", user.name)?;
        builder.assignment("login", user.login)?;
        builder.r#where("id", CompOp::Equal, id)?;
        let sql = format!("UPDATE \"user\" {} RETURNING *;", builder.build());
        Ok(query_as_with(&sql, builder.args)
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn user_delete(&self, id: i32) -> Result<User, PostgresError> {
        let mut builder = PsqlRqBuilder::default();
        builder.r#where("id", CompOp::Equal, id)?;
        let sql = format!("DELETE FROM \"user\" {} RETURNING *;", builder.build());
        Ok(query_as_with(&sql, builder.args)
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn user_get_many(
        &self,
        opt_field: Option<UserField>,
        opt_order: Option<Order>,
        last: (Option<i32>, Option<PartialUser>),
        opt_limit: Option<u8>,
    ) -> Result<Vec<User>, PostgresError> {
        let order = opt_order.unwrap_or(Order::Asc);
        let limit = opt_limit.unwrap_or(20);
        let mut builder = PsqlRqBuilder::default();
        if let Some(field) = opt_field.clone() {
            builder.order_by(&field.to_string(), order);
        }

        if last.0.is_some()
            && let Some(user) = last.1
            && let Some(field) = opt_field
        {
            builder.order_by("id", Order::Asc);
            match field {
                UserField::Id => builder.r#where(&field.to_string(), CompOp::Greater, user.id)?,
                UserField::Login => builder.tuple_where(
                    &field.to_string(),
                    &"id".to_string(),
                    CompOp::Greater,
                    user.login,
                    user.id,
                )?,
                UserField::Name => builder.tuple_where(
                    &field.to_string(),
                    &"id".to_string(),
                    CompOp::Greater,
                    user.name,
                    user.id,
                )?,
                UserField::CreatedOn => builder.tuple_where(
                    &field.to_string(),
                    &"id".to_string(),
                    CompOp::Greater,
                    user.created_on,
                    user.id,
                )?,
            }
        }
        builder.limit(limit)?;
        let sql = format!("SELECT * FROM \"user\" {};", builder.build());
        let users: Vec<User> = query_as_with(&sql, builder.args)
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }

    pub async fn task_get_all(&self) -> Result<Vec<TaskWithAuthorLogin>, Error> {
        query_as(
            r#"
            SELECT
                t.author_id,
                t.project_id,
                t.name,
                t.deadline,
                t.id,
                t.created_on,
                u.login AS author_login
            FROM task t
            INNER JOIN "user" u
                ON u.id = t.author_id
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn task_get_by_author_id(
        &self,
        author_id: i32,
    ) -> Result<Vec<TaskWithAuthorLogin>, PostgresError> {
        let mut builder = PsqlRqBuilder::default();
        builder.r#where("t.author_id", CompOp::Equal, author_id)?;
        let sql = format!(
            r#"
            SELECT
                t.author_id,
                t.project_id,
                t.name,
                t.deadline,
                t.id,
                t.created_on,
                u.login AS author_login
            FROM task t
            INNER JOIN "user" u
                ON u.id = t.author_id
            {};
            "#,
            builder.build()
        );
        Ok(query_as_with(&sql, builder.args)
            .fetch_all(&self.pool)
            .await?)
    }

    pub async fn task_get_by_project_id(
        &self,
        project_id: Option<i32>,
    ) -> Result<Vec<TaskWithAuthorLogin>, PostgresError> {
        let mut builder = PsqlRqBuilder::default();
        builder.r#where("t.project_id", CompOp::Equal, project_id)?;
        let sql = format!(
            r#"
            SELECT
                t.author_id,
                t.project_id,
                t.name,
                t.deadline,
                t.id,
                t.created_on,
                u.login AS author_login
            FROM task t
            INNER JOIN "user" u
                ON u.id = t.author_id
            {};
            "#,
            builder.build()
        );
        Ok(query_as_with(&sql, builder.args)
            .fetch_all(&self.pool)
            .await?)
    }

    pub async fn task_get(&self, id: i32) -> Result<TaskWithAuthorLogin, PostgresError> {
        let mut builder = PsqlRqBuilder::default();
        builder.r#where("t.id", CompOp::Equal, id)?;
        let sql = format!(
            r#"
            SELECT
                t.author_id,
                t.project_id,
                t.name,
                t.deadline,
                t.id,
                t.created_on,
                u.login AS author_login
            FROM task t
            INNER JOIN "user" u
                ON u.id = t.author_id
            {};
            "#,
            builder.build()
        );
        Ok(query_as_with(&sql, builder.args)
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn task_add(&self, task: TaskOptions) -> Result<Task, Error> {
        query_as("insert into task (author_id, project_id, name, deadline) values ($1, $2, $3, $4) returning *;")
            .bind(task.author_id)
            .bind(task.project_id)
            .bind(task.name)
            .bind(task.deadline)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn task_edit(&self, id: i32, task: TaskOptions) -> Result<Task, PostgresError> {
        let mut builder = PsqlRqBuilder::default();
        builder.assignment("author_id", task.author_id)?;
        builder.assignment("project_id", task.project_id)?;
        builder.assignment("name", task.name)?;
        builder.assignment("deadline", task.deadline)?;
        builder.r#where("id", CompOp::Equal, id)?;
        let sql = format!("UPDATE task {} RETURNING *;", builder.build());
        Ok(query_as_with(&sql, builder.args)
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn task_delete(&self, id: i32) -> Result<Task, PostgresError> {
        let mut builder = PsqlRqBuilder::default();
        builder.r#where("id", CompOp::Equal, id)?;
        let sql = format!("DELETE FROM task {} RETURNING *;", builder.build());
        Ok(query_as_with(&sql, builder.args)
            .fetch_one(&self.pool)
            .await?)
    }

    pub async fn task_get_all_filtered<T>(
        &self,
        task_field: TaskWithAuthorIdField,
        order: Order,
        last: Option<(i32, T)>,
        limit: u8,
    ) -> Result<Vec<TaskWithAuthorLogin>, PostgresError>
    where
        for<'v> T: sqlx::Encode<'v, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        let column = match task_field {
            TaskWithAuthorIdField::AuthorLogin => "u.login".to_string(),
            _ => format!("t.{}", task_field),
        };

        let mut builder = PsqlRqBuilder::default();
        builder.order_by(&column, order);
        builder.limit(limit)?;
        if let Some(last) = &last {
            builder.r#where(&column, CompOp::GreaterOrEq, &last.1)?;
        }
        let sql = format!(
            r#"
            SELECT
                t.author_id,
                t.project_id,
                t.name,
                t.deadline,
                t.id,
                t.created_on,
                u.login AS author_login
            FROM task t
            INNER JOIN "user" u
                ON u.id = t.author_id
            {};
            "#,
            builder.build()
        );
        let tasks: Vec<TaskWithAuthorLogin> = query_as_with(&sql, builder.args)
            .fetch_all(&self.pool)
            .await?;

        if let Some(last) = &last {
            return Ok(match tasks.iter().position(|t| t.id == last.0) {
                Some(idx) => tasks[idx + 1..].to_vec(),
                None => vec![],
            });
        }
        Ok(tasks)
    }

    pub async fn session_set(&self, session_id: &str) -> Result<DateTime<Utc>, PostgresError> {
        let expires_on: DateTime<Utc> = Utc::now() + Duration::minutes(5);
        sqlx::query("insert into session (id, expires_on) values ($1, $2);")
            .bind(session_id)
            .bind(expires_on)
            .execute(&self.pool)
            .await?;
        Ok(expires_on)
    }

    pub async fn session_valid(&self, session_id: &str) -> Result<DateTime<Utc>, PostgresError> {
        let mut builder = PsqlRqBuilder::default();
        builder.r#where("id", CompOp::Equal, session_id)?;
        let sql = format!("select expires_on from session {}", builder.build());
        let expires_on: DateTime<Utc> = sqlx::query_scalar(&sql)
            .bind(session_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(expires_on)
    }

    pub async fn auth_add(&self, auth: Auth) -> Result<(), PostgresError> {
        sqlx::query("INSERT INTO auth (user_id, email, password) VALUES ($1, $2, $3)")
            .bind(auth.user_id)
            .bind(auth.email)
            .bind(auth.password)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn auth_get_by_email(&self, email: &str) -> Result<Auth, PostgresError> {
        let mut builder = PsqlRqBuilder::default();
        builder.r#where("email", CompOp::Equal, email)?;
        let sql = format!("select * from auth {};", builder.build());
        let auth_opt = query_as_with(&sql, builder.args)
            .fetch_optional(&self.pool)
            .await?;
        if let Some(auth) = auth_opt {
            return Ok(auth);
        }
        Err(PostgresError::NotFound)
    }
}
