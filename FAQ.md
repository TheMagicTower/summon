# FAQ / 트러블슈팅

summon 사용 중 자주 발생하는 문제와 해결 방법을 정리한 문서입니다.

---

## Case 1: "There's an issue with the selected model" 경고

**증상**

Claude Code에서 외부 모델(예: `glm-5`)을 선택하면 다음 메시지가 표시된다:

```
There's an issue with the selected model (glm-5). It may not exist or you may not have access to it.
Run /model to pick a different model.
```

**원인**

Claude Code 클라이언트가 모델명을 Anthropic의 알려진 모델 목록과 대조하여 인식하지 못할 때 표시하는 경고입니다. 프록시와 무관한 클라이언트 측 검증이며, 요청 자체는 정상적으로 프록시를 통해 전송될 수 있습니다.

**해결**

- 경고만 표시되고 실제 동작에 문제가 없으면 무시해도 됩니다.
- 동작이 멈추는 경우 `/model`로 다른 모델을 선택 후 다시 시도하세요.

---

## Case 2: 외부 모델이 되다가 안 되다가 반복

**증상**

외부 제공자(Z.AI, Kimi 등)로 라우팅되는 모델을 사용할 때, 정상 동작과 에러가 간헐적으로 반복된다.

**원인**

`fallback: true` (기본값) 설정 시, 외부 제공자가 일시적으로 실패하면 Anthropic API로 폴백합니다. 그런데 Anthropic은 `glm-5` 같은 외부 모델명을 인식하지 못하므로 에러를 반환합니다.

```
요청 → 외부 제공자 성공 → 정상 동작
요청 → 외부 제공자 실패 → Anthropic 폴백 → 모델명 미인식 → 에러
```

**해결**

Anthropic과 호환되지 않는 모델명을 사용하는 라우트에서는 `fallback: false`로 설정하세요:

```yaml
routes:
  - match: "glm"
    upstream:
      url: "https://api.z.ai/api/anthropic"
      auth:
        header: "x-api-key"
        value: "${ZAI_API_KEY}"
    fallback: false   # Anthropic 폴백 비활성화
```

---

## Case 3: 이중 설치 충돌 (글로벌 vs 로컬)

**증상**

- `summon start` 또는 `summon enable`이 간헐적으로 실패한다.
- 포트 18081이 이미 사용 중이라는 에러가 발생한다.
- `which summon`이 가리키는 바이너리와 실제 실행 중인 바이너리가 다르다.

**원인**

여러 사용자 또는 경로에 summon이 각각 설치되어 있고, 각각 별도의 systemd 서비스가 등록되어 동일 포트를 잡으려 하면 충돌이 발생합니다.

```
/opt/UserA/.local/bin/summon   ← systemd user service (실행 중)
/opt/UserB/.local/bin/summon   ← which summon이 가리킴
/etc/systemd/system/summon.service  ← system service (dead)
```

**진단**

```bash
# 실행 중인 summon 프로세스 확인
ps aux | grep summon | grep -v grep

# 포트 점유 확인
ss -tlnp | grep 18081

# 설치된 바이너리 위치 확인
which summon
find /opt /usr/local/bin -name summon -type f 2>/dev/null

# systemd 서비스 확인
systemctl --user status summon          # 유저 서비스
sudo systemctl status summon            # 시스템 서비스
```

**해결: 글로벌 단일 바이너리로 통합**

```bash
# 1. 최신 빌드
cd /path/to/summon && cargo build --release

# 2. 글로벌 설치
sudo cp target/release/summon /usr/local/bin/summon
sudo chmod +x /usr/local/bin/summon

# 3. 로컬 바이너리 제거
sudo rm /opt/UserA/.local/bin/summon
sudo rm /opt/UserB/.local/bin/summon

# 4. systemd 서비스 ExecStart 경로 수정
# 유저 서비스: ~/.config/systemd/user/summon.service
# ExecStart=/usr/local/bin/summon --config ~/.config/summon/config.yaml

# 5. 중복 서비스 제거 (system service가 별도로 있는 경우)
sudo systemctl stop summon
sudo systemctl disable summon
sudo rm /etc/systemd/system/summon.service
sudo systemctl daemon-reload

# 6. 유저 서비스 재시작
systemctl --user daemon-reload
systemctl --user restart summon
```

---

## Case 4: `summon update` 권한 오류 (글로벌 설치)

**증상**

`/usr/local/bin/summon`으로 글로벌 설치 후 `summon update` 실행 시 "바이너리 교체 실패: Permission denied" 에러가 발생한다.

**원인**

`summon update`는 `std::env::current_exe()` 경로에 새 바이너리를 덮어씁니다. `/usr/local/bin/summon`은 root 소유이므로 일반 유저 권한으로는 쓰기 불가합니다.

**해결**

```bash
# 방법 1: sudo로 업데이트
sudo summon update

# 방법 2: 소스에서 수동 빌드
cd /path/to/summon
cargo build --release
sudo cp target/release/summon /usr/local/bin/summon
```

---

## Case 5: Cargo.lock 버전 호환성

**증상**

`cargo build` 시 다음 에러가 발생한다:

```
error: failed to parse lock file at: .../Cargo.lock
Caused by: lock file version 4 requires `-Znext-lockfile-bump`
```

**원인**

Cargo.lock v4 형식은 Rust 1.78 이상이 필요합니다. 시스템에 설치된 Rust 버전이 오래된 경우 발생합니다.

**해결**

```bash
# Rust 버전 확인
rustc --version

# rustup이 있는 경우
rustup update stable

# rustup이 없는 경우 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# 빌드 재시도
cargo build --release
```

---

## Case 6: API 키 풀 동시 요청 제한 도달

**증상**

요청이 `429 Too Many Requests`로 거부되거나, 로그에 "모든 API 키가 동시 요청 제한에 도달" 경고가 나타난다.

**원인**

라우트에 `concurrency` 제한이 설정되어 있고, 풀 내 모든 키가 동시 요청 한도에 도달한 상태입니다.

**해결**

- `pool`에 API 키를 추가하여 처리 용량을 늘린다.
- `concurrency` 값을 높인다 (제공자의 실제 제한 범위 내에서).
- `fallback: true`로 설정하면 한도 초과 시 Anthropic API로 폴백합니다 (단, Case 2 주의).

```yaml
routes:
  - match: "glm-5"
    concurrency: 1          # 키당 동시 1개
    upstream:
      url: "https://api.z.ai/api/anthropic"
      auth:
        header: "x-api-key"
        value: "${ZAI_KEY_1}"
        pool:                # 키를 추가하여 총 처리량 증가
          - "${ZAI_KEY_2}"
          - "${ZAI_KEY_3}"
    fallback: false
```
