use crate::config::{AuthConfig, Config, RouteConfig, UpstreamConfig};
use dialoguer::{Confirm, Input, Select};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// 프로바이더 프리셋 정보
struct ProviderPreset {
    name: &'static str,
    match_pattern: &'static str,
    upstream_url: &'static str,
    auth_header: &'static str,
    /// API 키 → 인증 값 변환 포맷. `{}` 자리에 키가 삽입됨
    auth_value_format: &'static str,
}

const PRESETS: &[ProviderPreset] = &[
    ProviderPreset {
        name: "Z.AI (GLM 시리즈)",
        match_pattern: "glm",
        upstream_url: "https://api.z.ai/api/anthropic",
        auth_header: "x-api-key",
        auth_value_format: "{}",
    },
    ProviderPreset {
        name: "Kimi (Moonshot 시리즈)",
        match_pattern: "kimi",
        upstream_url: "https://api.kimi.com/coding",
        auth_header: "Authorization",
        auth_value_format: "Bearer {}",
    },
];

// ── 경로 헬퍼 ──

/// Claude 설정 디렉토리 (~/.claude)
fn claude_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CLAUDE_CONFIG_DIR") {
        PathBuf::from(dir)
    } else {
        dirs::home_dir()
            .expect("홈 디렉토리를 찾을 수 없습니다")
            .join(".claude")
    }
}

fn settings_json_path() -> PathBuf {
    claude_dir().join("settings.json")
}

fn settings_backup_path() -> PathBuf {
    claude_dir().join("settings.json.pre-router")
}

fn pid_file_path() -> PathBuf {
    claude_dir().join("model-router.pid")
}

fn log_file_path() -> PathBuf {
    claude_dir().join("model-router.log")
}

// ── settings.json 읽기/쓰기 ──

fn read_settings() -> Value {
    let path = settings_json_path();
    if path.exists() {
        let raw = fs::read_to_string(&path).expect("settings.json 읽기 실패");
        serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    }
}

fn write_settings(value: &Value) {
    let path = settings_json_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("디렉토리 생성 실패");
    }
    let raw = serde_json::to_string_pretty(value).expect("JSON 직렬화 실패");
    fs::write(&path, raw).expect("settings.json 저장 실패");
}

// ── PID 헬퍼 ──

/// PID 파일에서 프로세스 ID 읽기
fn read_pid() -> Option<u32> {
    let path = pid_file_path();
    if !path.exists() {
        return None;
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// 프로세스가 실행 중인지 확인 (kill -0)
fn is_process_running(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 현재 실행 파일의 절대 경로
fn current_exe_path() -> PathBuf {
    std::env::current_exe().expect("실행 파일 경로를 찾을 수 없습니다")
}

// ── 디스패치 ──

pub fn run(action: &str, config_path: &str) {
    match action {
        "enable" => enable(config_path),
        "disable" => disable(config_path),
        "start" => start(config_path),
        "stop" => stop(),
        "add" => add_route(config_path),
        "remove" => remove_route(config_path),
        "status" => status(config_path),
        "restore" => restore(),
        _ => eprintln!("알 수 없는 액션: {}", action),
    }
}

// ── start / stop ──

/// 백그라운드로 프록시 프로세스 시작
fn start(config_path: &str) {
    // 이미 실행 중인지 확인
    if let Some(pid) = read_pid() {
        if is_process_running(pid) {
            println!("이미 실행 중입니다 (PID: {})", pid);
            return;
        }
        // PID 파일은 있지만 프로세스가 없음 → 정리
        let _ = fs::remove_file(pid_file_path());
    }

    let exe = current_exe_path();
    let config_abs = fs::canonicalize(config_path).unwrap_or_else(|_| PathBuf::from(config_path));
    let log_path = log_file_path();

    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("로그 파일 열기 실패");

    let stderr_file = log_file
        .try_clone()
        .expect("로그 파일 복제 실패");

    let child = Command::new(exe)
        .args(["--config", config_abs.to_str().unwrap()])
        .stdout(log_file)
        .stderr(stderr_file)
        .spawn()
        .expect("프로세스 시작 실패");

    let pid = child.id();
    fs::write(pid_file_path(), pid.to_string()).expect("PID 파일 저장 실패");

    println!("프록시 시작됨 (PID: {})", pid);
    println!("  로그: {}", log_path.display());
}

/// 프록시 프로세스 중지
fn stop() {
    let pid = match read_pid() {
        Some(pid) => pid,
        None => {
            println!("실행 중인 프록시가 없습니다.");
            return;
        }
    };

    if !is_process_running(pid) {
        println!("프로세스가 이미 종료되었습니다 (PID: {})", pid);
        let _ = fs::remove_file(pid_file_path());
        return;
    }

    // SIGTERM 전송
    let result = Command::new("kill")
        .arg(pid.to_string())
        .output();

    match result {
        Ok(output) if output.status.success() => {
            let _ = fs::remove_file(pid_file_path());
            println!("프록시 중지됨 (PID: {})", pid);
        }
        _ => {
            eprintln!("프로세스 종료 실패 (PID: {})", pid);
        }
    }
}

// ── enable / disable ──

/// settings.json 수정 + 프로세스 시작
fn enable(config_path: &str) {
    // 1. settings.json 백업 및 수정
    let settings_path = settings_json_path();
    let backup_path = settings_backup_path();

    if settings_path.exists() && !backup_path.exists() {
        fs::copy(&settings_path, &backup_path).expect("settings.json 백업 실패");
        println!("백업 생성: {}", backup_path.display());
    }

    let mut settings = read_settings();
    let env = settings
        .as_object_mut()
        .unwrap()
        .entry("env")
        .or_insert_with(|| serde_json::json!({}));
    env.as_object_mut()
        .unwrap()
        .insert(
            "ANTHROPIC_BASE_URL".into(),
            serde_json::json!("http://127.0.0.1:18081"),
        );
    write_settings(&settings);
    println!("settings.json에 ANTHROPIC_BASE_URL 추가됨");

    // 2. 프로세스 시작
    start(config_path);

    println!("\n프록시 활성화 완료. Claude Code를 재시작하세요.");
}

/// 프로세스 중지 + settings.json 복원
fn disable(_config_path: &str) {
    // 1. 프로세스 중지
    stop();

    // 2. settings.json에서 ANTHROPIC_BASE_URL 제거
    let mut settings = read_settings();
    if let Some(env) = settings.get_mut("env").and_then(|e| e.as_object_mut()) {
        env.remove("ANTHROPIC_BASE_URL");
        if env.is_empty() {
            settings.as_object_mut().unwrap().remove("env");
        }
    }
    write_settings(&settings);

    println!("settings.json에서 ANTHROPIC_BASE_URL 제거됨");
    println!("\n프록시 비활성화 완료. Claude Code를 재시작하세요.");
}

// ── add / remove ──

fn add_route(config_path: &str) {
    let mut config = if std::path::Path::new(config_path).exists() {
        Config::load(config_path).expect("설정 파일 로드 실패")
    } else {
        Config::default_config()
    };

    let mut provider_names: Vec<String> = PRESETS.iter().map(|p| p.name.to_string()).collect();
    provider_names.push("커스텀".into());

    let selection = Select::new()
        .with_prompt("프로바이더 선택")
        .items(&provider_names)
        .default(0)
        .interact()
        .expect("선택 실패");

    let (match_pattern, upstream_url, auth_header, auth_value_format) = if selection < PRESETS.len()
    {
        let preset = &PRESETS[selection];
        let match_pattern: String = Input::new()
            .with_prompt("매칭 패턴")
            .default(preset.match_pattern.into())
            .interact_text()
            .expect("입력 실패");
        (
            match_pattern,
            preset.upstream_url.to_string(),
            preset.auth_header.to_string(),
            preset.auth_value_format.to_string(),
        )
    } else {
        let match_pattern: String = Input::new()
            .with_prompt("매칭 패턴 (모델명에 포함될 문자열)")
            .interact_text()
            .expect("입력 실패");
        let upstream_url: String = Input::new()
            .with_prompt("업스트림 URL")
            .interact_text()
            .expect("입력 실패");
        let auth_header: String = Input::new()
            .with_prompt("인증 헤더 이름")
            .default("Authorization".into())
            .interact_text()
            .expect("입력 실패");
        let auth_value_format: String = Input::new()
            .with_prompt("인증 값 포맷 ({}에 API 키 삽입)")
            .default("Bearer {}".into())
            .interact_text()
            .expect("입력 실패");
        (match_pattern, upstream_url, auth_header, auth_value_format)
    };

    let api_key: String = Input::new()
        .with_prompt("API 키")
        .interact_text()
        .expect("입력 실패");

    let auth_value = auth_value_format.replace("{}", &api_key);

    let route = RouteConfig {
        match_pattern: match_pattern.clone(),
        upstream: UpstreamConfig {
            url: upstream_url.clone(),
            auth: AuthConfig {
                auth_type: "api_key".into(),
                header: Some(auth_header),
                value: Some(auth_value),
                client_id: None,
                client_secret: None,
                refresh_token: None,
                token_url: None,
            },
        },
        transformer: None,
        model_map: None,
    };

    config.routes.push(route);
    config.save(config_path).expect("설정 파일 저장 실패");

    println!("프로바이더 추가 완료");
    println!("  매칭 패턴: {}", match_pattern);
    println!("  업스트림: {}", upstream_url);
}

fn remove_route(config_path: &str) {
    if !std::path::Path::new(config_path).exists() {
        eprintln!("설정 파일이 없습니다: {}", config_path);
        return;
    }

    let mut config = Config::load(config_path).expect("설정 파일 로드 실패");

    if config.routes.is_empty() {
        println!("등록된 라우트가 없습니다.");
        return;
    }

    let route_labels: Vec<String> = config
        .routes
        .iter()
        .enumerate()
        .map(|(i, r)| {
            format!(
                "{}. match=\"{}\" → {}",
                i + 1,
                r.match_pattern,
                r.upstream.url
            )
        })
        .collect();

    let selection = Select::new()
        .with_prompt("제거할 라우트 선택")
        .items(&route_labels)
        .interact()
        .expect("선택 실패");

    let removed = config.routes.remove(selection);
    config.save(config_path).expect("설정 파일 저장 실패");

    println!("라우트 제거 완료: match=\"{}\"", removed.match_pattern);
}

// ── status ──

fn status(config_path: &str) {
    println!("=== Claude Code Model Router 상태 ===\n");

    // 1. 프로세스 상태
    match read_pid() {
        Some(pid) if is_process_running(pid) => {
            println!("프로세스: 실행 중 (PID: {})", pid);
        }
        Some(pid) => {
            println!("프로세스: 중지됨 (PID 파일에 {} 있으나 프로세스 없음)", pid);
        }
        None => {
            println!("프로세스: 중지됨");
        }
    }

    // 2. settings.json 상태
    let settings = read_settings();
    let base_url = settings
        .get("env")
        .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
        .and_then(|v| v.as_str());

    match base_url {
        Some(url) => println!("연동: 활성 ({})", url),
        None => println!("연동: 비활성"),
    }

    // 3. 백업
    let backup = settings_backup_path();
    if backup.exists() {
        println!("백업: 존재 ({})", backup.display());
    } else {
        println!("백업: 없음");
    }

    // 4. 라우트 목록
    println!();
    if std::path::Path::new(config_path).exists() {
        match Config::load(config_path) {
            Ok(config) => {
                if config.routes.is_empty() {
                    println!("라우트: 없음");
                } else {
                    println!("라우트 ({}):", config.routes.len());
                    for (i, r) in config.routes.iter().enumerate() {
                        let transformer_str = r
                            .transformer
                            .as_deref()
                            .map(|t| format!(" [변환: {}]", t))
                            .unwrap_or_default();
                        println!(
                            "  {}. match=\"{}\" → {}{}",
                            i + 1,
                            r.match_pattern,
                            r.upstream.url,
                            transformer_str
                        );
                    }
                }
            }
            Err(e) => eprintln!("설정 파일 로드 실패: {}", e),
        }
    } else {
        println!("설정 파일 없음: {}", config_path);
    }

    // 5. 경로 정보
    println!("\n경로:");
    println!("  settings.json: {}", settings_json_path().display());
    println!("  config.yaml:   {}", config_path);
    println!("  PID 파일:      {}", pid_file_path().display());
    println!("  로그 파일:     {}", log_file_path().display());
}

// ── restore ──

fn restore() {
    let backup = settings_backup_path();
    let settings = settings_json_path();

    if !backup.exists() {
        eprintln!("백업 파일이 없습니다: {}", backup.display());
        eprintln!("  'configure enable'을 실행한 적이 없거나 이미 복원되었습니다.");
        return;
    }

    let confirmed = Confirm::new()
        .with_prompt(format!(
            "settings.json을 백업에서 복원하시겠습니까?\n  백업: {}",
            backup.display()
        ))
        .default(false)
        .interact()
        .expect("확인 실패");

    if !confirmed {
        println!("복원 취소됨");
        return;
    }

    fs::copy(&backup, &settings).expect("복원 실패");
    fs::remove_file(&backup).expect("백업 파일 삭제 실패");

    println!("복원 완료");
    println!("  settings.json이 백업에서 복원되었습니다.");
}
