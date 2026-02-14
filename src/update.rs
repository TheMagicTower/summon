use dialoguer::Confirm;
use std::process::Command;

/// 현재 버전 (Cargo.toml에서 가져옴)
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 버전을 (major, minor, patch) 튜플로 파싱
fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let v = version.strip_prefix('v').unwrap_or(version);
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

/// GitHub API에서 최신 릴리스 태그 조회
fn fetch_latest_version() -> Result<String, String> {
    let output = Command::new("curl")
        .args([
            "-sL",
            "-H",
            "Accept: application/vnd.github+json",
            "https://api.github.com/repos/TheMagicTower/summon/releases/latest",
        ])
        .output()
        .map_err(|e| format!("curl 실행 실패: {}", e))?;

    if !output.status.success() {
        return Err("GitHub API 요청 실패".into());
    }

    let body = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("JSON 파싱 실패: {}", e))?;

    json.get("tag_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "tag_name을 찾을 수 없습니다".into())
}

/// 플랫폼에 맞는 다운로드 URL 구성
fn build_download_url(tag: &str) -> Result<String, String> {
    let os = match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "darwin",
        _ => return Err("Windows는 자동 업데이트를 지원하지 않습니다.\ninstall.ps1을 사용하세요.".into()),
    };

    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => return Err(format!("지원하지 않는 아키텍처: {}", std::env::consts::ARCH)),
    };

    Ok(format!(
        "https://github.com/TheMagicTower/summon/releases/download/{}/summon-{}-{}.tar.gz",
        tag, os, arch
    ))
}

/// 업데이트 실행
fn perform_update(download_url: &str) -> Result<(), String> {
    let tmp_dir = std::env::temp_dir().join("summon-update");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir)
        .map_err(|e| format!("임시 디렉토리 생성 실패: {}", e))?;

    let archive_path = tmp_dir.join("summon.tar.gz");

    // 다운로드
    println!("다운로드 중...");
    let status = Command::new("curl")
        .args([
            "-sL",
            "-o",
            archive_path.to_str().unwrap(),
            download_url,
        ])
        .status()
        .map_err(|e| format!("다운로드 실패: {}", e))?;

    if !status.success() {
        return Err("다운로드 실패".into());
    }

    // 압축 해제
    let status = Command::new("tar")
        .args([
            "xzf",
            archive_path.to_str().unwrap(),
            "-C",
            tmp_dir.to_str().unwrap(),
        ])
        .status()
        .map_err(|e| format!("압축 해제 실패: {}", e))?;

    if !status.success() {
        return Err("압축 해제 실패".into());
    }

    // 현재 실행 파일 경로
    let current_exe =
        std::env::current_exe().map_err(|e| format!("현재 실행 파일 경로 확인 실패: {}", e))?;

    let new_exe = tmp_dir.join("summon");
    if !new_exe.exists() {
        return Err("압축 해제된 바이너리를 찾을 수 없습니다".into());
    }

    // 바이너리 교체
    std::fs::copy(&new_exe, &current_exe)
        .map_err(|e| format!("바이너리 교체 실패: {}", e))?;

    // 실행 권한 부여
    let _ = Command::new("chmod")
        .args(["+x", current_exe.to_str().unwrap()])
        .status();

    // 임시 디렉토리 정리
    let _ = std::fs::remove_dir_all(&tmp_dir);

    Ok(())
}

/// `summon update` 엔트리포인트
pub fn run() {
    println!("현재 버전: v{}", CURRENT_VERSION);
    println!("최신 버전 확인 중...");

    let latest_tag = match fetch_latest_version() {
        Ok(tag) => tag,
        Err(e) => {
            eprintln!("버전 확인 실패: {}", e);
            std::process::exit(1);
        }
    };

    let current = match parse_version(CURRENT_VERSION) {
        Some(v) => v,
        None => {
            eprintln!("현재 버전 파싱 실패: {}", CURRENT_VERSION);
            std::process::exit(1);
        }
    };

    let latest = match parse_version(&latest_tag) {
        Some(v) => v,
        None => {
            eprintln!("최신 버전 파싱 실패: {}", latest_tag);
            std::process::exit(1);
        }
    };

    if current >= latest {
        println!("이미 최신 버전입니다 (v{}).", CURRENT_VERSION);
        return;
    }

    println!("새 버전 발견: v{} → {}", CURRENT_VERSION, latest_tag);

    let confirmed = Confirm::new()
        .with_prompt("업데이트하시겠습니까?")
        .default(true)
        .interact()
        .unwrap_or(false);

    if !confirmed {
        println!("업데이트 취소됨");
        return;
    }

    let download_url = match build_download_url(&latest_tag) {
        Ok(url) => url,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    match perform_update(&download_url) {
        Ok(()) => {
            println!("업데이트 완료! v{} → {}", CURRENT_VERSION, latest_tag);
        }
        Err(e) => {
            eprintln!("업데이트 실패: {}", e);
            std::process::exit(1);
        }
    }
}
