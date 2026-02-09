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

# Get latest release version
get_latest_version() {
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

    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT

    # Download
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/summon-$PLATFORM.tar.gz"
    echo "  다운로드: $DOWNLOAD_URL"
    curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/summon.tar.gz"

    # Extract
    tar -xzf "$TMP_DIR/summon.tar.gz" -C "$TMP_DIR"

    # Install
    mkdir -p "$INSTALL_DIR"
    cp "$TMP_DIR/summon-$PLATFORM" "$INSTALL_DIR/summon"
    chmod +x "$INSTALL_DIR/summon"

    echo ""
    echo "✅ Summon이 설치되었습니다: $INSTALL_DIR/summon"

    # Check if in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo ""
        echo "⚠️  $INSTALL_DIR이 PATH에 없습니다. 다음을 ~/.bashrc 또는 ~/.zshrc에 추가하세요:"
        echo "   export PATH=\"$INSTALL_DIR:\$PATH\""
    fi

    # Create sample config if doesn't exist
    CONFIG_FILE="${CONFIG_FILE:-$HOME/.config/summon/config.yaml}"
    if [ ! -f "$CONFIG_FILE" ]; then
        mkdir -p "$(dirname "$CONFIG_FILE")"
        cat > "$CONFIG_FILE" << 'EOF'
server:
  host: "127.0.0.1"
  port: 18081

default:
  url: "https://api.anthropic.com"

routes: []
  # 예시:
  # - match: "claude-haiku"
  #   upstream:
  #     url: "https://api.z.ai/api/anthropic"
  #     auth:
  #       header: "x-api-key"
  #       value: "${Z_AI_API_KEY}"
EOF
        echo ""
        echo "📝 샘플 설정 파일이 생성되었습니다: $CONFIG_FILE"
    fi

    echo ""
    echo "🚀 사용법:"
    echo "   summon --config $CONFIG_FILE"
    echo ""

    # WSL-specific instructions
    if is_wsl; then
        WSL_IP=$(get_wsl_host_ip)
        echo "💡 WSL 환경이 감지되었습니다!"
        echo ""
        echo "   WSL 낸에서 Claude Code 사용 시:"
        echo "   ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude"
        echo ""
        echo "   Windows측에서 Claude Code 사용 시:"
        echo "   1. summon 실행: summon --config $CONFIG_FILE"
        echo "   2. Windows 터미널에서: ANTHROPIC_BASE_URL=http://$WSL_IP:18081 claude"
    else
        echo "   Claude Code 연동:"
        echo "   ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude"
    fi

    # Service installation prompt
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
}

main "$@"
