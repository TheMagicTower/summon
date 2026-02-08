mod config;
mod proxy;

use config::Config;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use http_body_util::Full;
use bytes::Bytes;
use axum::Router;
use tower_http::trace::TraceLayer;

/// 프록시 HTTP 클라이언트 타입
pub type HttpClient = Client<HttpsConnector<HttpConnector>, Full<Bytes>>;

/// 애플리케이션 상태 (axum에서 공유)
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub client: HttpClient,
}

#[tokio::main]
async fn main() {
    // 1. tracing 초기화
    tracing_subscriber::fmt::init();

    // 2. CLI 인자 파싱 (--config)
    let args: Vec<String> = std::env::args().collect();
    let config_path = if let Some(pos) = args.iter().position(|a| a == "--config") {
        args.get(pos + 1).map(|s| s.as_str()).unwrap_or("config.yaml")
    } else {
        "config.yaml"
    };

    // 3. 설정 파일 로드
    let config = Config::load(config_path).expect("설정 파일 로드 실패");
    tracing::info!(host = %config.server.host, port = config.server.port, "설정 로드 완료");

    // 4. HTTPS 클라이언트 구축
    let https = HttpsConnector::new();
    let client: HttpClient = Client::builder(TokioExecutor::new()).build(https);

    // 5. AppState 생성 및 바인딩 주소 추출
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let state = AppState { config: config.clone(), client };

    // 6. axum 라우터 구성
    let app = Router::new()
        .fallback(proxy::proxy_handler)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // 7. 서버 시작
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("바인딩 실패");
    tracing::info!(addr = %addr, "프록시 서버 시작");
    axum::serve(listener, app).await.expect("서버 실행 실패");
}
