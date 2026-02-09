# Summon

モデル名に基づいてClaude CodeのAPIリクエストを異なるLLMプロバイダーにルーティングするRust製軽量リバースプロキシ。

既存のAnthropicサブスクリプション（OAuth）認証を維持しながら、特定のモデルのみを外部プロバイダー（Z.AI、Kimiなど）に分岐させます。

## アーキテクチャ

```
Claude Code CLI
  │ ANTHROPIC_BASE_URL=http://127.0.0.1:18081
  ▼
プロキシ (axumサーバー)
  ├─ /v1/messages POST → modelフィールド解析 → ルーティング決定
  │   ├─ マッチ → 外部プロバイダー（ヘッダー/認証置換）
  │   └─ 非マッチ → Anthropic API（パススルー）
  └─ その他のリクエスト → Anthropic API（パススルー）
```

## インストール

### ワンライナーインストール（推奨）

**Linux/macOS/WSL:**
```bash
curl -fsSL https://raw.githubusercontent.com/TheMagicTower/summon/master/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/TheMagicTower/summon/master/install.ps1 | iex
```

> 💡 **WSLユーザー**: WSL側とWindows側の両方でClaude Codeを使用できます。詳細は以下の[WSL使用方法](#wsl使用方法)セクションを参照してください。

### バイナリダウンロード

[Releases](https://github.com/TheMagicTower/summon/releases)ページからプラットフォームに適したバイナリをダウンロードしてください。

| プラットフォーム | ファイル |
|----------------|---------|
| Linux x86_64 | `summon-linux-amd64.tar.gz` |
| Linux ARM64 | `summon-linux-arm64.tar.gz` |
| macOS Intel | `summon-darwin-amd64.tar.gz` |
| macOS Apple Silicon | `summon-darwin-arm64.tar.gz` |
| Windows x86_64 | `summon-windows-amd64.zip` |
| Windows ARM64 | `summon-windows-arm64.zip` |

```bash
# 例: macOS Apple Silicon
tar xzf summon-darwin-arm64.tar.gz
chmod +x summon-darwin-arm64
sudo mv summon-darwin-arm64 /usr/local/bin/summon
```

### ソースからビルド

```bash
cargo build --release
```

## 設定

### 設定ファイルの場所

summonは以下の優先順位で設定ファイルを検索します:

| 優先順位 | 場所 | 説明 |
|---------|------|------|
| 1 | `--config <パス>` | 明示的な指定 |
| 2 | `SUMMON_CONFIG`環境変数 | 環境変数で指定されたパス |
| 3 | `~/.config/summon/config.yaml` | ユーザー別設定（XDG） |
| 4 | `/etc/summon/config.yaml` | システム全体設定 |
| 5 | `./config.yaml` | 現在のディレクトリ |

### マルチユーザー環境

各ユーザーが独自の設定を使用するには:
```bash
mkdir -p ~/.config/summon
cp /path/to/config.yaml ~/.config/summon/
```

システム管理者がデフォルト設定を提供するには:
```bash
sudo mkdir -p /etc/summon
sudo cp config.yaml /etc/summon/
```

### 設定ファイル例

`config.yaml`ファイルを作成します:

```yaml
server:
  host: "127.0.0.1"
  port: 18081

default:
  url: "https://api.anthropic.com"

routes:
  - match: "claude-haiku"
    upstream:
      url: "https://api.z.ai/api/anthropic"
      auth:
        header: "x-api-key"
        value: "${Z_AI_API_KEY}"

  - match: "claude-sonnet"
    upstream:
      url: "https://api.kimi.ai/v1"
      auth:
        header: "Authorization"
        value: "Bearer ${KIMI_API_KEY}"
```

- `match`: モデル名にこの文字列が含まれる場合にマッチ（上→下の順、最初のマッチを適用）
- `${ENV_VAR}`: 環境変数参照（APIキーを設定ファイルに直接記述しません）
- マッチしないモデルは`default.url`（Anthropic API）にパススルー

## 実行

```bash
# 環境変数設定
export Z_AI_API_KEY="your-z-ai-key"
export KIMI_API_KEY="your-kimi-key"

# プロキシ起動（設定ファイル自動検出）
summon

# または設定ファイルを直接指定
summon --config /path/to/config.yaml

# Claude Code連携
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

## WSL使用方法

WSL（Windows Subsystem for Linux）でもsummonを使用できます。

### WSL側でClaude Codeを使用

```bash
# WSLターミナルで（設定ファイルを ~/.config/summon/config.yaml に配置した場合）
summon

# 別のWSLターミナルで
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

### Windows側でClaude Codeを使用（WSLでsummon実行）

```bash
# WSLでsummon実行（0.0.0.0にバインドしてWindowsからアクセス可能に）
summon

# Windowsターミナル（PowerShell/CMD）で
# WSL IP確認: ip addr show eth0 | grep 'inet '
ANTHROPIC_BASE_URL=http://$(wsl hostname -I | awk '{print $1}'):18081 claude
```

または、`config.yaml`で`server.host`を`"0.0.0.0"`に設定してWindowsからアクセスできます。

## バックグラウンドサービスとして登録

### macOS (launchd)

**1. LaunchAgent plistファイル作成:**

```bash
cat > ~/Library/LaunchAgents/com.themagictower.summon.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.themagictower.summon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/YOUR_USERNAME/.local/bin/summon</string>
        <string>--config</string>
        <string>/Users/YOUR_USERNAME/.config/summon/config.yaml</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/Users/YOUR_USERNAME/.local/share/summon/summon.log</string>
    <key>StandardErrorPath</key>
    <string>/Users/YOUR_USERNAME/.local/share/summon/summon.error.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/Users/YOUR_USERNAME/.local/bin:/usr/local/bin:/usr/bin:/bin</string>
    </dict>
</dict>
</plist>
EOF
```

**2. ログディレクトリ作成とサービス登録:**

```bash
mkdir -p ~/.local/share/summon
launchctl load ~/Library/LaunchAgents/com.themagictower.summon.plist
launchctl start com.themagictower.summon
```

**3. サービス管理:**

```bash
# ステータス確認
launchctl list | grep com.themagictower.summon

# 停止
launchctl stop com.themagictower.summon

# 再起動
launchctl stop com.themagictower.summon && launchctl start com.themagictower.summon

# 削除
launchctl unload ~/Library/LaunchAgents/com.themagictower.summon.plist
rm ~/Library/LaunchAgents/com.themagictower.summon.plist
```

### Windows (Windows Service)

**PowerShell（管理者権限が必要）:**

```powershell
# 1. summonをWindows Serviceとして登録（nssm推奨）
# nssmインストール: winget install nssm

# サービス登録
nssm install Summon "$env:LOCALAPPDATA\summon\bin\summon.exe"
nssm set Summon AppParameters "--config `"$env:APPDATA\summon\config.yaml`""
nssm set Summon DisplayName "Summon LLM Proxy"
nssm set Summon Start SERVICE_AUTO_START

# サービス開始
Start-Service Summon

# サービス管理
Get-Service Summon      # ステータス確認
Stop-Service Summon     # 停止
Restart-Service Summon  # 再起動
sc delete Summon        # 削除
```

**またはWinSW使用:**

```powershell
# WinSWダウンロードと設定
# https://github.com/winsw/winsw/releases

# summon-service.xml作成:
@"
<service>
  <id>summon</id>
  <name>Summon LLM Proxy</name>
  <description>Model-based routing proxy for Claude Code</description>
  <executable>%LOCALAPPDATA%\summon\bin\summon.exe</executable>
  <arguments>--config "%APPDATA%\summon\config.yaml"</arguments>
  <log mode="roll-by-size">
    <sizeThreshold>10240</sizeThreshold>
    <keepFiles>8</keepFiles>
  </log>
</service>
"@ | Out-File "$env:LOCALAPPDATA\summon\bin\summon-service.xml" -Encoding UTF8

# サービス登録と開始
winsw install $env:LOCALAPPDATA\summon\bin\summon-service.xml
winsw start $env:LOCALAPPDATA\summon\bin\summon-service.xml
```

### Linux (systemd) - WSL含む

インストールスクリプトは環境を自動検出して適切なサービスタイプを選択します:
- **ユーザーサービス**: デスクトップ環境
- **システムサービス**: ヘッドレスサーバー（SSHセッション等）

#### 方法1: ユーザーサービス（デスクトップ環境）

**1. systemdサービスファイル作成:**

```bash
cat > ~/.config/systemd/user/summon.service << 'EOF'
[Unit]
Description=Summon LLM Proxy
After=network.target

[Service]
Type=simple
ExecStart=%h/.local/bin/summon --config %h/.config/summon/config.yaml
Restart=always
RestartSec=5
Environment="PATH=%h/.local/bin:/usr/local/bin:/usr/bin:/bin"

[Install]
WantedBy=default.target
EOF
```

**2. サービス登録と開始:**

```bash
# ユーザーサービスロード
systemctl --user daemon-reload
systemctl --user enable summon.service
systemctl --user start summon.service

# サービス管理
systemctl --user status summon    # ステータス確認
systemctl --user stop summon      # 停止
systemctl --user restart summon   # 再起動
systemctl --user disable summon   # 自動開始無効化
```

#### 方法2: システムサービス（ヘッドレスサーバー）

SSHセッション等D-Busユーザーセッションがない環境ではシステムレベルサービスを使用します。**sudo権限が必要です。**

**1. systemdサービスファイル作成（sudo必要）:**

```bash
sudo tee /etc/systemd/system/summon.service > /dev/null << 'EOF'
[Unit]
Description=Summon LLM Proxy
After=network.target

[Service]
Type=simple
User=$(whoami)
Group=$(id -gn)
ExecStart=/home/$(whoami)/.local/bin/summon --config /home/$(whoami)/.config/summon/config.yaml
Restart=always
RestartSec=5
Environment="PATH=/home/$(whoami)/.local/bin:/usr/local/bin:/usr/bin:/bin"

[Install]
WantedBy=multi-user.target
EOF
```

**2. サービス登録と開始（sudo必要）:**

```bash
# システムサービスロード
sudo systemctl daemon-reload
sudo systemctl enable summon.service
sudo systemctl start summon.service

# サービス管理
sudo systemctl status summon    # ステータス確認
sudo systemctl stop summon      # 停止
sudo systemctl restart summon   # 再起動
sudo systemctl disable summon   # 自動開始無効化

# ログ確認
journalctl -u summon -f
```

> **注**: WSL2でsystemdを使用するには、`/etc/wsl.conf`に`[boot] systemd=true`設定が必要な場合があります。

## 主な機能

- **透過的なプロキシ**: Claude Codeからプロキシの存在を認識できない
- **モデルベースルーティング**: `/v1/messages` POSTの`model`フィールドでルーティング決定
- **SSEストリーミング**: チャンク単位リアルタイムパススルー
- **サブスクリプション認証併用**: Anthropic OAuthトークンはそのまま維持、外部プロバイダーのみAPIキー置換
- **セキュリティ**: `127.0.0.1`のみにバインド、APIキーは環境変数参照

## ⚠️ 既知の制限

### 外部モデルに切り替え後Anthropic thinkingモデル使用不可

**一度外部プロバイダー（Kimi、Z.AI等）のモデルに切り替えた会話は、その後Anthropicのthinkingモデル（Claude Opus、Sonnet等）で継続できません。**

これはシステムアーキテクチャ上の制限であり解決できない問題です:
- 外部プロバイダーはAnthropicのネイティブメッセージ形式と完全互換ではない
- Thinkingモデルは特定のネイティブフィールドとコンテキスト構造に依存
- 外部モデルの応答はthinkingモデルが要求するコンテキスト形式を満たさない

**推奨使用方法:**
- 同一会話セッション内でモデルを切り替える必要がある場合、外部モデル↔外部モデル間でのみ切り替えてください
- Anthropic thinkingモデルが必要な場合、**新しい会話を開始**してください

## ロードマップ

- **v0.1** (現在): パススルー + モデルベースルーティング + SSEストリーミング
- **v0.2**: トランスフォーマー（リクエスト/応答変換 — 非互換プロバイダー対応）
- **v0.3**: ログ、ヘルスチェック、ホットリロード、タイムアウト

## ライセンス

MIT
