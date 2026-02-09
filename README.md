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
