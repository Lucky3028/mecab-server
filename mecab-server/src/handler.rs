mod parse;
pub use parse::*;

use crate::shared::ErrMsgJsonGenerator;
use axum::{
    http::{StatusCode, Uri},
    response::IntoResponse,
};

pub async fn fallback(uri: Uri) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        ErrMsgJsonGenerator::new(format!("No route for '{}'", uri)).generate(),
    )
}
