use serde_json::{json, Value};
use uuid::Uuid;

use super::{StreamContext, TransformError, TransformedRequest, Transformer};

pub struct GeminiTransformer;

/// Anthropic role → Gemini role
fn map_role(role: &str) -> &str {
    match role {
        "assistant" => "model",
        other => other,
    }
}

/// Anthropic content → Gemini parts
fn to_parts(content: &Value) -> Value {
    match content {
        Value::String(s) => json!([{"text": s}]),
        Value::Array(arr) => {
            let parts: Vec<Value> = arr
                .iter()
                .filter_map(|block| {
                    block["text"].as_str().map(|t| json!({"text": t}))
                })
                .collect();
            json!(parts)
        }
        _ => json!([{"text": ""}]),
    }
}

impl Transformer for GeminiTransformer {
    fn transform_request(
        &self,
        body: Value,
        model_map: Option<&str>,
        is_stream: bool,
    ) -> Result<TransformedRequest, TransformError> {
        let obj = body.as_object().ok_or_else(|| {
            TransformError("요청 본문이 JSON 객체가 아닙니다".into())
        })?;

        // 모델 결정 (model_map 우선, 없으면 원본)
        let model = model_map
            .map(|s| s.to_string())
            .or_else(|| obj.get("model").and_then(|v| v.as_str()).map(String::from))
            .unwrap_or_else(|| "gemini-2.0-flash".to_string());

        // 경로 결정
        let path = if is_stream {
            format!("/v1beta/models/{}:streamGenerateContent?alt=sse", model)
        } else {
            format!("/v1beta/models/{}:generateContent", model)
        };

        // messages → contents
        let contents: Vec<Value> = obj
            .get("messages")
            .and_then(|m| m.as_array())
            .map(|msgs| {
                msgs.iter()
                    .filter(|m| m["role"].as_str() != Some("system"))
                    .map(|m| {
                        json!({
                            "role": map_role(m["role"].as_str().unwrap_or("user")),
                            "parts": to_parts(&m["content"]),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut gemini_body = json!({"contents": contents});

        // system → systemInstruction
        if let Some(system) = obj.get("system") {
            let system_text = match system {
                Value::String(s) => s.clone(),
                Value::Array(arr) => arr
                    .iter()
                    .filter_map(|b| b["text"].as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
                _ => String::new(),
            };
            if !system_text.is_empty() {
                gemini_body["systemInstruction"] = json!({
                    "parts": [{"text": system_text}]
                });
            }
        }

        // generationConfig
        let mut gen_config = json!({});
        if let Some(max) = obj.get("max_tokens") {
            gen_config["maxOutputTokens"] = max.clone();
        }
        if let Some(temp) = obj.get("temperature") {
            gen_config["temperature"] = temp.clone();
        }
        if let Some(top_p) = obj.get("top_p") {
            gen_config["topP"] = top_p.clone();
        }
        if let Some(stop) = obj.get("stop_sequences") {
            gen_config["stopSequences"] = stop.clone();
        }
        if gen_config.as_object().map_or(false, |o| !o.is_empty()) {
            gemini_body["generationConfig"] = gen_config;
        }

        Ok(TransformedRequest {
            path,
            body: gemini_body,
            extra_headers: vec![],
        })
    }

    fn transform_response(
        &self,
        body: Value,
        model: &str,
    ) -> Result<Value, TransformError> {
        let text = body["candidates"]
            .get(0)
            .and_then(|c| c["content"]["parts"].get(0))
            .and_then(|p| p["text"].as_str())
            .unwrap_or("");

        let finish = body["candidates"]
            .get(0)
            .and_then(|c| c["finishReason"].as_str())
            .unwrap_or("STOP");

        let stop_reason = match finish {
            "STOP" => "end_turn",
            "MAX_TOKENS" => "max_tokens",
            other => other,
        };

        let input_tokens = body["usageMetadata"]["promptTokenCount"]
            .as_u64()
            .unwrap_or(0);
        let output_tokens = body["usageMetadata"]["candidatesTokenCount"]
            .as_u64()
            .unwrap_or(0);

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
        let data: Value = serde_json::from_str(chunk)
            .map_err(|e| TransformError(format!("SSE JSON 파싱 실패: {e}")))?;

        let mut events = Vec::new();

        // 텍스트 추출
        let text = data["candidates"]
            .get(0)
            .and_then(|c| c["content"]["parts"].get(0))
            .and_then(|p| p["text"].as_str())
            .unwrap_or("");

        // usage 추적
        if let Some(usage) = data.get("usageMetadata") {
            if let Some(pt) = usage["promptTokenCount"].as_u64() {
                ctx.input_tokens = pt as u32;
            }
            if let Some(ct) = usage["candidatesTokenCount"].as_u64() {
                ctx.output_tokens = ct as u32;
            }
        }

        // 첫 번째 청크: content_block_start
        if !ctx.started {
            ctx.started = true;
            let block_start = json!({
                "type": "content_block_start",
                "index": ctx.block_index,
                "content_block": {"type": "text", "text": ""}
            });
            events.push(format!(
                "event: content_block_start\ndata: {}\n\n",
                block_start
            ));
        }

        // 텍스트 델타
        if !text.is_empty() {
            let delta = json!({
                "type": "content_block_delta",
                "index": ctx.block_index,
                "delta": {"type": "text_delta", "text": text}
            });
            events.push(format!("event: content_block_delta\ndata: {}\n\n", delta));
        }

        // finishReason 확인 → 종료 이벤트
        let finish = data["candidates"]
            .get(0)
            .and_then(|c| c["finishReason"].as_str());

        if let Some(reason) = finish {
            let stop_reason = match reason {
                "STOP" => "end_turn",
                "MAX_TOKENS" => "max_tokens",
                other => other,
            };

            let block_stop = json!({
                "type": "content_block_stop",
                "index": ctx.block_index,
            });
            events.push(format!(
                "event: content_block_stop\ndata: {}\n\n",
                block_stop
            ));

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

    fn make_transformer() -> GeminiTransformer {
        GeminiTransformer
    }

    fn make_ctx() -> StreamContext {
        StreamContext {
            model: "gemini-2.0-flash".into(),
            message_id: "msg_test456".into(),
            input_tokens: 0,
            output_tokens: 0,
            block_index: 0,
            started: false,
        }
    }

    /// 요청 변환: contents 구조, systemInstruction, URL 경로
    #[test]
    fn test_transform_request_basic() {
        let t = make_transformer();
        let body = json!({
            "model": "gemini-2.0-flash",
            "system": "도움이 되는 어시스턴트.",
            "max_tokens": 100,
            "messages": [
                {"role": "user", "content": "안녕"},
                {"role": "assistant", "content": "네, 안녕하세요!"},
                {"role": "user", "content": "잘 지내?"}
            ]
        });
        let result = t.transform_request(body, None, false).unwrap();

        assert_eq!(
            result.path,
            "/v1beta/models/gemini-2.0-flash:generateContent"
        );

        // system은 contents에 포함되지 않아야 함
        assert!(result.body.get("model").is_none());
        assert!(result.body.get("stream").is_none());

        // systemInstruction
        assert_eq!(
            result.body["systemInstruction"]["parts"][0]["text"],
            "도움이 되는 어시스턴트."
        );

        // contents
        let contents = result.body["contents"].as_array().unwrap();
        assert_eq!(contents.len(), 3);
        assert_eq!(contents[0]["role"], "user");
        assert_eq!(contents[1]["role"], "model"); // assistant → model
        assert_eq!(contents[2]["role"], "user");

        // generationConfig
        assert_eq!(result.body["generationConfig"]["maxOutputTokens"], 100);
    }

    /// 요청 변환: 스트리밍 URL
    #[test]
    fn test_transform_request_streaming_path() {
        let t = make_transformer();
        let body = json!({"model": "gemini-2.0-flash", "max_tokens": 10, "messages": []});
        let result = t.transform_request(body, None, true).unwrap();
        assert!(result.path.contains("streamGenerateContent?alt=sse"));
    }

    /// 요청 변환: model_map 적용
    #[test]
    fn test_transform_request_model_map() {
        let t = make_transformer();
        let body = json!({"model": "gemini-old", "max_tokens": 10, "messages": []});
        let result = t.transform_request(body, Some("gemini-2.0-flash"), false).unwrap();
        assert!(result.path.contains("gemini-2.0-flash"));
    }

    /// 응답 변환: Gemini → Anthropic
    #[test]
    fn test_transform_response() {
        let t = make_transformer();
        let body = json!({
            "candidates": [{
                "content": {"parts": [{"text": "안녕하세요!"}]},
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 10,
                "candidatesTokenCount": 5
            }
        });
        let result = t.transform_response(body, "gemini-2.0-flash").unwrap();

        assert_eq!(result["type"], "message");
        assert_eq!(result["role"], "assistant");
        assert_eq!(result["model"], "gemini-2.0-flash");
        assert_eq!(result["content"][0]["text"], "안녕하세요!");
        assert_eq!(result["stop_reason"], "end_turn");
        assert_eq!(result["usage"]["input_tokens"], 10);
        assert_eq!(result["usage"]["output_tokens"], 5);
    }

    /// 응답 변환: MAX_TOKENS → max_tokens
    #[test]
    fn test_transform_response_max_tokens() {
        let t = make_transformer();
        let body = json!({
            "candidates": [{
                "content": {"parts": [{"text": "..."}]},
                "finishReason": "MAX_TOKENS"
            }],
            "usageMetadata": {"promptTokenCount": 10, "candidatesTokenCount": 100}
        });
        let result = t.transform_response(body, "gemini-2.0-flash").unwrap();
        assert_eq!(result["stop_reason"], "max_tokens");
    }

    /// SSE 변환: 첫 번째 청크 → block_start + delta
    #[test]
    fn test_stream_chunk_first() {
        let t = make_transformer();
        let mut ctx = make_ctx();
        let chunk = r#"{"candidates":[{"content":{"parts":[{"text":"안녕"}]}}],"usageMetadata":{"promptTokenCount":5}}"#;
        let events = t.transform_stream_chunk(chunk, &mut ctx).unwrap();

        assert!(ctx.started);
        assert_eq!(events.len(), 2); // block_start + delta
        assert!(events[0].contains("content_block_start"));
        assert!(events[1].contains("text_delta"));
        assert!(events[1].contains("안녕"));
        assert_eq!(ctx.input_tokens, 5);
    }

    /// SSE 변환: finishReason → block_stop + message_delta
    #[test]
    fn test_stream_chunk_finish() {
        let t = make_transformer();
        let mut ctx = make_ctx();
        ctx.started = true;
        let chunk = r#"{"candidates":[{"content":{"parts":[{"text":""}]},"finishReason":"STOP"}],"usageMetadata":{"promptTokenCount":10,"candidatesTokenCount":8}}"#;
        let events = t.transform_stream_chunk(chunk, &mut ctx).unwrap();

        assert!(events.iter().any(|e| e.contains("content_block_stop")));
        assert!(events.iter().any(|e| e.contains("message_delta")));
        assert!(events.iter().any(|e| e.contains("end_turn")));
        assert_eq!(ctx.input_tokens, 10);
        assert_eq!(ctx.output_tokens, 8);
    }

    /// 스트림 시작/종료 이벤트
    #[test]
    fn test_stream_start_end_events() {
        let t = make_transformer();
        let mut ctx = make_ctx();

        let start = t.stream_start_events(&ctx);
        assert_eq!(start.len(), 1);
        assert!(start[0].contains("message_start"));
        assert!(start[0].contains("msg_test456"));

        let end = t.stream_end_events(&mut ctx);
        assert_eq!(end.len(), 1);
        assert!(end[0].contains("message_stop"));
    }
}
