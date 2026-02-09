#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"

$Repo = "TheMagicTower/summon"
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "$env:LOCALAPPDATA\summon\bin" }

function Detect-Platform {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "windows-amd64" }
        "ARM64" { return "windows-arm64" }
        default { throw "Unsupported architecture: $arch" }
    }
}

function Get-LatestVersion {
    $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    return $response.tag_name
}

Write-Host "🔮 Summon 설치 중..." -ForegroundColor Cyan

try {
    $Platform = Detect-Platform
    $Version = Get-LatestVersion

    Write-Host "  플랫폼: $Platform" -ForegroundColor Gray
    Write-Host "  버전: $Version" -ForegroundColor Gray

    $TempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
    $DownloadUrl = "https://github.com/$Repo/releases/download/$Version/summon-$Platform.tar.gz"

    Write-Host "  다운로드: $DownloadUrl" -ForegroundColor Gray
    $TarPath = Join-Path $TempDir "summon.tar.gz"
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TarPath -UseBasicParsing

    # Extract (PowerShell 5.1 doesn't have native tar, use .NET or fallback)
    Write-Host "  압축 해제 중..." -ForegroundColor Gray
    if (Get-Command tar -ErrorAction SilentlyContinue) {
        tar -xzf $TarPath -C $TempDir
    } else {
        # Fallback for systems without tar
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        [System.IO.Compression.ZipFile]::ExtractToDirectory($TarPath, $TempDir)
    }

    # Install
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    $BinaryName = if ($Platform -like "windows*") { "summon.exe" } else { "summon" }
    $SourcePath = Join-Path $TempDir "summon-$Platform"
    if (Test-Path $SourcePath) {
        Copy-Item $SourcePath (Join-Path $InstallDir $BinaryName) -Force
    } else {
        # Try without platform suffix
        $SourcePath = Join-Path $TempDir "summon"
        Copy-Item $SourcePath (Join-Path $InstallDir $BinaryName) -Force
    }

    Write-Host ""
    Write-Host "✅ Summon이 설치되었습니다: $InstallDir\$BinaryName" -ForegroundColor Green

    # Check PATH
    $PathDirs = $env:PATH -split ";"
    if ($PathDirs -notcontains $InstallDir) {
        Write-Host ""
        Write-Host "⚠️  $InstallDir이 PATH에 없습니다." -ForegroundColor Yellow
        Write-Host "   PowerShell에서 다음을 실행하세요:" -ForegroundColor Yellow
        Write-Host "   [Environment]::SetEnvironmentVariable('Path', `"$InstallDir;`$env:Path`", 'User')" -ForegroundColor White
    }

    # Create sample config
    $ConfigDir = "$env:APPDATA\summon"
    $ConfigFile = "$ConfigDir\config.yaml"
    if (-not (Test-Path $ConfigFile)) {
        New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
        @"
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
  #       value: "`${env:Z_AI_API_KEY}"
"@ | Out-File -FilePath $ConfigFile -Encoding UTF8

        Write-Host ""
        Write-Host "📝 샘플 설정 파일이 생성되었습니다: $ConfigFile" -ForegroundColor Cyan
    }

    Write-Host ""
    Write-Host "🚀 사용법:" -ForegroundColor Green
    Write-Host "   summon --config `"$ConfigFile`"" -ForegroundColor White
    Write-Host ""
    Write-Host "   Claude Code 연동:" -ForegroundColor Green
    Write-Host "   `$env:ANTHROPIC_BASE_URL='http://127.0.0.1:18081'; claude" -ForegroundColor White

} catch {
    Write-Host "❌ 설치 실패: $_" -ForegroundColor Red
    exit 1
} finally {
    if ($TempDir -and (Test-Path $TempDir)) {
        Remove-Item $TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
