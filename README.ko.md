🌐 [English](README.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Español](README.es.md) | [Deutsch](README.de.md) | [Tiếng Việt](README.vi.md)

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
curl -fsSL https://raw.githubusercontent.com/TheMagicTower/summon/master/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/TheMagicTower/summon/master/install.ps1 | iex
```

> 💡 **WSL 사용자**: WSL 내부와 Windows측 모두에서 Claude Code를 사용할 수 있습니다. 자세한 내용은 아래 [WSL 사용법](#wsl-사용법) 섹션을 참조하세요.

### 바이너리 다운로드

[Releases](https://github.com/TheMagicTower/summon/releases) 페이지에서 플랫폼에 맞는 바이너리를 다운로드하세요.

| 플랫폼 | 파일 |
|--------|------|
| Linux x86_64 | `summon-linux-amd64.tar.gz` |
| Linux ARM64 | `summon-linux-arm64.tar.gz` |
| macOS Intel | `summon-darwin-amd64.tar.gz` |
| macOS Apple Silicon | `summon-darwin-arm64.tar.gz` |
| Windows x86_64 | `summon-windows-amd64.zip` |
| Windows ARM64 | `summon-windows-arm64.zip` |

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

### 설정 파일 위치

summon은 다음 우선순위로 설정 파일을 검색합니다:

| 우선순위 | 위치 | 설명 |
|---------|------|------|
| 1 | `--config <경로>` | 명시적 지정 |
| 2 | `SUMMON_CONFIG` 환경변수 | 환경변수로 지정된 경로 |
| 3 | `~/.config/summon/config.yaml` | 사용자별 설정 (XDG) |
| 4 | `/etc/summon/config.yaml` | 시스템 와이드 설정 |
| 5 | `./config.yaml` | 현재 디렉토리 |

### 다중 사용자 환경

각 사용자가 자신만의 설정을 사용하려면:
```bash
mkdir -p ~/.config/summon
cp /path/to/config.yaml ~/.config/summon/
```

시스템 관리자가 기본 설정을 제공하려면:
```bash
sudo mkdir -p /etc/summon
sudo cp config.yaml /etc/summon/
```

### 설정 방식

제공자와 용도에 따라 두 가지 방식을 선택할 수 있습니다.

#### 방안 1: 호환 제공자 (모델명 그대로 전달)

Anthropic 모델명을 그대로 이해하는 제공자(Z.AI, Kimi 등)에 적합합니다. Claude Code가 보내는 원래 모델명이 그대로 전달됩니다.

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
      url: "https://api.kimi.com/coding"
      auth:
        header: "Authorization"
        value: "Bearer ${KIMI_API_KEY}"
```

- Claude Code가 `model: "claude-haiku-4-5-20251001"`을 전송 → `"claude-haiku"` 매칭 → Z.AI로 라우팅
- 제공자가 Anthropic 모델명에 대해 실제 어떤 모델을 사용할지 결정
- 간단한 설정, 별도의 Claude Code 설정 불필요

#### 방안 2: 특정 모델 지정 (settings.json 오버라이드)

제공자가 매핑하는 기본 모델이 아닌 특정 모델을 사용하고 싶을 때 (예: `claude-haiku` 대신 `glm-4.7` 지정). Claude Code의 `settings.json`에서 모델명을 오버라이드합니다:

**Step 1.** Claude Code가 원하는 모델명을 전송하도록 설정:

```json
// ~/.claude/settings.json
{
  "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:18081",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "glm-4.7",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "kimi-for-coding"
  }
}
```

| 환경변수 | 설명 |
|---------|------|
| `ANTHROPIC_BASE_URL` | 프록시 주소 (매번 기동 시 지정할 필요 없음) |
| `ANTHROPIC_DEFAULT_HAIKU_MODEL` | Haiku 티어 선택 시 전송되는 모델명 |
| `ANTHROPIC_DEFAULT_SONNET_MODEL` | Sonnet 티어 선택 시 전송되는 모델명 |
| `ANTHROPIC_DEFAULT_OPUS_MODEL` | Opus 티어 선택 시 전송되는 모델명 |

**Step 2.** 오버라이드된 모델명에 맞춰 `config.yaml` 작성:

```yaml
server:
  host: "127.0.0.1"
  port: 18081

default:
  url: "https://api.anthropic.com"

routes:
  - match: "glm"
    upstream:
      url: "https://api.z.ai/api/anthropic"
      auth:
        header: "x-api-key"
        value: "${Z_AI_API_KEY}"

  - match: "kimi"
    upstream:
      url: "https://api.kimi.com/coding"
      auth:
        header: "Authorization"
        value: "Bearer ${KIMI_API_KEY}"
```

- Claude Code가 `model: "glm-4.7"` (오버라이드됨)을 전송 → `"glm"` 매칭 → Z.AI에서 정확한 모델로 처리
- 제공자가 사용하는 모델을 정확히 제어 가능
- `ANTHROPIC_BASE_URL`을 `settings.json`에 넣으면 환경변수 없이 `claude`만 실행 가능

### 설정 참조

- `match`: 모델명에 이 문자열이 포함되면 매칭 (위→아래 순서, 첫 매칭 적용)
- `${ENV_VAR}`: 환경변수 참조 (API 키를 설정 파일에 직접 기입하지 않음)
- `upstream.auth.pool`: 부하 분산을 위한 추가 API 키 값 (`auth.header`와 동일한 헤더 사용)
- `concurrency`: 키별 동시 요청 제한 (초과 시 Anthropic으로 폴백 또는 429 반환)
- `fallback`: 제공자 실패 시 Anthropic API로 폴백 여부 (기본값: `true`)
- 매칭되지 않는 모델은 `default.url`(Anthropic API)로 패스스루

### API 키 풀 (동시성 제한 처리)

일부 제공자는 API 키당 동시 요청 수를 제한합니다 (예: GLM-5는 키당 1개 동시 요청만 허용). 여러 API 키를 풀로 등록하여 전체 동시성을 높일 수 있습니다:

```yaml
routes:
  - match: "glm-5"
    concurrency: 1           # 키별 동시 요청 제한
    upstream:
      url: "https://open.bigmodel.cn/api/paas/v4"
      auth:
        header: "Authorization"
        value: "Bearer ${GLM_KEY_1}"
        pool:                 # 추가 키 (동일한 헤더)
          - "Bearer ${GLM_KEY_2}"
          - "Bearer ${GLM_KEY_3}"
    transformer: "openai"
    model_map: "glm-5"
```

**작동 방식:**

- 요청은 가장 적은 활성 연결을 가진 키로 분배됩니다 (**Least-Connections**)
- 각 키의 동시 사용량은 `concurrency` 설정에 의해 추적 및 제한됩니다
- 모든 키가 한도에 도달하면: Anthropic으로 폴백 (`fallback: true`인 경우) 또는 HTTP 429 반환
- 스트리밍 응답은 스트림이 끝날 때 키를 자동으로 해제합니다

## 실행

```bash
# 환경변수 설정
export Z_AI_API_KEY="your-z-ai-key"
export KIMI_API_KEY="your-kimi-key"

# 프록시 시작 (설정 파일 자동 검색)
summon

# 또는 설정 파일 직접 지정
summon --config /path/to/config.yaml
```

### Claude Code 연결

**방법 A: 수동 (세션마다)**
```bash
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

**방법 B: 자동 (권장)**

`~/.claude/settings.json`에 추가하면 매번 URL을 지정할 필요가 없습니다:
```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:18081"
  }
}
```

이후 간단히 실행:
```bash
claude
```

## CLI 관리

### 자체 업데이트

새 릴리스를 확인하고 바이너리를 제자리에서 업데이트합니다:

```bash
summon update
```

업데이트 명령은:
1. 현재 버전을 최신 GitHub 릴리스와 비교합니다
2. 새 버전이 있으면 확인을 요청합니다
3. 바이너리를 자동으로 다운로드하고 교체합니다

> Windows: 자체 업데이트는 지원되지 않습니다. 대신 `install.ps1`을 사용하세요.

### 직접 명령

모든 관리 명령은 최상위 수준에서 사용할 수 있습니다:

```bash
summon status          # 현재 상태 표시
summon enable          # 프록시 활성화 (settings.json 수정 + 시작)
summon disable         # 프록시 비활성화 (중지 + settings.json 복원)
summon start           # 백그라운드에서 프록시 시작
summon stop            # 프록시 중지
summon add             # 제공자 라우트 추가
summon remove          # 제공자 라우트 제거
summon restore         # 백업에서 settings.json 복원
```

### 대화형 구성

`summon configure`를 실행하면 사용 가능한 모든 작업이 포함된 대화형 메뉴가 열립니다:

```bash
summon configure
```

## WSL 사용법

WSL(Windows Subsystem for Linux)에서도 summon을 사용할 수 있습니다.

### WSL 내부에서 Claude Code 사용

```bash
# WSL 터미널에서 (설정 파일을 ~/.config/summon/config.yaml에 배치한 경우)
summon

# 다른 WSL 터미널에서
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

### Windows측에서 Claude Code 사용 (WSL에서 summon 실행)

```bash
# WSL에서 summon 실행 (0.0.0.0으로 바인딩하여 Windows에서 접근 가능하도록)
summon

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

설치 스크립트는 환경을 자동 감지하여 적절한 서비스 타입을 선택합니다:
- **사용자 서비스** (user service): 데스크톱 환경
- **시스템 서비스** (system service): 헤드리스 서버 (SSH 세션 등)

#### 방법 1: 사용자 서비스 (Desktop 환경)

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

#### 방법 2: 시스템 서비스 (헤드리스 서버)

SSH 세션 등 D-Bus 사용자 세션이 없는 환경에서는 시스템 레벨 서비스를 사용합니다. **sudo 권한이 필요합니다.**

**1. systemd 서비스 파일 생성 (sudo 필요):**

```bash
sudo tee /etc/systemd/system/summon.service > /dev/null << 'EOF'
[Unit]
Description=Summon LLM Proxy
After=network.target

[Service]
Type=simple
User=$(whoami)
Group=$(id -gn)
ExecStart=/home/$(whoami)/.local/bin/summon --config /home/$(whoami)/.config/summon/config.yaml
Restart=always
RestartSec=5
Environment="PATH=/home/$(whoami)/.local/bin:/usr/local/bin:/usr/bin:/bin"

[Install]
WantedBy=multi-user.target
EOF
```

**2. 서비스 등록 및 시작 (sudo 필요):**

```bash
# 시스템 서비스 로드
sudo systemctl daemon-reload
sudo systemctl enable summon.service
sudo systemctl start summon.service

# 서비스 관리
sudo systemctl status summon    # 상태 확인
sudo systemctl stop summon      # 중지
sudo systemctl restart summon   # 재시작
sudo systemctl disable summon   # 자동 시작 비활성화

# 로그 확인
journalctl -u summon -f
```

> **참고**: WSL2에서 systemd를 사용하려면 `/etc/wsl.conf`에 `[boot] systemd=true` 설정이 필요할 수 있습니다.

## 주요 기능

- **투명한 프록시**: Claude Code 입장에서 프록시의 존재를 인식하지 못함
- **모델 기반 라우팅**: `/v1/messages` POST의 `model` 필드로 라우팅 결정
- **SSE 스트리밍**: 청크 단위 실시간 패스스루
- **구독 인증 병행**: Anthropic OAuth 토큰은 그대로 유지, 외부 제공자만 API 키 교체
- **API 키 풀**: 키당 동시성 제한이 있는 제공자를 위해 Least-Connections 분배로 라우트당 여러 API 키 지원
- **보안**: `127.0.0.1`에만 바인딩, API 키는 환경변수 참조

## ⚠️ 주의사항 (Known Limitations)

### 외부 모델로 교체 후 Anthropic thinking 모델 사용 불가

**한 번 외부 제공자(Kimi, Z.AI 등)의 모델로 교체된 대화는 이후 Anthropic의 thinking 모델(Claude Opus, Sonnet 등)에서 이어서 진행할 수 없습니다.**

이는 시스템 아키텍처상의 제한사항이며 해결할 수 없는 문제입니다:
- 외부 제공자는 Anthropic의 네이티브 메시지 형식과 완전히 호환되지 않음
- Thinking 모델은 특정 네이티브 필드와 컨텍스트 구조에 의존
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
