# Changelog

## [v0.3.0] - 2026-02-16

### 추가
- 계정 단위 동시성 제한 (Account-Level Concurrency Limiting)
  - `account_concurrency` 설정 필드 추가 (라우트별)
  - `AccountSemaphore` 구조체 구현 (tokio::Semaphore 기반)
  - `SemaphoreGuard` 패턴으로 생명주기 자동 관리
  - 세마포어 획득 타임아웃 (500분)
  - 모든 응답 경로에 permit 적용 (스트리밍 포함)
  - 타임아웃 시 자동 폴백 연계
- BigModel 등 계정 단위 속도 제한 제공자에 대한 429 에러 사전 차단
- 여러 Claude Code 동시 실행 시 안정성 확보

## [v0.2.8] - 2026-02-14

### 추가
- 429(TOO_MANY_REQUESTS) 응답 시 API 키 쿨다운 적용
  - `Retry-After` 응답 헤더 파싱하여 쿨다운 시간 결정
  - 헤더 없을 시 기본 60초 쿨다운 적용
  - 쿨다운 중인 키는 자동으로 건너뛰어 불필요한 재시도 방지
  - 쿨다운 진입 시 WARN 레벨 로깅으로 운영 모니터링 지원

## [v0.1.0] - 2026-02-08

### 추가
- 프로젝트 초기 릴리즈
- `config.yaml` 기반 설정 시스템 (환경변수 `${VAR}` 치환 지원)
- 모델명 부분 문자열 매칭 기반 라우팅
- `/v1/messages` POST 요청의 `model` 필드 파싱
- 라우팅 대상 모델: 지정된 업스트림으로 인증 헤더 교체 후 포워딩
- 비대상 모델 및 기타 요청: Anthropic API로 패스스루
- SSE 스트리밍 지원 (버퍼링 없이 청크 단위 전달)
- `--config` CLI 인자 (기본값: `config.yaml`)
- 단위 테스트 7개 (환경변수 치환, YAML 파싱, 라우트 매칭)
- GitHub Actions 릴리즈 CI/CD (Linux amd64/arm64, macOS amd64/arm64)
