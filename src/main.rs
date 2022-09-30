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
use mecab_server::{
    middleware as my_middleware,
    shared::{ApiError, ErrMsgJsonGenerator},
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(FromArgs)]
/// CLI arg
struct Arg {
    #[argh(switch, short = 'v')]
    /// whether or not to log debug
    is_verbose: bool,
}

fn init_logger(is_verbose: bool) {
    let crate_log_level = if is_verbose {
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
    init_logger(arg.is_verbose);

    let socket = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server listening on {}", socket);
    let app = Router::new()
        .route("/parse", post(parse))
        .fallback(fallback.into_service())
        .layer(middleware::from_fn(my_middleware::print_request_response));

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
