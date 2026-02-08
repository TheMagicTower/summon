use regex::Regex;
use serde::Deserialize;
use std::fs;

/// 서버 바인딩 설정
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// 기본 업스트림 (Anthropic API)
#[derive(Debug, Deserialize, Clone)]
pub struct DefaultConfig {
    pub url: String,
}

/// 인증 헤더 설정
#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub header: String,
    pub value: String,
}

/// 업스트림 제공자 설정
#[derive(Debug, Deserialize, Clone)]
pub struct UpstreamConfig {
    pub url: String,
    pub auth: AuthConfig,
}

/// 라우팅 규칙
#[derive(Debug, Deserialize, Clone)]
pub struct RouteConfig {
    /// 모델명 부분 문자열 매칭 패턴
    #[serde(rename = "match")]
    pub match_pattern: String,
    pub upstream: UpstreamConfig,
}

/// 최상위 설정
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub default: DefaultConfig,
    pub routes: Vec<RouteConfig>,
}

/// 환경변수 치환: `${VAR_NAME}` → 실제 값
pub fn resolve_env(raw: &str) -> String {
    todo!()
}

impl Config {
    /// YAML 파일에서 설정 로드 (환경변수 치환 포함)
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        todo!()
    }

    /// 모델명으로 라우트 검색 (첫 번째 매칭 반환)
    pub fn find_route(&self, model: &str) -> Option<&RouteConfig> {
        todo!()
    }
}
