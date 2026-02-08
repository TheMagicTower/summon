mod config;
mod proxy;

use config::Config;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use http_body_util::Full;
use bytes::Bytes;

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
    todo!()
}
