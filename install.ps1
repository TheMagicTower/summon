#Requires -Version 5.1
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Repo = "TheMagicTower/summon"
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "$env:LOCALAPPDATA\summon\bin" }

# 언어 감지 (SUMMON_LANG 환경변수로 오버라이드 가능)
function Detect-Language {
    if ($env:SUMMON_LANG) { return $env:SUMMON_LANG }

    # Windows UI 언어 감지
    try {
        $culture = (Get-Culture).TwoLetterISOLanguageName
    } catch {
        $culture = "en"
    }

    switch ($culture) {
        "ko" { return "ko" }
        "ja" { return "ja" }
        "zh" { return "zh" }
        "es" { return "es" }
        "de" { return "de" }
        "vi" { return "vi" }
        default { return "en" }
    }
}

# 메시지 설정
function Set-Messages {
    param([string]$Lang)

    $script:M = @{}

    switch ($Lang) {
        "ko" {
            $M.Installing = "🔮 Summon 설치 중..."
            $M.Platform = "  플랫폼"
            $M.Version = "  버전"
            $M.LocalBinary = "  로컬 바이너리"
            $M.Downloading = "  다운로드"
            $M.Installed = "✅ Summon이 설치되었습니다"
            $M.PathAdded = "✅ PATH에 {0} 를 추가했습니다. (새 터미널에서 적용)"
            $M.ProviderTitle = "=== 외부 LLM 프로바이더 설정 ==="
            $M.ProviderDesc = "Claude Code의 요청을 외부 LLM으로 라우팅할 수 있습니다."
            $M.ProviderSelect = "사용할 프로바이더를 선택하세요:"
            $M.ProviderAll = "모두 설정"
            $M.ProviderSkip = "건너뛰기"
            $M.Select = "선택"
            $M.ApiKeyKimi = "  Kimi API 키"
            $M.ApiKeyGlm = "  Z.AI (GLM) API 키"
            $M.ConfigCreated = "📝 설정 파일이 생성되었습니다"
            $M.ModelTitle = "=== 모델 바인딩 ==="
            $M.ModelDesc = "Claude Code의 기본 모델을 외부 프로바이더로 교체할 수 있습니다."
            $M.ModelHaiku = "Haiku 모델:"
            $M.ModelSonnet = "Sonnet 모델:"
            $M.ModelDefault = "기본값 유지 (Anthropic)"
            $M.SettingsUpdated = "   Claude Code 설정이 업데이트되었습니다"
            $M.Usage = "🚀 사용법:"
            $M.SetupComplete = "✅ 설정 완료! Claude Code를 재시작하면 자동으로 적용됩니다."
            $M.ClaudeIntegration = "   Claude Code 연동:"
            $M.ServiceTitle = "🔧 로그인 시 자동 시작으로 등록하시겠습니까?"
            $M.ServiceDesc = "   Windows 작업 스케줄러를 사용하여 로그인 시 자동으로 summon을 시작합니다."
            $M.ServicePrompt = "   등록하시겠습니까? (y/N)"
            $M.ServiceDone = "   ✅ 작업 스케줄러 등록 완료"
            $M.ServiceCommands = "   📋 관리 명령어:"
            $M.ServiceFailed = "   ⚠️  작업 스케줄러 등록에 실패했습니다. 관리자 권한으로 다시 시도해주세요."
            $M.InstallFailed = "❌ 설치 실패"
            $M.VersionError = "최신 버전 정보를 가져올 수 없습니다."
        }
        "ja" {
            $M.Installing = "🔮 Summon をインストール中..."
            $M.Platform = "  プラットフォーム"
            $M.Version = "  バージョン"
            $M.LocalBinary = "  ローカルバイナリ"
            $M.Downloading = "  ダウンロード"
            $M.Installed = "✅ Summon がインストールされました"
            $M.PathAdded = "✅ PATH に {0} を追加しました。（新しいターミナルで有効）"
            $M.ProviderTitle = "=== 外部 LLM プロバイダー設定 ==="
            $M.ProviderDesc = "Claude Code のリクエストを外部 LLM にルーティングできます。"
            $M.ProviderSelect = "使用するプロバイダーを選択してください:"
            $M.ProviderAll = "すべて設定"
            $M.ProviderSkip = "スキップ"
            $M.Select = "選択"
            $M.ApiKeyKimi = "  Kimi API キー"
            $M.ApiKeyGlm = "  Z.AI (GLM) API キー"
            $M.ConfigCreated = "📝 設定ファイルが作成されました"
            $M.ModelTitle = "=== モデルバインディング ==="
            $M.ModelDesc = "Claude Code のデフォルトモデルを外部プロバイダーに変更できます。"
            $M.ModelHaiku = "Haiku モデル:"
            $M.ModelSonnet = "Sonnet モデル:"
            $M.ModelDefault = "デフォルトを維持 (Anthropic)"
            $M.SettingsUpdated = "   Claude Code の設定が更新されました"
            $M.Usage = "🚀 使い方:"
            $M.SetupComplete = "✅ 設定完了！Claude Code を再起動すると自動的に適用されます。"
            $M.ClaudeIntegration = "   Claude Code 連携:"
            $M.ServiceTitle = "🔧 ログイン時に自動起動として登録しますか？"
            $M.ServiceDesc = "   Windows タスクスケジューラを使用して、ログイン時に自動的に summon を起動します。"
            $M.ServicePrompt = "   登録しますか？ (y/N)"
            $M.ServiceDone = "   ✅ タスクスケジューラ登録完了"
            $M.ServiceCommands = "   📋 管理コマンド:"
            $M.ServiceFailed = "   ⚠️  タスクスケジューラの登録に失敗しました。管理者権限で再試行してください。"
            $M.InstallFailed = "❌ インストール失敗"
            $M.VersionError = "最新バージョン情報を取得できません。"
        }
        "zh" {
            $M.Installing = "🔮 正在安装 Summon..."
            $M.Platform = "  平台"
            $M.Version = "  版本"
            $M.LocalBinary = "  本地二进制文件"
            $M.Downloading = "  下载"
            $M.Installed = "✅ Summon 已安装"
            $M.PathAdded = "✅ 已将 {0} 添加到 PATH。（在新终端中生效）"
            $M.ProviderTitle = "=== 外部 LLM 提供商设置 ==="
            $M.ProviderDesc = "可以将 Claude Code 的请求路由到外部 LLM。"
            $M.ProviderSelect = "请选择要使用的提供商:"
            $M.ProviderAll = "全部设置"
            $M.ProviderSkip = "跳过"
            $M.Select = "选择"
            $M.ApiKeyKimi = "  Kimi API 密钥"
            $M.ApiKeyGlm = "  Z.AI (GLM) API 密钥"
            $M.ConfigCreated = "📝 配置文件已创建"
            $M.ModelTitle = "=== 模型绑定 ==="
            $M.ModelDesc = "可以将 Claude Code 的默认模型替换为外部提供商。"
            $M.ModelHaiku = "Haiku 模型:"
            $M.ModelSonnet = "Sonnet 模型:"
            $M.ModelDefault = "保持默认 (Anthropic)"
            $M.SettingsUpdated = "   Claude Code 设置已更新"
            $M.Usage = "🚀 用法:"
            $M.SetupComplete = "✅ 设置完成！重启 Claude Code 后自动生效。"
            $M.ClaudeIntegration = "   Claude Code 集成:"
            $M.ServiceTitle = "🔧 是否注册为登录时自动启动？"
            $M.ServiceDesc = "   使用 Windows 任务计划程序在登录时自动启动 summon。"
            $M.ServicePrompt = "   是否注册？ (y/N)"
            $M.ServiceDone = "   ✅ 任务计划程序注册完成"
            $M.ServiceCommands = "   📋 管理命令:"
            $M.ServiceFailed = "   ⚠️  任务计划程序注册失败。请以管理员权限重试。"
            $M.InstallFailed = "❌ 安装失败"
            $M.VersionError = "无法获取最新版本信息。"
        }
        "es" {
            $M.Installing = "🔮 Instalando Summon..."
            $M.Platform = "  Plataforma"
            $M.Version = "  Versión"
            $M.LocalBinary = "  Binario local"
            $M.Downloading = "  Descarga"
            $M.Installed = "✅ Summon ha sido instalado"
            $M.PathAdded = "✅ {0} se ha añadido al PATH. (Efectivo en nueva terminal)"
            $M.ProviderTitle = "=== Configuración de proveedores LLM externos ==="
            $M.ProviderDesc = "Puedes enrutar las solicitudes de Claude Code a un LLM externo."
            $M.ProviderSelect = "Selecciona el proveedor que deseas usar:"
            $M.ProviderAll = "Configurar todos"
            $M.ProviderSkip = "Omitir"
            $M.Select = "Selección"
            $M.ApiKeyKimi = "  Clave API de Kimi"
            $M.ApiKeyGlm = "  Clave API de Z.AI (GLM)"
            $M.ConfigCreated = "📝 Archivo de configuración creado"
            $M.ModelTitle = "=== Enlace de modelos ==="
            $M.ModelDesc = "Puedes reemplazar los modelos predeterminados de Claude Code con proveedores externos."
            $M.ModelHaiku = "Modelo Haiku:"
            $M.ModelSonnet = "Modelo Sonnet:"
            $M.ModelDefault = "Mantener predeterminado (Anthropic)"
            $M.SettingsUpdated = "   La configuración de Claude Code ha sido actualizada"
            $M.Usage = "🚀 Uso:"
            $M.SetupComplete = "✅ ¡Configuración completada! Se aplicará automáticamente al reiniciar Claude Code."
            $M.ClaudeIntegration = "   Integración con Claude Code:"
            $M.ServiceTitle = "🔧 ¿Registrar inicio automático al iniciar sesión?"
            $M.ServiceDesc = "   Usa el Programador de tareas de Windows para iniciar summon automáticamente."
            $M.ServicePrompt = "   ¿Registrar? (y/N)"
            $M.ServiceDone = "   ✅ Tarea programada registrada"
            $M.ServiceCommands = "   📋 Comandos de gestión:"
            $M.ServiceFailed = "   ⚠️  Falló el registro en el Programador de tareas. Inténtalo con permisos de administrador."
            $M.InstallFailed = "❌ Instalación fallida"
            $M.VersionError = "No se pudo obtener la información de la última versión."
        }
        "de" {
            $M.Installing = "🔮 Summon wird installiert..."
            $M.Platform = "  Plattform"
            $M.Version = "  Version"
            $M.LocalBinary = "  Lokale Binärdatei"
            $M.Downloading = "  Download"
            $M.Installed = "✅ Summon wurde installiert"
            $M.PathAdded = "✅ {0} wurde zum PATH hinzugefügt. (Gilt in neuem Terminal)"
            $M.ProviderTitle = "=== Externe LLM-Anbieter einrichten ==="
            $M.ProviderDesc = "Du kannst Claude Code-Anfragen an externe LLMs weiterleiten."
            $M.ProviderSelect = "Wähle den gewünschten Anbieter:"
            $M.ProviderAll = "Alle einrichten"
            $M.ProviderSkip = "Überspringen"
            $M.Select = "Auswahl"
            $M.ApiKeyKimi = "  Kimi API-Schlüssel"
            $M.ApiKeyGlm = "  Z.AI (GLM) API-Schlüssel"
            $M.ConfigCreated = "📝 Konfigurationsdatei erstellt"
            $M.ModelTitle = "=== Modellbindung ==="
            $M.ModelDesc = "Du kannst die Standardmodelle von Claude Code durch externe Anbieter ersetzen."
            $M.ModelHaiku = "Haiku-Modell:"
            $M.ModelSonnet = "Sonnet-Modell:"
            $M.ModelDefault = "Standard beibehalten (Anthropic)"
            $M.SettingsUpdated = "   Claude Code-Einstellungen wurden aktualisiert"
            $M.Usage = "🚀 Verwendung:"
            $M.SetupComplete = "✅ Einrichtung abgeschlossen! Wird nach Neustart von Claude Code automatisch angewendet."
            $M.ClaudeIntegration = "   Claude Code-Integration:"
            $M.ServiceTitle = "🔧 Als Autostart bei Anmeldung registrieren?"
            $M.ServiceDesc = "   Verwendet den Windows Aufgabenplaner, um summon automatisch zu starten."
            $M.ServicePrompt = "   Registrieren? (y/N)"
            $M.ServiceDone = "   ✅ Aufgabenplaner-Registrierung abgeschlossen"
            $M.ServiceCommands = "   📋 Verwaltungsbefehle:"
            $M.ServiceFailed = "   ⚠️  Registrierung im Aufgabenplaner fehlgeschlagen. Bitte mit Administratorrechten erneut versuchen."
            $M.InstallFailed = "❌ Installation fehlgeschlagen"
            $M.VersionError = "Die neueste Version konnte nicht abgerufen werden."
        }
        "vi" {
            $M.Installing = "🔮 Đang cài đặt Summon..."
            $M.Platform = "  Nền tảng"
            $M.Version = "  Phiên bản"
            $M.LocalBinary = "  Binary cục bộ"
            $M.Downloading = "  Tải xuống"
            $M.Installed = "✅ Summon đã được cài đặt"
            $M.PathAdded = "✅ Đã thêm {0} vào PATH. (Có hiệu lực trong terminal mới)"
            $M.ProviderTitle = "=== Cấu hình nhà cung cấp LLM bên ngoài ==="
            $M.ProviderDesc = "Bạn có thể định tuyến yêu cầu của Claude Code đến LLM bên ngoài."
            $M.ProviderSelect = "Chọn nhà cung cấp bạn muốn sử dụng:"
            $M.ProviderAll = "Cấu hình tất cả"
            $M.ProviderSkip = "Bỏ qua"
            $M.Select = "Chọn"
            $M.ApiKeyKimi = "  Khóa API Kimi"
            $M.ApiKeyGlm = "  Khóa API Z.AI (GLM)"
            $M.ConfigCreated = "📝 Tệp cấu hình đã được tạo"
            $M.ModelTitle = "=== Liên kết mô hình ==="
            $M.ModelDesc = "Bạn có thể thay thế mô hình mặc định của Claude Code bằng nhà cung cấp bên ngoài."
            $M.ModelHaiku = "Mô hình Haiku:"
            $M.ModelSonnet = "Mô hình Sonnet:"
            $M.ModelDefault = "Giữ mặc định (Anthropic)"
            $M.SettingsUpdated = "   Cấu hình Claude Code đã được cập nhật"
            $M.Usage = "🚀 Cách dùng:"
            $M.SetupComplete = "✅ Cấu hình hoàn tất! Sẽ tự động áp dụng khi khởi động lại Claude Code."
            $M.ClaudeIntegration = "   Tích hợp Claude Code:"
            $M.ServiceTitle = "🔧 Đăng ký tự động khởi động khi đăng nhập?"
            $M.ServiceDesc = "   Sử dụng Task Scheduler của Windows để tự động khởi động summon."
            $M.ServicePrompt = "   Đăng ký? (y/N)"
            $M.ServiceDone = "   ✅ Đã đăng ký Task Scheduler"
            $M.ServiceCommands = "   📋 Lệnh quản lý:"
            $M.ServiceFailed = "   ⚠️  Đăng ký Task Scheduler thất bại. Vui lòng thử lại với quyền quản trị."
            $M.InstallFailed = "❌ Cài đặt thất bại"
            $M.VersionError = "Không thể lấy thông tin phiên bản mới nhất."
        }
        default {
            $M.Installing = "🔮 Installing Summon..."
            $M.Platform = "  Platform"
            $M.Version = "  Version"
            $M.LocalBinary = "  Local binary"
            $M.Downloading = "  Download"
            $M.Installed = "✅ Summon has been installed"
            $M.PathAdded = "✅ {0} has been added to PATH. (Effective in new terminal)"
            $M.ProviderTitle = "=== External LLM Provider Setup ==="
            $M.ProviderDesc = "You can route Claude Code requests to external LLMs."
            $M.ProviderSelect = "Select a provider to use:"
            $M.ProviderAll = "Set up all"
            $M.ProviderSkip = "Skip"
            $M.Select = "Select"
            $M.ApiKeyKimi = "  Kimi API key"
            $M.ApiKeyGlm = "  Z.AI (GLM) API key"
            $M.ConfigCreated = "📝 Configuration file created"
            $M.ModelTitle = "=== Model Binding ==="
            $M.ModelDesc = "You can replace Claude Code's default models with external providers."
            $M.ModelHaiku = "Haiku model:"
            $M.ModelSonnet = "Sonnet model:"
            $M.ModelDefault = "Keep default (Anthropic)"
            $M.SettingsUpdated = "   Claude Code settings have been updated"
            $M.Usage = "🚀 Usage:"
            $M.SetupComplete = "✅ Setup complete! Changes will be applied when Claude Code restarts."
            $M.ClaudeIntegration = "   Claude Code integration:"
            $M.ServiceTitle = "🔧 Register auto-start on login?"
            $M.ServiceDesc = "   Uses Windows Task Scheduler to auto-start summon on login."
            $M.ServicePrompt = "   Register? (y/N)"
            $M.ServiceDone = "   ✅ Task Scheduler registration complete"
            $M.ServiceCommands = "   📋 Management commands:"
            $M.ServiceFailed = "   ⚠️  Task Scheduler registration failed. Please retry with administrator privileges."
            $M.InstallFailed = "❌ Installation failed"
            $M.VersionError = "Could not fetch the latest version information."
        }
    }
}

# 언어 감지 및 메시지 초기화
$LangCode = Detect-Language
Set-Messages -Lang $LangCode

# 플랫폼 감지
function Detect-Platform {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "windows-amd64" }
        "ARM64" { return "windows-arm64" }
        default { throw "Unsupported architecture: $arch" }
    }
}

# 최신 릴리즈 버전 가져오기
function Get-LatestVersion {
    if ($env:SUMMON_VERSION) {
        return $env:SUMMON_VERSION
    }
    $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    return $response.tag_name
}

# settings.json 업데이트
function Update-SettingsJson {
    param(
        [string]$SettingsFile,
        [string]$HaikuModel,
        [string]$SonnetModel
    )

    $baseUrl = "http://127.0.0.1:18081"

    if (Test-Path $SettingsFile) {
        try {
            $data = Get-Content $SettingsFile -Raw | ConvertFrom-Json
        } catch {
            $data = [PSCustomObject]@{}
        }
    } else {
        $data = [PSCustomObject]@{}
    }

    if (-not ($data.PSObject.Properties.Name -contains "env")) {
        $data | Add-Member -NotePropertyName "env" -NotePropertyValue ([PSCustomObject]@{})
    }

    if ($data.env.PSObject.Properties.Name -contains "ANTHROPIC_BASE_URL") {
        $data.env.ANTHROPIC_BASE_URL = $baseUrl
    } else {
        $data.env | Add-Member -NotePropertyName "ANTHROPIC_BASE_URL" -NotePropertyValue $baseUrl
    }

    if ($HaikuModel) {
        if ($data.env.PSObject.Properties.Name -contains "ANTHROPIC_DEFAULT_HAIKU_MODEL") {
            $data.env.ANTHROPIC_DEFAULT_HAIKU_MODEL = $HaikuModel
        } else {
            $data.env | Add-Member -NotePropertyName "ANTHROPIC_DEFAULT_HAIKU_MODEL" -NotePropertyValue $HaikuModel
        }
    }

    if ($SonnetModel) {
        if ($data.env.PSObject.Properties.Name -contains "ANTHROPIC_DEFAULT_SONNET_MODEL") {
            $data.env.ANTHROPIC_DEFAULT_SONNET_MODEL = $SonnetModel
        } else {
            $data.env | Add-Member -NotePropertyName "ANTHROPIC_DEFAULT_SONNET_MODEL" -NotePropertyValue $SonnetModel
        }
    }

    $data | ConvertTo-Json -Depth 10 | Set-Content $SettingsFile -Encoding UTF8
    Write-Host ""
    Write-Host "$($M.SettingsUpdated): $SettingsFile"
}

# 메인 설치
Write-Host $M.Installing

$TempDir = $null
try {
    $Platform = Detect-Platform
    $Version = Get-LatestVersion

    if (-not $Version) {
        Write-Error $M.VersionError
        exit 1
    }

    Write-Host "$($M.Platform): $Platform"
    Write-Host "$($M.Version): $Version"

    # 설치
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    $DestBinary = Join-Path $InstallDir "summon.exe"

    if ($env:SUMMON_BINARY) {
        Write-Host "$($M.LocalBinary): $($env:SUMMON_BINARY)"
        Copy-Item $env:SUMMON_BINARY $DestBinary -Force
    } else {
        $TempDir = Join-Path ([System.IO.Path]::GetTempPath()) "summon-install-$([guid]::NewGuid().ToString('N').Substring(0,8))"
        New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

        $DownloadUrl = "https://github.com/$Repo/releases/download/$Version/summon-$Platform.zip"
        $ZipPath = Join-Path $TempDir "summon.zip"
        Write-Host "$($M.Downloading): $DownloadUrl"
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing

        Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

        $SourcePath = Join-Path $TempDir "summon-$Platform.exe"
        Copy-Item $SourcePath $DestBinary -Force
    }

    Write-Host ""
    Write-Host "$($M.Installed): $DestBinary"

    # PATH 확인 및 추가
    if ($env:SUMMON_NON_INTERACTIVE -ne "1") {
        $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($UserPath -notlike "*$InstallDir*") {
            [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$UserPath", "User")
            $env:Path = "$InstallDir;$env:Path"
            Write-Host ""
            Write-Host ($M.PathAdded -f $InstallDir)
        }
    } else {
        $env:Path = "$InstallDir;$env:Path"
    }

    # config.yaml 생성 (없을 때만)
    $ConfigDir = Join-Path $env:USERPROFILE ".config\summon"
    $ConfigFile = if ($env:CONFIG_FILE) { $env:CONFIG_FILE } else { Join-Path $ConfigDir "config.yaml" }
    $KimiKey = ""
    $GlmKey = ""
    $HasAnyKey = $false

    if (-not (Test-Path $ConfigFile)) {
        New-Item -ItemType Directory -Path (Split-Path $ConfigFile) -Force | Out-Null

        if ($env:SUMMON_NON_INTERACTIVE -eq "1") {
            $KimiKey = ""
            $GlmKey = ""
        } else {
            Write-Host ""
            Write-Host $M.ProviderTitle
            Write-Host $M.ProviderDesc
            Write-Host $M.ProviderSelect
            Write-Host ""
            Write-Host "  1) Kimi (Moonshot AI - kimi.com)"
            Write-Host "  2) Z.AI (GLM - z.ai)"
            Write-Host "  3) $($M.ProviderAll)"
            Write-Host "  4) $($M.ProviderSkip)"
            Write-Host ""
            $ProviderChoice = Read-Host "$($M.Select) (4)"
            if (-not $ProviderChoice) { $ProviderChoice = "4" }

            switch ($ProviderChoice) {
                "1" {
                    $KimiKey = Read-Host $M.ApiKeyKimi
                }
                "2" {
                    $GlmKey = Read-Host $M.ApiKeyGlm
                }
                "3" {
                    $KimiKey = Read-Host $M.ApiKeyKimi
                    $GlmKey = Read-Host $M.ApiKeyGlm
                }
                default {
                    $KimiKey = ""
                    $GlmKey = ""
                }
            }
        }

        # routes 생성
        $Routes = ""
        if ($KimiKey) {
            $HasAnyKey = $true
            $Routes += @"

  - match: "kimi"
    upstream:
      url: "https://api.kimi.com/coding"
      auth:
        header: "Authorization"
        value: "Bearer $KimiKey"
"@
        }
        if ($GlmKey) {
            $HasAnyKey = $true
            $Routes += @"

  - match: "glm"
    upstream:
      url: "https://api.z.ai/api/anthropic"
      auth:
        header: "x-api-key"
        value: "$GlmKey"
"@
        }

        if (-not $Routes) {
            $Routes = " []"
        }

        $ConfigContent = @"
server:
  host: "127.0.0.1"
  port: 18081

default:
  url: "https://api.anthropic.com"

routes:$Routes
"@
        Set-Content -Path $ConfigFile -Value $ConfigContent -Encoding UTF8
        Write-Host ""
        Write-Host "$($M.ConfigCreated): $ConfigFile"
    }

    # 모델 바인딩 (API 키가 하나라도 있을 때만)
    $HaikuModel = ""
    $SonnetModel = ""
    $ModelBindingSet = $false

    if ($HasAnyKey -and $env:SUMMON_NON_INTERACTIVE -ne "1") {
        Write-Host ""
        Write-Host $M.ModelTitle
        Write-Host $M.ModelDesc
        Write-Host ""

        # Haiku 모델 선택
        Write-Host $M.ModelHaiku
        Write-Host "  1) $($M.ModelDefault)"
        $HaikuIdx = 2
        $HaikuKimiIdx = 0
        $HaikuGlmIdx = 0
        if ($KimiKey) {
            Write-Host "  $HaikuIdx) Kimi"
            $HaikuKimiIdx = $HaikuIdx
            $HaikuIdx++
        }
        if ($GlmKey) {
            Write-Host "  $HaikuIdx) GLM"
            $HaikuGlmIdx = $HaikuIdx
            $HaikuIdx++
        }
        $HaikuChoice = Read-Host "$($M.Select) (1)"
        if (-not $HaikuChoice) { $HaikuChoice = "1" }

        if ($HaikuChoice -ne "1") {
            if ($HaikuKimiIdx -and $HaikuChoice -eq "$HaikuKimiIdx") {
                $HaikuModel = "kimi-for-coding"
                $ModelBindingSet = $true
            } elseif ($HaikuGlmIdx -and $HaikuChoice -eq "$HaikuGlmIdx") {
                $HaikuModel = "glm-4.7"
                $ModelBindingSet = $true
            }
        }

        Write-Host ""

        # Sonnet 모델 선택
        Write-Host $M.ModelSonnet
        Write-Host "  1) $($M.ModelDefault)"
        $SonnetIdx = 2
        $SonnetKimiIdx = 0
        $SonnetGlmIdx = 0
        if ($KimiKey) {
            Write-Host "  $SonnetIdx) Kimi"
            $SonnetKimiIdx = $SonnetIdx
            $SonnetIdx++
        }
        if ($GlmKey) {
            Write-Host "  $SonnetIdx) GLM"
            $SonnetGlmIdx = $SonnetIdx
            $SonnetIdx++
        }
        $SonnetChoice = Read-Host "$($M.Select) (1)"
        if (-not $SonnetChoice) { $SonnetChoice = "1" }

        if ($SonnetChoice -ne "1") {
            if ($SonnetKimiIdx -and $SonnetChoice -eq "$SonnetKimiIdx") {
                $SonnetModel = "kimi-for-coding"
                $ModelBindingSet = $true
            } elseif ($SonnetGlmIdx -and $SonnetChoice -eq "$SonnetGlmIdx") {
                $SonnetModel = "glm-4.7"
                $ModelBindingSet = $true
            }
        }
    }

    # settings.json 업데이트
    if ($HasAnyKey) {
        $ClaudeDir = Join-Path $env:USERPROFILE ".claude"
        New-Item -ItemType Directory -Path $ClaudeDir -Force | Out-Null
        $SettingsFile = Join-Path $ClaudeDir "settings.json"
        Update-SettingsJson -SettingsFile $SettingsFile -HaikuModel $HaikuModel -SonnetModel $SonnetModel
    }

    Write-Host ""
    Write-Host "$($M.Usage)"
    Write-Host "   summon --config `"$ConfigFile`""
    Write-Host ""

    if ($ModelBindingSet) {
        Write-Host $M.SetupComplete
    } else {
        Write-Host $M.ClaudeIntegration
        Write-Host "   `$env:ANTHROPIC_BASE_URL='http://127.0.0.1:18081'; claude"
    }

    # 서비스 등록 (작업 스케줄러)
    if ($env:SUMMON_NON_INTERACTIVE -ne "1") {
        Write-Host ""
        Write-Host $M.ServiceTitle
        Write-Host $M.ServiceDesc
        $InstallService = Read-Host $M.ServicePrompt

        if ($InstallService -match "^[Yy]$") {
            $TaskName = "Summon LLM Proxy"
            try {
                $Action = New-ScheduledTaskAction -Execute $DestBinary -Argument "--config `"$ConfigFile`""
                $Trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
                $Settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1)
                Register-ScheduledTask -TaskName $TaskName -Action $Action -Trigger $Trigger -Settings $Settings -Force | Out-Null
                Write-Host ""
                Write-Host "$($M.ServiceDone): $TaskName"
                Write-Host $M.ServiceCommands
                Write-Host "      schtasks /run /tn `"$TaskName`""
                Write-Host "      schtasks /end /tn `"$TaskName`""
                Write-Host "      schtasks /query /tn `"$TaskName`""
                Write-Host "      schtasks /delete /tn `"$TaskName`""
            } catch {
                Write-Host $M.ServiceFailed
            }
        }
    }

} catch {
    Write-Host "$($M.InstallFailed): $_" -ForegroundColor Red
    exit 1
} finally {
    if ($TempDir -and (Test-Path $TempDir)) {
        Remove-Item $TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
