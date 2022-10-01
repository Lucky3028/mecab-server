use crate::parser::Parser;
use itertools::Itertools;
use mecab::Tagger;
use std::path::PathBuf;

#[derive(derive_new::new, Debug, Eq, PartialEq)]
pub struct MecabParserResult {
    pub word: String,
    pub details: Vec<String>,
}

pub struct MecabParser(Tagger);

impl Parser for MecabParser {
    type ParserResult = MecabParserResult;

    fn new(args: Option<String>) -> Self {
        Self(Tagger::new(args.unwrap_or_default()))
    }

    fn parse<T: ToString>(&self, input: T) -> Vec<Self::ParserResult> {
        // 形態素解析
        let parsed = self.0.parse_str(input.to_string());
        // 各単語ごとに分割
        let parsed = parsed
            .split("\n")
            .collect_vec()
            .into_iter()
            // 「EOS」と改行が含まれているので排除
            .dropping_back(2);

        parsed
            // 単語と形態の配列に分離
            .map(|s| s.split("\t").collect_vec())
            .flat_map(|v| v.split_first().map(|t| (t.0.to_owned(), t.1.to_owned())))
            .map(|(word, details)| {
                MecabParserResult::new(
                    word.to_string(),
                    details
                        .into_iter()
                        .flat_map(|s| s.split(",").collect_vec())
                        .map(|s| s.to_string())
                        .collect_vec(),
                )
            })
            .collect_vec()
    }
}

impl MecabParser {
    pub fn with_custom_dic<T: Into<PathBuf>>(
        dic_path: T,
        other_args: Option<String>,
    ) -> anyhow::Result<Self> {
        let dic_path = dic_path.into();
        anyhow::ensure!(dic_path.exists(), "The path to dictionary doesn't exist");
        let dic_path = dic_path.to_str();
        anyhow::ensure!(
            dic_path.is_some(),
            "The string of the path to dictionary must be valid unicode"
        );

        let dic_path = format!("-d {}", dic_path.unwrap());
        let args = other_args
            .map(|s| format!("{} {}", dic_path, s))
            .unwrap_or(dic_path);

        Ok(Self::new(Some(args)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_parser_with_illegal_dic_path_should_return_err() {
        let res = MecabParser::with_custom_dic("The dir path doesn't exist", None);
        assert!(res.is_err());
    }

    #[test]
    fn parse_empty_string_should_return_empty_vec() {
        let parser = MecabParser::new(None);
        assert!(parser.parse("").is_empty())
    }

    #[test]
    fn parse_string_should_return_result() {
        // NOTE: Mecab default dictionary should be IPADIC, so the expected result format is IPADIC.
        let parser = MecabParser::new(None);
        let expected = MecabParserResult::new(
            "あ".to_string(),
            vec!["フィラー", "*", "*", "*", "*", "*", "あ", "ア", "ア"]
                .into_iter()
                .map(|s| s.to_string())
                .collect_vec(),
        );
        assert_eq!(parser.parse("あ"), vec![expected]);
    }
}
