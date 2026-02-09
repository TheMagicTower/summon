# Summon

모델명 기반으로 Claude Code의 API 요청을 다른 LLM 제공자에게 라우팅하는 Rust 경량 리버스 프록시.

기존 Anthropic 구독(OAuth) 인증을 유지하면서 특정 모델만 외부 제공자(Z.AI, Kimi 등)로 분기합니다.

## 아키텍처

```
Claude Code CLI
  │ ANTHROPIC_BASE_URL=http://127.0.0.1:18081
  ▼
프록시 (axum 서버)
  ├─ /v1/messages POST → model 필드 파싱 → 라우팅 결정
  │   ├─ 매칭 → 외부 제공자 (헤더/인증 교체)
  │   └─ 미매칭 → Anthropic API (패스스루)
  └─ 그 외 요청 → Anthropic API (패스스루)
```

## 설치

### 원라인 설치 (권장)

**Linux/macOS/WSL:**
```bash
curl -fsSL https://raw.githubusercontent.com/TheMagicTower/summon/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/TheMagicTower/summon/main/install.ps1 | iex
```

> 💡 **WSL 사용자**: WSL 낸과 Windows측 모두에서 Claude Code를 사용할 수 있습니다. 자세한 내용은 아래 [WSL 사용법](#wsl-사용법) 섹션을 참조하세요.

### 바이너리 다운로드

[Releases](https://github.com/TheMagicTower/summon/releases) 페이지에서 플랫폼에 맞는 바이너리를 다운로드하세요.

| 플랫폼 | 파일 |
|--------|------|
| Linux x86_64 | `summon-linux-amd64.tar.gz` |
| Linux ARM64 | `summon-linux-arm64.tar.gz` |
| macOS Intel | `summon-darwin-amd64.tar.gz` |
| macOS Apple Silicon | `summon-darwin-arm64.tar.gz` |

```bash
# 예: macOS Apple Silicon
tar xzf summon-darwin-arm64.tar.gz
chmod +x summon-darwin-arm64
sudo mv summon-darwin-arm64 /usr/local/bin/summon
```

### 소스에서 빌드

```bash
cargo build --release
```

## 설정

`config.yaml` 파일을 생성합니다:

```yaml
server:
  host: "127.0.0.1"
  port: 18081

default:
  url: "https://api.anthropic.com"

routes:
  - match: "claude-haiku"
    upstream:
      url: "https://api.z.ai/api/anthropic"
      auth:
        header: "x-api-key"
        value: "${Z_AI_API_KEY}"

  - match: "claude-sonnet"
    upstream:
      url: "https://api.kimi.ai/v1"
      auth:
        header: "Authorization"
        value: "Bearer ${KIMI_API_KEY}"
```

- `match`: 모델명에 이 문자열이 포함되면 매칭 (위→아래 순서, 첫 매칭 적용)
- `${ENV_VAR}`: 환경변수 참조 (API 키를 설정 파일에 직접 기입하지 않음)
- 매칭되지 않는 모델은 `default.url`(Anthropic API)로 패스스루

## 실행

```bash
# 환경변수 설정
export Z_AI_API_KEY="your-z-ai-key"
export KIMI_API_KEY="your-kimi-key"

# 프록시 시작
summon --config config.yaml

# Claude Code 연동
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

## WSL 사용법

WSL(Windows Subsystem for Linux)에서도 summon을 사용할 수 있습니다.

### WSL 낸에서 Claude Code 사용

```bash
# WSL 터미널에서
summon --config ~/.config/summon/config.yaml

# 다른 WSL 터미널에서
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

### Windows측에서 Claude Code 사용 (WSL에서 summon 실행)

```bash
# WSL에서 summon 실행 (0.0.0.0으로 바인딩하여 Windows에서 접근 가능하도록)
summon --config ~/.config/summon/config.yaml

# Windows 터미널(PowerShell/CMD)에서
# WSL IP 확인: ip addr show eth0 | grep 'inet '
ANTHROPIC_BASE_URL=http://$(wsl hostname -I | awk '{print $1}'):18081 claude
```

또는 `config.yaml`에서 `server.host`를 `"0.0.0.0"`으로 설정하여 Windows에서 접근할 수 있습니다.

## 백그라운드 서비스로 등록

### macOS (launchd)

**1. LaunchAgent plist 파일 생성:**

```bash
cat > ~/Library/LaunchAgents/com.themagictower.summon.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.themagictower.summon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/YOUR_USERNAME/.local/bin/summon</string>
        <string>--config</string>
        <string>/Users/YOUR_USERNAME/.config/summon/config.yaml</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/Users/YOUR_USERNAME/.local/share/summon/summon.log</string>
    <key>StandardErrorPath</key>
    <string>/Users/YOUR_USERNAME/.local/share/summon/summon.error.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/Users/YOUR_USERNAME/.local/bin:/usr/local/bin:/usr/bin:/bin</string>
    </dict>
</dict>
</plist>
EOF
```

**2. 로그 디렉토리 생성 및 서비스 등록:**

```bash
mkdir -p ~/.local/share/summon
launchctl load ~/Library/LaunchAgents/com.themagictower.summon.plist
launchctl start com.themagictower.summon
```

**3. 서비스 관리:**

```bash
# 상태 확인
launchctl list | grep com.themagictower.summon

# 중지
launchctl stop com.themagictower.summon

# 재시작
launchctl stop com.themagictower.summon && launchctl start com.themagictower.summon

# 제거
launchctl unload ~/Library/LaunchAgents/com.themagictower.summon.plist
rm ~/Library/LaunchAgents/com.themagictower.summon.plist
```

### Windows (Windows Service)

**PowerShell (관리자 권한 필요):**

```powershell
# 1. summon을 Windows Service로 등록 (nssm 사용 권장)
# nssm 설치: winget install nssm

# 서비스 등록
nssm install Summon "$env:LOCALAPPDATA\summon\bin\summon.exe"
nssm set Summon AppParameters "--config `"$env:APPDATA\summon\config.yaml`""
nssm set Summon DisplayName "Summon LLM Proxy"
nssm set Summon Start SERVICE_AUTO_START

# 서비스 시작
Start-Service Summon

# 서비스 관리
Get-Service Summon      # 상태 확인
Stop-Service Summon     # 중지
Restart-Service Summon  # 재시작
sc delete Summon        # 제거
```

**또는 WinSW 사용:**

```powershell
# WinSW 다운로드 및 설정
# https://github.com/winsw/winsw/releases

# summon-service.xml 생성:
@"
<service>
  <id>summon</id>
  <name>Summon LLM Proxy</name>
  <description>Model-based routing proxy for Claude Code</description>
  <executable>%LOCALAPPDATA%\summon\bin\summon.exe</executable>
  <arguments>--config "%APPDATA%\summon\config.yaml"</arguments>
  <log mode="roll-by-size">
    <sizeThreshold>10240</sizeThreshold>
    <keepFiles>8</keepFiles>
  </log>
</service>
"@ | Out-File "$env:LOCALAPPDATA\summon\bin\summon-service.xml" -Encoding UTF8

# 서비스 등록 및 시작
winsw install $env:LOCALAPPDATA\summon\bin\summon-service.xml
winsw start $env:LOCALAPPDATA\summon\bin\summon-service.xml
```

### Linux (systemd) - WSL 포함

**1. systemd 서비스 파일 생성:**

```bash
cat > ~/.config/systemd/user/summon.service << 'EOF'
[Unit]
Description=Summon LLM Proxy
After=network.target

[Service]
Type=simple
ExecStart=%h/.local/bin/summon --config %h/.config/summon/config.yaml
Restart=always
RestartSec=5
Environment="PATH=%h/.local/bin:/usr/local/bin:/usr/bin:/bin"

[Install]
WantedBy=default.target
EOF
```

**2. 서비스 등록 및 시작:**

```bash
# 사용자 서비스 로드
systemctl --user daemon-reload
systemctl --user enable summon.service
systemctl --user start summon.service

# 서비스 관리
systemctl --user status summon    # 상태 확인
systemctl --user stop summon      # 중지
systemctl --user restart summon   # 재시작
systemctl --user disable summon   # 자동 시작 비활성화
```

> **참고**: WSL2에서 systemd를 사용하려면 `/etc/wsl.conf`에 `[boot] systemd=true` 설정이 필요할 수 있습니다.

## 주요 기능

- **투명한 프록시**: Claude Code 입장에서 프록시의 존재를 인식하지 못함
- **모델 기반 라우팅**: `/v1/messages` POST의 `model` 필드로 라우팅 결정
- **SSE 스트리밍**: 청크 단위 실시간 패스스루
- **구독 인증 병행**: Anthropic OAuth 토큰은 그대로 유지, 외부 제공자만 API 키 교체
- **보안**: `127.0.0.1`에만 바인딩, API 키는 환경변수 참조

## ⚠️ 주의사항 (Known Limitations)

### 외부 모델로 교체 후 Anthropic thinking 모델 사용 불가

**한 번 외부 제공자(Kimi, Z.AI 등)의 모델로 교첼된 대화는 이후 Anthropic의 thinking 모델(Claude Opus, Sonnet 등)에서 이어서 진행할 수 없습니다.**

이는 시스템 아키텍처상의 제한사항이며 해결할 수 없는 문제입니다:
- 외부 제공자는 Anthropic의 나이티브 메시지 형식과 완전히 호환되지 않음
- Thinking 모델은 특정 나이티브 필드와 컨텍스트 구조에 의존
- 외부 모델의 응답은 thinking 모델이 요구하는 컨텍스트 형식을 충족하지 못함

**권장 사용 방식:**
- 동일한 대화 세션 내에서 모델을 교체해야 할 경우, 외부 모델 ↔ 외부 모델 간에만 전환하세요
- Anthropic thinking 모델이 필요한 경우, **새로운 대화를 시작**하세요

## 로드맵

- **v0.1** (현재): 패스스루 + 모델 기반 라우팅 + SSE 스트리밍
- **v0.2**: 트랜스포머 (요청/응답 변환 — 비호환 제공자 지원)
- **v0.3**: 로깅, 헬스체크, 핫 리로드, 타임아웃

## 라이선스

MIT
