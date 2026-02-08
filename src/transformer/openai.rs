use serde_json::{json, Value};
use uuid::Uuid;

use super::{StreamContext, TransformError, TransformedRequest, Transformer};

pub struct OpenAITransformer;

impl Transformer for OpenAITransformer {
    fn transform_request(
        &self,
        mut body: Value,
        model_map: Option<&str>,
        _is_stream: bool,
    ) -> Result<TransformedRequest, TransformError> {
        let obj = body.as_object_mut().ok_or_else(|| {
            TransformError("요청 본문이 JSON 객체가 아닙니다".into())
        })?;

        // 모델 교체
        if let Some(mapped) = model_map {
            obj.insert("model".into(), json!(mapped));
        }

        // system 필드 → messages 첫 항목으로 이동
        if let Some(system) = obj.remove("system") {
            let system_msg = json!({"role": "system", "content": system});
            if let Some(messages) = obj.get_mut("messages").and_then(|m| m.as_array_mut()) {
                messages.insert(0, system_msg);
            }
        }

        // max_tokens → max_completion_tokens
        if let Some(max) = obj.remove("max_tokens") {
            obj.insert("max_completion_tokens".into(), max);
        }

        // stop_sequences → stop
        if let Some(stop) = obj.remove("stop_sequences") {
            obj.insert("stop".into(), stop);
        }

        // Anthropic 전용 필드 제거
        for key in &["top_k", "metadata", "anthropic_version"] {
            obj.remove(*key);
        }

        Ok(TransformedRequest {
            path: "/v1/chat/completions".to_string(),
            body,
            extra_headers: vec![],
        })
    }

    fn transform_response(
        &self,
        body: Value,
        model: &str,
    ) -> Result<Value, TransformError> {
        let text = body["choices"]
            .get(0)
            .and_then(|c| c["message"]["content"].as_str())
            .unwrap_or("");

        let finish = body["choices"]
            .get(0)
            .and_then(|c| c["finish_reason"].as_str())
            .unwrap_or("stop");

        let stop_reason = match finish {
            "stop" => "end_turn",
            "length" => "max_tokens",
            other => other,
        };

        let input_tokens = body["usage"]["prompt_tokens"].as_u64().unwrap_or(0);
        let output_tokens = body["usage"]["completion_tokens"].as_u64().unwrap_or(0);

        Ok(json!({
            "id": format!("msg_{}", Uuid::new_v4().simple()),
            "type": "message",
            "role": "assistant",
            "model": model,
            "content": [{"type": "text", "text": text}],
            "stop_reason": stop_reason,
            "stop_sequence": null,
            "usage": {
                "input_tokens": input_tokens,
                "output_tokens": output_tokens,
            }
        }))
    }

    fn stream_start_events(&self, ctx: &StreamContext) -> Vec<String> {
        let msg_start = json!({
            "type": "message_start",
            "message": {
                "id": &ctx.message_id,
                "type": "message",
                "role": "assistant",
                "model": &ctx.model,
                "content": [],
                "stop_reason": null,
                "stop_sequence": null,
                "usage": {"input_tokens": 0, "output_tokens": 0}
            }
        });
        vec![format!("event: message_start\ndata: {}\n\n", msg_start)]
    }

    fn transform_stream_chunk(
        &self,
        chunk: &str,
        ctx: &mut StreamContext,
    ) -> Result<Vec<String>, TransformError> {
        // OpenAI 종료 시그널
        if chunk.trim() == "[DONE]" {
            return Ok(vec![]);
        }

        let data: Value = serde_json::from_str(chunk)
            .map_err(|e| TransformError(format!("SSE JSON 파싱 실패: {e}")))?;

        let mut events = Vec::new();
        let choice = &data["choices"][0];

        // 첫 번째 청크: content_block_start
        if !ctx.started {
            ctx.started = true;
            let block_start = json!({
                "type": "content_block_start",
                "index": ctx.block_index,
                "content_block": {"type": "text", "text": ""}
            });
            events.push(format!("event: content_block_start\ndata: {}\n\n", block_start));
        }

        // 텍스트 델타
        if let Some(content) = choice["delta"]["content"].as_str() {
            if !content.is_empty() {
                ctx.output_tokens += 1; // 근사치
                let delta = json!({
                    "type": "content_block_delta",
                    "index": ctx.block_index,
                    "delta": {"type": "text_delta", "text": content}
                });
                events.push(format!("event: content_block_delta\ndata: {}\n\n", delta));
            }
        }

        // finish_reason이 있으면 종료 이벤트 생성
        if let Some(reason) = choice["finish_reason"].as_str() {
            let stop_reason = match reason {
                "stop" => "end_turn",
                "length" => "max_tokens",
                other => other,
            };

            // usage 정보 (존재 시)
            if let Some(usage) = data.get("usage") {
                ctx.input_tokens = usage["prompt_tokens"].as_u64().unwrap_or(0) as u32;
                ctx.output_tokens = usage["completion_tokens"].as_u64().unwrap_or(0) as u32;
            }

            let block_stop = json!({
                "type": "content_block_stop",
                "index": ctx.block_index,
            });
            events.push(format!("event: content_block_stop\ndata: {}\n\n", block_stop));

            let msg_delta = json!({
                "type": "message_delta",
                "delta": {"stop_reason": stop_reason, "stop_sequence": null},
                "usage": {"output_tokens": ctx.output_tokens}
            });
            events.push(format!("event: message_delta\ndata: {}\n\n", msg_delta));
        }

        Ok(events)
    }

    fn stream_end_events(&self, _ctx: &mut StreamContext) -> Vec<String> {
        let msg_stop = json!({"type": "message_stop"});
        vec![format!("event: message_stop\ndata: {}\n\n", msg_stop)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_transformer() -> OpenAITransformer {
        OpenAITransformer
    }

    fn make_ctx() -> StreamContext {
        StreamContext {
            model: "gpt-4o".into(),
            message_id: "msg_test123".into(),
            input_tokens: 0,
            output_tokens: 0,
            block_index: 0,
            started: false,
        }
    }

    /// 요청 변환: system → messages, max_tokens → max_completion_tokens
    #[test]
    fn test_transform_request_basic() {
        let t = make_transformer();
        let body = json!({
            "model": "gpt-4o",
            "system": "너는 도움이 되는 어시스턴트야.",
            "max_tokens": 100,
            "messages": [{"role": "user", "content": "안녕"}]
        });
        let result = t.transform_request(body, None, false).unwrap();

        assert_eq!(result.path, "/v1/chat/completions");
        assert!(result.body.get("system").is_none());
        assert!(result.body.get("max_tokens").is_none());
        assert_eq!(result.body["max_completion_tokens"], 100);

        let msgs = result.body["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(msgs[1]["role"], "user");
    }

    /// 요청 변환: model_map 적용
    #[test]
    fn test_transform_request_model_map() {
        let t = make_transformer();
        let body = json!({"model": "gpt-old", "max_tokens": 10, "messages": []});
        let result = t.transform_request(body, Some("gpt-4o"), false).unwrap();
        assert_eq!(result.body["model"], "gpt-4o");
    }

    /// 요청 변환: Anthropic 전용 필드 제거
    #[test]
    fn test_transform_request_removes_anthropic_fields() {
        let t = make_transformer();
        let body = json!({
            "model": "gpt-4o",
            "max_tokens": 10,
            "top_k": 5,
            "metadata": {"user_id": "test"},
            "messages": []
        });
        let result = t.transform_request(body, None, false).unwrap();
        assert!(result.body.get("top_k").is_none());
        assert!(result.body.get("metadata").is_none());
    }

    /// 요청 변환: stop_sequences → stop
    #[test]
    fn test_transform_request_stop_sequences() {
        let t = make_transformer();
        let body = json!({
            "model": "gpt-4o",
            "max_tokens": 10,
            "stop_sequences": ["END"],
            "messages": []
        });
        let result = t.transform_request(body, None, false).unwrap();
        assert!(result.body.get("stop_sequences").is_none());
        assert_eq!(result.body["stop"][0], "END");
    }

    /// 응답 변환: OpenAI → Anthropic
    #[test]
    fn test_transform_response() {
        let t = make_transformer();
        let body = json!({
            "choices": [{"message": {"content": "안녕하세요!"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5}
        });
        let result = t.transform_response(body, "gpt-4o").unwrap();

        assert_eq!(result["type"], "message");
        assert_eq!(result["role"], "assistant");
        assert_eq!(result["model"], "gpt-4o");
        assert_eq!(result["content"][0]["type"], "text");
        assert_eq!(result["content"][0]["text"], "안녕하세요!");
        assert_eq!(result["stop_reason"], "end_turn");
        assert_eq!(result["usage"]["input_tokens"], 10);
        assert_eq!(result["usage"]["output_tokens"], 5);
    }

    /// 응답 변환: length → max_tokens
    #[test]
    fn test_transform_response_max_tokens() {
        let t = make_transformer();
        let body = json!({
            "choices": [{"message": {"content": "..."}, "finish_reason": "length"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 100}
        });
        let result = t.transform_response(body, "gpt-4o").unwrap();
        assert_eq!(result["stop_reason"], "max_tokens");
    }

    /// SSE 변환: 첫 번째 청크 → content_block_start + delta
    #[test]
    fn test_stream_chunk_first() {
        let t = make_transformer();
        let mut ctx = make_ctx();
        let chunk = r#"{"choices":[{"delta":{"content":"안녕"},"finish_reason":null}]}"#;
        let events = t.transform_stream_chunk(chunk, &mut ctx).unwrap();

        assert!(ctx.started);
        assert_eq!(events.len(), 2); // block_start + delta
        assert!(events[0].contains("content_block_start"));
        assert!(events[1].contains("text_delta"));
        assert!(events[1].contains("안녕"));
    }

    /// SSE 변환: [DONE] → 빈 벡터
    #[test]
    fn test_stream_chunk_done() {
        let t = make_transformer();
        let mut ctx = make_ctx();
        let events = t.transform_stream_chunk("[DONE]", &mut ctx).unwrap();
        assert!(events.is_empty());
    }

    /// SSE 변환: finish_reason → block_stop + message_delta
    #[test]
    fn test_stream_chunk_finish() {
        let t = make_transformer();
        let mut ctx = make_ctx();
        ctx.started = true;
        let chunk = r#"{"choices":[{"delta":{},"finish_reason":"stop"}],"usage":{"prompt_tokens":10,"completion_tokens":5}}"#;
        let events = t.transform_stream_chunk(chunk, &mut ctx).unwrap();

        assert!(events.iter().any(|e| e.contains("content_block_stop")));
        assert!(events.iter().any(|e| e.contains("message_delta")));
        assert!(events.iter().any(|e| e.contains("end_turn")));
        assert_eq!(ctx.input_tokens, 10);
        assert_eq!(ctx.output_tokens, 5);
    }

    /// 스트림 시작/종료 이벤트
    #[test]
    fn test_stream_start_end_events() {
        let t = make_transformer();
        let mut ctx = make_ctx();

        let start = t.stream_start_events(&ctx);
        assert_eq!(start.len(), 1);
        assert!(start[0].contains("message_start"));
        assert!(start[0].contains("msg_test123"));

        let end = t.stream_end_events(&mut ctx);
        assert_eq!(end.len(), 1);
        assert!(end[0].contains("message_stop"));
    }
}
