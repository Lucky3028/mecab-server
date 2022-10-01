use argh::FromArgs;
use axum::{
    handler::Handler,
    headers::ContentType,
    http::{StatusCode, Uri},
    middleware,
    response::IntoResponse,
    routing::post,
    Json, Router, TypedHeader,
};
use derive_new::new;
use itertools::Itertools;
use log::LevelFilter;
use mecab_server::{
    middleware as my_middleware,
    shared::{ApiError, ErrMsgJsonGenerator},
};
use mecab_wrapper::parser::{NeoglogdParser, NeologdParserResult, Parser};
use serde::{Deserialize, Serialize};
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

#[derive(Deserialize)]
struct ParseRequest {
    texts: Vec<String>,
}

#[derive(Serialize, new)]
struct ParserResultResponse {
    pub input: String,
    pub part_of_speech: String,
    pub parts_of_speech_subtyping: Vec<String>,
    pub conjugation_type: String,
    pub conjugated_form: String,
    pub original_form: String,
    pub reading: String,
}

impl From<NeologdParserResult> for ParserResultResponse {
    fn from(res: NeologdParserResult) -> Self {
        Self::new(
            res.input,
            res.part_of_speech.unwrap_or_default(),
            res.parts_of_speech_subtyping,
            res.conjugation_type.unwrap_or_default(),
            res.conjugated_form.unwrap_or_default(),
            res.original_form.unwrap_or_default(),
            res.reading.unwrap_or_default(),
        )
    }
}

#[derive(Serialize)]
struct ParseResponse {
    results: Vec<ParserResultResponse>,
}

async fn parse(Json(parse_req): Json<ParseRequest>) -> Result<impl IntoResponse, ApiError> {
    let parser = NeoglogdParser::new(None)?;
    let results = parse_req
        .texts
        .into_iter()
        .flat_map(|s| parser.parse(s))
        .flat_map(|v| v.into_iter().map(|r| r.into()).collect_vec())
        .collect_vec();
    let results = Json(ParseResponse { results });

    Ok((StatusCode::OK, results))
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
