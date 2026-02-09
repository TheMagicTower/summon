# Summon

Một proxy ngược nhẹ bằng Rust định tuyến các yêu cầu API của Claude Code đến các nhà cung cấp LLM khác nhau dựa trên tên mô hình.

Duy trì xác thực đăng ký Anthropic (OAuth) hiện có của bạn trong khi chuyển hướng các mô hình cụ thể đến các nhà cung cấp bên ngoài (Z.AI, Kimi, v.v.).

## Kiến trúc

```
Claude Code CLI
  │ ANTHROPIC_BASE_URL=http://127.0.0.1:18081
  ▼
Proxy (máy chủ axum)
  ├─ /v1/messages POST → phân tích trường model → quyết định định tuyến
  │   ├─ Khớp → Nhà cung cấp bên ngoài (thay thế header/xác thực)
  │   └─ Không khớp → Anthropic API (passthrough)
  └─ Các yêu cầu khác → Anthropic API (passthrough)
```

## Cài đặt

### Cài đặt một dòng (Khuyến nghị)

**Linux/macOS/WSL:**
```bash
curl -fsSL https://raw.githubusercontent.com/TheMagicTower/summon/master/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/TheMagicTower/summon/master/install.ps1 | iex
```

> 💡 **Người dùng WSL**: Bạn có thể sử dụng Claude Code từ cả phía WSL và Windows. Xem phần [Cách sử dụng WSL](#cách-sử-dụng-wsl) bên dưới để biết chi tiết.

### Tải xuống Binary

Tải xuống binary cho nền tảng của bạn từ trang [Releases](https://github.com/TheMagicTower/summon/releases).

| Nền tảng | Tệp |
|----------|------|
| Linux x86_64 | `summon-linux-amd64.tar.gz` |
| Linux ARM64 | `summon-linux-arm64.tar.gz` |
| macOS Intel | `summon-darwin-amd64.tar.gz` |
| macOS Apple Silicon | `summon-darwin-arm64.tar.gz` |
| Windows x86_64 | `summon-windows-amd64.zip` |
| Windows ARM64 | `summon-windows-arm64.zip` |

```bash
# Ví dụ: macOS Apple Silicon
tar xzf summon-darwin-arm64.tar.gz
chmod +x summon-darwin-arm64
sudo mv summon-darwin-arm64 /usr/local/bin/summon
```

### Biên dịch từ nguồn

```bash
cargo build --release
```

## Cấu hình

### Vị trí tệp cấu hình

summon tìm kiếm tệp cấu hình theo thứ tự ưu tiên sau:

| Ưu tiên | Vị trí | Mô tả |
|---------|--------|------|
| 1 | `--config <đường_dẫn>` | Chỉ định rõ ràng |
| 2 | Biến môi trường `SUMMON_CONFIG` | Đường dẫn được chỉ định bởi biến môi trường |
| 3 | `~/.config/summon/config.yaml` | Cấu hình cụ thể của người dùng (XDG) |
| 4 | `/etc/summon/config.yaml` | Cấu hình hệ thống |
| 5 | `./config.yaml` | Thư mục hiện tại |

### Môi trường đa người dùng

Để mỗi người dùng có cấu hình riêng:
```bash
mkdir -p ~/.config/summon
cp /path/to/config.yaml ~/.config/summon/
```

Để quản trị viên hệ thống cung cấp cấu hình mặc định:
```bash
sudo mkdir -p /etc/summon
sudo cp config.yaml /etc/summon/
```

### Ví dụ tệp cấu hình

Tạo tệp `config.yaml`:

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

- `match`: Khớp nếu chuỗi này có trong tên mô hình (thứ tự từ trên xuống dưới, khớp đầu tiên được áp dụng)
- `${ENV_VAR}`: Tham chiếu biến môi trường (không viết khóa API trực tiếp vào tệp cấu hình)
- Các mô hình không khớp được chuyển qua `default.url` (Anthropic API)

## Chạy

```bash
# Thiết lập biến môi trường
export Z_AI_API_KEY="your-z-ai-key"
export KIMI_API_KEY="your-kimi-key"

# Khởi động proxy (tệp cấu hình được tự động phát hiện)
summon

# Hoặc chỉ định tệp cấu hình trực tiếp
summon --config /path/to/config.yaml

# Tích hợp với Claude Code
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

## Cách sử dụng WSL

Bạn cũng có thể sử dụng summon từ WSL (Windows Subsystem for Linux).

### Sử dụng Claude Code từ phía WSL

```bash
# Trong terminal WSL (giả sử tệp cấu hình được đặt tại ~/.config/summon/config.yaml)
summon

# Trong terminal WSL khác
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

### Sử dụng Claude Code từ phía Windows (summon chạy trong WSL)

```bash
# Chạy summon trong WSL (bind đến 0.0.0.0 để có thể truy cập từ Windows)
summon

# Trong terminal Windows (PowerShell/CMD)
# Kiểm tra IP WSL: ip addr show eth0 | grep 'inet '
ANTHROPIC_BASE_URL=http://$(wsl hostname -I | awk '{print $1}'):18081 claude
```

Ngoài ra, bạn có thể đặt `server.host` thành `"0.0.0.0"` trong `config.yaml` để có thể truy cập từ Windows.

## Đăng ký làm dịch vụ nền

### macOS (launchd)

**1. Tạo tệp plist LaunchAgent:**

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

**2. Tạo thư mục log và đăng ký dịch vụ:**

```bash
mkdir -p ~/.local/share/summon
launchctl load ~/Library/LaunchAgents/com.themagictower.summon.plist
launchctl start com.themagictower.summon
```

**3. Quản lý dịch vụ:**

```bash
# Kiểm tra trạng thái
launchctl list | grep com.themagictower.summon

# Dừng
launchctl stop com.themagictower.summon

# Khởi động lại
launchctl stop com.themagictower.summon && launchctl start com.themagictower.summon

# Xóa
launchctl unload ~/Library/LaunchAgents/com.themagictower.summon.plist
rm ~/Library/LaunchAgents/com.themagictower.summon.plist
```

### Windows (Windows Service)

**PowerShell (yêu cầu quyền quản trị viên):**

```powershell
# 1. Đăng ký summon làm Windows Service (khuyến nghị nssm)
# Cài đặt nssm: winget install nssm

# Đăng ký dịch vụ
nssm install Summon "$env:LOCALAPPDATA\summon\bin\summon.exe"
nssm set Summon AppParameters "--config `"$env:APPDATA\summon\config.yaml`""
nssm set Summon DisplayName "Summon LLM Proxy"
nssm set Summon Start SERVICE_AUTO_START

# Khởi động dịch vụ
Start-Service Summon

# Quản lý dịch vụ
Get-Service Summon      # Kiểm tra trạng thái
Stop-Service Summon     # Dừng
Restart-Service Summon  # Khởi động lại
sc delete Summon        # Xóa
```

**Hoặc sử dụng WinSW:**

```powershell
# Tải xuống và cấu hình WinSW
# https://github.com/winsw/winsw/releases

# Tạo summon-service.xml:
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

# Đăng ký và khởi động dịch vụ
winsw install $env:LOCALAPPDATA\summon\bin\summon-service.xml
winsw start $env:LOCALAPPDATA\summon\bin\summon-service.xml
```

### Linux (systemd) - Bao gồm WSL

Script cài đặt tự động phát hiện môi trường và chọn loại dịch vụ phù hợp:
- **Dịch vụ người dùng**: Môi trường desktop
- **Dịch vụ hệ thống**: Máy chủ không giao diện (ses SSH, v.v.)

#### Phương pháp 1: Dịch vụ người dùng (Môi trường Desktop)

**1. Tạo tệp dịch vụ systemd:**

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

**2. Đăng ký và khởi động dịch vụ:**

```bash
# Tải dịch vụ người dùng
systemctl --user daemon-reload
systemctl --user enable summon.service
systemctl --user start summon.service

# Quản lý dịch vụ
systemctl --user status summon    # Kiểm tra trạng thái
systemctl --user stop summon      # Dừng
systemctl --user restart summon   # Khởi động lại
systemctl --user disable summon   # Vô hiệu hóa tự động khởi động
```

#### Phương pháp 2: Dịch vụ hệ thống (Máy chủ không giao diện)

Đối với môi trường không có ses người dùng D-Bus như ses SSH, sử dụng dịch vụ cấp hệ thống. **Yêu cầu quyền sudo.**

**1. Tạo tệp dịch vụ systemd (yêu cầu sudo):**

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

**2. Đăng ký và khởi động dịch vụ (yêu cầu sudo):**

```bash
# Tải dịch vụ hệ thống
sudo systemctl daemon-reload
sudo systemctl enable summon.service
sudo systemctl start summon.service

# Quản lý dịch vụ
sudo systemctl status summon    # Kiểm tra trạng thái
sudo systemctl stop summon      # Dừng
sudo systemctl restart summon   # Khởi động lại
sudo systemctl disable summon   # Vô hiệu hóa tự động khởi động

# Xem nhật ký
journalctl -u summon -f
```

> **Lưu ý**: Để sử dụng systemd trong WSL2, bạn có thể cần đặt `[boot] systemd=true` trong `/etc/wsl.conf`.

## Tính năng chính

- **Proxy trong suốt**: Claude Code không nhận biết sự tồn tại của proxy
- **Định tuyến dựa trên mô hình**: Quyết định định tuyến dựa trên trường `model` trong `/v1/messages` POST
- **Streaming SSE**: Passthrough thời gian thực theo từng khối
- **Xác thực đăng ký đồng thời**: Token OAuth Anthropic được giữ nguyên, chỉ nhà cung cấp bên ngoài sử dụng khóa API
- **Bảo mật**: Chỉ bind đến `127.0.0.1`, khóa API được tham chiếu từ biến môi trường

## ⚠️ Hạn chế đã biết

### Không thể sử dụng mô hình thinking Anthropic sau khi chuyển sang mô hình bên ngoài

**Khi một cuộc hội thoại đã được chuyển sang mô hình của nhà cung cấp bên ngoài (Kimi, Z.AI, v.v.), bạn không thể tiếp tục với các mô hình thinking của Anthropic (Claude Opus, Sonnet, v.v.) trong cùng cuộc hội thoại đó.**

Đây là hạn chế về kiến trúc hệ thống không thể giải quyết:
- Nhà cung cấp bên ngoài không hoàn toàn tương thích với định dạng thư mục gốc của Anthropic
- Mô hình thinking phụ thuộc vào các trường gốc và cấu trúc bối cảnh cụ thể
- Phản hồi từ mô hình bên ngoài không đáp ứng định dạng bối cảnh được yêu cầu bởi mô hình thinking

**Cách sử dụng được khuyến nghị:**
- Khi chuyển đổi mô hình trong cùng một ses hội thoại, chỉ chuyển đổi giữa mô hình bên ngoài ↔ mô hình bên ngoài
- Nếu bạn cần mô hình thinking Anthropic, **hãy bắt đầu cuộc hội thoại mới**

## Lộ trình

- **v0.1** (hiện tại): Passthrough + định tuyến dựa trên mô hình + streaming SSE
- **v0.2**: Bộ biến đổi (chuyển đổi yêu cầu/phản hồi — cho các nhà cung cấp không tương thích)
- **v0.3**: Ghi nhật ký, kiểm tra sức khỏe, tải lại nóng, thời gian chờ

## Giấy phép

MIT
