# Claude Code Model Router

Claude Code의 API 요청을 모델명 기반으로 다른 LLM 제공자에게 라우팅하는 Rust 경량 리버스 프록시.

기존 Anthropic 구독(OAuth) 인증을 유지하면서 특정 모델만 외부 제공자(Z.AI, Kimi 등)로 분기합니다.

## 아키텍처

```
Claude Code CLI
  │ ANTHROPIC_BASE_URL=http://127.0.0.1:8080
  ▼
프록시 (axum 서버)
  ├─ /v1/messages POST → model 필드 파싱 → 라우팅 결정
  │   ├─ 매칭 → 외부 제공자 (헤더/인증 교체)
  │   └─ 미매칭 → Anthropic API (패스스루)
  └─ 그 외 요청 → Anthropic API (패스스루)
```

## 설치

### 바이너리 다운로드

[Releases](https://github.com/TheMagicTower/claude-code-model-router/releases) 페이지에서 플랫폼에 맞는 바이너리를 다운로드하세요.

| 플랫폼 | 파일 |
|--------|------|
| Linux x86_64 | `claude-code-model-router-linux-amd64.tar.gz` |
| Linux ARM64 | `claude-code-model-router-linux-arm64.tar.gz` |
| macOS Intel | `claude-code-model-router-darwin-amd64.tar.gz` |
| macOS Apple Silicon | `claude-code-model-router-darwin-arm64.tar.gz` |

```bash
# 예: macOS Apple Silicon
tar xzf claude-code-model-router-darwin-arm64.tar.gz
chmod +x claude-code-model-router-darwin-arm64
sudo mv claude-code-model-router-darwin-arm64 /usr/local/bin/claude-code-model-router
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
  port: 8080

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
claude-code-model-router --config config.yaml

# Claude Code 연동
ANTHROPIC_BASE_URL=http://127.0.0.1:8080 claude
```

## 주요 기능

- **투명한 프록시**: Claude Code 입장에서 프록시의 존재를 인식하지 못함
- **모델 기반 라우팅**: `/v1/messages` POST의 `model` 필드로 라우팅 결정
- **SSE 스트리밍**: 청크 단위 실시간 패스스루
- **구독 인증 병행**: Anthropic OAuth 토큰은 그대로 유지, 외부 제공자만 API 키 교체
- **보안**: `127.0.0.1`에만 바인딩, API 키는 환경변수 참조

## 로드맵

- **v0.1** (현재): 패스스루 + 모델 기반 라우팅 + SSE 스트리밍
- **v0.2**: 트랜스포머 (요청/응답 변환 — 비호환 제공자 지원)
- **v0.3**: 로깅, 헬스체크, 핫 리로드, 타임아웃

## 라이선스

MIT
