use argh::FromArgs;
use axum::{
    body::{Body, Bytes},
    handler::Handler,
    headers::ContentType,
    http::{Request, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router, TypedHeader,
};
use log::LevelFilter;
use serde_json::{json, Value};
use std::net::SocketAddr;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type ErrResponse = (StatusCode, Json<Value>);

#[derive(FromArgs)]
/// CLI arg
struct Arg {
    #[argh(switch, short = 'v')]
    /// whether or not to log debug
    is_verbose: bool,
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("{0:?}")]
    Unknown(anyhow::Error),
    #[error("Content-Type must be '{0}'")]
    UnexpectedContentType(ContentType),
}

struct ErrMsgJsonGenerator(String);

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

async fn parse(
    TypedHeader(content_type): TypedHeader<ContentType>,
) -> Result<impl IntoResponse, ApiError> {
    let expected_content_type = ContentType::json();
    if content_type != expected_content_type {
        return Err(ApiError::UnexpectedContentType(expected_content_type));
    }

    // TODO: run mecab
    Ok((StatusCode::OK, "OK"))
}

#[tokio::main]
async fn main() {
    let arg: Arg = argh::from_env();
    let crate_log_level = if arg.is_verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(format!(
            "mecab_server={}",
            crate_log_level.to_string()
        )))
        .with(tracing_subscriber::fmt::layer())
        .init();
    debug!("Logging with debug level.");

    let socket = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server listening on {}", socket);
    let app = Router::new()
        .route("/parse", post(parse))
        .fallback(fallback.into_service())
        .layer(middleware::from_fn(print_request_response));

    axum::Server::bind(&socket)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn fallback(uri: Uri) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        ErrMsgJsonGenerator::new(format!("No route for '{}'", uri)).generate(),
    )
}

async fn print_request_response(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<impl IntoResponse, ErrResponse> {
    let (parts, body) = req.into_parts();
    info!("Request: {} '{}'", parts.method, parts.uri);
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    info!("Response: {}", parts.status);
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, ErrResponse>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match hyper::body::to_bytes(body).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                ErrMsgJsonGenerator::new(format!("failed to read {} body: {}", direction, err))
                    .generate(),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        debug!("{} body = {:?}", direction, body);
    }

    Ok(bytes)
}
