mod mecab_parser;
pub use mecab_parser::*;

pub trait Parser {
    type ParserResult;

    fn new(args: Option<String>) -> anyhow::Result<Self>;
    fn parse<T: ToString>(&self, input: T) -> Vec<Self::ParserResult>;
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParserError {
    #[error("The path to dictionary doesn't exist")]
    DictionaryIsNotFound,
    #[error("The string of the path to dictionary must be valid unicode")]
    DictionaryPathMustBeEncodedWithUnicode,
    #[error("The env var is not set: {key}")]
    EnvVarIsNotSet { key: String },
    #[error("The parser result must have {at_least} detail elements at least")]
    ParserResultHasIllegalState { at_least: usize },
}
