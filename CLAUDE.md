# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 프로젝트 개요

Claude Code의 API 요청을 모델명 기반으로 다른 LLM 제공자에게 라우팅하는 Rust 경량 리버스 프록시.
Anthropic 구독(OAuth) 인증을 유지하면서 특정 모델만 외부 제공자(Z.AI, Kimi 등)로 분기한다.

스펙 문서: `SPEC.md`

## 기술 스택

- **언어**: Rust (stable)
- **HTTP 프레임워크**: axum 0.8
- **HTTP 클라이언트**: hyper-util (legacy Client) + hyper-tls
- **런타임**: tokio
- **설정**: serde + serde_yaml (`config.yaml`)
- **로깅**: tracing + tracing-subscriber

## 빌드 및 실행

```bash
# 빌드
cargo build --release

# 실행 (config.yaml 필요)
./target/release/summon --config config.yaml

# 테스트
cargo test

# Claude Code 연동
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

## 아키텍처

```
Claude Code CLI
  │ ANTHROPIC_BASE_URL=http://127.0.0.1:{PORT}
  ▼
프록시 (axum 서버)
  ├─ /v1/messages POST → model 필드 파싱 → 라우팅 결정
  │   ├─ 매칭 → 외부 제공자 (헤더/인증 교체)
  │   └─ 미매칭 → Anthropic API (패스스루)
  └─ 그 외 요청 → Anthropic API (패스스루)
```

### 소스 구조

```
src/
├── main.rs        # 엔트리포인트, CLI 파싱 (Configure/Update), AppState, axum 서버 시작
├── config.rs      # Config 구조체, YAML 로드, ${ENV_VAR} 치환, Fallback enum, find_route
├── proxy.rs       # 프록시 핸들러, 패스스루/라우팅 포워딩, 폴백 모델 교체, SSE 스트리밍
├── pool.rs        # API 키 풀 (Least-Connections 분배, PoolGuard 자동 해제)
├── configure.rs   # 대화형 설정 관리 (enable/disable/add/remove/status 등 + 대화형 메뉴)
├── transformer.rs # 요청/응답 변환 (비호환 제공자 지원)
└── update.rs      # 자체 업데이트 (GitHub 릴리스 확인 + 바이너리 교체)
```

### 핵심 흐름

1. `/v1/messages` POST만 본문 파싱하여 `model` 필드 추출
2. `config.yaml`의 `routes`를 순서대로 순회, `match` 문자열 포함 여부로 매칭
3. 매칭 시 해당 upstream URL + auth 헤더로 교체하여 포워딩
4. 미매칭 시 원본 헤더 그대로 Anthropic API로 패스스루
5. SSE 스트리밍은 hyper 응답 Body를 그대로 반환 (별도 처리 불필요)

### 설정 파일 (`config.yaml`)

- `server.host` / `server.port`: 바인딩 주소
- `default.url`: 기본 업스트림 (Anthropic API)
- `routes[].match`: 모델명 부분 문자열 매칭 (위→아래 순서, 첫 매칭 적용)
- `routes[].upstream.auth`: `header` + `value` (환경변수 `${VAR}` 참조 지원)
- `routes[].upstream.auth.pool`: 부하 분산용 추가 API 키
- `routes[].concurrency`: 키당 동시 요청 제한
- `routes[].fallback`: `false` / `true` / `"모델명"` (폴백 동작 설정)

### 계정 단위 동시성 제한 (v0.3)

BigModel 등 일부 제공자는 API 키별이 아닌 계정 전체의 동시 요청 수를 제한합니다.
이 경우 `account_concurrency`를 설정하여 프록시에서 요청을 순차 처리할 수 있습니다.

```yaml
routes:
  - match: "glm-5"
    account_concurrency: 1  # 계정 전체 동시 1개만
    concurrency: 1          # 키당 동시 1개만 (선택적)
    upstream:
      url: "https://open.bigmodel.cn/api/anthropic"
      auth:
        header: "x-api-key"
        value: "${GLM_KEY}"
```

**설정 설명:**
- `account_concurrency`: 계정 전체 동시 요청 수 제한 (선택적)
- 미설정 시 무제한
- 한도 초과 시 최대 500분까지 대기 후 타임아웃
- `concurrency`(키별 제한)와 독립적으로 동작

## 개발 규칙

- 모든 문서, 주석, 커밋 메시지는 한국어로 작성
- 커밋 접두사: `feat:`, `fix:`, `chore:`, `docs:`
- `127.0.0.1`에만 바인딩 — 외부 노출 금지
- API 키는 설정 파일에 직접 기입하지 않고 환경변수 참조
- OAuth 토큰 갱신 등 인증 관련 요청은 반드시 Anthropic API로 패스스루

## 버전 로드맵

- **v0.1**: 패스스루 + 모델 기반 라우팅 + SSE 스트리밍
- **v0.2** (현재): 트랜스포머, API 키 풀, 폴백 모델명, 대화형 CLI, 자체 업데이트
- **v0.3**: 로깅, 헬스체크, 핫 리로드, 타임아웃
