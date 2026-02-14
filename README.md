🌐 [한국어](README.ko.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Español](README.es.md) | [Deutsch](README.de.md) | [Tiếng Việt](README.vi.md)

# Summon

A lightweight reverse proxy in Rust that routes Claude Code API requests to different LLM providers based on model name.

Maintains your existing Anthropic subscription (OAuth) authentication while branching specific models to external providers (Z.AI, Kimi, etc.).

## Architecture

```
Claude Code CLI
  │ ANTHROPIC_BASE_URL=http://127.0.0.1:18081
  ▼
Proxy (axum server)
  ├─ /v1/messages POST → model field parsing → routing decision
  │   ├─ Match → External provider (header/auth replacement)
  │   └─ No match → Anthropic API (passthrough)
  └─ Other requests → Anthropic API (passthrough)
```

## Installation

### One-line Installation (Recommended)

**Linux/macOS/WSL:**
```bash
curl -fsSL https://raw.githubusercontent.com/TheMagicTower/summon/master/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/TheMagicTower/summon/master/install.ps1 | iex
```

> 💡 **WSL Users**: You can use Claude Code from both WSL and Windows sides. See the [WSL Usage](#wsl-usage) section below for details.

### Binary Download

Download the binary for your platform from the [Releases](https://github.com/TheMagicTower/summon/releases) page.

| Platform | File |
|----------|------|
| Linux x86_64 | `summon-linux-amd64.tar.gz` |
| Linux ARM64 | `summon-linux-arm64.tar.gz` |
| macOS Intel | `summon-darwin-amd64.tar.gz` |
| macOS Apple Silicon | `summon-darwin-arm64.tar.gz` |
| Windows x86_64 | `summon-windows-amd64.zip` |
| Windows ARM64 | `summon-windows-arm64.zip` |

```bash
# Example: macOS Apple Silicon
tar xzf summon-darwin-arm64.tar.gz
chmod +x summon-darwin-arm64
sudo mv summon-darwin-arm64 /usr/local/bin/summon
```

### Build from Source

```bash
cargo build --release
```

## Configuration

### Configuration File Location

summon searches for configuration files in the following priority order:

| Priority | Location | Description |
|----------|----------|-------------|
| 1 | `--config <path>` | Explicit specification |
| 2 | `SUMMON_CONFIG` environment variable | Path specified by environment variable |
| 3 | `~/.config/summon/config.yaml` | User-specific configuration (XDG) |
| 4 | `/etc/summon/config.yaml` | System-wide configuration |
| 5 | `./config.yaml` | Current directory |

### Multi-user Environment

For each user to have their own configuration:
```bash
mkdir -p ~/.config/summon
cp /path/to/config.yaml ~/.config/summon/
```

For system administrators to provide default configuration:
```bash
sudo mkdir -p /etc/summon
sudo cp config.yaml /etc/summon/
```

### Configuration Approaches

There are two approaches depending on your provider and use case.

#### Approach 1: Compatible Providers (Model Name Passthrough)

For providers that natively understand Anthropic model names (e.g., Z.AI, Kimi). The original model name from Claude Code is forwarded as-is.

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
      url: "https://api.kimi.com/coding"
      auth:
        header: "Authorization"
        value: "Bearer ${KIMI_API_KEY}"
```

- Claude Code sends `model: "claude-haiku-4-5-20251001"` → matches `"claude-haiku"` → routed to Z.AI
- The provider decides which actual model to use for the Anthropic model name
- Simple setup, no additional Claude Code configuration needed

#### Approach 2: Custom Model Binding (Specific Model Selection)

When you want to use a specific upstream model (e.g., `glm-4.7` instead of whatever the provider maps `claude-haiku` to). Override model names in Claude Code's `settings.json`:

**Step 1.** Configure Claude Code to send custom model names:

```json
// ~/.claude/settings.json
{
  "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:18081",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "glm-4.7",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "kimi-for-coding"
  }
}
```

| Environment Variable | Description |
|---------------------|-------------|
| `ANTHROPIC_BASE_URL` | Proxy address (also eliminates the need to specify it on every launch) |
| `ANTHROPIC_DEFAULT_HAIKU_MODEL` | Model name sent when Haiku tier is selected |
| `ANTHROPIC_DEFAULT_SONNET_MODEL` | Model name sent when Sonnet tier is selected |
| `ANTHROPIC_DEFAULT_OPUS_MODEL` | Model name sent when Opus tier is selected |

**Step 2.** Match on the overridden model names in `config.yaml`:

```yaml
server:
  host: "127.0.0.1"
  port: 18081

default:
  url: "https://api.anthropic.com"

routes:
  - match: "glm"
    upstream:
      url: "https://api.z.ai/api/anthropic"
      auth:
        header: "x-api-key"
        value: "${Z_AI_API_KEY}"

  - match: "kimi"
    upstream:
      url: "https://api.kimi.com/coding"
      auth:
        header: "Authorization"
        value: "Bearer ${KIMI_API_KEY}"
```

- Claude Code sends `model: "glm-4.7"` (overridden) → matches `"glm"` → routed to Z.AI with exact model
- You control exactly which model the provider uses
- `ANTHROPIC_BASE_URL` in `settings.json` means you can just run `claude` without extra env vars

### Configuration Reference

- `match`: Matches if this string is contained in the model name (top to bottom order, first match applies)
- `${ENV_VAR}`: Environment variable reference (API keys are not written directly in the configuration file)
- `upstream.auth.pool`: Additional API key values for load distribution (same header as `auth.header`)
- `concurrency`: Per-key concurrent request limit (when exceeded, falls back to Anthropic or returns 429)
- `fallback`: Whether to fall back to Anthropic API on provider failure (default: `true`)
- Models that don't match are passed through to `default.url` (Anthropic API)

### API Key Pool (Concurrency Limit Handling)

Some providers limit concurrent requests per API key (e.g., GLM-5 allows only 1 concurrent request per key). Register multiple API keys as a pool to increase total concurrency:

```yaml
routes:
  - match: "glm-5"
    concurrency: 1           # per-key concurrent request limit
    upstream:
      url: "https://open.bigmodel.cn/api/paas/v4"
      auth:
        header: "Authorization"
        value: "Bearer ${GLM_KEY_1}"
        pool:                 # additional keys (same header)
          - "Bearer ${GLM_KEY_2}"
          - "Bearer ${GLM_KEY_3}"
    transformer: "openai"
    model_map: "glm-5"
```

**How it works:**

- Requests are distributed to the key with the fewest active connections (**Least-Connections**)
- Each key's concurrent usage is tracked and limited by the `concurrency` setting
- When all keys reach their limit: fallback to Anthropic (if `fallback: true`) or return HTTP 429
- Streaming responses automatically release the key when the stream ends

## Running

```bash
# Set environment variables
export Z_AI_API_KEY="your-z-ai-key"
export KIMI_API_KEY="your-kimi-key"

# Start proxy (configuration file auto-detected)
summon

# Or specify configuration file directly
summon --config /path/to/config.yaml
```

### Connecting Claude Code

**Option A: Manual (per-session)**
```bash
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

**Option B: Automatic (recommended)**

Add to `~/.claude/settings.json` so you never need to specify the URL again:
```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:18081"
  }
}
```

Then simply run:
```bash
claude
```

## CLI Management

### Self-Update

Check for new releases and update the binary in place:

```bash
summon update
```

The update command:
1. Checks the current version against the latest GitHub release
2. Prompts for confirmation if a newer version is available
3. Downloads and replaces the binary automatically

> Windows: Self-update is not supported. Use `install.ps1` instead.

### Direct Commands

All management commands are top-level:

```bash
summon status          # Show current status
summon enable          # Enable proxy (modify settings.json + start)
summon disable         # Disable proxy (stop + restore settings.json)
summon start           # Start proxy in background
summon stop            # Stop proxy
summon add             # Add a provider route
summon remove          # Remove a provider route
summon restore         # Restore settings.json from backup
```

### Interactive Configuration

Running `summon configure` opens an interactive menu with all available actions:

```bash
summon configure
```

## WSL Usage

You can also use summon from WSL (Windows Subsystem for Linux).

### Using Claude Code from WSL Side

```bash
# In WSL terminal (assuming config file is placed at ~/.config/summon/config.yaml)
summon

# In another WSL terminal
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

### Using Claude Code from Windows Side (summon running in WSL)

```bash
# Run summon in WSL (bind to 0.0.0.0 to make it accessible from Windows)
summon

# In Windows terminal (PowerShell/CMD)
# Check WSL IP: ip addr show eth0 | grep 'inet '
ANTHROPIC_BASE_URL=http://$(wsl hostname -I | awk '{print $1}'):18081 claude
```

Alternatively, you can set `server.host` to `"0.0.0.0"` in `config.yaml` to make it accessible from Windows.

## Register as Background Service

### macOS (launchd)

**1. Create LaunchAgent plist file:**

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

**2. Create log directory and register service:**

```bash
mkdir -p ~/.local/share/summon
launchctl load ~/Library/LaunchAgents/com.themagictower.summon.plist
launchctl start com.themagictower.summon
```

**3. Service management:**

```bash
# Check status
launchctl list | grep com.themagictower.summon

# Stop
launchctl stop com.themagictower.summon

# Restart
launchctl stop com.themagictower.summon && launchctl start com.themagictower.summon

# Remove
launchctl unload ~/Library/LaunchAgents/com.themagictower.summon.plist
rm ~/Library/LaunchAgents/com.themagictower.summon.plist
```

### Windows (Windows Service)

**PowerShell (requires administrator privileges):**

```powershell
# 1. Register summon as Windows Service (nssm recommended)
# Install nssm: winget install nssm

# Register service
nssm install Summon "$env:LOCALAPPDATA\summon\bin\summon.exe"
nssm set Summon AppParameters "--config `"$env:APPDATA\summon\config.yaml`""
nssm set Summon DisplayName "Summon LLM Proxy"
nssm set Summon Start SERVICE_AUTO_START

# Start service
Start-Service Summon

# Service management
Get-Service Summon      # Check status
Stop-Service Summon     # Stop
Restart-Service Summon  # Restart
sc delete Summon        # Remove
```

**Or use WinSW:**

```powershell
# Download and configure WinSW
# https://github.com/winsw/winsw/releases

# Create summon-service.xml:
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

# Register and start service
winsw install $env:LOCALAPPDATA\summon\bin\summon-service.xml
winsw start $env:LOCALAPPDATA\summon\bin\summon-service.xml
```

### Linux (systemd) - Including WSL

The installation script automatically detects the environment and selects the appropriate service type:
- **User service**: Desktop environment
- **System service**: Headless server (SSH sessions, etc.)

#### Method 1: User Service (Desktop Environment)

**1. Create systemd service file:**

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

**2. Register and start service:**

```bash
# Load user service
systemctl --user daemon-reload
systemctl --user enable summon.service
systemctl --user start summon.service

# Service management
systemctl --user status summon    # Check status
systemctl --user stop summon      # Stop
systemctl --user restart summon   # Restart
systemctl --user disable summon   # Disable auto-start
```

#### Method 2: System Service (Headless Server)

For environments without D-Bus user sessions such as SSH sessions, use a system-level service. **Requires sudo privileges.**

**1. Create systemd service file (requires sudo):**

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

**2. Register and start service (requires sudo):**

```bash
# Load system service
sudo systemctl daemon-reload
sudo systemctl enable summon.service
sudo systemctl start summon.service

# Service management
sudo systemctl status summon    # Check status
sudo systemctl stop summon      # Stop
sudo systemctl restart summon   # Restart
sudo systemctl disable summon   # Disable auto-start

# View logs
journalctl -u summon -f
```

> **Note**: To use systemd in WSL2, you may need to set `[boot] systemd=true` in `/etc/wsl.conf`.

## Key Features

- **Transparent Proxy**: Claude Code is unaware of the proxy's existence
- **Model-based Routing**: Routing decision based on `model` field in `/v1/messages` POST
- **SSE Streaming**: Real-time passthrough in chunks
- **Concurrent Subscription Auth**: Anthropic OAuth tokens remain intact, only external providers use API keys
- **API Key Pool**: Multiple API keys per route with Least-Connections distribution for providers with per-key concurrency limits
- **Security**: Binds only to `127.0.0.1`, API keys referenced from environment variables

## ⚠️ Known Limitations

### Cannot Use Anthropic Thinking Models After Switching to External Models

**Once a conversation has been switched to an external provider's model (Kimi, Z.AI, etc.), you cannot continue with Anthropic's thinking models (Claude Opus, Sonnet, etc.) in the same conversation.**

This is a system architecture limitation that cannot be resolved:
- External providers are not fully compatible with Anthropic's native message format
- Thinking models depend on specific native fields and context structures
- External model responses do not meet the context format required by thinking models

**Recommended Usage:**
- When switching models within the same conversation session, only switch between external models ↔ external models
- If you need Anthropic thinking models, **start a new conversation**

## Roadmap

- **v0.1** (current): Passthrough + model-based routing + SSE streaming
- **v0.2**: Transformer (request/response transformation — for incompatible providers)
- **v0.3**: Logging, health check, hot reload, timeout

## License

MIT
