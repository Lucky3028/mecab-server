mod err_msg_json_gen;
pub use err_msg_json_gen::ErrMsgJsonGenerator;

mod api_error;
pub use api_error::ApiError;

pub type ErrResponse = (axum::http::StatusCode, axum::Json<serde_json::Value>);
