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
    let re = Regex::new(r"\$\{(\w+)\}").unwrap();
    re.replace_all(raw, |caps: &regex::Captures| {
        std::env::var(&caps[1]).unwrap_or_default()
    })
    .into_owned()
}

impl Config {
    /// YAML 파일에서 설정 로드 (환경변수 치환 포함)
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let raw = fs::read_to_string(path)?;
        let resolved = resolve_env(&raw);
        let config: Config = serde_yaml::from_str(&resolved)?;
        Ok(config)
    }

    /// 모델명으로 라우트 검색 (첫 번째 매칭 반환)
    pub fn find_route(&self, model: &str) -> Option<&RouteConfig> {
        self.routes.iter().find(|r| model.contains(&r.match_pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// resolve_env: 정상 치환
    #[test]
    fn test_resolve_env_substitutes_existing_var() {
        unsafe { std::env::set_var("TEST_KEY_CONFIG", "hello") };
        assert_eq!(resolve_env("${TEST_KEY_CONFIG}"), "hello");
        unsafe { std::env::remove_var("TEST_KEY_CONFIG") };
    }

    /// resolve_env: 미존재 변수 → 빈 문자열
    #[test]
    fn test_resolve_env_missing_var_becomes_empty() {
        assert_eq!(resolve_env("${NONEXISTENT_VAR_12345}"), "");
    }

    /// resolve_env: 치환 대상 없음 → 원본 그대로
    #[test]
    fn test_resolve_env_no_placeholder_unchanged() {
        assert_eq!(resolve_env("plain text"), "plain text");
    }

    /// YAML 파싱: 임시 파일 → Config::load → 필드 검증
    #[test]
    fn test_config_load_from_yaml() {
        let yaml = r#"
server:
  host: "127.0.0.1"
  port: 8080
default:
  url: "https://api.anthropic.com"
routes:
  - match: "zai"
    upstream:
      url: "https://api.z.ai"
      auth:
        header: "Authorization"
        value: "Bearer test-key"
"#;
        let path = "/tmp/_config_test_load.yaml";
        fs::write(path, yaml).expect("임시 파일 작성 실패");

        let config = Config::load(path).expect("설정 로드 실패");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.default.url, "https://api.anthropic.com");
        assert_eq!(config.routes.len(), 1);
        assert_eq!(config.routes[0].match_pattern, "zai");
        assert_eq!(config.routes[0].upstream.url, "https://api.z.ai");

        let _ = fs::remove_file(path);
    }

    /// find_route: 매칭되는 경우
    #[test]
    fn test_find_route_matches() {
        let config = make_test_config();
        let route = config.find_route("zai-model-v1");
        assert!(route.is_some());
        assert_eq!(route.unwrap().upstream.url, "https://api.z.ai");
    }

    /// find_route: 매칭 안 되는 경우 → None
    #[test]
    fn test_find_route_no_match() {
        let config = make_test_config();
        assert!(config.find_route("claude-3-opus").is_none());
    }

    /// find_route: 순서 우선순위 (첫 번째 매칭 반환)
    #[test]
    fn test_find_route_first_match_wins() {
        let config = Config {
            server: ServerConfig {
                host: "127.0.0.1".into(),
                port: 8080,
            },
            default: DefaultConfig {
                url: "https://api.anthropic.com".into(),
            },
            routes: vec![
                RouteConfig {
                    match_pattern: "kimi".into(),
                    upstream: UpstreamConfig {
                        url: "https://first.example.com".into(),
                        auth: AuthConfig {
                            header: "Authorization".into(),
                            value: "Bearer first".into(),
                        },
                    },
                },
                RouteConfig {
                    match_pattern: "kimi".into(),
                    upstream: UpstreamConfig {
                        url: "https://second.example.com".into(),
                        auth: AuthConfig {
                            header: "Authorization".into(),
                            value: "Bearer second".into(),
                        },
                    },
                },
            ],
        };
        let route = config.find_route("kimi-latest").unwrap();
        assert_eq!(route.upstream.url, "https://first.example.com");
    }

    /// 테스트용 Config 헬퍼
    fn make_test_config() -> Config {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".into(),
                port: 8080,
            },
            default: DefaultConfig {
                url: "https://api.anthropic.com".into(),
            },
            routes: vec![RouteConfig {
                match_pattern: "zai".into(),
                upstream: UpstreamConfig {
                    url: "https://api.z.ai".into(),
                    auth: AuthConfig {
                        header: "Authorization".into(),
                        value: "Bearer test".into(),
                    },
                },
            }],
        }
    }
}
