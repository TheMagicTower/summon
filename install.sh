#!/bin/bash
set -e

REPO="TheMagicTower/summon"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect WSL
is_wsl() {
    if [ -f /proc/sys/fs/binfmt_misc/WSLInterop ] || [ -n "${WSL_DISTRO_NAME:-}" ]; then
        return 0
    fi
    return 1
}

# Detect platform
detect_platform() {
    local os arch
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        linux)
            case "$arch" in
                x86_64) echo "linux-amd64" ;;
                aarch64|arm64) echo "linux-arm64" ;;
                *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
            esac
            ;;
        darwin)
            case "$arch" in
                x86_64) echo "darwin-amd64" ;;
                aarch64|arm64) echo "darwin-arm64" ;;
                *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
            esac
            ;;
        *) echo "Unsupported OS: $os" >&2; exit 1 ;;
    esac
}

# Get WSL host IP for Windows access
get_wsl_host_ip() {
    ip route show default | grep -oP '(?<=via )\d+\.\d+\.\d+\.\d+' || echo "127.0.0.1"
}

# Detect OS type for service installation
detect_os_type() {
    local os
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    case "$os" in
        darwin) echo "macos" ;;
        linux) echo "linux" ;;
        *) echo "unknown" ;;
    esac
}

# Install macOS LaunchAgent
install_macos_service() {
    local config_file="$1"
    local plist_path="$HOME/Library/LaunchAgents/com.themagictower.summon.plist"
    local log_dir="$HOME/.local/share/summon"

    echo ""
    echo "🍎 macOS LaunchAgent 설치 중..."

    mkdir -p "$log_dir"

    cat > "$plist_path" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.themagictower.summon</string>
    <key>ProgramArguments</key>
    <array>
        <string>$HOME/.local/bin/summon</string>
        <string>--config</string>
        <string>$config_file</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$log_dir/summon.log</string>
    <key>StandardErrorPath</key>
    <string>$log_dir/summon.error.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>$HOME/.local/bin:/usr/local/bin:/usr/bin:/bin</string>
    </dict>
</dict>
</plist>
EOF

    launchctl load "$plist_path" 2>/dev/null || true
    launchctl start com.themagictower.summon 2>/dev/null || true

    echo "   ✅ LaunchAgent 등록 완료: $plist_path"
    echo "   📋 관리 명령어:"
    echo "      launchctl stop com.themagictower.summon    # 중지"
    echo "      launchctl start com.themagictower.summon   # 시작"
    echo "      launchctl list | grep summon               # 상태 확인"
}

# Install Linux/WSL systemd user service
install_linux_service() {
    local config_file="$1"
    local service_dir="$HOME/.config/systemd/user"
    local service_path="$service_dir/summon.service"

    echo ""
    echo "🐧 systemd 사용자 서비스 설치 중..."

    mkdir -p "$service_dir"

    cat > "$service_path" << EOF
[Unit]
Description=Summon LLM Proxy
After=network.target

[Service]
Type=simple
ExecStart=$HOME/.local/bin/summon --config $config_file
Restart=always
RestartSec=5
Environment="PATH=$HOME/.local/bin:/usr/local/bin:/usr/bin:/bin"

[Install]
WantedBy=default.target
EOF

    systemctl --user daemon-reload 2>/dev/null || true
    systemctl --user enable summon.service 2>/dev/null || true
    systemctl --user start summon.service 2>/dev/null || true

    echo "   ✅ systemd 서비스 등록 완료: $service_path"
    echo "   📋 관리 명령어:"
    echo "      systemctl --user stop summon      # 중지"
    echo "      systemctl --user start summon     # 시작"
    echo "      systemctl --user status summon    # 상태 확인"

    if is_wsl; then
        echo ""
        echo "   💡 WSL에서 systemd를 사용하려면 /etc/wsl.conf에 다음 설정이 필요할 수 있습니다:"
        echo "      [boot]"
        echo "      systemd=true"
    fi
}

# settings.json 업데이트 (python3 → jq → 직접 생성)
update_settings_json() {
    local settings_file="$1"
    local haiku_model="$2"
    local sonnet_model="$3"

    # env 객체에 설정할 키-값 쌍 구성
    local base_url="http://127.0.0.1:18081"

    if command -v python3 &>/dev/null; then
        python3 << PYEOF
import json, os

settings_file = "$settings_file"
haiku_model = "$haiku_model"
sonnet_model = "$sonnet_model"
base_url = "$base_url"

# 기존 파일 읽기 또는 빈 객체
if os.path.exists(settings_file):
    with open(settings_file, "r") as f:
        try:
            data = json.load(f)
        except json.JSONDecodeError:
            data = {}
else:
    data = {}

# env 객체 확보
if "env" not in data or not isinstance(data["env"], dict):
    data["env"] = {}

data["env"]["ANTHROPIC_BASE_URL"] = base_url

if haiku_model:
    data["env"]["ANTHROPIC_DEFAULT_HAIKU_MODEL"] = haiku_model
if sonnet_model:
    data["env"]["ANTHROPIC_DEFAULT_SONNET_MODEL"] = sonnet_model

with open(settings_file, "w") as f:
    json.dump(data, f, indent=2, ensure_ascii=False)
    f.write("\n")
PYEOF
    elif command -v jq &>/dev/null; then
        local tmp_file
        tmp_file=$(mktemp)

        if [ -f "$settings_file" ]; then
            cp "$settings_file" "$tmp_file"
        else
            echo '{}' > "$tmp_file"
        fi

        local jq_expr=".env.ANTHROPIC_BASE_URL = \"$base_url\""
        if [ -n "$haiku_model" ]; then
            jq_expr="$jq_expr | .env.ANTHROPIC_DEFAULT_HAIKU_MODEL = \"$haiku_model\""
        fi
        if [ -n "$sonnet_model" ]; then
            jq_expr="$jq_expr | .env.ANTHROPIC_DEFAULT_SONNET_MODEL = \"$sonnet_model\""
        fi

        jq "$jq_expr" "$tmp_file" > "$settings_file"
        rm -f "$tmp_file"
    else
        # python3/jq 모두 없으면 직접 생성 (기존 파일 없는 경우만)
        if [ ! -f "$settings_file" ]; then
            local env_entries="\"ANTHROPIC_BASE_URL\": \"$base_url\""
            if [ -n "$haiku_model" ]; then
                env_entries="$env_entries,
      \"ANTHROPIC_DEFAULT_HAIKU_MODEL\": \"$haiku_model\""
            fi
            if [ -n "$sonnet_model" ]; then
                env_entries="$env_entries,
      \"ANTHROPIC_DEFAULT_SONNET_MODEL\": \"$sonnet_model\""
            fi
            cat > "$settings_file" << EOF
{
  "env": {
    $env_entries
  }
}
EOF
        else
            echo "   ⚠️  python3 또는 jq가 필요합니다. settings.json을 수동으로 수정하세요:"
            echo "      파일: $settings_file"
            echo "      추가할 env 키:"
            echo "        ANTHROPIC_BASE_URL: $base_url"
            [ -n "$haiku_model" ] && echo "        ANTHROPIC_DEFAULT_HAIKU_MODEL: $haiku_model"
            [ -n "$sonnet_model" ] && echo "        ANTHROPIC_DEFAULT_SONNET_MODEL: $sonnet_model"
            return
        fi
    fi

    echo ""
    echo "📝 Claude Code 설정이 업데이트되었습니다: $settings_file"
}

# Get latest release version
get_latest_version() {
    if [ -n "${SUMMON_VERSION:-}" ]; then
        echo "$SUMMON_VERSION"
        return
    fi
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep -o '"tag_name": "[^"]*"' | cut -d'"' -f4
}

# Main installation
main() {
    echo "🔮 Summon 설치 중..."

    PLATFORM=$(detect_platform)
    VERSION=$(get_latest_version)

    if [ -z "$VERSION" ]; then
        echo "최신 버전 정보를 가져올 수 없습니다." >&2
        exit 1
    fi

    echo "  플랫폼: $PLATFORM"
    echo "  버전: $VERSION"

    # Install binary
    mkdir -p "$INSTALL_DIR"

    if [ -n "${SUMMON_BINARY:-}" ]; then
        # 로컬 바이너리 사용 (CI/테스트용)
        echo "  로컬 바이너리: $SUMMON_BINARY"
        cp "$SUMMON_BINARY" "$INSTALL_DIR/summon"
        chmod +x "$INSTALL_DIR/summon"
    else
        # GitHub releases에서 다운로드
        TMP_DIR=$(mktemp -d)
        trap "rm -rf $TMP_DIR" EXIT

        DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/summon-$PLATFORM.tar.gz"
        echo "  다운로드: $DOWNLOAD_URL"
        curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/summon.tar.gz"

        tar -xzf "$TMP_DIR/summon.tar.gz" -C "$TMP_DIR"
        cp "$TMP_DIR/summon-$PLATFORM" "$INSTALL_DIR/summon"
        chmod +x "$INSTALL_DIR/summon"
    fi

    echo ""
    echo "✅ Summon이 설치되었습니다: $INSTALL_DIR/summon"

    # Check if in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo ""
        echo "⚠️  $INSTALL_DIR이 PATH에 없습니다. 다음을 ~/.bashrc 또는 ~/.zshrc에 추가하세요:"
        echo "   export PATH=\"$INSTALL_DIR:\$PATH\""
    fi

    # config.yaml 생성 (없을 때만)
    CONFIG_FILE="${CONFIG_FILE:-$HOME/.config/summon/config.yaml}"
    KIMI_KEY=""
    GLM_KEY=""
    HAS_ANY_KEY=false

    if [ ! -f "$CONFIG_FILE" ]; then
        mkdir -p "$(dirname "$CONFIG_FILE")"

        if [ "${SUMMON_NON_INTERACTIVE:-}" = "1" ]; then
            KIMI_KEY=""
            GLM_KEY=""
        else
            echo ""
            echo "=== API 키 설정 ==="
            echo "외부 LLM 프로바이더의 API 키를 입력하세요. (Enter로 건너뛰기)"
            echo ""

            read -rp "  Kimi API 키: " KIMI_KEY
            read -rp "  Z.AI (GLM) API 키: " GLM_KEY
        fi

        # routes 생성
        ROUTES=""
        if [ -n "$KIMI_KEY" ]; then
            HAS_ANY_KEY=true
            ROUTES="${ROUTES}
  - match: \"kimi\"
    upstream:
      url: \"https://api.kimi.com/coding\"
      auth:
        header: \"Authorization\"
        value: \"Bearer ${KIMI_KEY}\""
        fi
        if [ -n "$GLM_KEY" ]; then
            HAS_ANY_KEY=true
            ROUTES="${ROUTES}
  - match: \"glm\"
    upstream:
      url: \"https://api.z.ai/api/anthropic\"
      auth:
        header: \"x-api-key\"
        value: \"${GLM_KEY}\""
        fi

        if [ -z "$ROUTES" ]; then
            ROUTES=" []"
        fi

        cat > "$CONFIG_FILE" << EOF
server:
  host: "127.0.0.1"
  port: 18081

default:
  url: "https://api.anthropic.com"

routes:${ROUTES}
EOF
        echo ""
        echo "📝 설정 파일이 생성되었습니다: $CONFIG_FILE"
    fi

    # 모델 바인딩 (API 키가 하나라도 있을 때만)
    HAIKU_MODEL=""
    SONNET_MODEL=""
    MODEL_BINDING_SET=false

    if [ "$HAS_ANY_KEY" = true ] && [ "${SUMMON_NON_INTERACTIVE:-}" != "1" ]; then
        echo ""
        echo "=== 모델 바인딩 ==="
        echo "Claude Code의 기본 모델을 외부 프로바이더로 교체할 수 있습니다."
        echo ""

        # Haiku 모델 선택
        echo "Haiku 모델:"
        echo "  1) 기본값 유지 (Anthropic)"
        HAIKU_IDX=2
        HAIKU_KIMI_IDX=0
        HAIKU_GLM_IDX=0
        if [ -n "$KIMI_KEY" ]; then
            echo "  ${HAIKU_IDX}) Kimi"
            HAIKU_KIMI_IDX=$HAIKU_IDX
            HAIKU_IDX=$((HAIKU_IDX + 1))
        fi
        if [ -n "$GLM_KEY" ]; then
            echo "  ${HAIKU_IDX}) GLM"
            HAIKU_GLM_IDX=$HAIKU_IDX
            HAIKU_IDX=$((HAIKU_IDX + 1))
        fi
        read -rp "선택 (1): " HAIKU_CHOICE
        HAIKU_CHOICE="${HAIKU_CHOICE:-1}"

        if [ "$HAIKU_CHOICE" != "1" ]; then
            if [ "$HAIKU_CHOICE" = "$HAIKU_KIMI_IDX" ] 2>/dev/null; then
                HAIKU_MODEL="kimi-for-coding"
                MODEL_BINDING_SET=true
            elif [ "$HAIKU_CHOICE" = "$HAIKU_GLM_IDX" ] 2>/dev/null; then
                HAIKU_MODEL="glm-4.7"
                MODEL_BINDING_SET=true
            fi
        fi

        echo ""

        # Sonnet 모델 선택
        echo "Sonnet 모델:"
        echo "  1) 기본값 유지 (Anthropic)"
        SONNET_IDX=2
        SONNET_KIMI_IDX=0
        SONNET_GLM_IDX=0
        if [ -n "$KIMI_KEY" ]; then
            echo "  ${SONNET_IDX}) Kimi"
            SONNET_KIMI_IDX=$SONNET_IDX
            SONNET_IDX=$((SONNET_IDX + 1))
        fi
        if [ -n "$GLM_KEY" ]; then
            echo "  ${SONNET_IDX}) GLM"
            SONNET_GLM_IDX=$SONNET_IDX
            SONNET_IDX=$((SONNET_IDX + 1))
        fi
        read -rp "선택 (1): " SONNET_CHOICE
        SONNET_CHOICE="${SONNET_CHOICE:-1}"

        if [ "$SONNET_CHOICE" != "1" ]; then
            if [ "$SONNET_CHOICE" = "$SONNET_KIMI_IDX" ] 2>/dev/null; then
                SONNET_MODEL="kimi-for-coding"
                MODEL_BINDING_SET=true
            elif [ "$SONNET_CHOICE" = "$SONNET_GLM_IDX" ] 2>/dev/null; then
                SONNET_MODEL="glm-4.7"
                MODEL_BINDING_SET=true
            fi
        fi
    fi

    # settings.json 업데이트 (모델 바인딩 또는 API 키 설정 시)
    if [ "$HAS_ANY_KEY" = true ]; then
        SETTINGS_FILE="$HOME/.claude/settings.json"
        mkdir -p "$HOME/.claude"
        update_settings_json "$SETTINGS_FILE" "$HAIKU_MODEL" "$SONNET_MODEL"
    fi

    echo ""
    echo "🚀 사용법:"
    echo "   summon --config $CONFIG_FILE"
    echo ""

    if [ "$MODEL_BINDING_SET" = true ]; then
        echo "✅ 설정 완료! Claude Code를 재시작하면 자동으로 적용됩니다."
    else
        # WSL-specific instructions
        if is_wsl; then
            WSL_IP=$(get_wsl_host_ip)
            echo "💡 WSL 환경이 감지되었습니다!"
            echo ""
            echo "   WSL 내에서 Claude Code 사용 시:"
            echo "   ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude"
            echo ""
            echo "   Windows측에서 Claude Code 사용 시:"
            echo "   1. summon 실행: summon --config $CONFIG_FILE"
            echo "   2. Windows 터미널에서: ANTHROPIC_BASE_URL=http://$WSL_IP:18081 claude"
        else
            echo "   Claude Code 연동:"
            echo "   ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude"
        fi
    fi

    # Service installation prompt
    if [ "${SUMMON_NON_INTERACTIVE:-}" != "1" ]; then
        echo ""
        echo "🔧 백그라운드 서비스로 등록하시겠습니까?"
        echo "   이 설정은 부팅 시 자동으로 summon을 시작하고, 종료 시 자동으로 재시작합니다."
        read -rp "   서비스로 등록하시겠습니까? (y/N): " INSTALL_SERVICE

        if [[ "$INSTALL_SERVICE" =~ ^[Yy]$ ]]; then
            OS_TYPE=$(detect_os_type)
            case "$OS_TYPE" in
                macos)
                    install_macos_service "$CONFIG_FILE"
                    ;;
                linux)
                    install_linux_service "$CONFIG_FILE"
                    ;;
                *)
                    echo "   ⚠️  지원되지 않는 OS입니다. 수동으로 서비스를 등록해주세요."
                    ;;
            esac
        fi
    fi
}

main "$@"
