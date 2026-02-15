mod config;
mod configure;
mod pool;
mod proxy;
mod transformer;
mod update;

use clap::{Parser, Subcommand};
use std::sync::Arc;

use config::Config;
use pool::{KeyPool, AccountSemaphore};
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
    pub key_pool: Arc<KeyPool>,
    pub account_semaphore: Arc<AccountSemaphore>,
}

#[derive(Parser)]
#[command(name = "claude-code-model-router", version)]
struct Cli {
    /// 설정 파일 경로 (지정하지 않으면 자동 검색)
    #[arg(long)]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
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
    /// 대화형 설정 메뉴
    Configure,
    /// 최신 버전으로 업데이트
    Update,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // 설정 파일 경로 해결
    let config_path = match cli.config {
        Some(path) => path,
        None => {
            match Config::find_config_path() {
                Some(path) => path.to_string_lossy().to_string(),
                None => {
                    eprintln!("설정 파일을 찾을 수 없습니다.");
                    eprintln!("다음 위치 중 하나에 config.yaml을 배치하세요:");
                    eprintln!("  - ~/.config/summon/config.yaml");
                    eprintln!("  - /etc/summon/config.yaml");
                    eprintln!("  - ./config.yaml");
                    eprintln!("또는 --config <경로>로 지정하세요.");
                    std::process::exit(1);
                }
            }
        }
    };

    match cli.command {
        Some(Commands::Enable) => configure::run("enable", &config_path),
        Some(Commands::Disable) => configure::run("disable", &config_path),
        Some(Commands::Start) => configure::run("start", &config_path),
        Some(Commands::Stop) => configure::run("stop", &config_path),
        Some(Commands::Add) => configure::run("add", &config_path),
        Some(Commands::Remove) => configure::run("remove", &config_path),
        Some(Commands::Status) => configure::run("status", &config_path),
        Some(Commands::Restore) => configure::run("restore", &config_path),
        Some(Commands::Configure) => configure::interactive_menu(&config_path),
        Some(Commands::Update) => update::run(),
        None => {
            // 기존 프록시 서버 실행
            run_server(&config_path).await;
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

    // 4. 키 풀 초기화 및 AppState 생성
    let key_pool = Arc::new(KeyPool::from_config(&config));
    let account_semaphore = Arc::new(AccountSemaphore::from_config(&config));
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let state = AppState { config: config.clone(), client, key_pool, account_semaphore };

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
