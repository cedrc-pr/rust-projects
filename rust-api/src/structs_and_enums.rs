use crate::{app_error::AppError, postgres::postgres_request_builder::Order};

#[derive(Debug, sqlx::FromRow, serde::Deserialize, serde::Serialize)]
pub struct Auth {
    pub user_id: i32,
    pub email: String,
    pub password: String,
}

#[derive(Debug, sqlx::FromRow, serde::Deserialize, serde::Serialize)]
pub struct Creds {
    pub email: String,
    pub password: String,
}

#[derive(Debug, sqlx::FromRow, Clone, serde::Serialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub login: String,
    pub created_on: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Deserialize, Debug)]
pub struct PartialUser {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub login: Option<String>,
    pub created_on: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(serde::Deserialize)]
pub struct UserOptions {
    pub name: String,
    pub login: String,
    pub email: String,
    pub password: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct UserQuery {
    pub field: Option<UserField>,
    pub limit: Option<u8>,
    pub order: Option<Order>,
    pub id: Option<i32>,
    pub name: Option<String>,
    pub login: Option<String>,
    pub created_on: Option<chrono::DateTime<chrono::Utc>>,
}

impl UserQuery {
    pub fn to_partial_user(&self) -> PartialUser {
        PartialUser {
            id: self.id,
            login: self.login.to_owned(),
            name: self.name.to_owned(),
            created_on: self.created_on,
        }
    }
}

#[derive(strum::Display, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum UserField {
    #[strum(to_string = "id")]
    Id,
    #[strum(to_string = "name")]
    Name,
    #[strum(to_string = "login")]
    Login,
    #[strum(to_string = "created_on")]
    CreatedOn,
}

impl PartialUser {
    pub fn has_id_if_field(&self, field: UserField, id: Option<i32>) -> Result<(), AppError> {
        let res = self.has_field(field);
        if res.is_ok() && id.is_none() {
            return Err(AppError::BadRequest("missing id".to_owned()));
        }
        Ok(())
    }

    pub fn has_field(&self, field: UserField) -> Result<(), AppError> {
        match field {
            UserField::Id => {
                if self.id.is_some() {
                    Ok(())
                } else {
                    Err(AppError::BadRequest("missing id".to_owned()))
                }
            }
            UserField::Login => {
                if self.login.is_some() {
                    Ok(())
                } else {
                    Err(AppError::BadRequest("missing login".to_owned()))
                }
            }
            UserField::Name => {
                if self.name.is_some() {
                    Ok(())
                } else {
                    Err(AppError::BadRequest("missing name".to_owned()))
                }
            }
            UserField::CreatedOn => {
                if self.created_on.is_some() {
                    Ok(())
                } else {
                    Err(AppError::BadRequest("missing created_on".to_owned()))
                }
            }
        }
    }
}

pub struct Project {
    pub id: i32,
    pub author_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_on: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct TaskOptions {
    pub author_id: i32,
    pub project_id: Option<i32>,
    pub name: String,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Task {
    pub id: i32,
    pub author_id: i32,
    pub project_id: Option<i32>,
    pub name: String,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub created_on: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct TaskWithAuthorLogin {
    pub id: i32,
    pub author_id: i32,
    pub project_id: Option<i32>,
    pub name: String,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub created_on: chrono::DateTime<chrono::Utc>,
    pub author_login: String,
}

#[derive(strum::Display)]
pub enum TaskWithAuthorIdField {
    #[strum(to_string = "id")]
    Id,
    #[strum(to_string = "author_id")]
    AuthorId,
    #[strum(to_string = "author_login")]
    AuthorLogin,
    #[strum(to_string = "project_id")]
    ProjectId,
    #[strum(to_string = "name")]
    Name,
    #[strum(to_string = "deadline")]
    Deadline,
    #[strum(to_string = "created_on")]
    CreatedOn,
}

#[derive(strum::Display)]
pub enum SortDirection {
    #[strum(to_string = "ASC")]
    ASC,
    #[strum(to_string = "DESC")]
    DESC,
}
