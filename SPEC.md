# Summon

## 개요

모델명 기반으로 Claude Code의 API 요청을 다른 LLM 제공자에게 라우팅하는 경량 리버스 프록시.
기존 Anthropic 구독(OAuth) 인증을 유지하면서 특정 모델만 외부 제공자로 분기한다.

## 핵심 요구사항

### 1. 투명한 프록시

- Claude Code → 프록시 → Anthropic API (기본 경로)
- OAuth 토큰, 헤더, 본문을 **변경 없이 그대로 전달**
- Claude Code 입장에서 프록시의 존재를 인식할 수 없어야 함

### 2. 모델 기반 라우팅

- `/v1/messages` POST 요청만 본문 파싱
- `model` 필드 값에 따라 라우팅 결정
- 라우팅 대상 모델: 헤더(인증)를 교체하고 지정된 업스트림으로 전달
- 라우팅 비대상 모델: Anthropic API로 그대로 전달

### 3. SSE 스트리밍 지원

- `"stream": true` 요청 시 SSE 이벤트를 실시간 패스스루
- 버퍼링 없이 청크 단위 전달

### 4. 트랜스포머 (v0.2)

- 라우트별로 트랜스포머를 지정할 수 있음 (지정하지 않으면 변환 없이 패스스루)
- **요청 트랜스포머**: 모델명 치환, 비지원 필드 제거, 필드 추가, 헤더 변환
- **응답 트랜스포머**: 모델명 복원, 필드명 매핑, 누락 필드 기본값 삽입
- Anthropic 호환 제공자는 트랜스포머 불필요, 비호환 제공자만 설정

### 5. API 키 풀 (v0.2)

- 라우트별로 여러 API 키를 풀로 등록, Least-Connections 방식으로 분배
- `concurrency` 설정으로 키당 동시 요청 수 제한
- 모든 키 소진 시 fallback 동작에 따라 Anthropic 폴백 또는 429 반환
- `PoolGuard`를 통한 스트리밍 응답 자동 해제

### 6. 폴백 모델명 (v0.2)

- `fallback` 필드가 `false` / `true` / `"모델명"` 3가지 형태 지원
- `"모델명"` 지정 시, 외부 제공자 실패 또는 키 풀 소진 시 해당 모델명으로 교체하여 Anthropic API로 폴백
- 비-Anthropic 모델명(glm-5 등) 사용 시 Anthropic이 인식 가능한 모델명으로 안전하게 폴백

### 7. 구독 인증 병행

- Anthropic OAuth 토큰을 그대로 Anthropic API로 전달
- 외부 제공자 요청 시에만 API 키 교체
- 토큰 갱신 등 인증 관련 요청은 전부 Anthropic API로 패스스루

### 8. 계정 단위 동시성 제한 (v0.3)

- 라우트별로 `account_concurrency` 설정 가능
- 계정 전체의 동시 요청 수를 세마포어로 제어
- 한도 초과 시 요청을 대기시켜 순차 처리 (최대 500분)
- BigModel 등 계정 단위 제한이 있는 제공자에 대응

## 아키텍처

```
Claude Code CLI
  │
  │ ANTHROPIC_BASE_URL=http://127.0.0.1:{PORT}
  │
  ▼
┌─────────────────────────────┐
│          Summon             │
│     (Rust 리버스 프록시)     │
│                             │
│  1. 요청 수신               │
│  2. /v1/messages POST인가?  │
│     ├─ 아니오 → 패스스루     │
│     └─ 예 → model 필드 확인 │
│         ├─ 라우팅 대상 →     │
│         │   업스트림 + API키  │
│         └─ 비대상 →          │
│             Anthropic 패스스루│
└─────────────────────────────┘
  │                    │
  ▼                    ▼
Anthropic API      외부 제공자
(api.anthropic.com)  (Kimi, Z.AI 등)
```

## 기술 스택

- **언어**: Rust (stable)
- **HTTP 프레임워크**: axum 0.8
- **HTTP 클라이언트**: hyper-util (legacy Client) + hyper-tls
- **런타임**: tokio
- **설정**: serde + serde_yaml
- **로깅**: tracing + tracing-subscriber
- **미들웨어**: tower-http (트레이싱)
- **빌드**: 단일 바이너리 (`cargo build --release`)

## 프로젝트 구조

```
summon/
├── Cargo.toml
├── config.yaml           # 설정 예시
├── SPEC.md
├── src/
│   ├── main.rs           # 엔트리포인트, CLI 파싱, 서버 시작
│   ├── config.rs         # Config 구조체, YAML 로드, 환경변수 치환, Fallback enum
│   ├── proxy.rs          # 프록시 핸들러 (패스스루 + 라우팅 + 폴백 모델 교체)
│   ├── pool.rs           # API 키 풀 (Least-Connections 분배 + PoolGuard)
│   ├── configure.rs      # 대화형 설정 관리 (enable/disable/add/remove/status 등)
│   ├── transformer.rs    # 요청/응답 변환 (비호환 제공자 지원)
│   └── update.rs         # 자체 업데이트 (GitHub 릴리스 확인 + 바이너리 교체)
```

## 설정 파일

`config.yaml`

```yaml
server:
  host: "127.0.0.1"
  port: 8080

# 기본 업스트림 (라우팅 비대상 모델 + 모든 비-메시지 요청)
default:
  url: "https://api.anthropic.com"
  # 인증 헤더를 그대로 전달 (passthrough)

# 모델별 라우팅 규칙
routes:
  - match: "claude-haiku"        # model 필드에 이 문자열이 포함되면 매칭
    upstream:
      url: "https://api.z.ai/api/anthropic"
      auth:
        header: "x-api-key"
        value: "${Z_AI_API_KEY}"  # 환경변수 참조

  - match: "glm-5"
    concurrency: 1               # 키당 동시 요청 제한
    fallback: "claude-sonnet-4-5-20250929"  # 폴백 시 호환 모델명으로 교체
    upstream:
      url: "https://open.bigmodel.cn/api/paas/v4"
      auth:
        header: "Authorization"
        value: "Bearer ${GLM_KEY_1}"
        pool:                     # 추가 키 (동일 헤더)
          - "Bearer ${GLM_KEY_2}"
    transformer: "openai"
    model_map: "glm-5"

  - match: "claude-sonnet"
    upstream:
      url: "https://api.kimi.ai/v1"
      auth:
        header: "Authorization"
        value: "Bearer ${KIMI_API_KEY}"

# 매칭 순서: 위에서 아래로, 첫 번째 매칭 적용
# 매칭되지 않으면 default로 전달
# fallback: false (폴백 비활성화) / true (원본 모델명 폴백) / "모델명" (모델명 교체 폴백)
```

## 핵심 구현 상세

### 프록시 핸들러 흐름

```rust
async fn proxy_handler(
    State(state): State<AppState>,
    req: Request,
) -> Result<Response, StatusCode> {
    let (parts, body) = req.into_parts();

    // 1. /v1/messages POST인지 확인
    let is_messages = parts.method == Method::POST
        && parts.uri.path() == "/v1/messages";

    if !is_messages {
        // 패스스루: 그대로 default로 포워딩
        return forward(&state, parts, body, None).await;
    }

    // 2. 본문 읽기 → model 추출
    let bytes = body.collect().await?.to_bytes();
    let model = extract_model(&bytes)?;

    // 3. 라우팅 결정
    let route = state.config.find_route(&model);

    // 4. 포워딩 (route 있으면 업스트림/인증 교체)
    forward(&state, parts, bytes, route).await
}
```

### SSE 패스스루

hyper-util Client의 응답 Body는 이미 스트리밍.
axum이 `Response<Body>`를 그대로 반환하면 청크 단위 전달됨.
별도 SSE 처리 불필요.

### 환경변수 치환

```rust
fn resolve_env(raw: &str) -> String {
    // ${VAR_NAME} 패턴을 std::env::var("VAR_NAME")으로 치환
    regex::Regex::new(r"\$\{(\w+)\}")
        .unwrap()
        .replace_all(raw, |caps: &regex::Captures| {
            std::env::var(&caps[1]).unwrap_or_default()
        })
        .to_string()
}
```

## 의존성 (Cargo.toml)

```toml
[dependencies]
axum = "0.8"
hyper = "1"
hyper-util = { version = "0.1", features = ["client-legacy", "http1", "http2", "tokio"] }
hyper-tls = "0.6"
http-body-util = "0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
regex = "1"
tower = "0.5"
tower-http = { version = "0.6", features = ["trace"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## 구현 범위

### v0.1 (MVP) ✅ 완료

- [x] Cargo.toml 의존성 설정
- [x] config.rs: Config 구조체 정의 + YAML 로드 + 환경변수 치환 + find_route
- [x] proxy.rs: 프록시 핸들러 + 패스스루 + 업스트림 포워딩 + SSE 스트리밍
- [x] main.rs: 서버 시작 + --config CLI 인자 + AppState
- [x] 단위 테스트: config 파싱, 환경변수 치환, 모델 매칭
- [x] 빌드 및 기본 동작 검증

### v0.2 ← 현재

- [x] transformer.rs: 요청/응답 변환 (OpenAI 호환 등)
- [x] pool.rs: API 키 풀 (Least-Connections 분배 + concurrency 제한)
- [x] config.rs: Fallback enum (false / true / "모델명")
- [x] proxy.rs: 폴백 시 모델명 교체 (replace_model + apply_fallback_model)
- [x] configure.rs: 대화형 CLI (enable/disable/add/remove/status + 대화형 메뉴)
- [x] update.rs: 자체 업데이트 (GitHub 릴리스 확인 + 바이너리 교체)

### v0.3 (개선)

- [x] 계정 단위 동시성 제한 (`account_concurrency`)
- [ ] 요청/응답 로깅 (선택적)
- [ ] 헬스체크 엔드포인트 (`/health`)
- [ ] 설정 파일 핫 리로드
- [ ] 연결 타임아웃 설정

## 실행 방법

```bash
# 빌드
cargo build --release

# 실행
./target/release/summon --config config.yaml

# Claude Code 연동
ANTHROPIC_BASE_URL=http://127.0.0.1:8080 claude
```

## 검증 방법

```bash
# 패스스루 테스트 (default → Anthropic)
curl http://127.0.0.1:8080/v1/messages \
  -H "x-api-key: test" \
  -H "Content-Type: application/json" \
  -d '{"model":"claude-opus-4-6","max_tokens":10,"messages":[{"role":"user","content":"hi"}]}'

# 라우팅 테스트 (haiku → Z.AI)
curl http://127.0.0.1:8080/v1/messages \
  -H "Content-Type: application/json" \
  -d '{"model":"claude-haiku-4-5-20251001","max_tokens":10,"messages":[{"role":"user","content":"hi"}]}'
```

## 제약사항

1. **v0.1은 트랜스포머 없음** — Anthropic API 호환 제공자만 지원
2. **요청 본문 이중 읽기** — `/v1/messages` POST만 본문을 읽고 다시 전달 (성능 영향 최소)
3. **OAuth 토큰 갱신** — 인증 관련 요청은 반드시 Anthropic API로 전달
4. **보안** — 기본적으로 `127.0.0.1`에만 바인딩, 외부 노출 금지
5. **API 키 보안** — 설정 파일에 직접 기입하지 않고 환경변수 참조 권장
