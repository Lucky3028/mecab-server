use axum::Json;
use serde_json::{json, Value};

pub struct ErrMsgJsonGenerator(String);

impl ErrMsgJsonGenerator {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    #[allow(dead_code)]
    pub fn from_impl(s: impl ToString) -> Self {
        Self(s.to_string())
    }

    pub fn generate(self) -> Json<Value> {
        Json(json!({ "error": self.0 }))
    }
}
