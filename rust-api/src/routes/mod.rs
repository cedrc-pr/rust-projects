use poem::http::StatusCode;
use tp_sqlx::app_error::AppError;

pub mod auth;
pub mod sessions;
pub mod users;

pub struct Cookies {
    pub session_id: String,
}

impl<'a> poem::FromRequest<'a> for Cookies {
    async fn from_request(
        req: &'a poem::Request,
        _body: &mut poem::RequestBody,
    ) -> poem::Result<Self> {
        let cookie = req
            .headers()
            .get("cookie")
            .ok_or_else(|| AppError::BadRequest("missing cookie".to_string()))?;

        if let Ok(to_split) = cookie.to_str()
            && let Some((_, session_id)) = to_split.split_once("=")
        {
            return Ok(Self {
                session_id: session_id.to_string(),
            });
        }

        Err(poem::Error::from_status(StatusCode::BAD_REQUEST))
    }
}
