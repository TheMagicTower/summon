use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, Response, StatusCode};

use crate::AppState;

/// 모든 요청을 처리하는 프록시 핸들러
/// - POST /v1/messages → 모델 기반 라우팅
/// - 그 외 → Anthropic API 패스스루
pub async fn proxy_handler(
    State(state): State<AppState>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    todo!()
}
