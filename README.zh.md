🌐 [English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [Español](README.es.md) | [Deutsch](README.de.md) | [Tiếng Việt](README.vi.md)

# Summon

一个基于Rust的轻量级反向代理，根据模型名称将Claude Code的API请求路由到不同的LLM提供商。

在保持现有Anthropic订阅（OAuth）身份验证的同时，将特定模型分支到外部提供商（Z.AI、Kimi等）。

## 架构

```
Claude Code CLI
  │ ANTHROPIC_BASE_URL=http://127.0.0.1:18081
  ▼
代理 (axum服务器)
  ├─ /v1/messages POST → 解析model字段 → 路由决策
  │   ├─ 匹配 → 外部提供商（替换头部/身份验证）
  │   └─ 不匹配 → Anthropic API（透传）
  └─ 其他请求 → Anthropic API（透传）
```

## 安装

### 一键安装（推荐）

**Linux/macOS/WSL:**
```bash
curl -fsSL https://raw.githubusercontent.com/TheMagicTower/summon/master/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/TheMagicTower/summon/master/install.ps1 | iex
```

> 💡 **WSL用户**: 您可以在WSL和Windows两侧都使用Claude Code。详情请参阅下面的[WSL使用方法](#wsl使用方法)部分。

### 二进制下载

从[Releases](https://github.com/TheMagicTower/summon/releases)页面下载适合您平台的二进制文件。

| 平台 | 文件 |
|----------|----------|
| Linux x86_64 | `summon-linux-amd64.tar.gz` |
| Linux ARM64 | `summon-linux-arm64.tar.gz` |
| macOS Intel | `summon-darwin-amd64.tar.gz` |
| macOS Apple Silicon | `summon-darwin-arm64.tar.gz` |
| Windows x86_64 | `summon-windows-amd64.zip` |
| Windows ARM64 | `summon-windows-arm64.zip` |

```bash
# 示例：macOS Apple Silicon
tar xzf summon-darwin-arm64.tar.gz
chmod +x summon-darwin-arm64
sudo mv summon-darwin-arm64 /usr/local/bin/summon
```

### 从源代码构建

```bash
cargo build --release
```

## 配置

### 配置文件位置

summon按以下优先级搜索配置文件：

| 优先级 | 位置 | 说明 |
|---------|------|------|
| 1 | `--config <路径>` | 显式指定 |
| 2 | `SUMMON_CONFIG`环境变量 | 环境变量指定的路径 |
| 3 | `~/.config/summon/config.yaml` | 用户特定配置（XDG） |
| 4 | `/etc/summon/config.yaml` | 系统级配置 |
| 5 | `./config.yaml` | 当前目录 |

### 多用户环境

为每个用户提供自己的配置：
```bash
mkdir -p ~/.config/summon
cp /path/to/config.yaml ~/.config/summon/
```

为系统管理员提供默认配置：
```bash
sudo mkdir -p /etc/summon
sudo cp config.yaml /etc/summon/
```

### 配置方式

根据提供商和用例，有两种配置方式。

#### 方式1：兼容提供商（模型名原样传递）

适用于原生理解Anthropic模型名的提供商（如Z.AI、Kimi）。Claude Code发送的原始模型名将原样转发。

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

- Claude Code发送`model: "claude-haiku-4-5-20251001"` → 匹配`"claude-haiku"` → 路由到Z.AI
- 提供商决定使用哪个实际模型来处理Anthropic模型名
- 简单设置，无需额外的Claude Code配置

#### 方式2：自定义模型绑定（指定特定模型）

当您想使用特定的上游模型时（例如用`glm-4.7`代替提供商映射的`claude-haiku`）。在Claude Code的`settings.json`中覆盖模型名：

**步骤1.** 配置Claude Code发送自定义模型名：

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

| 环境变量 | 说明 |
|---------|------|
| `ANTHROPIC_BASE_URL` | 代理地址（无需每次启动时指定） |
| `ANTHROPIC_DEFAULT_HAIKU_MODEL` | 选择Haiku级别时发送的模型名 |
| `ANTHROPIC_DEFAULT_SONNET_MODEL` | 选择Sonnet级别时发送的模型名 |
| `ANTHROPIC_DEFAULT_OPUS_MODEL` | 选择Opus级别时发送的模型名 |

**步骤2.** 在`config.yaml`中匹配覆盖后的模型名：

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

- Claude Code发送`model: "glm-4.7"`（已覆盖） → 匹配`"glm"` → 路由到Z.AI并使用精确模型
- 您可以精确控制提供商使用哪个模型
- 在`settings.json`中设置`ANTHROPIC_BASE_URL`后，可以直接运行`claude`而无需额外环境变量

### 配置参考

- `match`: 如果模型名包含此字符串则匹配（从上到下顺序，应用第一个匹配）
- `${ENV_VAR}`: 环境变量引用（API密钥不直接写入配置文件）
- `upstream.auth.pool`: 用于负载均衡的额外API密钥值（使用与`auth.header`相同的头部）
- `concurrency`: 每个密钥的并发请求限制（超过时回退到Anthropic或返回429）
- `fallback`: 提供商失败时的回退行为（默认：`true`）
  - `false`: 不回退，原样返回错误
  - `true`: 使用原始模型名回退到Anthropic API
  - `"模型名"`: 使用指定的模型名替换后回退到Anthropic API（非Anthropic模型名推荐使用）
- 不匹配的模型透传到`default.url`（Anthropic API）

### API 密钥池（并发限制处理）

某些提供商限制每个API密钥的并发请求数（例如：GLM-5每个密钥仅允许1个并发请求）。可以将多个API密钥注册为池以提高总并发数：

```yaml
routes:
  - match: "glm-5"
    concurrency: 1           # 每个密钥的并发请求限制
    upstream:
      url: "https://open.bigmodel.cn/api/paas/v4"
      auth:
        header: "Authorization"
        value: "Bearer ${GLM_KEY_1}"
        pool:                 # 额外密钥（相同的头部）
          - "Bearer ${GLM_KEY_2}"
          - "Bearer ${GLM_KEY_3}"
    transformer: "openai"
    model_map: "glm-5"
```

**工作原理：**

- 请求被分发到活动连接最少的密钥（**Least-Connections**）
- 每个密钥的并发使用量由`concurrency`设置跟踪和限制
- 当所有密钥都达到限制时：回退到Anthropic（如果启用了`fallback`）或返回HTTP 429。使用`fallback: "claude-sonnet-4-5-20250929"`可以安全地以兼容模型名回退
- 流式响应在流结束时自动释放密钥

## 运行

```bash
# 设置环境变量
export Z_AI_API_KEY="your-z-ai-key"
export KIMI_API_KEY="your-kimi-key"

# 启动代理（自动检测配置文件）
summon

# 或直接指定配置文件
summon --config /path/to/config.yaml
```

### 连接Claude Code

**选项A：手动（每次会话）**
```bash
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

**选项B：自动（推荐）**

添加到`~/.claude/settings.json`，这样无需每次都指定URL：
```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:18081"
  }
}
```

然后直接运行：
```bash
claude
```

## CLI管理

### 自我更新

检查新版本并原地更新二进制文件：

```bash
summon update
```

更新命令会：
1. 将当前版本与最新的GitHub发布版本进行比较
2. 如果有新版本可用，提示确认
3. 自动下载并替换二进制文件

> Windows: 不支持自我更新。请改用`install.ps1`。

### 直接命令

所有管理命令都是顶级命令：

```bash
summon status          # 显示当前状态
summon enable          # 启用代理（修改settings.json + 启动）
summon disable         # 禁用代理（停止 + 恢复settings.json）
summon start           # 在后台启动代理
summon stop            # 停止代理
summon add             # 添加提供商路由
summon remove          # 删除提供商路由
summon restore         # 从备份恢复settings.json
```

### 交互式配置

运行`summon configure`会打开包含所有可用操作的交互式菜单：

```bash
summon configure
```

## WSL使用方法

您也可以在WSL（Windows Subsystem for Linux）中使用summon。

### 在WSL侧使用Claude Code

```bash
# 在WSL终端中（假设配置文件放在 ~/.config/summon/config.yaml）
summon

# 在另一个WSL终端中
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

### 在Windows侧使用Claude Code（在WSL中运行summon）

```bash
# 在WSL中运行summon（绑定到0.0.0.0以使其可从Windows访问）
summon

# 在Windows终端（PowerShell/CMD）中
# 检查WSL IP：ip addr show eth0 | grep 'inet '
ANTHROPIC_BASE_URL=http://$(wsl hostname -I | awk '{print $1}'):18081 claude
```

或者，您可以在`config.yaml`中将`server.host`设置为`"0.0.0.0"`以使其可从Windows访问。

## 注册为后台服务

### macOS (launchd)

**1. 创建LaunchAgent plist文件：**

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

**2. 创建日志目录并注册服务：**

```bash
mkdir -p ~/.local/share/summon
launchctl load ~/Library/LaunchAgents/com.themagictower.summon.plist
launchctl start com.themagictower.summon
```

**3. 服务管理：**

```bash
# 检查状态
launchctl list | grep com.themagictower.summon

# 停止
launchctl stop com.themagictower.summon

# 重启
launchctl stop com.themagictower.summon && launchctl start com.themagictower.summon

# 删除
launchctl unload ~/Library/LaunchAgents/com.themagictower.summon.plist
rm ~/Library/LaunchAgents/com.themagictower.summon.plist
```

### Windows (Windows Service)

**PowerShell（需要管理员权限）：**

```powershell
# 1. 将summon注册为Windows服务（推荐使用nssm）
# 安装nssm：winget install nssm

# 注册服务
nssm install Summon "$env:LOCALAPPDATA\summon\bin\summon.exe"
nssm set Summon AppParameters "--config `"$env:APPDATA\summon\config.yaml`""
nssm set Summon DisplayName "Summon LLM Proxy"
nssm set Summon Start SERVICE_AUTO_START

# 启动服务
Start-Service Summon

# 服务管理
Get-Service Summon      # 检查状态
Stop-Service Summon     # 停止
Restart-Service Summon  # 重启
sc delete Summon        # 删除
```

**或使用WinSW：**

```powershell
# 下载并配置WinSW
# https://github.com/winsw/winsw/releases

# 创建summon-service.xml：
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

# 注册并启动服务
winsw install $env:LOCALAPPDATA\summon\bin\summon-service.xml
winsw start $env:LOCALAPPDATA\summon\bin\summon-service.xml
```

### Linux (systemd) - 包括WSL

安装脚本会自动检测环境并选择适当的服务类型：
- **用户服务**：桌面环境
- **系统服务**：无头服务器（SSH会话等）

#### 方法1：用户服务（桌面环境）

**1. 创建systemd服务文件：**

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

**2. 注册并启动服务：**

```bash
# 加载用户服务
systemctl --user daemon-reload
systemctl --user enable summon.service
systemctl --user start summon.service

# 服务管理
systemctl --user status summon    # 检查状态
systemctl --user stop summon      # 停止
systemctl --user restart summon   # 重启
systemctl --user disable summon   # 禁用自动启动
```

#### 方法2：系统服务（无头服务器）

对于没有D-Bus用户会话的环境（如SSH会话），使用系统级服务。**需要sudo权限。**

**1. 创建systemd服务文件（需要sudo）：**

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

**2. 注册并启动服务（需要sudo）：**

```bash
# 加载系统服务
sudo systemctl daemon-reload
sudo systemctl enable summon.service
sudo systemctl start summon.service

# 服务管理
sudo systemctl status summon    # 检查状态
sudo systemctl stop summon      # 停止
sudo systemctl restart summon   # 重启
sudo systemctl disable summon   # 禁用自动启动

# 查看日志
journalctl -u summon -f
```

> **注意**：要在WSL2中使用systemd，您可能需要在`/etc/wsl.conf`中设置`[boot] systemd=true`。

## 主要功能

- **透明代理**：Claude Code感知不到代理的存在
- **基于模型的路由**：基于`/v1/messages` POST中的`model`字段进行路由决策
- **SSE流式传输**：按块实时透传
- **并发订阅身份验证**：Anthropic OAuth令牌保持不变，仅外部提供商使用API密钥
- **API密钥池**：为有每密钥并发限制的提供商提供支持，通过Least-Connections分配实现每个路由多个API密钥
- **回退模型名**：使用非Anthropic模型名时，指定兼容的Anthropic模型名以实现安全回退
- **安全性**：仅绑定到`127.0.0.1`，API密钥从环境变量引用

## ⚠️ 已知限制

### 切换到外部模型后无法使用Anthropic思考模型

**一旦对话切换到外部提供商的模型（Kimi、Z.AI等），您就无法在同一对话中继续使用Anthropic的思考模型（Claude Opus、Sonnet等）。**

这是系统架构的限制，无法解决：
- 外部提供商与Anthropic的原生消息格式不完全兼容
- 思考模型依赖于特定的原生字段和上下文结构
- 外部模型的响应不满足思考模型所需的上下文格式

**推荐用法：**
- 在同一对话会话中切换模型时，仅在外部模型↔外部模型之间切换
- 如果需要Anthropic思考模型，**请开始新对话**

## 路线图

- **v0.1**：透传 + 基于模型的路由 + SSE流式传输
- **v0.2**（当前）：转换器、API密钥池、回退模型名、交互式CLI、自我更新
- **v0.3**：日志记录、健康检查、热重载、超时

## 许可证

MIT
