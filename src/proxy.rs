use axum::body::Body;
use axum::extract::State;
use axum::http::uri::PathAndQuery;
use axum::http::{Method, Request, Response, StatusCode};
use bytes::{Bytes, BytesMut};
use http_body_util::{BodyExt, Full};
use uuid::Uuid;

use std::sync::Arc;

use crate::config::RouteConfig;
use crate::transformer::{self, StreamContext, Transformer};
use crate::AppState;

/// 인증 관련 헤더인지 확인
fn is_auth_header(name: &str) -> bool {
    name.eq_ignore_ascii_case("x-api-key") || name.eq_ignore_ascii_case("authorization")
}

/// JSON 본문에서 model 필드 추출
fn extract_model(bytes: &[u8]) -> Result<String, StatusCode> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| StatusCode::BAD_REQUEST)?;
    value["model"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or(StatusCode::BAD_REQUEST)
}

/// JSON 본문에서 stream 필드 추출
fn is_stream_request(bytes: &[u8]) -> bool {
    serde_json::from_slice::<serde_json::Value>(bytes)
        .ok()
        .and_then(|v| v["stream"].as_bool())
        .unwrap_or(false)
}

/// 모든 요청을 처리하는 프록시 핸들러
/// - POST /v1/messages → 모델 기반 라우팅 (+ 트랜스포머 변환)
/// - 그 외 → Anthropic API 패스스루
pub async fn proxy_handler(
    State(state): State<AppState>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let (parts, body) = req.into_parts();

    let is_messages = parts.method == Method::POST && parts.uri.path() == "/v1/messages";

    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if !is_messages {
        return forward(&state, &parts, bytes, None).await;
    }

    let model = extract_model(&bytes)?;
    let route = state.config.find_route(&model);

    tracing::info!(model = %model, routed = route.is_some(), "라우팅 결정");

    match route {
        Some(route) if route.fallback => {
            // 폴백 활성화: 외부 제공자 실패 시 Anthropic API로 재시도
            match forward(&state, &parts, bytes.clone(), Some(route)).await {
                Ok(resp) if resp.status().is_success() => Ok(resp),
                Ok(resp) => {
                    tracing::warn!(
                        status = %resp.status(),
                        "외부 제공자 비성공 응답, Anthropic API로 폴백"
                    );
                    forward(&state, &parts, bytes, None).await
                }
                Err(_) => {
                    tracing::warn!("외부 제공자 연결 실패, Anthropic API로 폴백");
                    forward(&state, &parts, bytes, None).await
                }
            }
        }
        _ => {
            // 폴백 비활성화 또는 라우팅 미매칭: 기존 동작 유지
            forward(&state, &parts, bytes, route).await
        }
    }
}

/// 업스트림으로 요청 포워딩
/// - route가 Some이고 transformer가 있으면 프로토콜 변환
/// - route가 Some이고 transformer가 없으면 라우팅만 (인증 헤더 교체)
/// - route가 None이면 기본 Anthropic API로 패스스루
async fn forward(
    state: &AppState,
    parts: &axum::http::request::Parts,
    body_bytes: Bytes,
    route: Option<&RouteConfig>,
) -> Result<Response<Body>, StatusCode> {
    // 트랜스포머 결정
    let transformer_opt: Option<Arc<dyn Transformer>> = route
        .and_then(|r| r.transformer.as_deref())
        .and_then(transformer::create_transformer)
        .map(Arc::from);

    // 트랜스포머가 있으면 변환 분기
    if let Some(ref tf) = transformer_opt {
        return forward_with_transform(state, parts, body_bytes, route.unwrap(), tf.clone()).await;
    }

    // 기존 패스스루/라우팅 로직
    let base_url = match route {
        Some(r) => &r.upstream.url,
        None => &state.config.default.url,
    };

    let path_and_query = parts
        .uri
        .path_and_query()
        .map(PathAndQuery::as_str)
        .unwrap_or("/");
    let uri_string = format!("{}{}", base_url, path_and_query);

    let mut builder = hyper::Request::builder()
        .method(parts.method.clone())
        .uri(&uri_string);

    // 헤더 복사 (host 제외, 라우팅 시 기존 인증 헤더도 제외)
    for (key, value) in parts.headers.iter() {
        if key == hyper::header::HOST {
            continue;
        }
        if route.is_some() && is_auth_header(key.as_str()) {
            continue;
        }
        builder = builder.header(key, value);
    }

    // 라우팅 시 새 인증 헤더 추가
    if let Some(r) = route {
        builder = builder.header(
            r.upstream.auth.header_name(),
            r.upstream.auth.header_value(),
        );
    }

    let req = builder
        .body(Full::new(body_bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let resp = state.client.request(req).await.map_err(|e| {
        tracing::error!(error = %e, "업스트림 요청 실패");
        StatusCode::BAD_GATEWAY
    })?;

    // hyper Incoming → axum Body 변환 (SSE 스트리밍 자동 지원)
    let (resp_parts, incoming) = resp.into_parts();
    let body = Body::new(incoming);
    Ok(Response::from_parts(resp_parts, body))
}

/// 트랜스포머를 사용하여 요청/응답 변환
async fn forward_with_transform(
    state: &AppState,
    parts: &axum::http::request::Parts,
    body_bytes: Bytes,
    route: &RouteConfig,
    transformer: Arc<dyn Transformer>,
) -> Result<Response<Body>, StatusCode> {
    // 원본 본문 파싱
    let body_json: serde_json::Value =
        serde_json::from_slice(&body_bytes).map_err(|_| StatusCode::BAD_REQUEST)?;

    let is_stream = is_stream_request(&body_bytes);
    let original_model = body_json["model"].as_str().unwrap_or("unknown").to_string();
    let upstream_model = route
        .model_map
        .as_deref()
        .unwrap_or(&original_model);

    // 요청 변환
    let transformed = transformer
        .transform_request(body_json, route.model_map.as_deref(), is_stream)
        .map_err(|e| {
            tracing::error!(error = %e, "요청 변환 실패");
            StatusCode::BAD_REQUEST
        })?;

    // URI 구축
    let uri_string = format!("{}{}", route.upstream.url, transformed.path);

    let mut builder = hyper::Request::builder()
        .method(parts.method.clone())
        .uri(&uri_string);

    // 기본 헤더: Content-Type
    builder = builder.header("Content-Type", "application/json");

    // 원본 헤더 중 필요한 것만 전달 (host, auth, content-type 제외)
    for (key, value) in parts.headers.iter() {
        if key == hyper::header::HOST
            || key == hyper::header::CONTENT_TYPE
            || key == hyper::header::CONTENT_LENGTH
            || is_auth_header(key.as_str())
        {
            continue;
        }
        builder = builder.header(key, value);
    }

    // 인증 헤더 추가
    builder = builder.header(
        route.upstream.auth.header_name(),
        route.upstream.auth.header_value(),
    );

    // 추가 헤더
    for (name, value) in &transformed.extra_headers {
        builder = builder.header(name.as_str(), value.as_str());
    }

    let req_body = serde_json::to_vec(&transformed.body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let req = builder
        .body(Full::new(Bytes::from(req_body)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!(
        uri = %uri_string,
        stream = is_stream,
        model = %upstream_model,
        "변환된 요청 전송"
    );

    let resp = state.client.request(req).await.map_err(|e| {
        tracing::error!(error = %e, "업스트림 요청 실패");
        StatusCode::BAD_GATEWAY
    })?;

    let (resp_parts, incoming) = resp.into_parts();

    if !is_stream {
        // 비스트리밍: 전체 수집 → 변환 → JSON 반환
        let resp_bytes = incoming
            .collect()
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?
            .to_bytes();

        let resp_json: serde_json::Value =
            serde_json::from_slice(&resp_bytes).map_err(|e| {
                tracing::error!(error = %e, body = %String::from_utf8_lossy(&resp_bytes), "응답 JSON 파싱 실패");
                StatusCode::BAD_GATEWAY
            })?;

        let anthropic_resp = transformer
            .transform_response(resp_json, &original_model)
            .map_err(|e| {
                tracing::error!(error = %e, "응답 변환 실패");
                StatusCode::BAD_GATEWAY
            })?;

        let resp_body = serde_json::to_vec(&anthropic_resp)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut response = Response::new(Body::from(resp_body));
        *response.status_mut() = resp_parts.status;
        response.headers_mut().insert(
            hyper::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        return Ok(response);
    }

    // 스트리밍: SSE 변환 스트림
    let ctx = StreamContext {
        model: original_model.clone(),
        message_id: format!("msg_{}", Uuid::new_v4().simple()),
        input_tokens: 0,
        output_tokens: 0,
        block_index: 0,
        started: false,
    };

    let body = transform_sse_stream(incoming, transformer, ctx);

    let mut response = Response::new(body);
    *response.status_mut() = resp_parts.status;
    response.headers_mut().insert(
        hyper::header::CONTENT_TYPE,
        "text/event-stream".parse().unwrap(),
    );
    response.headers_mut().insert(
        hyper::header::CACHE_CONTROL,
        "no-cache".parse().unwrap(),
    );

    Ok(response)
}

/// 업스트림 SSE 스트림을 Anthropic SSE 형식으로 변환
fn transform_sse_stream(
    incoming: hyper::body::Incoming,
    transformer: Arc<dyn Transformer>,
    mut ctx: StreamContext,
) -> Body {
    let stream = async_stream::stream! {
        // 1. 스트림 시작 이벤트 전송
        for event in transformer.stream_start_events(&ctx) {
            yield Ok::<Bytes, std::io::Error>(Bytes::from(event));
        }

        // 2. incoming → 바이트 청크 → 줄 단위 SSE 파싱
        let mut buf = BytesMut::new();
        let mut body = incoming;

        loop {
            match body.frame().await {
                Some(Ok(frame)) => {
                    if let Ok(data) = frame.into_data() {
                        buf.extend_from_slice(&data);

                        // 줄 단위 처리
                        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                            let line = buf.split_to(pos + 1);
                            let line_str = String::from_utf8_lossy(&line);
                            let trimmed = line_str.trim();

                            // SSE "data: " 접두사 처리
                            if let Some(payload) = trimmed.strip_prefix("data: ").or_else(|| trimmed.strip_prefix("data:")) {
                                let payload = payload.trim();
                                if payload.is_empty() {
                                    continue;
                                }

                                match transformer.transform_stream_chunk(payload, &mut ctx) {
                                    Ok(events) => {
                                        for event in events {
                                            yield Ok(Bytes::from(event));
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!(error = %e, "SSE 청크 변환 실패, 건너뜀");
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    tracing::error!(error = %e, "SSE 스트림 읽기 오류");
                    break;
                }
                None => break,
            }
        }

        // 남은 버퍼 처리
        if !buf.is_empty() {
            let line_str = String::from_utf8_lossy(&buf);
            let trimmed = line_str.trim();
            if let Some(payload) = trimmed.strip_prefix("data: ").or_else(|| trimmed.strip_prefix("data:")) {
                let payload = payload.trim();
                if !payload.is_empty() {
                    if let Ok(events) = transformer.transform_stream_chunk(payload, &mut ctx) {
                        for event in events {
                            yield Ok(Bytes::from(event));
                        }
                    }
                }
            }
        }

        // 3. 스트림 종료 이벤트 전송
        for event in transformer.stream_end_events(&mut ctx) {
            yield Ok(Bytes::from(event));
        }
    };

    Body::from_stream(stream)
}
