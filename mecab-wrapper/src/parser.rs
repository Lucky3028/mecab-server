mod mecab_parser;
pub use mecab_parser::*;

pub trait Parser {
    type ParserResult;

    fn new(args: Option<String>) -> anyhow::Result<Self>;
    fn parse<T: ToString>(&self, input: T) -> Vec<Self::ParserResult>;
}
