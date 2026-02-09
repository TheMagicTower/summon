#!/bin/bash
set -e

REPO="TheMagicTower/summon"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# 언어 감지 (SUMMON_LANG 환경변수로 오버라이드 가능)
detect_language() {
    if [ -n "${SUMMON_LANG:-}" ]; then
        echo "$SUMMON_LANG"
        return
    fi
    local lang="${LANG:-${LC_ALL:-en}}"
    case "$lang" in
        ko*) echo "ko" ;;
        ja*) echo "ja" ;;
        zh*) echo "zh" ;;
        es*) echo "es" ;;
        de*) echo "de" ;;
        vi*) echo "vi" ;;
        *) echo "en" ;;
    esac
}

# 메시지 설정
set_messages() {
    case "$1" in
        ko)
            MSG_INSTALLING="🔮 Summon 설치 중..."
            MSG_PLATFORM="  플랫폼"
            MSG_VERSION="  버전"
            MSG_LOCAL_BINARY="  로컬 바이너리"
            MSG_DOWNLOADING="  다운로드"
            MSG_INSTALLED="✅ Summon이 설치되었습니다"
            MSG_PATH_WARN="⚠️  %s이 PATH에 없습니다. 다음을 ~/.bashrc 또는 ~/.zshrc에 추가하세요:"
            MSG_PROVIDER_TITLE="=== 외부 LLM 프로바이더 설정 ==="
            MSG_PROVIDER_DESC="Claude Code의 요청을 외부 LLM으로 라우팅할 수 있습니다."
            MSG_PROVIDER_SELECT="사용할 프로바이더를 선택하세요:"
            MSG_PROVIDER_ALL="모두 설정"
            MSG_PROVIDER_SKIP="건너뛰기"
            MSG_SELECT="선택"
            MSG_API_KEY_KIMI="  Kimi API 키: "
            MSG_API_KEY_GLM="  Z.AI (GLM) API 키: "
            MSG_CONFIG_CREATED="📝 설정 파일이 생성되었습니다"
            MSG_MODEL_TITLE="=== 모델 바인딩 ==="
            MSG_MODEL_DESC="Claude Code의 기본 모델을 외부 프로바이더로 교체할 수 있습니다."
            MSG_MODEL_HAIKU="Haiku 모델:"
            MSG_MODEL_SONNET="Sonnet 모델:"
            MSG_MODEL_DEFAULT="기본값 유지 (Anthropic)"
            MSG_SETTINGS_UPDATED="📝 Claude Code 설정이 업데이트되었습니다"
            MSG_SETTINGS_MANUAL="⚠️  python3 또는 jq가 필요합니다. settings.json을 수동으로 수정하세요:"
            MSG_SETTINGS_FILE="파일"
            MSG_SETTINGS_ADD_KEYS="추가할 env 키:"
            MSG_USAGE="🚀 사용법:"
            MSG_SETUP_COMPLETE="✅ 설정 완료! Claude Code를 재시작하면 자동으로 적용됩니다."
            MSG_CLAUDE_INTEGRATION="   Claude Code 연동:"
            MSG_WSL_DETECTED="💡 WSL 환경이 감지되었습니다!"
            MSG_WSL_INSIDE="   WSL 내에서 Claude Code 사용 시:"
            MSG_WSL_WINDOWS="   Windows측에서 Claude Code 사용 시:"
            MSG_WSL_STEP1="   1. summon 실행:"
            MSG_WSL_STEP2="   2. Windows 터미널에서:"
            MSG_SERVICE_TITLE="🔧 백그라운드 서비스로 등록하시겠습니까?"
            MSG_SERVICE_DESC="   이 설정은 부팅 시 자동으로 summon을 시작하고, 종료 시 자동으로 재시작합니다."
            MSG_SERVICE_PROMPT="   서비스로 등록하시겠습니까? (y/N): "
            MSG_SERVICE_UNSUPPORTED="   ⚠️  지원되지 않는 OS입니다. 수동으로 서비스를 등록해주세요."
            MSG_MACOS_INSTALLING="🍎 macOS LaunchAgent 설치 중..."
            MSG_MACOS_DONE="   ✅ LaunchAgent 등록 완료"
            MSG_MACOS_COMMANDS="   📋 관리 명령어:"
            MSG_LINUX_INSTALLING="🐧 systemd 사용자 서비스 설치 중..."
            MSG_LINUX_DONE="   ✅ systemd 서비스 등록 완료"
            MSG_LINUX_COMMANDS="   📋 관리 명령어:"
            MSG_WSL_SYSTEMD="   💡 WSL에서 systemd를 사용하려면 /etc/wsl.conf에 다음 설정이 필요할 수 있습니다:"
            MSG_LINUX_NO_USER_SESSION="   ⚠️  사용자 systemd 세션을 사용할 수 없습니다. 대안을 시도합니다..."
            MSG_LINUX_LINGER_TRYING="   🔄 loginctl enable-linger 시도 중..."
            MSG_LINUX_SYSTEM_INSTALLING="🐧 시스템 레벨 systemd 서비스로 설치 중..."
            MSG_LINUX_SUDO_REQUIRED="   ⚠️  시스템 서비스 설치에 sudo 권한이 필요합니다."
            MSG_VERSION_ERROR="최신 버전 정보를 가져올 수 없습니다."
            ;;
        ja)
            MSG_INSTALLING="🔮 Summon をインストール中..."
            MSG_PLATFORM="  プラットフォーム"
            MSG_VERSION="  バージョン"
            MSG_LOCAL_BINARY="  ローカルバイナリ"
            MSG_DOWNLOADING="  ダウンロード"
            MSG_INSTALLED="✅ Summon がインストールされました"
            MSG_PATH_WARN="⚠️  %s が PATH にありません。以下を ~/.bashrc または ~/.zshrc に追加してください:"
            MSG_PROVIDER_TITLE="=== 外部 LLM プロバイダー設定 ==="
            MSG_PROVIDER_DESC="Claude Code のリクエストを外部 LLM にルーティングできます。"
            MSG_PROVIDER_SELECT="使用するプロバイダーを選択してください:"
            MSG_PROVIDER_ALL="すべて設定"
            MSG_PROVIDER_SKIP="スキップ"
            MSG_SELECT="選択"
            MSG_API_KEY_KIMI="  Kimi API キー: "
            MSG_API_KEY_GLM="  Z.AI (GLM) API キー: "
            MSG_CONFIG_CREATED="📝 設定ファイルが作成されました"
            MSG_MODEL_TITLE="=== モデルバインディング ==="
            MSG_MODEL_DESC="Claude Code のデフォルトモデルを外部プロバイダーに変更できます。"
            MSG_MODEL_HAIKU="Haiku モデル:"
            MSG_MODEL_SONNET="Sonnet モデル:"
            MSG_MODEL_DEFAULT="デフォルトを維持 (Anthropic)"
            MSG_SETTINGS_UPDATED="📝 Claude Code の設定が更新されました"
            MSG_SETTINGS_MANUAL="⚠️  python3 または jq が必要です。settings.json を手動で編集してください:"
            MSG_SETTINGS_FILE="ファイル"
            MSG_SETTINGS_ADD_KEYS="追加する env キー:"
            MSG_USAGE="🚀 使い方:"
            MSG_SETUP_COMPLETE="✅ 設定完了！Claude Code を再起動すると自動的に適用されます。"
            MSG_CLAUDE_INTEGRATION="   Claude Code 連携:"
            MSG_WSL_DETECTED="💡 WSL 環境が検出されました！"
            MSG_WSL_INSIDE="   WSL 内で Claude Code を使用する場合:"
            MSG_WSL_WINDOWS="   Windows 側で Claude Code を使用する場合:"
            MSG_WSL_STEP1="   1. summon を実行:"
            MSG_WSL_STEP2="   2. Windows ターミナルで:"
            MSG_SERVICE_TITLE="🔧 バックグラウンドサービスとして登録しますか？"
            MSG_SERVICE_DESC="   この設定により、起動時に自動的に summon を開始し、終了時に自動的に再起動します。"
            MSG_SERVICE_PROMPT="   サービスとして登録しますか？ (y/N): "
            MSG_SERVICE_UNSUPPORTED="   ⚠️  サポートされていない OS です。手動でサービスを登録してください。"
            MSG_MACOS_INSTALLING="🍎 macOS LaunchAgent をインストール中..."
            MSG_MACOS_DONE="   ✅ LaunchAgent 登録完了"
            MSG_MACOS_COMMANDS="   📋 管理コマンド:"
            MSG_LINUX_INSTALLING="🐧 systemd ユーザーサービスをインストール中..."
            MSG_LINUX_DONE="   ✅ systemd サービス登録完了"
            MSG_LINUX_COMMANDS="   📋 管理コマンド:"
            MSG_WSL_SYSTEMD="   💡 WSL で systemd を使用するには /etc/wsl.conf に以下の設定が必要な場合があります:"
            MSG_LINUX_NO_USER_SESSION="   ⚠️  ユーザー systemd セッションが利用できません。代替方法を試みます..."
            MSG_LINUX_LINGER_TRYING="   🔄 loginctl enable-linger を試行中..."
            MSG_LINUX_SYSTEM_INSTALLING="🐧 システムレベル systemd サービスとしてインストール中..."
            MSG_LINUX_SUDO_REQUIRED="   ⚠️  システムサービスのインストールには sudo 権限が必要です。"
            MSG_VERSION_ERROR="最新バージョン情報を取得できません。"
            ;;
        zh)
            MSG_INSTALLING="🔮 正在安装 Summon..."
            MSG_PLATFORM="  平台"
            MSG_VERSION="  版本"
            MSG_LOCAL_BINARY="  本地二进制文件"
            MSG_DOWNLOADING="  下载"
            MSG_INSTALLED="✅ Summon 已安装"
            MSG_PATH_WARN="⚠️  %s 不在 PATH 中。请将以下内容添加到 ~/.bashrc 或 ~/.zshrc:"
            MSG_PROVIDER_TITLE="=== 外部 LLM 提供商设置 ==="
            MSG_PROVIDER_DESC="可以将 Claude Code 的请求路由到外部 LLM。"
            MSG_PROVIDER_SELECT="请选择要使用的提供商:"
            MSG_PROVIDER_ALL="全部设置"
            MSG_PROVIDER_SKIP="跳过"
            MSG_SELECT="选择"
            MSG_API_KEY_KIMI="  Kimi API 密钥: "
            MSG_API_KEY_GLM="  Z.AI (GLM) API 密钥: "
            MSG_CONFIG_CREATED="📝 配置文件已创建"
            MSG_MODEL_TITLE="=== 模型绑定 ==="
            MSG_MODEL_DESC="可以将 Claude Code 的默认模型替换为外部提供商。"
            MSG_MODEL_HAIKU="Haiku 模型:"
            MSG_MODEL_SONNET="Sonnet 模型:"
            MSG_MODEL_DEFAULT="保持默认 (Anthropic)"
            MSG_SETTINGS_UPDATED="📝 Claude Code 设置已更新"
            MSG_SETTINGS_MANUAL="⚠️  需要 python3 或 jq。请手动编辑 settings.json:"
            MSG_SETTINGS_FILE="文件"
            MSG_SETTINGS_ADD_KEYS="要添加的 env 键:"
            MSG_USAGE="🚀 用法:"
            MSG_SETUP_COMPLETE="✅ 设置完成！重启 Claude Code 后自动生效。"
            MSG_CLAUDE_INTEGRATION="   Claude Code 集成:"
            MSG_WSL_DETECTED="💡 检测到 WSL 环境！"
            MSG_WSL_INSIDE="   在 WSL 中使用 Claude Code:"
            MSG_WSL_WINDOWS="   在 Windows 端使用 Claude Code:"
            MSG_WSL_STEP1="   1. 运行 summon:"
            MSG_WSL_STEP2="   2. 在 Windows 终端中:"
            MSG_SERVICE_TITLE="🔧 是否注册为后台服务？"
            MSG_SERVICE_DESC="   此设置将在启动时自动启动 summon，并在退出时自动重启。"
            MSG_SERVICE_PROMPT="   是否注册为服务？ (y/N): "
            MSG_SERVICE_UNSUPPORTED="   ⚠️  不支持的操作系统。请手动注册服务。"
            MSG_MACOS_INSTALLING="🍎 正在安装 macOS LaunchAgent..."
            MSG_MACOS_DONE="   ✅ LaunchAgent 注册完成"
            MSG_MACOS_COMMANDS="   📋 管理命令:"
            MSG_LINUX_INSTALLING="🐧 正在安装 systemd 用户服务..."
            MSG_LINUX_DONE="   ✅ systemd 服务注册完成"
            MSG_LINUX_COMMANDS="   📋 管理命令:"
            MSG_WSL_SYSTEMD="   💡 在 WSL 中使用 systemd 可能需要在 /etc/wsl.conf 中添加以下设置:"
            MSG_LINUX_NO_USER_SESSION="   ⚠️  无法使用用户 systemd 会话。正在尝试替代方案..."
            MSG_LINUX_LINGER_TRYING="   🔄 正在尝试 loginctl enable-linger..."
            MSG_LINUX_SYSTEM_INSTALLING="🐧 正在安装系统级 systemd 服务..."
            MSG_LINUX_SUDO_REQUIRED="   ⚠️  安装系统服务需要 sudo 权限。"
            MSG_VERSION_ERROR="无法获取最新版本信息。"
            ;;
        es)
            MSG_INSTALLING="🔮 Instalando Summon..."
            MSG_PLATFORM="  Plataforma"
            MSG_VERSION="  Versión"
            MSG_LOCAL_BINARY="  Binario local"
            MSG_DOWNLOADING="  Descarga"
            MSG_INSTALLED="✅ Summon ha sido instalado"
            MSG_PATH_WARN="⚠️  %s no está en PATH. Añade lo siguiente a ~/.bashrc o ~/.zshrc:"
            MSG_PROVIDER_TITLE="=== Configuración de proveedores LLM externos ==="
            MSG_PROVIDER_DESC="Puedes enrutar las solicitudes de Claude Code a un LLM externo."
            MSG_PROVIDER_SELECT="Selecciona el proveedor que deseas usar:"
            MSG_PROVIDER_ALL="Configurar todos"
            MSG_PROVIDER_SKIP="Omitir"
            MSG_SELECT="Selección"
            MSG_API_KEY_KIMI="  Clave API de Kimi: "
            MSG_API_KEY_GLM="  Clave API de Z.AI (GLM): "
            MSG_CONFIG_CREATED="📝 Archivo de configuración creado"
            MSG_MODEL_TITLE="=== Enlace de modelos ==="
            MSG_MODEL_DESC="Puedes reemplazar los modelos predeterminados de Claude Code con proveedores externos."
            MSG_MODEL_HAIKU="Modelo Haiku:"
            MSG_MODEL_SONNET="Modelo Sonnet:"
            MSG_MODEL_DEFAULT="Mantener predeterminado (Anthropic)"
            MSG_SETTINGS_UPDATED="📝 La configuración de Claude Code ha sido actualizada"
            MSG_SETTINGS_MANUAL="⚠️  Se requiere python3 o jq. Edita settings.json manualmente:"
            MSG_SETTINGS_FILE="Archivo"
            MSG_SETTINGS_ADD_KEYS="Claves env a añadir:"
            MSG_USAGE="🚀 Uso:"
            MSG_SETUP_COMPLETE="✅ ¡Configuración completada! Se aplicará automáticamente al reiniciar Claude Code."
            MSG_CLAUDE_INTEGRATION="   Integración con Claude Code:"
            MSG_WSL_DETECTED="💡 ¡Entorno WSL detectado!"
            MSG_WSL_INSIDE="   Para usar Claude Code dentro de WSL:"
            MSG_WSL_WINDOWS="   Para usar Claude Code desde Windows:"
            MSG_WSL_STEP1="   1. Ejecutar summon:"
            MSG_WSL_STEP2="   2. En la terminal de Windows:"
            MSG_SERVICE_TITLE="🔧 ¿Deseas registrarlo como servicio en segundo plano?"
            MSG_SERVICE_DESC="   Esta configuración iniciará summon automáticamente al arrancar y lo reiniciará si se detiene."
            MSG_SERVICE_PROMPT="   ¿Registrar como servicio? (y/N): "
            MSG_SERVICE_UNSUPPORTED="   ⚠️  SO no soportado. Registra el servicio manualmente."
            MSG_MACOS_INSTALLING="🍎 Instalando macOS LaunchAgent..."
            MSG_MACOS_DONE="   ✅ LaunchAgent registrado"
            MSG_MACOS_COMMANDS="   📋 Comandos de gestión:"
            MSG_LINUX_INSTALLING="🐧 Instalando servicio de usuario systemd..."
            MSG_LINUX_DONE="   ✅ Servicio systemd registrado"
            MSG_LINUX_COMMANDS="   📋 Comandos de gestión:"
            MSG_WSL_SYSTEMD="   💡 Para usar systemd en WSL, puede que necesites añadir lo siguiente en /etc/wsl.conf:"
            MSG_LINUX_NO_USER_SESSION="   ⚠️  No se puede usar la sesión systemd de usuario. Intentando alternativas..."
            MSG_LINUX_LINGER_TRYING="   🔄 Intentando loginctl enable-linger..."
            MSG_LINUX_SYSTEM_INSTALLING="🐧 Instalando como servicio systemd a nivel de sistema..."
            MSG_LINUX_SUDO_REQUIRED="   ⚠️  Se requieren permisos sudo para instalar el servicio del sistema."
            MSG_VERSION_ERROR="No se pudo obtener la información de la última versión."
            ;;
        de)
            MSG_INSTALLING="🔮 Summon wird installiert..."
            MSG_PLATFORM="  Plattform"
            MSG_VERSION="  Version"
            MSG_LOCAL_BINARY="  Lokale Binärdatei"
            MSG_DOWNLOADING="  Download"
            MSG_INSTALLED="✅ Summon wurde installiert"
            MSG_PATH_WARN="⚠️  %s ist nicht im PATH. Füge Folgendes zu ~/.bashrc oder ~/.zshrc hinzu:"
            MSG_PROVIDER_TITLE="=== Externe LLM-Anbieter einrichten ==="
            MSG_PROVIDER_DESC="Du kannst Claude Code-Anfragen an externe LLMs weiterleiten."
            MSG_PROVIDER_SELECT="Wähle den gewünschten Anbieter:"
            MSG_PROVIDER_ALL="Alle einrichten"
            MSG_PROVIDER_SKIP="Überspringen"
            MSG_SELECT="Auswahl"
            MSG_API_KEY_KIMI="  Kimi API-Schlüssel: "
            MSG_API_KEY_GLM="  Z.AI (GLM) API-Schlüssel: "
            MSG_CONFIG_CREATED="📝 Konfigurationsdatei erstellt"
            MSG_MODEL_TITLE="=== Modellbindung ==="
            MSG_MODEL_DESC="Du kannst die Standardmodelle von Claude Code durch externe Anbieter ersetzen."
            MSG_MODEL_HAIKU="Haiku-Modell:"
            MSG_MODEL_SONNET="Sonnet-Modell:"
            MSG_MODEL_DEFAULT="Standard beibehalten (Anthropic)"
            MSG_SETTINGS_UPDATED="📝 Claude Code-Einstellungen wurden aktualisiert"
            MSG_SETTINGS_MANUAL="⚠️  python3 oder jq erforderlich. Bearbeite settings.json manuell:"
            MSG_SETTINGS_FILE="Datei"
            MSG_SETTINGS_ADD_KEYS="Hinzuzufügende env-Schlüssel:"
            MSG_USAGE="🚀 Verwendung:"
            MSG_SETUP_COMPLETE="✅ Einrichtung abgeschlossen! Wird nach Neustart von Claude Code automatisch angewendet."
            MSG_CLAUDE_INTEGRATION="   Claude Code-Integration:"
            MSG_WSL_DETECTED="💡 WSL-Umgebung erkannt!"
            MSG_WSL_INSIDE="   Claude Code in WSL verwenden:"
            MSG_WSL_WINDOWS="   Claude Code von Windows aus verwenden:"
            MSG_WSL_STEP1="   1. summon ausführen:"
            MSG_WSL_STEP2="   2. Im Windows-Terminal:"
            MSG_SERVICE_TITLE="🔧 Als Hintergrunddienst registrieren?"
            MSG_SERVICE_DESC="   Diese Einstellung startet summon automatisch beim Hochfahren und startet es bei Beendigung neu."
            MSG_SERVICE_PROMPT="   Als Dienst registrieren? (y/N): "
            MSG_SERVICE_UNSUPPORTED="   ⚠️  Nicht unterstütztes Betriebssystem. Registriere den Dienst manuell."
            MSG_MACOS_INSTALLING="🍎 macOS LaunchAgent wird installiert..."
            MSG_MACOS_DONE="   ✅ LaunchAgent registriert"
            MSG_MACOS_COMMANDS="   📋 Verwaltungsbefehle:"
            MSG_LINUX_INSTALLING="🐧 systemd-Benutzerdienst wird installiert..."
            MSG_LINUX_DONE="   ✅ systemd-Dienst registriert"
            MSG_LINUX_COMMANDS="   📋 Verwaltungsbefehle:"
            MSG_WSL_SYSTEMD="   💡 Für systemd in WSL muss möglicherweise Folgendes in /etc/wsl.conf eingetragen werden:"
            MSG_LINUX_NO_USER_SESSION="   ⚠️  Benutzer-systemd-Sitzung nicht verfügbar. Versuche Alternativen..."
            MSG_LINUX_LINGER_TRYING="   🔄 Versuche loginctl enable-linger..."
            MSG_LINUX_SYSTEM_INSTALLING="🐧 Installiere als systemd-Systemdienst..."
            MSG_LINUX_SUDO_REQUIRED="   ⚠️  Für die Installation des Systemdienstes werden sudo-Rechte benötigt."
            MSG_VERSION_ERROR="Die neueste Version konnte nicht abgerufen werden."
            ;;
        vi)
            MSG_INSTALLING="🔮 Đang cài đặt Summon..."
            MSG_PLATFORM="  Nền tảng"
            MSG_VERSION="  Phiên bản"
            MSG_LOCAL_BINARY="  Binary cục bộ"
            MSG_DOWNLOADING="  Tải xuống"
            MSG_INSTALLED="✅ Summon đã được cài đặt"
            MSG_PATH_WARN="⚠️  %s không có trong PATH. Thêm dòng sau vào ~/.bashrc hoặc ~/.zshrc:"
            MSG_PROVIDER_TITLE="=== Cấu hình nhà cung cấp LLM bên ngoài ==="
            MSG_PROVIDER_DESC="Bạn có thể định tuyến yêu cầu của Claude Code đến LLM bên ngoài."
            MSG_PROVIDER_SELECT="Chọn nhà cung cấp bạn muốn sử dụng:"
            MSG_PROVIDER_ALL="Cấu hình tất cả"
            MSG_PROVIDER_SKIP="Bỏ qua"
            MSG_SELECT="Chọn"
            MSG_API_KEY_KIMI="  Khóa API Kimi: "
            MSG_API_KEY_GLM="  Khóa API Z.AI (GLM): "
            MSG_CONFIG_CREATED="📝 Tệp cấu hình đã được tạo"
            MSG_MODEL_TITLE="=== Liên kết mô hình ==="
            MSG_MODEL_DESC="Bạn có thể thay thế mô hình mặc định của Claude Code bằng nhà cung cấp bên ngoài."
            MSG_MODEL_HAIKU="Mô hình Haiku:"
            MSG_MODEL_SONNET="Mô hình Sonnet:"
            MSG_MODEL_DEFAULT="Giữ mặc định (Anthropic)"
            MSG_SETTINGS_UPDATED="📝 Cấu hình Claude Code đã được cập nhật"
            MSG_SETTINGS_MANUAL="⚠️  Cần python3 hoặc jq. Vui lòng chỉnh sửa settings.json thủ công:"
            MSG_SETTINGS_FILE="Tệp"
            MSG_SETTINGS_ADD_KEYS="Các khóa env cần thêm:"
            MSG_USAGE="🚀 Cách dùng:"
            MSG_SETUP_COMPLETE="✅ Cấu hình hoàn tất! Sẽ tự động áp dụng khi khởi động lại Claude Code."
            MSG_CLAUDE_INTEGRATION="   Tích hợp Claude Code:"
            MSG_WSL_DETECTED="💡 Phát hiện môi trường WSL!"
            MSG_WSL_INSIDE="   Sử dụng Claude Code trong WSL:"
            MSG_WSL_WINDOWS="   Sử dụng Claude Code từ Windows:"
            MSG_WSL_STEP1="   1. Chạy summon:"
            MSG_WSL_STEP2="   2. Trong terminal Windows:"
            MSG_SERVICE_TITLE="🔧 Bạn có muốn đăng ký làm dịch vụ nền không?"
            MSG_SERVICE_DESC="   Cấu hình này sẽ tự động khởi động summon khi boot và tự động khởi động lại khi thoát."
            MSG_SERVICE_PROMPT="   Đăng ký làm dịch vụ? (y/N): "
            MSG_SERVICE_UNSUPPORTED="   ⚠️  Hệ điều hành không được hỗ trợ. Vui lòng đăng ký dịch vụ thủ công."
            MSG_MACOS_INSTALLING="🍎 Đang cài đặt macOS LaunchAgent..."
            MSG_MACOS_DONE="   ✅ Đã đăng ký LaunchAgent"
            MSG_MACOS_COMMANDS="   📋 Lệnh quản lý:"
            MSG_LINUX_INSTALLING="🐧 Đang cài đặt dịch vụ người dùng systemd..."
            MSG_LINUX_DONE="   ✅ Đã đăng ký dịch vụ systemd"
            MSG_LINUX_COMMANDS="   📋 Lệnh quản lý:"
            MSG_WSL_SYSTEMD="   💡 Để sử dụng systemd trong WSL, bạn có thể cần thêm cấu hình sau vào /etc/wsl.conf:"
            MSG_LINUX_NO_USER_SESSION="   ⚠️  Không thể sử dụng phiên systemd người dùng. Đang thử phương án thay thế..."
            MSG_LINUX_LINGER_TRYING="   🔄 Đang thử loginctl enable-linger..."
            MSG_LINUX_SYSTEM_INSTALLING="🐧 Đang cài đặt dịch vụ systemd cấp hệ thống..."
            MSG_LINUX_SUDO_REQUIRED="   ⚠️  Cần quyền sudo để cài đặt dịch vụ hệ thống."
            MSG_VERSION_ERROR="Không thể lấy thông tin phiên bản mới nhất."
            ;;
        en|*)
            MSG_INSTALLING="🔮 Installing Summon..."
            MSG_PLATFORM="  Platform"
            MSG_VERSION="  Version"
            MSG_LOCAL_BINARY="  Local binary"
            MSG_DOWNLOADING="  Download"
            MSG_INSTALLED="✅ Summon has been installed"
            MSG_PATH_WARN="⚠️  %s is not in PATH. Add the following to ~/.bashrc or ~/.zshrc:"
            MSG_PROVIDER_TITLE="=== External LLM Provider Setup ==="
            MSG_PROVIDER_DESC="You can route Claude Code requests to external LLMs."
            MSG_PROVIDER_SELECT="Select a provider to use:"
            MSG_PROVIDER_ALL="Set up all"
            MSG_PROVIDER_SKIP="Skip"
            MSG_SELECT="Select"
            MSG_API_KEY_KIMI="  Kimi API key: "
            MSG_API_KEY_GLM="  Z.AI (GLM) API key: "
            MSG_CONFIG_CREATED="📝 Configuration file created"
            MSG_MODEL_TITLE="=== Model Binding ==="
            MSG_MODEL_DESC="You can replace Claude Code's default models with external providers."
            MSG_MODEL_HAIKU="Haiku model:"
            MSG_MODEL_SONNET="Sonnet model:"
            MSG_MODEL_DEFAULT="Keep default (Anthropic)"
            MSG_SETTINGS_UPDATED="📝 Claude Code settings have been updated"
            MSG_SETTINGS_MANUAL="⚠️  python3 or jq is required. Please edit settings.json manually:"
            MSG_SETTINGS_FILE="File"
            MSG_SETTINGS_ADD_KEYS="env keys to add:"
            MSG_USAGE="🚀 Usage:"
            MSG_SETUP_COMPLETE="✅ Setup complete! Changes will be applied when Claude Code restarts."
            MSG_CLAUDE_INTEGRATION="   Claude Code integration:"
            MSG_WSL_DETECTED="💡 WSL environment detected!"
            MSG_WSL_INSIDE="   Using Claude Code inside WSL:"
            MSG_WSL_WINDOWS="   Using Claude Code from Windows:"
            MSG_WSL_STEP1="   1. Run summon:"
            MSG_WSL_STEP2="   2. In Windows terminal:"
            MSG_SERVICE_TITLE="🔧 Register as a background service?"
            MSG_SERVICE_DESC="   This will auto-start summon on boot and restart it if it stops."
            MSG_SERVICE_PROMPT="   Register as service? (y/N): "
            MSG_SERVICE_UNSUPPORTED="   ⚠️  Unsupported OS. Please register the service manually."
            MSG_MACOS_INSTALLING="🍎 Installing macOS LaunchAgent..."
            MSG_MACOS_DONE="   ✅ LaunchAgent registered"
            MSG_MACOS_COMMANDS="   📋 Management commands:"
            MSG_LINUX_INSTALLING="🐧 Installing systemd user service..."
            MSG_LINUX_DONE="   ✅ systemd service registered"
            MSG_LINUX_COMMANDS="   📋 Management commands:"
            MSG_WSL_SYSTEMD="   💡 To use systemd in WSL, you may need to add the following to /etc/wsl.conf:"
            MSG_LINUX_NO_USER_SESSION="   ⚠️  User systemd session unavailable. Trying alternatives..."
            MSG_LINUX_LINGER_TRYING="   🔄 Trying loginctl enable-linger..."
            MSG_LINUX_SYSTEM_INSTALLING="🐧 Installing as system-level systemd service..."
            MSG_LINUX_SUDO_REQUIRED="   ⚠️  sudo privileges required to install system service."
            MSG_VERSION_ERROR="Could not fetch the latest version information."
            ;;
    esac
}

# 언어 감지 및 메시지 초기화
LANG_CODE=$(detect_language)
set_messages "$LANG_CODE"

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

# 사용자 systemd 세션 사용 가능 여부 감지
can_use_user_systemd() {
    systemctl --user status >/dev/null 2>&1
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
    echo "$MSG_MACOS_INSTALLING"

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

    echo "$MSG_MACOS_DONE: $plist_path"
    echo "$MSG_MACOS_COMMANDS"
    echo "      launchctl stop com.themagictower.summon"
    echo "      launchctl start com.themagictower.summon"
    echo "      launchctl list | grep summon"
}

# Install Linux/WSL systemd user service
install_linux_service() {
    local config_file="$1"

    if can_use_user_systemd; then
        install_linux_user_service "$config_file"
        return
    fi

    # 사용자 세션 불가 → linger 활성화 시도
    echo ""
    echo "$MSG_LINUX_NO_USER_SESSION"
    echo "$MSG_LINUX_LINGER_TRYING"

    loginctl enable-linger "$(whoami)" 2>/dev/null || true
    export XDG_RUNTIME_DIR="/run/user/$(id -u)"

    if can_use_user_systemd; then
        install_linux_user_service "$config_file"
        return
    fi

    # linger로도 불가 → 시스템 서비스 폴백
    install_linux_system_service "$config_file"
}

# systemd 사용자 서비스 설치 (내부용)
install_linux_user_service() {
    local config_file="$1"
    local service_dir="$HOME/.config/systemd/user"
    local service_path="$service_dir/summon.service"

    echo ""
    echo "$MSG_LINUX_INSTALLING"

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

    echo "$MSG_LINUX_DONE: $service_path"
    echo "$MSG_LINUX_COMMANDS"
    echo "      systemctl --user stop summon"
    echo "      systemctl --user start summon"
    echo "      systemctl --user status summon"

    if is_wsl; then
        echo ""
        echo "$MSG_WSL_SYSTEMD"
        echo "      [boot]"
        echo "      systemd=true"
    fi
}

# systemd 시스템 레벨 서비스 설치 (헤드리스 서버 폴백)
install_linux_system_service() {
    local config_file="$1"
    local service_path="/etc/systemd/system/summon.service"
    local current_user
    local current_group
    current_user="$(whoami)"
    current_group="$(id -gn)"

    echo ""
    echo "$MSG_LINUX_SYSTEM_INSTALLING"

    if ! command -v sudo &>/dev/null || ! sudo -n true 2>/dev/null; then
        echo "$MSG_LINUX_SUDO_REQUIRED"
    fi

    sudo tee "$service_path" > /dev/null << EOF
[Unit]
Description=Summon LLM Proxy
After=network.target

[Service]
Type=simple
User=$current_user
Group=$current_group
ExecStart=$HOME/.local/bin/summon --config $config_file
Restart=always
RestartSec=5
Environment="PATH=$HOME/.local/bin:/usr/local/bin:/usr/bin:/bin"

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl daemon-reload
    sudo systemctl enable summon.service
    sudo systemctl start summon.service

    echo "$MSG_LINUX_DONE: $service_path"
    echo "$MSG_LINUX_COMMANDS"
    echo "      sudo systemctl stop summon"
    echo "      sudo systemctl start summon"
    echo "      sudo systemctl status summon"
    echo "      journalctl -u summon -f"
}

# settings.json 업데이트 (python3 → jq → 직접 생성)
update_settings_json() {
    local settings_file="$1"
    local haiku_model="$2"
    local sonnet_model="$3"

    local base_url="http://127.0.0.1:18081"

    if command -v python3 &>/dev/null; then
        python3 << PYEOF
import json, os

settings_file = "$settings_file"
haiku_model = "$haiku_model"
sonnet_model = "$sonnet_model"
base_url = "$base_url"

if os.path.exists(settings_file):
    with open(settings_file, "r") as f:
        try:
            data = json.load(f)
        except json.JSONDecodeError:
            data = {}
else:
    data = {}

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
            echo "   $MSG_SETTINGS_MANUAL"
            echo "      $MSG_SETTINGS_FILE: $settings_file"
            echo "      $MSG_SETTINGS_ADD_KEYS"
            echo "        ANTHROPIC_BASE_URL: $base_url"
            [ -n "$haiku_model" ] && echo "        ANTHROPIC_DEFAULT_HAIKU_MODEL: $haiku_model"
            [ -n "$sonnet_model" ] && echo "        ANTHROPIC_DEFAULT_SONNET_MODEL: $sonnet_model"
            return
        fi
    fi

    echo ""
    echo "$MSG_SETTINGS_UPDATED: $settings_file"
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
    echo "$MSG_INSTALLING"

    PLATFORM=$(detect_platform)
    VERSION=$(get_latest_version)

    if [ -z "$VERSION" ]; then
        echo "$MSG_VERSION_ERROR" >&2
        exit 1
    fi

    echo "$MSG_PLATFORM: $PLATFORM"
    echo "$MSG_VERSION: $VERSION"

    # Install binary
    mkdir -p "$INSTALL_DIR"

    if [ -n "${SUMMON_BINARY:-}" ]; then
        echo "$MSG_LOCAL_BINARY: $SUMMON_BINARY"
        cp "$SUMMON_BINARY" "$INSTALL_DIR/summon"
        chmod +x "$INSTALL_DIR/summon"
    else
        TMP_DIR=$(mktemp -d)
        trap "rm -rf $TMP_DIR" EXIT

        DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/summon-$PLATFORM.tar.gz"
        echo "$MSG_DOWNLOADING: $DOWNLOAD_URL"
        curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/summon.tar.gz"

        tar -xzf "$TMP_DIR/summon.tar.gz" -C "$TMP_DIR"
        cp "$TMP_DIR/summon-$PLATFORM" "$INSTALL_DIR/summon"
        chmod +x "$INSTALL_DIR/summon"
    fi

    echo ""
    echo "$MSG_INSTALLED: $INSTALL_DIR/summon"

    # Check if in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo ""
        printf "$MSG_PATH_WARN\n" "$INSTALL_DIR"
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
            echo "$MSG_PROVIDER_TITLE"
            echo "$MSG_PROVIDER_DESC"
            echo "$MSG_PROVIDER_SELECT"
            echo ""
            echo "  1) Kimi (Moonshot AI - kimi.com)"
            echo "  2) Z.AI (GLM - z.ai)"
            echo "  3) $MSG_PROVIDER_ALL"
            echo "  4) $MSG_PROVIDER_SKIP"
            echo ""
            read -rp "$MSG_SELECT (4): " PROVIDER_CHOICE < /dev/tty
            PROVIDER_CHOICE="${PROVIDER_CHOICE:-4}"

            case "$PROVIDER_CHOICE" in
                1)
                    read -rp "$MSG_API_KEY_KIMI" KIMI_KEY < /dev/tty
                    ;;
                2)
                    read -rp "$MSG_API_KEY_GLM" GLM_KEY < /dev/tty
                    ;;
                3)
                    read -rp "$MSG_API_KEY_KIMI" KIMI_KEY < /dev/tty
                    read -rp "$MSG_API_KEY_GLM" GLM_KEY < /dev/tty
                    ;;
                *)
                    KIMI_KEY=""
                    GLM_KEY=""
                    ;;
            esac
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
        echo "$MSG_CONFIG_CREATED: $CONFIG_FILE"
    fi

    # 모델 바인딩 (API 키가 하나라도 있을 때만)
    HAIKU_MODEL=""
    SONNET_MODEL=""
    MODEL_BINDING_SET=false

    if [ "$HAS_ANY_KEY" = true ] && [ "${SUMMON_NON_INTERACTIVE:-}" != "1" ]; then
        echo ""
        echo "$MSG_MODEL_TITLE"
        echo "$MSG_MODEL_DESC"
        echo ""

        # Haiku 모델 선택
        echo "$MSG_MODEL_HAIKU"
        echo "  1) $MSG_MODEL_DEFAULT"
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
        read -rp "$MSG_SELECT (1): " HAIKU_CHOICE < /dev/tty
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
        echo "$MSG_MODEL_SONNET"
        echo "  1) $MSG_MODEL_DEFAULT"
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
        read -rp "$MSG_SELECT (1): " SONNET_CHOICE < /dev/tty
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
    echo "$MSG_USAGE"
    echo "   summon --config $CONFIG_FILE"
    echo ""

    if [ "$MODEL_BINDING_SET" = true ]; then
        echo "$MSG_SETUP_COMPLETE"
    else
        if is_wsl; then
            WSL_IP=$(get_wsl_host_ip)
            echo "$MSG_WSL_DETECTED"
            echo ""
            echo "$MSG_WSL_INSIDE"
            echo "   ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude"
            echo ""
            echo "$MSG_WSL_WINDOWS"
            echo "$MSG_WSL_STEP1 summon --config $CONFIG_FILE"
            echo "$MSG_WSL_STEP2 ANTHROPIC_BASE_URL=http://$WSL_IP:18081 claude"
        else
            echo "$MSG_CLAUDE_INTEGRATION"
            echo "   ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude"
        fi
    fi

    # Service installation prompt
    if [ "${SUMMON_NON_INTERACTIVE:-}" != "1" ]; then
        echo ""
        echo "$MSG_SERVICE_TITLE"
        echo "$MSG_SERVICE_DESC"
        read -rp "$MSG_SERVICE_PROMPT" INSTALL_SERVICE < /dev/tty

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
                    echo "$MSG_SERVICE_UNSUPPORTED"
                    ;;
            esac
        fi
    fi
}

main "$@"
