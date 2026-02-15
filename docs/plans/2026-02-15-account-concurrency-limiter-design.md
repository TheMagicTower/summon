# 계정 단위 동시성 제한 설계

**작성일:** 2026-02-15
**버전:** v0.3 기능
**목적:** BigModel 등 계정 단위 속도 제한이 있는 제공자에 대해 프록시에서 요청을 순차 처리하여 429 에러 방지

## 배경

### 문제 상황

BigModel API는 계정 단위로 동시 요청 수를 제한합니다:

```json
{
  "error": {
    "code": "1302",
    "message": "您的账户已达到速率限制，请您控制请求频率"
  }
}
```

**현재 동작:**
- 여러 API 키를 풀로 사용해도 같은 계정이면 전체 합산으로 제한
- 2개 Claude Code가 동시 요청 → 계정 한도 초과 → 429 발생
- 기존 `concurrency` 설정은 키별 제한이라 계정 제한에 대응 불가

**요구사항:**
- 계정 전체의 동시 요청 수를 프록시에서 제어
- 한도 초과 시 429 발생 대신 요청을 대기시켜 순차 처리
- 기존 키별 `concurrency`와 독립적으로 동작

## 설계 개요

### 핵심 개념

라우트별로 **계정 단위 세마포어**를 추가하여 전체 동시 요청 수를 제한합니다.

```
설정: account_concurrency: 1

요청 1 (Claude Code A) → 세마포어 획득 (1/1 사용 중) → 외부 API 호출
요청 2 (Claude Code B) → 세마포어 대기 (슬롯 없음)
                         ↓
요청 1 완료 → 세마포어 해제 (0/1 사용 중)
                         ↓
요청 2 대기 종료 → 세마포어 획득 (1/1 사용 중) → 외부 API 호출
```

### 기존 시스템과의 관계

| 구분 | 키별 `concurrency` | 계정 `account_concurrency` |
|------|-------------------|---------------------------|
| **제어 단위** | API 키당 | 라우트(계정) 전체 |
| **기존/신규** | 기존 유지 | 신규 추가 |
| **설정 위치** | `routes[].concurrency` | `routes[].account_concurrency` |
| **동작 순서** | 2단계 (계정 세마포어 획득 후) | 1단계 (먼저 획득) |

**처리 순서:**
1. 계정 세마포어 획득 (account_concurrency)
2. 키 풀에서 API 키 할당 (concurrency)
3. 요청 처리
4. 키 해제
5. 계정 세마포어 해제

## 설정 구조

### config.yaml 변경

```yaml
routes:
  - match: "glm-5"
    account_concurrency: 1        # 신규: 계정 전체 동시 요청 제한
    concurrency: 1                # 기존: 키당 동시 요청 제한
    upstream:
      url: "https://open.bigmodel.cn/api/anthropic"
      auth:
        header: "x-api-key"
        value: "key1"
        pool:
          - "key2"
          - "key3"
    fallback: false
```

### 설정 필드

**`account_concurrency`**: (선택적)
- 타입: `Option<usize>`
- 의미: 해당 라우트의 모든 API 키를 합쳐서 동시 N개 요청만 허용
- 미설정 시: 제한 없음 (기존 동작 유지)
- 사용 예: BigModel처럼 계정 단위 제한이 있는 제공자

**`concurrency`**: (기존 필드, 변경 없음)
- 타입: `Option<usize>`
- 의미: 개별 API 키당 동시 요청 제한
- `account_concurrency`와 독립적으로 동작

### 사용 예시

```yaml
# Case 1: 계정 제한 + 키별 제한 (BigModel - 보수적)
account_concurrency: 1
concurrency: 1

# Case 2: 계정 제한만
account_concurrency: 5
# concurrency 없음 = 키당 무제한

# Case 3: 제한 없음 (Anthropic 등)
# 둘 다 없음 = 무제한
```

## 구현 상세

### 1. AccountSemaphore 구조 (pool.rs)

```rust
/// 라우트별 계정 단위 동시성 제어
pub struct AccountSemaphore {
    /// 라우트별 세마포어 (None = 제한 없음)
    semaphores: Vec<Option<Arc<tokio::sync::Semaphore>>>,
}

impl AccountSemaphore {
    pub fn from_config(config: &Config) -> Self {
        let semaphores = config.routes.iter()
            .map(|route| {
                route.account_concurrency
                    .map(|limit| Arc::new(tokio::sync::Semaphore::new(limit)))
            })
            .collect();

        AccountSemaphore { semaphores }
    }

    /// 세마포어 획득 (비동기 대기)
    /// - Some(permit): 제한이 있고 획득 성공
    /// - None: 제한 없음
    pub async fn acquire(&self, route_idx: usize)
        -> Option<tokio::sync::SemaphorePermit<'_>>
    {
        match self.semaphores.get(route_idx)? {
            Some(sem) => Some(sem.acquire().await.ok()?),
            None => None,
        }
    }
}
```

**특징:**
- `tokio::sync::Semaphore` 사용 (비동기 대기 지원)
- `SemaphorePermit`는 Drop 시 자동 해제
- 라우트별 독립적인 세마포어

### 2. AppState 변경 (main.rs)

```rust
pub struct AppState {
    pub config: Arc<Config>,
    pub client: Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub key_pool: Arc<KeyPool>,
    pub account_semaphore: Arc<AccountSemaphore>,  // 신규 추가
}
```

### 3. proxy.rs 통합

#### 세마포어 획득

```rust
// 라우팅 대상인 경우
let (route_idx, route) = route_match;

// 1. 계정 세마포어 획득 (타임아웃 적용)
const SEMAPHORE_TIMEOUT_MS: u64 = 30_000_000;  // 500분

let permit = tokio::time::timeout(
    Duration::from_millis(SEMAPHORE_TIMEOUT_MS),
    state.account_semaphore.acquire(route_idx)
).await;

let account_permit = match permit {
    Ok(Some(p)) => {
        tracing::info!(
            route_idx,
            "계정 세마포어 획득, 요청 처리 시작"
        );
        Some(p)
    }
    Ok(None) => None,  // 제한 없음
    Err(_) => {
        tracing::error!(
            route_idx,
            timeout_mins = 500,
            "계정 세마포어 대기 타임아웃"
        );

        if route.fallback.is_enabled() {
            let fallback_bytes = apply_fallback_model(&bytes, &route.fallback)?;
            return forward(&state, &parts, fallback_bytes, None, None).await;
        }

        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
};

// 2. 키 풀에서 키 할당 (기존 로직)
let key_idx = state.key_pool.acquire_sticky(route_idx, session_hash)?;
let pool_guard = PoolGuard::new(state.key_pool.clone(), route_idx, key_idx);

// 3. 요청 전송
let resp = forward(...).await?;

// 4. 응답 반환 (permit들을 스트림과 함께 유지)
Ok(attach_permits(resp, account_permit, pool_guard))
```

#### Permit 관리

스트리밍 응답에서 Permit이 스트림 종료까지 유지되도록:

```rust
fn attach_permits(
    resp: Response<Body>,
    account_permit: Option<SemaphorePermit<'_>>,
    pool_guard: PoolGuard,
) -> Response<Body> {
    let (parts, body) = resp.into_parts();

    let guarded_body = wrap_body_with_permits(body, account_permit, pool_guard);

    Response::from_parts(parts, guarded_body)
}

fn wrap_body_with_permits(
    body: Body,
    account_permit: Option<SemaphorePermit<'_>>,
    pool_guard: PoolGuard,
) -> Body {
    let stream = async_stream::stream! {
        // permit들을 스트림 생명주기에 묶음
        let _account_permit = account_permit;
        let _pool_guard = pool_guard;

        let mut body = body;
        loop {
            match body.frame().await {
                Some(Ok(frame)) => {
                    if let Ok(data) = frame.into_data() {
                        yield Ok::<Bytes, std::io::Error>(data);
                    }
                }
                Some(Err(e)) => {
                    tracing::error!(error = %e, "응답 스트림 읽기 오류");
                    break;
                }
                None => break,
            }
        }
        // 스트림 종료 시 _account_permit, _pool_guard 자동 drop
    };

    Body::from_stream(stream)
}
```

### 4. 타임아웃 정책

**계산 근거:**
- API_TIMEOUT_MS = 3,000,000ms (50분)
- 세마포어 타임아웃 = API_TIMEOUT × 10 = 30,000,000ms (500분)
- 합리성: 최악의 경우 10개 요청이 각각 50분씩 대기 = 500분

**동작:**
- 500분 이내 슬롯 확보: 정상 처리
- 500분 초과: 타임아웃
  - 폴백 활성화 시 → Anthropic API로 폴백
  - 폴백 비활성화 시 → 503 Service Unavailable

### 5. 로깅

```rust
// 대기 시작
tracing::debug!(route_idx, "계정 세마포어 대기 시작");

// 획득 완료
tracing::info!(
    route_idx,
    available_permits = semaphore.available_permits(),
    "계정 세마포어 획득, 요청 처리"
);

// 타임아웃
tracing::error!(
    route_idx,
    timeout_mins = 500,
    "계정 세마포어 대기 타임아웃"
);

// 해제 (자동, DEBUG 레벨)
tracing::debug!(route_idx, "계정 세마포어 해제");
```

## 테스트 전략

### 단위 테스트

**`pool.rs` - AccountSemaphore 테스트:**

```rust
#[tokio::test]
async fn test_account_semaphore_basic() {
    // account_concurrency: 2
    let sem = AccountSemaphore::from_config(&test_config());

    let permit1 = sem.acquire(0).await.unwrap();
    let permit2 = sem.acquire(0).await.unwrap();

    // 3번째는 대기
    let result = tokio::time::timeout(
        Duration::from_millis(100),
        sem.acquire(0)
    ).await;
    assert!(result.is_err());

    // permit1 해제 후 획득 가능
    drop(permit1);
    let permit3 = sem.acquire(0).await.unwrap();
    assert!(permit3.is_some());
}

#[tokio::test]
async fn test_account_semaphore_no_limit() {
    // account_concurrency 없음
    let sem = AccountSemaphore::from_config(&test_config());

    let p1 = sem.acquire(1).await;
    let p2 = sem.acquire(1).await;

    assert!(p1.is_none());
    assert!(p2.is_none());
}
```

### 통합 테스트 시나리오

**시나리오 1: 순차 처리 검증**
```yaml
# 설정
account_concurrency: 1
```
```bash
# 2개 Claude Code 동시 실행

# 예상 결과:
# - 첫 번째 요청: 즉시 처리
# - 두 번째 요청: 첫 번째 완료까지 대기
# - 429 없이 모두 성공
```

**시나리오 2: 타임아웃 검증**
```bash
# 매우 긴 요청 (50분+) 10개 대기 중

# 예상 결과:
# - 500분 이내 완료: 정상 처리
# - 500분 초과: 타임아웃 → 폴백 또는 503
```

**시나리오 3: 키별 + 계정 병행**
```yaml
account_concurrency: 2
concurrency: 1
# 키 3개 풀
```
```bash
# 요청 3개 동시 발생

# 예상 결과:
# - 요청 1, 2: 즉시 처리 (서로 다른 키)
# - 요청 3: 계정 세마포어 대기
# - 요청 1 또는 2 완료 후 3 처리
```

### 로그 검증

```
INFO 계정 세마포어 획득, 요청 처리 시작 route_idx=0
INFO 라우팅 결정 model=glm-5 routed=true
INFO 키 풀에서 키 할당 (LC) route_idx=0 key_idx=1 active_count=1
...
INFO 키 해제 route_idx=0 key_idx=1 active_count=0
DEBUG 계정 세마포어 해제 route_idx=0
```

## 구현 체크리스트

- [ ] `config.rs`: `RouteConfig`에 `account_concurrency: Option<usize>` 필드 추가
- [ ] `pool.rs`: `AccountSemaphore` 구조체 구현
- [ ] `main.rs`: `AppState`에 `account_semaphore` 추가, 초기화
- [ ] `proxy.rs`: 세마포어 획득/해제 로직 통합
- [ ] `proxy.rs`: `attach_permits` 함수로 스트림 생명주기 관리
- [ ] `proxy.rs`: 타임아웃 처리 (500분)
- [ ] 단위 테스트: `AccountSemaphore` 기본 동작
- [ ] 단위 테스트: 제한 없는 라우트
- [ ] 통합 테스트: 2개 Claude Code 순차 처리
- [ ] 로깅: 세마포어 대기/획득/해제
- [ ] 문서: `SPEC.md` v0.3 섹션 업데이트
- [ ] 문서: `CLAUDE.md` 설정 예시 추가

## 기대 효과

1. **429 에러 방지**: 계정 한도를 프록시에서 준수하여 429 발생 사전 차단
2. **안정적인 다중 세션**: 여러 Claude Code 동시 사용 시에도 순차 처리로 안정성 확보
3. **투명한 대기**: 사용자는 대기 중임을 인지하지 못하고, 프록시가 자동 조절
4. **폴백 연계**: 타임아웃 시 Anthropic API로 자동 폴백하여 가용성 유지
5. **유연한 설정**: 제공자별로 account_concurrency 선택적 적용 가능

## 향후 개선 가능성

- 대기 큐 우선순위 (현재는 FIFO)
- 동적 concurrency 조정 (429 발생 빈도 기반)
- 메트릭 수집 (평균 대기 시간, 타임아웃 빈도)
- 라우트별 타임아웃 설정 (현재는 고정 500분)
