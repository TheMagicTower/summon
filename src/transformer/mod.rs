pub mod gemini;
pub mod openai;

use std::fmt;

/// 변환 오류
#[derive(Debug)]
pub struct TransformError(pub String);

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TransformError: {}", self.0)
    }
}

impl std::error::Error for TransformError {}

/// 변환된 요청 (경로 + 본문 + 추가 헤더)
pub struct TransformedRequest {
    pub path: String,
    pub body: serde_json::Value,
    pub extra_headers: Vec<(String, String)>,
}

/// SSE 스트림 상태 추적
pub struct StreamContext {
    pub model: String,
    pub message_id: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub block_index: u32,
    pub started: bool,
}

/// 프로토콜 변환 트레이트
pub trait Transformer: Send + Sync {
    /// Anthropic 요청 → 제공자 요청
    fn transform_request(
        &self,
        body: serde_json::Value,
        model_map: Option<&str>,
        is_stream: bool,
    ) -> Result<TransformedRequest, TransformError>;

    /// 제공자 비스트리밍 응답 → Anthropic 응답
    fn transform_response(
        &self,
        body: serde_json::Value,
        model: &str,
    ) -> Result<serde_json::Value, TransformError>;

    /// 제공자 SSE 청크 → Anthropic SSE 이벤트(들)
    fn transform_stream_chunk(
        &self,
        chunk: &str,
        ctx: &mut StreamContext,
    ) -> Result<Vec<String>, TransformError>;

    /// 스트림 시작 이벤트
    fn stream_start_events(&self, ctx: &StreamContext) -> Vec<String>;

    /// 스트림 종료 이벤트
    fn stream_end_events(&self, ctx: &mut StreamContext) -> Vec<String>;
}

/// 트랜스포머 팩토리
pub fn create_transformer(name: &str) -> Option<Box<dyn Transformer>> {
    match name {
        "openai" => Some(Box::new(openai::OpenAITransformer)),
        "gemini" => Some(Box::new(gemini::GeminiTransformer)),
        _ => None,
    }
}
