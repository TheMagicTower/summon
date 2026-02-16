use axum::body::Body;
use axum::extract::State;
use axum::http::uri::PathAndQuery;
use axum::http::{Method, Request, Response, StatusCode};
use bytes::{Bytes, BytesMut};
use http_body_util::{BodyExt, Full};
use uuid::Uuid;

use std::sync::Arc;
use std::time::Duration;

use crate::config::RouteConfig;
use crate::pool::{PoolGuard, SemaphoreGuard};
use crate::transformer::{self, StreamContext, Transformer};
use crate::AppState;

/// 세마포어 대기 최대 시간 (500분 = 30,000,000ms)
const SEMAPHORE_TIMEOUT_MS: u64 = 30_000_000;

/// 인증 관련 헤더인지 확인
fn is_auth_header(name: &str) -> bool {
    name.eq_ignore_ascii_case("x-api-key") || name.eq_ignore_ascii_case("authorization")
}

/// 요청 본문에서 세션 식별자 해시 추출
///
/// system 프롬프트의 앞부분을 해시하여 동일 세션/프로젝트의 요청이
/// 동일한 API 키를 사용하도록 한다. Claude Code 세션마다 고유한
/// 프로젝트 경로, CLAUDE.md 등이 system 프롬프트에 포함되므로
/// 세션 구분이 가능하다.
fn session_hash(body: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    let prefix = serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|v| {
            match &v["system"] {
                serde_json::Value::String(s) => {
                    let end = s.len().min(512);
                    Some(s[..end].to_string())
                }
                serde_json::Value::Array(arr) => {
                    arr.first()
                        .and_then(|item| item["text"].as_str())
                        .map(|s| {
                            let end = s.len().min(512);
                            s[..end].to_string()
                        })
                }
                _ => None,
            }
        })
        .unwrap_or_default();

    let mut hasher = DefaultHasher::new();
    prefix.hash(&mut hasher);
    hasher.finish()
}

/// 응답에서 Retry-After 헤더 값을 초 단위로 파싱
/// - 숫자: 그대로 초 단위 반환
/// - 파싱 실패 또는 헤더 없음: None (호출자가 기본값 사용)
fn parse_retry_after(resp: &Response<Body>) -> Option<u64> {
    resp.headers()
        .get("retry-after")?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
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

/// JSON 본문의 model 필드를 교체
fn replace_model(bytes: &Bytes, new_model: &str) -> Result<Bytes, StatusCode> {
    let mut json: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| StatusCode::BAD_REQUEST)?;
    json["model"] = serde_json::Value::String(new_model.to_string());
    let replaced = serde_json::to_vec(&json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Bytes::from(replaced))
}

/// 폴백 시 모델명 교체 적용
fn apply_fallback_model(bytes: &Bytes, fallback: &crate::config::Fallback) -> Result<Bytes, StatusCode> {
    match fallback.model() {
        Some(model) => replace_model(bytes, model),
        None => Ok(bytes.clone()),
    }
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
        return forward(&state, &parts, bytes, None, None).await;
    }

    let model = extract_model(&bytes)?;
    let route_match = state.config.find_route(&model);

    tracing::info!(model = %model, routed = route_match.is_some(), "라우팅 결정");

    // 라우팅 대상이 아니면 패스스루
    let (route_idx, route) = match route_match {
        Some(pair) => pair,
        None => {
            return forward(&state, &parts, bytes, None, None).await;
        }
    };

    // 계정 세마포어 획득 (타임아웃 적용)
    let account_permit = match tokio::time::timeout(
        Duration::from_millis(SEMAPHORE_TIMEOUT_MS),
        state.account_semaphore.acquire(route_idx),
    )
    .await
    {
        Ok(Some(permit)) => {
            tracing::info!(route_idx, "계정 세마포어 획득, 요청 처리 시작");
            Some(permit)
        }
        Ok(None) => {
            tracing::debug!(route_idx, "계정 세마포어 제한 없음");
            None
        }
        Err(_) => {
            tracing::error!(
                route_idx,
                timeout_mins = 500,
                "계정 세마포어 대기 타임아웃"
            );

            if route.fallback.is_enabled() {
                tracing::warn!("타임아웃 발생, Anthropic API로 폴백");
                let fallback_bytes = apply_fallback_model(&bytes, &route.fallback)?;
                // 폴백은 Anthropic API로 가므로 permit 없이 전달
                return forward(&state, &parts, fallback_bytes, None, None).await;
            }

            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    // 키 풀이 있는 라우트: 429 시 다른 키로 재시도하는 루프
    if route.upstream.auth.has_pool() {
        let mut tried_keys: Vec<usize> = Vec::new();
        let sess_hash = session_hash(&bytes);

        loop {
            // 첫 시도: 세션 친화로 동일 키 재사용 (프롬프트 캐시 활용)
            // 재시도: 이미 시도한 키를 제외하고 LC로 할당
            let key_idx = if tried_keys.is_empty() {
                state.key_pool.acquire_sticky(route_idx, sess_hash)
            } else {
                state.key_pool.acquire_excluding(route_idx, &tried_keys)
            };

            match key_idx {
                Some(key_idx) => {
                    let values = route.upstream.auth.all_values();
                    let selected = &values[key_idx];
                    let guard = PoolGuard::new(state.key_pool.clone(), route_idx, key_idx);
                    tracing::debug!(route_idx, key_idx, tried = ?tried_keys, "키 풀에서 키 선택");

                    match forward(&state, &parts, bytes.clone(), Some(route), Some(selected.as_str())).await {
                        Ok(resp) if resp.status() == StatusCode::TOO_MANY_REQUESTS => {
                            let retry_after = parse_retry_after(&resp);
                            state.key_pool.set_cooldown(route_idx, key_idx, retry_after);
                            drop(guard);
                            tried_keys.push(key_idx);
                            continue;
                        }
                        Ok(resp) if resp.status().is_success() => {
                            return Ok(attach_permits(resp, account_permit, Some(guard)));
                        }
                        Ok(resp) if route.fallback.is_enabled() => {
                            tracing::warn!(
                                status = %resp.status(),
                                "외부 제공자 비성공 응답, Anthropic API로 폴백"
                            );
                            drop(guard);
                            let fallback_bytes = apply_fallback_model(&bytes, &route.fallback)?;
                            let resp = forward(&state, &parts, fallback_bytes, None, None).await?;
                            // 폴백은 Anthropic API이므로 account_permit만 전달 (guard는 이미 drop)
                            return Ok(attach_permits(resp, account_permit, None));
                        }
                        Ok(resp) => {
                            return Ok(attach_permits(resp, account_permit, Some(guard)));
                        }
                        Err(_) if route.fallback.is_enabled() => {
                            tracing::warn!("외부 제공자 연결 실패, Anthropic API로 폴백");
                            drop(guard);
                            let fallback_bytes = apply_fallback_model(&bytes, &route.fallback)?;
                            let resp = forward(&state, &parts, fallback_bytes, None, None).await?;
                            // 폴백은 Anthropic API이므로 account_permit만 전달 (guard는 이미 drop)
                            return Ok(attach_permits(resp, account_permit, None));
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                None => {
                    // 모든 키 소진 (concurrency 제한 또는 전부 429)
                    tracing::warn!(
                        model = %model,
                        tried = tried_keys.len(),
                        "사용 가능한 API 키 없음"
                    );
                    if route.fallback.is_enabled() {
                        tracing::info!("Anthropic API로 폴백");
                        let fallback_bytes = apply_fallback_model(&bytes, &route.fallback)?;
                        let resp = forward(&state, &parts, fallback_bytes, None, None).await?;
                        // 폴백은 Anthropic API이므로 account_permit만 전달
                        return Ok(attach_permits(resp, account_permit, None));
                    } else {
                        return Err(StatusCode::TOO_MANY_REQUESTS);
                    }
                }
            }
        }
    }

    // 풀이 없는 라우트: 단일 키로 시도
    match route.fallback.is_enabled() {
        true => {
            match forward(&state, &parts, bytes.clone(), Some(route), None).await {
                Ok(resp) if resp.status().is_success() => {
                    Ok(attach_permits(resp, account_permit, None))
                }
                Ok(resp) => {
                    tracing::warn!(
                        status = %resp.status(),
                        "외부 제공자 비성공 응답, Anthropic API로 폴백"
                    );
                    let fallback_bytes = apply_fallback_model(&bytes, &route.fallback)?;
                    let resp = forward(&state, &parts, fallback_bytes, None, None).await?;
                    // 폴백은 Anthropic API이므로 account_permit만 전달
                    Ok(attach_permits(resp, account_permit, None))
                }
                Err(_) => {
                    tracing::warn!("외부 제공자 연결 실패, Anthropic API로 폴백");
                    let fallback_bytes = apply_fallback_model(&bytes, &route.fallback)?;
                    let resp = forward(&state, &parts, fallback_bytes, None, None).await?;
                    // 폴백은 Anthropic API이므로 account_permit만 전달
                    Ok(attach_permits(resp, account_permit, None))
                }
            }
        }
        false => {
            let resp = forward(&state, &parts, bytes, Some(route), None).await?;
            Ok(attach_permits(resp, account_permit, None))
        }
    }
}

/// 응답 Body에 PoolGuard와 SemaphoreGuard를 부착하여 스트림 종료 시 자동 해제
fn attach_permits(
    resp: Response<Body>,
    account_permit: Option<SemaphoreGuard>,
    guard: Option<PoolGuard>,
) -> Response<Body> {
    if account_permit.is_none() && guard.is_none() {
        return resp;
    }

    let (parts, body) = resp.into_parts();
    let guarded = wrap_body_with_permits(body, account_permit, guard);
    Response::from_parts(parts, guarded)
}

/// Body를 permit들과 함께 감싸서 스트림 종료 시 자동 해제
fn wrap_body_with_permits(
    body: Body,
    account_permit: Option<SemaphoreGuard>,
    guard: Option<PoolGuard>,
) -> Body {
    let stream = async_stream::stream! {
        let _account_permit = account_permit;
        let _guard = guard;
        let mut body = body;
        loop {
            match body.frame().await {
                Some(Ok(frame)) => {
                    if let Ok(data) = frame.into_data() {
                        yield Ok::<Bytes, std::io::Error>(data);
                    }
                }
                Some(Err(e)) => {
                    tracing::error!(error = %e, "응답 스트림 읽기 오류");
                    break;
                }
                None => break,
            }
        }
        tracing::debug!("스트림 종료, permit들 자동 해제");
    };
    Body::from_stream(stream)
}

/// 업스트림으로 요청 포워딩
/// - route가 Some이고 transformer가 있으면 프로토콜 변환
/// - route가 Some이고 transformer가 없으면 라우팅만 (인증 헤더 교체)
/// - route가 None이면 기본 Anthropic API로 패스스루
/// - auth_value_override가 Some이면 풀에서 선택된 키 값 사용
async fn forward(
    state: &AppState,
    parts: &axum::http::request::Parts,
    body_bytes: Bytes,
    route: Option<&RouteConfig>,
    auth_value_override: Option<&str>,
) -> Result<Response<Body>, StatusCode> {
    // 트랜스포머 결정
    let transformer_opt: Option<Arc<dyn Transformer>> = route
        .and_then(|r| r.transformer.as_deref())
        .and_then(transformer::create_transformer)
        .map(Arc::from);

    // 트랜스포머가 있으면 변환 분기
    if let Some(ref tf) = transformer_opt {
        return forward_with_transform(state, parts, body_bytes, route.unwrap(), tf.clone(), auth_value_override).await;
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

    // 헤더 복사 (host, content-length 제외, 라우팅 시 기존 인증 헤더도 제외)
    // content-length는 hyper가 실제 body 크기로 자동 설정 (폴백 시 body 변경 대응)
    for (key, value) in parts.headers.iter() {
        if key == hyper::header::HOST || key == hyper::header::CONTENT_LENGTH {
            continue;
        }
        if route.is_some() && is_auth_header(key.as_str()) {
            continue;
        }
        builder = builder.header(key, value);
    }

    // 라우팅 시 새 인증 헤더 추가
    if let Some(r) = route {
        let value = auth_value_override.unwrap_or_else(|| r.upstream.auth.header_value());
        builder = builder.header(r.upstream.auth.header_name(), value);
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
    auth_value_override: Option<&str>,
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

    // 인증 헤더 추가 (풀 오버라이드 적용)
    let auth_value = auth_value_override.unwrap_or_else(|| route.upstream.auth.header_value());
    builder = builder.header(route.upstream.auth.header_name(), auth_value);

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
