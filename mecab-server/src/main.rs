use argh::FromArgs;
use axum::{
    handler::Handler,
    http::{StatusCode, Uri},
    middleware,
    response::IntoResponse,
    routing::post,
    Router,
};
use log::LevelFilter;
use mecab_server::{handler, middleware as my_middleware, shared::ErrMsgJsonGenerator};
use std::net::SocketAddr;
use tracing::info;
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
            crate_log_level
        )))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() {
    let arg: Arg = argh::from_env();
    init_logger(arg.is_verbose);

    let socket = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server listening on {}", socket);
    let app = Router::new()
        .route("/parse", post(handler::parse))
        .fallback(handler::fallback.into_service())
        .layer(middleware::from_fn(my_middleware::print_request_response));

    axum::Server::bind(&socket)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
