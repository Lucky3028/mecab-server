mod mecab_parser;
pub use mecab_parser::*;

mod neologd_parser;
pub use neologd_parser::*;

pub trait Parser {
    type Parsed;

    fn new(args: Option<String>) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn parse<T: ToString>(&self, input: T) -> anyhow::Result<Vec<Self::Parsed>>;
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ParserError {
    #[error("The path to dictionary doesn't exist")]
    DictionaryIsNotFound,
    #[error("The string of the path to dictionary must be valid unicode")]
    DictionaryPathMustBeEncodedWithUnicode,
    #[error("The env var ({key}) is not set")]
    EnvVarIsNotSet { key: String },
    #[error("The parser result must have {at_least} detail elements at least")]
    ParserResultHasIllegalState { at_least: usize },
}
