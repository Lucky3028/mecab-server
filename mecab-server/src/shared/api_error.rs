use crate::shared::ErrMsgJsonGenerator;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mecab_wrapper::parser::ParserError;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    ParserError(#[from] ParserError),
    #[error(transparent)]
    Unknown(anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, err_msg) = match self {
            Self::ParserError(_) => {
                error!(
                    "Parser Error! Here's the error message: {:?}",
                    self.to_string()
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal error has occurred while parsing contents".to_string(),
                )
            }
            Self::Unknown(_) => {
                let err_msg = self.to_string();
                error!("Unknown Error! Here's the error message: {:?}", err_msg);
                (StatusCode::INTERNAL_SERVER_ERROR, err_msg)
            }
        };
        let body = ErrMsgJsonGenerator::new(err_msg).generate();

        (status, body).into_response()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        e.downcast::<ParserError>()
            .map(Self::ParserError)
            .unwrap_or_else(Self::Unknown)
    }
}
