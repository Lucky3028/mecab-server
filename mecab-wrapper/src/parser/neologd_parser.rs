use super::{MecabParser, MecabParserResult, Parser, ParserError};
use itertools::Itertools;

#[derive(derive_new::new, Debug)]
pub struct NeologdParserResult {
    /// 単語
    pub input: String,
    /// 品詞
    pub part_of_speech: String,
    /// 品詞（詳細）
    pub parts_of_speech_subtyping: Vec<String>,
    /// 活用型
    pub conjugation_type: String,
    /// 活用形
    pub conjugated_form: String,
    /// 原形
    pub original_form: String,
    /// 読み
    pub reading: String,
}

const EXPECTED_DETAILS_ELEMENTS: usize = 8;

impl From<MecabParserResult> for Option<NeologdParserResult> {
    fn from(value: MecabParserResult) -> Self {
        if value.details.len() < EXPECTED_DETAILS_ELEMENTS {
            return None;
        }

        let parts_of_speech_subtyping = (1..=3)
            .map(|idx| value.details.get(idx).unwrap().to_owned())
            .filter(|s| s != "*")
            .collect_vec();

        Some(NeologdParserResult::new(
            value.word,
            value.details.get(0).unwrap().to_owned(),
            parts_of_speech_subtyping,
            value.details.get(4).unwrap().to_owned(),
            value.details.get(5).unwrap().to_owned(),
            value.details.get(6).unwrap().to_owned(),
            value.details.get(7).unwrap().to_owned(),
        ))
    }
}

pub struct NeoglogdParser(MecabParser);

impl Parser for NeoglogdParser {
    type Parsed = NeologdParserResult;

    fn new(args: Option<String>) -> anyhow::Result<Self> {
        let neologd_dic_path_env_name = "NEOLOGD_DIC_PATH";
        let neologd_dic_path =
            std::env::var(neologd_dic_path_env_name).map_err(|_| ParserError::EnvVarIsNotSet {
                key: neologd_dic_path_env_name.to_string(),
            })?;

        Ok(Self(MecabParser::with_custom_dic(neologd_dic_path, args)?))
    }

    fn parse<T: ToString>(&self, input: T) -> anyhow::Result<Vec<Self::Parsed>> {
        let parsed: Vec<Option<NeologdParserResult>> =
            self.0.parse(input)?.into_iter().map(|r| r.into()).collect();
        anyhow::ensure!(
            parsed.iter().all(|r| r.is_some()),
            ParserError::ParserResultHasIllegalState {
                at_least: EXPECTED_DETAILS_ELEMENTS
            }
        );

        Ok(parsed.into_iter().flat_map(|v| v).collect())
    }
}
