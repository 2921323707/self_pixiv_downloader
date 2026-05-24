use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::domain::R18Policy;
use crate::errors::{AppError, ErrorCode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeepSeekConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartParseInput {
    pub prompt: String,
    pub count_hint: u32,
    pub max_count: u32,
    pub r18_policy: R18Policy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SmartParsePlan {
    pub tags: Vec<String>,
    pub negative_tags: Vec<String>,
    pub count_recommend: u32,
    pub r18_policy: R18Policy,
    pub confidence: f64,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeepSeekConnectionStatus {
    pub configured: bool,
    pub status: String,
    pub model: String,
}

pub trait AiClient: Send + Sync {
    fn parse_smart_prompt(&self, input: &SmartParseInput) -> Result<SmartParsePlan, AppError>;
    fn test_connection(&self) -> Result<DeepSeekConnectionStatus, AppError>;
}

pub struct DeepSeekHttpClient {
    config: DeepSeekConfig,
    http: Client,
}

impl DeepSeekHttpClient {
    pub fn new(config: DeepSeekConfig) -> Result<Self, AppError> {
        if config.api_key.trim().is_empty() {
            return Err(AppError::new(
                ErrorCode::AiConfigMissing,
                "DeepSeek API key is required in settings or DEEPSEEK_API_KEY",
            ));
        }
        Ok(Self {
            config,
            http: Client::new(),
        })
    }
}

impl AiClient for DeepSeekHttpClient {
    fn parse_smart_prompt(&self, input: &SmartParseInput) -> Result<SmartParsePlan, AppError> {
        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );
        let request = DeepSeekChatRequest {
            model: self.config.model.clone(),
            temperature: 0.2,
            response_format: DeepSeekResponseFormat {
                response_type: "json_object".to_owned(),
            },
            messages: vec![
                DeepSeekMessage {
                    role: "system".to_owned(),
                    content: smart_parse_system_prompt(),
                },
                DeepSeekMessage {
                    role: "user".to_owned(),
                    content: smart_parse_user_prompt(input),
                },
            ],
        };

        let response = self
            .http
            .post(url)
            .bearer_auth(&self.config.api_key)
            .json(&request)
            .send()
            .map_err(|error| AppError::new(ErrorCode::AiParseFailed, error.to_string()))?;

        if !response.status().is_success() {
            return Err(AppError::new(
                ErrorCode::AiParseFailed,
                format!("DeepSeek parse failed with status {}", response.status()),
            ));
        }

        let body = response
            .json::<DeepSeekChatResponse>()
            .map_err(|error| AppError::new(ErrorCode::AiParseFailed, error.to_string()))?;
        let content = body
            .choices
            .first()
            .map(|choice| choice.message.content.as_str())
            .ok_or_else(|| {
                AppError::new(ErrorCode::AiParseFailed, "DeepSeek returned no choices")
            })?;

        parse_smart_plan_json(content, input, &self.config.model)
    }

    fn test_connection(&self) -> Result<DeepSeekConnectionStatus, AppError> {
        let url = format!("{}/models", self.config.base_url.trim_end_matches('/'));
        let response = self
            .http
            .get(url)
            .bearer_auth(&self.config.api_key)
            .send()
            .map_err(|error| AppError::new(ErrorCode::AiParseFailed, error.to_string()))?;

        if !response.status().is_success() {
            return Err(AppError::new(
                ErrorCode::AiConfigMissing,
                format!(
                    "DeepSeek connection failed with status {}",
                    response.status()
                ),
            ));
        }

        let models = response
            .json::<DeepSeekModelsResponse>()
            .map_err(|error| AppError::new(ErrorCode::AiParseFailed, error.to_string()))?;
        let found = models
            .data
            .iter()
            .any(|model| model.id == self.config.model);

        Ok(DeepSeekConnectionStatus {
            configured: true,
            status: if found {
                "ok".to_owned()
            } else {
                "model_not_listed".to_owned()
            },
            model: self.config.model.clone(),
        })
    }
}

pub fn parse_smart_plan_json(
    content: &str,
    input: &SmartParseInput,
    model: &str,
) -> Result<SmartParsePlan, AppError> {
    let raw = serde_json::from_str::<RawSmartParsePlan>(content)
        .map_err(|error| AppError::new(ErrorCode::AiParseFailed, error.to_string()))?;
    normalize_raw_plan(raw, input, model)
}

fn normalize_raw_plan(
    raw: RawSmartParsePlan,
    input: &SmartParseInput,
    model: &str,
) -> Result<SmartParsePlan, AppError> {
    let tags = normalize_tags(raw.tags.into());
    if tags.is_empty() {
        return Err(AppError::new(
            ErrorCode::AiParseFailed,
            "DeepSeek returned no tags",
        ));
    }

    let negative_tags = normalize_tags(raw.negative_tags.map(Vec::from).unwrap_or_default());
    let count_recommend = raw
        .count_recommend
        .unwrap_or(input.count_hint)
        .clamp(1, input.max_count);
    let r18_policy = input.r18_policy;
    let confidence = raw.confidence.unwrap_or(0.7).clamp(0.0, 1.0);

    Ok(SmartParsePlan {
        tags,
        negative_tags,
        count_recommend,
        r18_policy,
        confidence,
        model: model.to_owned(),
    })
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for tag in tags {
        let tag = tag.trim();
        if tag.is_empty() {
            continue;
        }
        if !normalized.iter().any(|existing: &String| existing == tag) {
            normalized.push(tag.to_owned());
        }
        if normalized.len() == 12 {
            break;
        }
    }
    normalized
}

fn smart_parse_system_prompt() -> String {
    [
        "You convert natural-language Pixiv download requests into strict JSON.",
        "Return only JSON with keys: tags, negative_tags, count_recommend, r18_policy, confidence.",
        "tags and negative_tags must be short Pixiv-searchable tags.",
        "Prefer Japanese Pixiv tags first. Use English only when it is more likely to match Pixiv search. Avoid Chinese generic terms unless no Japanese or English tag is suitable.",
        "Keep the user's R18 policy unchanged. Return the provided default r18_policy exactly.",
        "Do not include markdown or explanations.",
    ]
    .join("\n")
}

fn smart_parse_user_prompt(input: &SmartParseInput) -> String {
    format!(
        "Prompt: {}\nDefault count: {}\nMax count: {}\nDefault r18_policy: {}\nReturn strict JSON.",
        input.prompt,
        input.count_hint,
        input.max_count,
        input.r18_policy.as_str()
    )
}

#[derive(Debug, Serialize)]
struct DeepSeekChatRequest {
    model: String,
    messages: Vec<DeepSeekMessage>,
    temperature: f64,
    response_format: DeepSeekResponseFormat,
}

#[derive(Debug, Serialize)]
struct DeepSeekMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct DeepSeekResponseFormat {
    #[serde(rename = "type")]
    response_type: String,
}

#[derive(Debug, Deserialize)]
struct DeepSeekChatResponse {
    choices: Vec<DeepSeekChoice>,
}

#[derive(Debug, Deserialize)]
struct DeepSeekChoice {
    message: DeepSeekResponseMessage,
}

#[derive(Debug, Deserialize)]
struct DeepSeekResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct DeepSeekModelsResponse {
    data: Vec<DeepSeekModel>,
}

#[derive(Debug, Deserialize)]
struct DeepSeekModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct RawSmartParsePlan {
    tags: FlexibleTags,
    negative_tags: Option<FlexibleTags>,
    count_recommend: Option<u32>,
    #[serde(rename = "r18_policy")]
    _r18_policy: Option<String>,
    confidence: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FlexibleTags {
    One(String),
    Many(Vec<String>),
}

impl From<FlexibleTags> for Vec<String> {
    fn from(tags: FlexibleTags) -> Self {
        match tags {
            FlexibleTags::One(tag) => vec![tag],
            FlexibleTags::Many(tags) => tags,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ai::{SmartParseInput, parse_smart_plan_json};
    use crate::domain::R18Policy;
    use crate::errors::ErrorCode;

    #[test]
    fn req_ai_001_parses_and_normalizes_deepseek_json_plan() {
        let input = SmartParseInput {
            prompt: "blue cyberpunk girl".to_owned(),
            count_hint: 20,
            max_count: 100,
            r18_policy: R18Policy::Exclude,
        };

        let plan = parse_smart_plan_json(
            r#"{
              "tags": [" blue hair ", "cyberpunk", "girl", "cyberpunk"],
              "negative_tags": ["low quality", ""],
              "count_recommend": 120,
              "r18_policy": "include_blurred",
              "confidence": 0.82
            }"#,
            &input,
            "deepseek-v4-flash",
        )
        .unwrap();

        assert_eq!(plan.tags, vec!["blue hair", "cyberpunk", "girl"]);
        assert_eq!(plan.negative_tags, vec!["low quality"]);
        assert_eq!(plan.count_recommend, 100);
        assert_eq!(plan.r18_policy, R18Policy::Exclude);
        assert_eq!(plan.model, "deepseek-v4-flash");
    }

    #[test]
    fn req_ai_001_accepts_single_string_tags_from_deepseek() {
        let input = SmartParseInput {
            prompt: "一些白丝的图片".to_owned(),
            count_hint: 20,
            max_count: 100,
            r18_policy: R18Policy::Exclude,
        };

        let plan = parse_smart_plan_json(
            r#"{
              "tags": "白タイツ",
              "negative_tags": "low quality",
              "count_recommend": 8,
              "r18_policy": "exclude",
              "confidence": 0.74
            }"#,
            &input,
            "deepseek-v4-flash",
        )
        .unwrap();

        assert_eq!(plan.tags, vec!["白タイツ"]);
        assert_eq!(plan.negative_tags, vec!["low quality"]);
        assert_eq!(plan.count_recommend, 8);
    }

    #[test]
    fn req_ai_005_rejects_empty_deepseek_tag_plan() {
        let input = SmartParseInput {
            prompt: "anything".to_owned(),
            count_hint: 20,
            max_count: 100,
            r18_policy: R18Policy::Exclude,
        };

        let error = parse_smart_plan_json(
            r#"{"tags":[],"negative_tags":[],"confidence":0.5}"#,
            &input,
            "deepseek-v4-flash",
        )
        .unwrap_err();

        assert_eq!(error.code, ErrorCode::AiParseFailed);
    }
}
