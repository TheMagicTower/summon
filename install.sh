#!/bin/bash
set -e

REPO="TheMagicTower/summon"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

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
    echo "   Claude Code 연동:"
    echo "   ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude"
}

main "$@"
