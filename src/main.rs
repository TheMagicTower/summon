mod config;
mod configure;
mod proxy;
mod transformer;

use clap::{Parser, Subcommand};
use config::Config;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use http_body_util::Full;
use bytes::Bytes;
use axum::Router;
use tower_http::trace::TraceLayer;

/// 프록시 HTTP 클라이언트 타입
pub type HttpClient = Client<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, Full<Bytes>>;

/// 애플리케이션 상태 (axum에서 공유)
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub client: HttpClient,
}

#[derive(Parser)]
#[command(name = "claude-code-model-router", version)]
struct Cli {
    /// 설정 파일 경로
    #[arg(long, default_value = "config.yaml")]
    config: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 설정 관리
    Configure {
        #[command(subcommand)]
        action: ConfigureAction,
    },
}

#[derive(Subcommand)]
enum ConfigureAction {
    /// 프록시 활성화 (settings.json 수정 + 프로세스 시작)
    Enable,
    /// 프록시 비활성화 (프로세스 중지 + settings.json 복원)
    Disable,
    /// 프록시 프로세스 백그라운드 시작
    Start,
    /// 프록시 프로세스 중지
    Stop,
    /// 프로바이더 추가 (라우트 + API 키)
    Add,
    /// 프로바이더 제거
    Remove,
    /// 현재 상태 표시
    Status,
    /// settings.json 백업에서 복구
    Restore,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Configure { action }) => {
            let action_str = match action {
                ConfigureAction::Enable => "enable",
                ConfigureAction::Disable => "disable",
                ConfigureAction::Start => "start",
                ConfigureAction::Stop => "stop",
                ConfigureAction::Add => "add",
                ConfigureAction::Remove => "remove",
                ConfigureAction::Status => "status",
                ConfigureAction::Restore => "restore",
            };
            configure::run(action_str, &cli.config);
        }
        None => {
            // 기존 프록시 서버 실행
            run_server(&cli.config).await;
        }
    }
}

/// 프록시 서버 시작
async fn run_server(config_path: &str) {
    // 1. tracing 초기화
    tracing_subscriber::fmt::init();

    // 2. 설정 파일 로드
    let config = Config::load(config_path).expect("설정 파일 로드 실패");
    tracing::info!(host = %config.server.host, port = config.server.port, "설정 로드 완료");

    // 3. HTTPS 클라이언트 구축 (rustls — 순수 Rust TLS, 시스템 OpenSSL 불필요)
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("CryptoProvider 설치 실패");

    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .expect("시스템 루트 인증서 로드 실패")
        .https_or_http()
        .enable_all_versions()
        .build();
    let client: HttpClient = Client::builder(TokioExecutor::new()).build(https);

    // 4. AppState 생성 및 바인딩 주소 추출
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let state = AppState { config: config.clone(), client };

    // 5. axum 라우터 구성
    let app = Router::new()
        .fallback(proxy::proxy_handler)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // 6. 서버 시작
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("바인딩 실패");
    tracing::info!(addr = %addr, "프록시 서버 시작");
    axum::serve(listener, app).await.expect("서버 실행 실패");
}
