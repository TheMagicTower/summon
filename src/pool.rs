use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::config::Config;

/// 라우트별 API 키 풀 — Least-Connections 방식 분배
///
/// 각 라우트의 `auth.pool`에 복수 키가 설정된 경우,
/// 키당 활성 연결 수를 추적하고 가장 여유 있는 키를 선택한다.
/// `concurrency` 제한이 있으면 해당 키의 활성 연결이 제한에 도달하면 건너뛴다.
pub struct KeyPool {
    entries: Vec<Option<PoolEntry>>,
}

struct PoolEntry {
    /// 키별 활성 연결 수
    active: Vec<AtomicUsize>,
    /// 키당 동시 요청 제한 (None = 무제한)
    concurrency: Option<usize>,
}

impl KeyPool {
    /// Config로부터 KeyPool 생성
    pub fn from_config(config: &Config) -> Self {
        let entries = config
            .routes
            .iter()
            .map(|route| {
                if route.upstream.auth.has_pool() {
                    let pool_size = route.upstream.auth.all_values().len();
                    Some(PoolEntry {
                        active: (0..pool_size).map(|_| AtomicUsize::new(0)).collect(),
                        concurrency: route.concurrency,
                    })
                } else {
                    None
                }
            })
            .collect();
        KeyPool { entries }
    }

    /// Least-Connections 방식으로 키 획득
    ///
    /// concurrency 제한 내에서 활성 연결이 가장 적은 키의 인덱스를 반환.
    /// 모든 키가 제한에 도달하면 None 반환.
    pub fn acquire(&self, route_idx: usize) -> Option<usize> {
        let entry = self.entries.get(route_idx)?.as_ref()?;
        let limit = entry.concurrency.unwrap_or(usize::MAX);

        // 제한 이하인 키 중 가장 활성 연결이 적은 키 선택
        let best = entry
            .active
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                let count = c.load(Ordering::Relaxed);
                if count < limit {
                    Some((i, count))
                } else {
                    None
                }
            })
            .min_by_key(|(_, count)| *count);

        match best {
            Some((idx, _)) => {
                entry.active[idx].fetch_add(1, Ordering::Relaxed);
                Some(idx)
            }
            None => None, // 모든 키가 concurrency 제한에 도달
        }
    }

    /// 키 해제 (활성 연결 카운터 감소)
    pub fn release(&self, route_idx: usize, key_idx: usize) {
        if let Some(Some(entry)) = self.entries.get(route_idx) {
            if let Some(c) = entry.active.get(key_idx) {
                c.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }
}

/// 키 풀 자동 해제 가드
///
/// Drop 시 활성 연결 카운터를 자동으로 감소시킨다.
/// 스트리밍 응답의 Body에 부착하여 스트림 종료 시 해제.
pub struct PoolGuard {
    pool: Arc<KeyPool>,
    route_idx: usize,
    key_idx: usize,
}

impl PoolGuard {
    pub fn new(pool: Arc<KeyPool>, route_idx: usize, key_idx: usize) -> Self {
        PoolGuard {
            pool,
            route_idx,
            key_idx,
        }
    }
}

impl Drop for PoolGuard {
    fn drop(&mut self) {
        self.pool.release(self.route_idx, self.key_idx);
        tracing::debug!(
            route = self.route_idx,
            key = self.key_idx,
            "키 풀 연결 해제"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    fn make_pool_config() -> Config {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".into(),
                port: 18081,
            },
            default: DefaultConfig {
                url: "https://api.anthropic.com".into(),
            },
            routes: vec![
                // 라우트 0: 풀 있음, concurrency 1
                RouteConfig {
                    match_pattern: "glm-5".into(),
                    upstream: UpstreamConfig {
                        url: "https://open.bigmodel.cn".into(),
                        auth: AuthConfig {
                            auth_type: "api_key".into(),
                            header: Some("Authorization".into()),
                            value: Some("Bearer key1".into()),
                            client_id: None,
                            client_secret: None,
                            refresh_token: None,
                            token_url: None,
                            pool: Some(vec![
                                "Bearer key2".into(),
                                "Bearer key3".into(),
                            ]),
                        },
                    },
                    transformer: None,
                    model_map: None,
                    fallback: Fallback::Passthrough,
                    concurrency: Some(1),
                },
                // 라우트 1: 풀 없음
                RouteConfig {
                    match_pattern: "kimi".into(),
                    upstream: UpstreamConfig {
                        url: "https://api.kimi.com".into(),
                        auth: AuthConfig {
                            auth_type: "api_key".into(),
                            header: Some("Authorization".into()),
                            value: Some("Bearer kimi-key".into()),
                            client_id: None,
                            client_secret: None,
                            refresh_token: None,
                            token_url: None,
                            pool: None,
                        },
                    },
                    transformer: None,
                    model_map: None,
                    fallback: Fallback::Passthrough,
                    concurrency: None,
                },
            ],
        }
    }

    #[test]
    fn test_acquire_least_connections() {
        let config = make_pool_config();
        let pool = KeyPool::from_config(&config);

        // 첫 3회 acquire: key1(0), key2(1), key3(2) 순서로 분배
        let k0 = pool.acquire(0).unwrap();
        let k1 = pool.acquire(0).unwrap();
        let k2 = pool.acquire(0).unwrap();

        assert_eq!(k0, 0);
        assert_eq!(k1, 1);
        assert_eq!(k2, 2);
    }

    #[test]
    fn test_concurrency_limit_reached() {
        let config = make_pool_config();
        let pool = KeyPool::from_config(&config);

        // concurrency=1이므로 키 3개 → 최대 3개 동시 요청
        pool.acquire(0).unwrap(); // key0: 1/1
        pool.acquire(0).unwrap(); // key1: 1/1
        pool.acquire(0).unwrap(); // key2: 1/1

        // 4번째는 None (모든 키 한계)
        assert!(pool.acquire(0).is_none());
    }

    #[test]
    fn test_release_frees_slot() {
        let config = make_pool_config();
        let pool = KeyPool::from_config(&config);

        let k0 = pool.acquire(0).unwrap();
        pool.acquire(0).unwrap();
        pool.acquire(0).unwrap();

        // 모든 키 소진
        assert!(pool.acquire(0).is_none());

        // key0 해제 → 다시 사용 가능
        pool.release(0, k0);
        let k = pool.acquire(0).unwrap();
        assert_eq!(k, 0);
    }

    #[test]
    fn test_no_pool_returns_none() {
        let config = make_pool_config();
        let pool = KeyPool::from_config(&config);

        // 라우트 1은 풀 없음
        assert!(pool.acquire(1).is_none());
    }

    #[test]
    fn test_guard_auto_release() {
        let config = make_pool_config();
        let pool = Arc::new(KeyPool::from_config(&config));

        pool.acquire(0).unwrap(); // key0: 1/1
        pool.acquire(0).unwrap(); // key1: 1/1

        {
            let k2 = pool.acquire(0).unwrap(); // key2: 1/1
            let _guard = PoolGuard::new(pool.clone(), 0, k2);
            assert!(pool.acquire(0).is_none()); // 모두 소진
        }
        // guard drop → key2 해제

        let freed = pool.acquire(0).unwrap();
        assert_eq!(freed, 2);
    }
}
