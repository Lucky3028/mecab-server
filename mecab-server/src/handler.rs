use crate::shared::ApiError;
use axum::{http::StatusCode, response::IntoResponse, Json};
use derive_new::new;
use itertools::Itertools;
use mecab_wrapper::parser::{NeoglogdParser, NeologdParserResult, Parser};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ParseRequest {
    texts: Vec<String>,
}

#[derive(Serialize, new)]
pub struct ParserResultResponse {
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
            res.part_of_speech,
            res.parts_of_speech_subtyping,
            res.conjugation_type,
            res.conjugated_form,
            res.original_form,
            res.reading,
        )
    }
}

#[derive(Serialize)]
pub struct ParseResponse {
    results: Vec<Vec<ParserResultResponse>>,
}

pub async fn parse(Json(parse_req): Json<ParseRequest>) -> Result<impl IntoResponse, ApiError> {
    let parser = NeoglogdParser::new(None)?;
    let results = parse_req
        .texts
        .into_iter()
        .flat_map(|s| parser.parse(s))
        .map(|v| v.into_iter().map(|r| r.into()).collect_vec())
        .collect_vec();
    let results = Json(ParseResponse { results });

    Ok((StatusCode::OK, results))
}
