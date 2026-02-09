#Requires -Version 5.1
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Repo = "TheMagicTower/summon"
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "$env:LOCALAPPDATA\summon\bin" }

# 플랫폼 감지
function Detect-Platform {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "windows-amd64" }
        "ARM64" { return "windows-arm64" }
        default { throw "지원되지 않는 아키텍처: $arch" }
    }
}

# 최신 릴리즈 버전 가져오기
function Get-LatestVersion {
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

    # env 객체 확보
    if (-not ($data.PSObject.Properties.Name -contains "env")) {
        $data | Add-Member -NotePropertyName "env" -NotePropertyValue ([PSCustomObject]@{})
    }

    # ANTHROPIC_BASE_URL 설정
    if ($data.env.PSObject.Properties.Name -contains "ANTHROPIC_BASE_URL") {
        $data.env.ANTHROPIC_BASE_URL = $baseUrl
    } else {
        $data.env | Add-Member -NotePropertyName "ANTHROPIC_BASE_URL" -NotePropertyValue $baseUrl
    }

    # Haiku 모델 설정
    if ($HaikuModel) {
        if ($data.env.PSObject.Properties.Name -contains "ANTHROPIC_DEFAULT_HAIKU_MODEL") {
            $data.env.ANTHROPIC_DEFAULT_HAIKU_MODEL = $HaikuModel
        } else {
            $data.env | Add-Member -NotePropertyName "ANTHROPIC_DEFAULT_HAIKU_MODEL" -NotePropertyValue $HaikuModel
        }
    }

    # Sonnet 모델 설정
    if ($SonnetModel) {
        if ($data.env.PSObject.Properties.Name -contains "ANTHROPIC_DEFAULT_SONNET_MODEL") {
            $data.env.ANTHROPIC_DEFAULT_SONNET_MODEL = $SonnetModel
        } else {
            $data.env | Add-Member -NotePropertyName "ANTHROPIC_DEFAULT_SONNET_MODEL" -NotePropertyValue $SonnetModel
        }
    }

    $data | ConvertTo-Json -Depth 10 | Set-Content $SettingsFile -Encoding UTF8
    Write-Host ""
    Write-Host "   Claude Code 설정이 업데이트되었습니다: $SettingsFile"
}

# 메인 설치
Write-Host "🔮 Summon 설치 중..."

try {
    $Platform = Detect-Platform
    $Version = Get-LatestVersion

    if (-not $Version) {
        Write-Error "최신 버전 정보를 가져올 수 없습니다."
        exit 1
    }

    Write-Host "  플랫폼: $Platform"
    Write-Host "  버전: $Version"

    # 임시 디렉토리
    $TempDir = Join-Path ([System.IO.Path]::GetTempPath()) "summon-install-$([guid]::NewGuid().ToString('N').Substring(0,8))"
    New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

    # 다운로드 (.zip)
    $DownloadUrl = "https://github.com/$Repo/releases/download/$Version/summon-$Platform.zip"
    $ZipPath = Join-Path $TempDir "summon.zip"
    Write-Host "  다운로드: $DownloadUrl"
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing

    # 압축 해제
    Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

    # 설치
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    $SourcePath = Join-Path $TempDir "summon-$Platform.exe"
    $DestBinary = Join-Path $InstallDir "summon.exe"
    Copy-Item $SourcePath $DestBinary -Force

    Write-Host ""
    Write-Host "✅ Summon이 설치되었습니다: $DestBinary"

    # PATH 확인 및 추가
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$UserPath", "User")
        $env:Path = "$InstallDir;$env:Path"
        Write-Host ""
        Write-Host "✅ PATH에 $InstallDir 를 추가했습니다. (새 터미널에서 적용)"
    }

    # config.yaml 생성 (없을 때만)
    $ConfigDir = Join-Path $env:USERPROFILE ".config\summon"
    $ConfigFile = if ($env:CONFIG_FILE) { $env:CONFIG_FILE } else { Join-Path $ConfigDir "config.yaml" }
    $KimiKey = ""
    $GlmKey = ""
    $HasAnyKey = $false

    if (-not (Test-Path $ConfigFile)) {
        New-Item -ItemType Directory -Path (Split-Path $ConfigFile) -Force | Out-Null

        Write-Host ""
        Write-Host "=== API 키 설정 ==="
        Write-Host "외부 LLM 프로바이더의 API 키를 입력하세요. (Enter로 건너뛰기)"
        Write-Host ""

        $KimiKey = Read-Host "  Kimi API 키"
        $GlmKey = Read-Host "  Z.AI (GLM) API 키"

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
        Write-Host "📝 설정 파일이 생성되었습니다: $ConfigFile"
    }

    # 모델 바인딩 (API 키가 하나라도 있을 때만)
    $HaikuModel = ""
    $SonnetModel = ""
    $ModelBindingSet = $false

    if ($HasAnyKey) {
        Write-Host ""
        Write-Host "=== 모델 바인딩 ==="
        Write-Host "Claude Code의 기본 모델을 외부 프로바이더로 교체할 수 있습니다."
        Write-Host ""

        # Haiku 모델 선택
        Write-Host "Haiku 모델:"
        Write-Host "  1) 기본값 유지 (Anthropic)"
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
        $HaikuChoice = Read-Host "선택 (1)"
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
        Write-Host "Sonnet 모델:"
        Write-Host "  1) 기본값 유지 (Anthropic)"
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
        $SonnetChoice = Read-Host "선택 (1)"
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
    Write-Host "🚀 사용법:"
    Write-Host "   summon --config `"$ConfigFile`""
    Write-Host ""

    if ($ModelBindingSet) {
        Write-Host "✅ 설정 완료! Claude Code를 재시작하면 자동으로 적용됩니다."
    } else {
        Write-Host "   Claude Code 연동:"
        Write-Host "   `$env:ANTHROPIC_BASE_URL='http://127.0.0.1:18081'; claude"
    }

    # 서비스 등록 (작업 스케줄러)
    Write-Host ""
    Write-Host "🔧 로그인 시 자동 시작으로 등록하시겠습니까?"
    Write-Host "   Windows 작업 스케줄러를 사용하여 로그인 시 자동으로 summon을 시작합니다."
    $InstallService = Read-Host "   등록하시겠습니까? (y/N)"

    if ($InstallService -match "^[Yy]$") {
        $TaskName = "Summon LLM Proxy"
        try {
            $Action = New-ScheduledTaskAction -Execute $DestBinary -Argument "--config `"$ConfigFile`""
            $Trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
            $Settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1)
            Register-ScheduledTask -TaskName $TaskName -Action $Action -Trigger $Trigger -Settings $Settings -Force | Out-Null
            Write-Host ""
            Write-Host "   ✅ 작업 스케줄러 등록 완료: $TaskName"
            Write-Host "   📋 관리 명령어:"
            Write-Host "      schtasks /run /tn `"$TaskName`"       # 즉시 시작"
            Write-Host "      schtasks /end /tn `"$TaskName`"       # 중지"
            Write-Host "      schtasks /query /tn `"$TaskName`"     # 상태 확인"
            Write-Host "      schtasks /delete /tn `"$TaskName`"    # 삭제"
        } catch {
            Write-Host "   ⚠️  작업 스케줄러 등록에 실패했습니다. 관리자 권한으로 다시 시도해주세요."
        }
    }

} catch {
    Write-Host "❌ 설치 실패: $_" -ForegroundColor Red
    exit 1
} finally {
    if ($TempDir -and (Test-Path $TempDir)) {
        Remove-Item $TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
