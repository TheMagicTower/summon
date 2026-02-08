use axum::body::Body;
use axum::extract::State;
use axum::http::uri::PathAndQuery;
use axum::http::{Method, Request, Response, StatusCode};
use bytes::Bytes;
use http_body_util::Full;

use crate::AppState;
use crate::config::RouteConfig;

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

/// 모든 요청을 처리하는 프록시 핸들러
/// - POST /v1/messages → 모델 기반 라우팅
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
        return forward(&state, parts, bytes, None).await;
    }

    let model = extract_model(&bytes)?;
    let route = state.config.find_route(&model);

    tracing::info!(model = %model, routed = route.is_some(), "라우팅 결정");

    forward(&state, parts, bytes, route).await
}

/// 업스트림으로 요청 포워딩
/// - route가 Some이면 해당 제공자로 라우팅 (인증 헤더 교체)
/// - route가 None이면 기본 Anthropic API로 패스스루
async fn forward(
    state: &AppState,
    parts: axum::http::request::Parts,
    body_bytes: Bytes,
    route: Option<&RouteConfig>,
) -> Result<Response<Body>, StatusCode> {
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
        .method(parts.method)
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
        builder = builder.header(r.upstream.auth.header.as_str(), r.upstream.auth.value.as_str());
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
