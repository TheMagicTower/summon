use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 서버 바인딩 설정
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// 기본 업스트림 (Anthropic API)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DefaultConfig {
    pub url: String,
}

/// 인증 헤더 설정
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    /// 인증 타입: "api_key" (기본) 또는 "oauth" (향후 v0.3+)
    #[serde(rename = "type", default = "default_auth_type", skip_serializing_if = "is_default_auth_type")]
    pub auth_type: String,

    // API Key 방식
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    // OAuth 방식 (v0.3+ — 현재는 파싱만)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_url: Option<String>,

    /// 추가 API 키 풀 (같은 header, 다른 value)
    /// Least-Connections 방식으로 분배됨
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool: Option<Vec<String>>,
}

fn default_auth_type() -> String {
    "api_key".to_string()
}

fn is_default_auth_type(s: &String) -> bool {
    s == "api_key"
}

/// 외부 제공자 실패 시 폴백 설정
/// - false: 폴백 없음
/// - true: 원본 모델명 그대로 Anthropic API로 폴백
/// - "모델명": 지정된 모델명으로 교체 후 Anthropic API로 폴백
#[derive(Debug, Clone)]
pub enum Fallback {
    /// 폴백 비활성화
    Disabled,
    /// 원본 모델명 그대로 폴백
    Passthrough,
    /// 지정된 모델명으로 교체 후 폴백
    Model(String),
}

impl Default for Fallback {
    fn default() -> Self {
        Fallback::Passthrough
    }
}

impl Fallback {
    /// 폴백이 활성화되어 있는지 확인
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Fallback::Disabled)
    }

    /// 폴백 시 교체할 모델명 (None이면 원본 유지)
    pub fn model(&self) -> Option<&str> {
        match self {
            Fallback::Model(m) => Some(m.as_str()),
            _ => None,
        }
    }
}

impl Serialize for Fallback {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Fallback::Disabled => serializer.serialize_bool(false),
            Fallback::Passthrough => serializer.serialize_bool(true),
            Fallback::Model(m) => serializer.serialize_str(m),
        }
    }
}

impl<'de> Deserialize<'de> for Fallback {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_yaml::Value::deserialize(deserializer)?;
        match value {
            serde_yaml::Value::Bool(true) => Ok(Fallback::Passthrough),
            serde_yaml::Value::Bool(false) => Ok(Fallback::Disabled),
            serde_yaml::Value::String(s) if s.is_empty() => Ok(Fallback::Disabled),
            serde_yaml::Value::String(s) => Ok(Fallback::Model(s)),
            _ => Err(serde::de::Error::custom("fallback은 bool 또는 모델명 문자열이어야 합니다")),
        }
    }
}

fn is_default_fallback(f: &Fallback) -> bool {
    matches!(f, Fallback::Passthrough)
}

impl AuthConfig {
    /// 하위 호환: header/value를 직접 반환 (api_key 방식)
    pub fn header_name(&self) -> &str {
        self.header.as_deref().unwrap_or("Authorization")
    }

    pub fn header_value(&self) -> &str {
        self.value.as_deref().unwrap_or("")
    }

    /// 키 풀이 설정되어 있는지 확인
    pub fn has_pool(&self) -> bool {
        self.pool.as_ref().is_some_and(|p| !p.is_empty())
    }

    /// 기본 value + pool을 합친 전체 키 목록
    pub fn all_values(&self) -> Vec<String> {
        let mut keys = vec![self.header_value().to_string()];
        if let Some(pool) = &self.pool {
            keys.extend(pool.iter().cloned());
        }
        keys
    }
}

/// 업스트림 제공자 설정
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpstreamConfig {
    pub url: String,
    pub auth: AuthConfig,
}

/// 라우팅 규칙
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RouteConfig {
    /// 모델명 부분 문자열 매칭 패턴
    #[serde(rename = "match")]
    pub match_pattern: String,
    pub upstream: UpstreamConfig,
    /// 트랜스포머 이름: "openai", "gemini" 등 (None이면 패스스루)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformer: Option<String>,
    /// 업스트림 모델명 (원본 모델명을 이 값으로 교체)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_map: Option<String>,
    /// 외부 제공자 실패 시 Anthropic API로 폴백
    /// - false: 폴백 없음
    /// - true: 원본 모델명 그대로 폴백 (기본값)
    /// - "모델명": 지정된 모델명으로 교체 후 폴백
    #[serde(default, skip_serializing_if = "is_default_fallback")]
    pub fallback: Fallback,
    /// API 키당 동시 요청 제한 (기본값: 제한 없음)
    /// 예: GLM-5는 키당 1개, GLM-4-Plus는 키당 20개
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<usize>,
}

/// 최상위 설정
#[derive(Debug, Deserialize, Serialize, Clone)]
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

    /// 기본 설정 생성 (config.yaml이 없을 때)
    pub fn default_config() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".into(),
                port: 18081,
            },
            default: DefaultConfig {
                url: "https://api.anthropic.com".into(),
            },
            routes: vec![],
        }
    }

    /// YAML 파일로 설정 저장
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let yaml = serde_yaml::to_string(self)?;
        fs::write(path, yaml)?;
        Ok(())
    }

    /// 모델명으로 라우트 검색 (첫 번째 매칭의 인덱스 + 참조 반환)
    pub fn find_route(&self, model: &str) -> Option<(usize, &RouteConfig)> {
        self.routes
            .iter()
            .enumerate()
            .find(|(_, r)| model.contains(&r.match_pattern))
    }

    /// XDG Base Directory 준수 설정 파일 검색
    ///
    /// 검색 우선순위:
    /// 1. SUMMON_CONFIG 환경변수
    /// 2. ~/.config/summon/config.yaml (XDG_CONFIG_HOME 또는 ~/.config)
    /// 3. /etc/summon/config.yaml (시스템 와이드)
    /// 4. ./config.yaml (현재 디렉토리)
    pub fn find_config_path() -> Option<PathBuf> {
        // 1. 환경변수 SUMMON_CONFIG 확인
        if let Ok(path) = std::env::var("SUMMON_CONFIG") {
            let p = PathBuf::from(&path);
            if p.exists() {
                return Some(p);
            }
        }

        // 2. XDG_CONFIG_HOME 또는 ~/.config/summon/config.yaml
        if let Some(home) = dirs::home_dir() {
            let xdg_config = std::env::var("XDG_CONFIG_HOME")
                .unwrap_or_else(|_| home.join(".config").to_string_lossy().to_string());
            let user_config = PathBuf::from(xdg_config).join("summon/config.yaml");
            if user_config.exists() {
                return Some(user_config);
            }
        }

        // 3. 시스템 와이드 설정
        let system_config = PathBuf::from("/etc/summon/config.yaml");
        if system_config.exists() {
            return Some(system_config);
        }

        // 4. 현재 디렉토리
        let cwd_config = PathBuf::from("config.yaml");
        if cwd_config.exists() {
            return Some(cwd_config);
        }

        None
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

    /// YAML 파싱: 기존 형식 (header/value 직접) — 하위 호환
    #[test]
    fn test_config_load_legacy_auth() {
        let yaml = r#"
server:
  host: "127.0.0.1"
  port: 18081
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
        let path = "/tmp/_config_test_legacy.yaml";
        fs::write(path, yaml).expect("임시 파일 작성 실패");

        let config = Config::load(path).expect("설정 로드 실패");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 18081);
        assert_eq!(config.default.url, "https://api.anthropic.com");
        assert_eq!(config.routes.len(), 1);
        assert_eq!(config.routes[0].match_pattern, "zai");
        assert_eq!(config.routes[0].upstream.url, "https://api.z.ai");
        assert_eq!(config.routes[0].upstream.auth.auth_type, "api_key");
        assert_eq!(config.routes[0].upstream.auth.header_name(), "Authorization");
        assert_eq!(config.routes[0].upstream.auth.header_value(), "Bearer test-key");
        assert!(config.routes[0].transformer.is_none());
        assert!(config.routes[0].model_map.is_none());

        let _ = fs::remove_file(path);
    }

    /// YAML 파싱: v0.2 형식 (transformer, model_map, auth type)
    #[test]
    fn test_config_load_v02_fields() {
        let yaml = r#"
server:
  host: "127.0.0.1"
  port: 18081
default:
  url: "https://api.anthropic.com"
routes:
  - match: "gpt"
    upstream:
      url: "https://api.openai.com"
      auth:
        type: "api_key"
        header: "Authorization"
        value: "Bearer sk-test"
    transformer: "openai"
    model_map: "gpt-4o"
  - match: "gemini"
    upstream:
      url: "https://generativelanguage.googleapis.com"
      auth:
        type: "api_key"
        header: "x-goog-api-key"
        value: "test-gemini-key"
    transformer: "gemini"
    model_map: "gemini-2.0-flash"
"#;
        let path = "/tmp/_config_test_v02.yaml";
        fs::write(path, yaml).expect("임시 파일 작성 실패");

        let config = Config::load(path).expect("설정 로드 실패");
        assert_eq!(config.routes.len(), 2);

        // OpenAI 라우트
        let openai = &config.routes[0];
        assert_eq!(openai.match_pattern, "gpt");
        assert_eq!(openai.transformer.as_deref(), Some("openai"));
        assert_eq!(openai.model_map.as_deref(), Some("gpt-4o"));
        assert_eq!(openai.upstream.auth.auth_type, "api_key");

        // Gemini 라우트
        let gemini = &config.routes[1];
        assert_eq!(gemini.match_pattern, "gemini");
        assert_eq!(gemini.transformer.as_deref(), Some("gemini"));
        assert_eq!(gemini.model_map.as_deref(), Some("gemini-2.0-flash"));
        assert_eq!(gemini.upstream.auth.header_name(), "x-goog-api-key");

        let _ = fs::remove_file(path);
    }

    /// find_route: 매칭되는 경우
    #[test]
    fn test_find_route_matches() {
        let config = make_test_config();
        let result = config.find_route("zai-model-v1");
        assert!(result.is_some());
        let (idx, route) = result.unwrap();
        assert_eq!(idx, 0);
        assert_eq!(route.upstream.url, "https://api.z.ai");
    }

    /// find_route: 매칭 안 되는 경우 → None
    #[test]
    fn test_find_route_no_match() {
        let config = make_test_config();
        assert!(config.find_route("claude-3-opus").is_none());
    }

    /// find_config_path: SUMMON_CONFIG 환경변수 우선순위
    #[test]
    fn test_find_config_path_env_var() {
        let temp_dir = std::env::temp_dir();
        let test_config = temp_dir.join("summon_test_config.yaml");
        fs::write(&test_config, "server:\n  host: 127.0.0.1\n  port: 18081\ndefault:\n  url: https://api.anthropic.com\nroutes: []").unwrap();

        unsafe { std::env::set_var("SUMMON_CONFIG", test_config.to_str().unwrap()) };
        let result = Config::find_config_path();
        unsafe { std::env::remove_var("SUMMON_CONFIG") };

        assert_eq!(result, Some(test_config.clone()));
        let _ = fs::remove_file(&test_config);
    }

    /// find_config_path: 존재하지 않는 파일은 무시
    #[test]
    fn test_find_config_path_nonexistent_env_var() {
        unsafe { std::env::set_var("SUMMON_CONFIG", "/nonexistent/path/config.yaml") };
        // 다른 경로들도 없으므로 None 또는 다른 경로가 반환됨
        let _result = Config::find_config_path();
        unsafe { std::env::remove_var("SUMMON_CONFIG") };
        // 환경변수에 지정된 경로가 없으면 다음 우선순위로 넘어감
    }

    /// find_config_path: 현재 디렉토리 config.yaml
    #[test]
    fn test_find_config_path_cwd() {
        // 현재 디렉토리에 config.yaml이 있으면 Some을 반환
        // 없으면 None을 반환
        let result = Config::find_config_path();
        let cwd_config = PathBuf::from("config.yaml");

        if cwd_config.exists() {
            assert!(result.is_some());
        } else {
            // 다른 경로도 모두 없는 경우 None
            // (테스트 환경에 따라 다름)
        }
    }

    /// find_route: 순서 우선순위 (첫 번째 매칭 반환)
    #[test]
    fn test_find_route_first_match_wins() {
        let config = Config {
            server: ServerConfig {
                host: "127.0.0.1".into(),
                port: 18081,
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
                            auth_type: "api_key".into(),
                            header: Some("Authorization".into()),
                            value: Some("Bearer first".into()),
                            client_id: None,
                            client_secret: None,
                            refresh_token: None,
                            token_url: None,
                            pool: None,
                        },
                    },
                    transformer: None,
                    model_map: None,
                    fallback: Fallback::Passthrough,
                    concurrency: None,
                },
                RouteConfig {
                    match_pattern: "kimi".into(),
                    upstream: UpstreamConfig {
                        url: "https://second.example.com".into(),
                        auth: AuthConfig {
                            auth_type: "api_key".into(),
                            header: Some("Authorization".into()),
                            value: Some("Bearer second".into()),
                            client_id: None,
                            client_secret: None,
                            refresh_token: None,
                            token_url: None,
                            pool: None,
                        },
                    },
                    transformer: None,
                    model_map: None,
                    fallback: Fallback::Passthrough,
                    concurrency: None,
                },
            ],
        };
        let (idx, route) = config.find_route("kimi-latest").unwrap();
        assert_eq!(idx, 0);
        assert_eq!(route.upstream.url, "https://first.example.com");
    }

    /// 테스트용 Config 헬퍼
    fn make_test_config() -> Config {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".into(),
                port: 18081,
            },
            default: DefaultConfig {
                url: "https://api.anthropic.com".into(),
            },
            routes: vec![RouteConfig {
                match_pattern: "zai".into(),
                upstream: UpstreamConfig {
                    url: "https://api.z.ai".into(),
                    auth: AuthConfig {
                        auth_type: "api_key".into(),
                        header: Some("Authorization".into()),
                        value: Some("Bearer test".into()),
                        client_id: None,
                        client_secret: None,
                        refresh_token: None,
                        token_url: None,
                        pool: None,
                    },
                },
                transformer: None,
                model_map: None,
                fallback: Fallback::Passthrough,
                concurrency: None,
            }],
        }
    }
}
