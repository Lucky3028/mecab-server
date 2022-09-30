use crate::shared::ErrMsgJsonGenerator;
use axum::{
    headers::ContentType,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0:?}")]
    Unknown(anyhow::Error),
    #[error("Content-Type must be '{0}'")]
    UnexpectedContentType(ContentType),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, err_msg) = match self {
            Self::Unknown(ref e) => {
                info!("Unknown Error! Here's stacktrace: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            Self::UnexpectedContentType(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };
        let body = ErrMsgJsonGenerator::new(err_msg).generate();

        (status, body).into_response()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(inner: anyhow::Error) -> Self {
        ApiError::Unknown(inner)
    }
}
