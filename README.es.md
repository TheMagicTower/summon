# Summon

Un proxy inverso ligero en Rust que enruta las solicitudes de API de Claude Code a diferentes proveedores de LLM según el nombre del modelo.

Mantiene tu autenticación de suscripción existente de Anthropic (OAuth) mientras deriva modelos específicos a proveedores externos (Z.AI, Kimi, etc.).

## Arquitectura

```
Claude Code CLI
  │ ANTHROPIC_BASE_URL=http://127.0.0.1:18081
  ▼
Proxy (servidor axum)
  ├─ /v1/messages POST → análisis del campo model → decisión de enrutamiento
  │   ├─ Coincidencia → Proveedor externo (reemplazo de encabezados/autenticación)
  │   └─ Sin coincidencia → Anthropic API (passthrough)
  └─ Otras solicitudes → Anthropic API (passthrough)
```

## Instalación

### Instalación en una línea (Recomendado)

**Linux/macOS/WSL:**
```bash
curl -fsSL https://raw.githubusercontent.com/TheMagicTower/summon/master/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/TheMagicTower/summon/master/install.ps1 | iex
```

> 💡 **Usuarios de WSL**: Puedes usar Claude Code tanto desde WSL como desde Windows. Consulta la sección [Uso de WSL](#uso-de-wsl) a continuación para obtener más detalles.

### Descarga de binarios

Descarga el binario para tu plataforma desde la página de [Releases](https://github.com/TheMagicTower/summon/releases).

| Plataforma | Archivo |
|------------|---------|
| Linux x86_64 | `summon-linux-amd64.tar.gz` |
| Linux ARM64 | `summon-linux-arm64.tar.gz` |
| macOS Intel | `summon-darwin-amd64.tar.gz` |
| macOS Apple Silicon | `summon-darwin-arm64.tar.gz` |
| Windows x86_64 | `summon-windows-amd64.zip` |
| Windows ARM64 | `summon-windows-arm64.zip` |

```bash
# Ejemplo: macOS Apple Silicon
tar xzf summon-darwin-arm64.tar.gz
chmod +x summon-darwin-arm64
sudo mv summon-darwin-arm64 /usr/local/bin/summon
```

### Construir desde el código fuente

```bash
cargo build --release
```

## Configuración

### Ubicación del archivo de configuración

summon busca archivos de configuración en el siguiente orden de prioridad:

| Prioridad | Ubicación | Descripción |
|-----------|-----------|-------------|
| 1 | `--config <ruta>` | Especificación explícita |
| 2 | Variable de entorno `SUMMON_CONFIG` | Ruta especificada por variable de entorno |
| 3 | `~/.config/summon/config.yaml` | Configuración específica de usuario (XDG) |
| 4 | `/etc/summon/config.yaml` | Configuración de todo el sistema |
| 5 | `./config.yaml` | Directorio actual |

### Entorno multiusuario

Para que cada usuario tenga su propia configuración:
```bash
mkdir -p ~/.config/summon
cp /path/to/config.yaml ~/.config/summon/
```

Para que los administradores del sistema proporcionen una configuración predeterminada:
```bash
sudo mkdir -p /etc/summon
sudo cp config.yaml /etc/summon/
```

### Ejemplo de archivo de configuración

Crea un archivo `config.yaml`:

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

- `match`: Coincide si esta cadena está contenida en el nombre del modelo (orden de arriba a abajo, se aplica la primera coincidencia)
- `${ENV_VAR}`: Referencia a variable de entorno (las claves de API no se escriben directamente en el archivo de configuración)
- Los modelos que no coinciden se pasan a `default.url` (Anthropic API)

## Ejecución

```bash
# Establecer variables de entorno
export Z_AI_API_KEY="your-z-ai-key"
export KIMI_API_KEY="your-kimi-key"

# Iniciar proxy (archivo de configuración detectado automáticamente)
summon

# O especificar archivo de configuración directamente
summon --config /path/to/config.yaml

# Integración con Claude Code
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

## Uso de WSL

También puedes usar summon desde WSL (Windows Subsystem for Linux).

### Usar Claude Code desde el lado de WSL

```bash
# En terminal de WSL (asumiendo que el archivo de configuración está en ~/.config/summon/config.yaml)
summon

# En otra terminal de WSL
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

### Usar Claude Code desde el lado de Windows (summon ejecutándose en WSL)

```bash
# Ejecutar summon en WSL (enlazar a 0.0.0.0 para que sea accesible desde Windows)
summon

# En terminal de Windows (PowerShell/CMD)
# Verificar IP de WSL: ip addr show eth0 | grep 'inet '
ANTHROPIC_BASE_URL=http://$(wsl hostname -I | awk '{print $1}'):18081 claude
```

Alternativamente, puedes establecer `server.host` en `"0.0.0.0"` en `config.yaml` para que sea accesible desde Windows.

## Registrar como servicio en segundo plano

### macOS (launchd)

**1. Crear archivo plist de LaunchAgent:**

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

**2. Crear directorio de registros y registrar servicio:**

```bash
mkdir -p ~/.local/share/summon
launchctl load ~/Library/LaunchAgents/com.themagictower.summon.plist
launchctl start com.themagictower.summon
```

**3. Gestión del servicio:**

```bash
# Verificar estado
launchctl list | grep com.themagictower.summon

# Detener
launchctl stop com.themagictower.summon

# Reiniciar
launchctl stop com.themagictower.summon && launchctl start com.themagictower.summon

# Eliminar
launchctl unload ~/Library/LaunchAgents/com.themagictower.summon.plist
rm ~/Library/LaunchAgents/com.themagictower.summon.plist
```

### Windows (Windows Service)

**PowerShell (requiere privilegios de administrador):**

```powershell
# 1. Registrar summon como Windows Service (se recomienda nssm)
# Instalar nssm: winget install nssm

# Registrar servicio
nssm install Summon "$env:LOCALAPPDATA\summon\bin\summon.exe"
nssm set Summon AppParameters "--config `"$env:APPDATA\summon\config.yaml`""
nssm set Summon DisplayName "Summon LLM Proxy"
nssm set Summon Start SERVICE_AUTO_START

# Iniciar servicio
Start-Service Summon

# Gestión del servicio
Get-Service Summon      # Verificar estado
Stop-Service Summon     # Detener
Restart-Service Summon  # Reiniciar
sc delete Summon        # Eliminar
```

**O usar WinSW:**

```powershell
# Descargar y configurar WinSW
# https://github.com/winsw/winsw/releases

# Crear summon-service.xml：
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

# Registrar e iniciar servicio
winsw install $env:LOCALAPPDATA\summon\bin\summon-service.xml
winsw start $env:LOCALAPPDATA\summon\bin\summon-service.xml
```

### Linux (systemd) - Incluyendo WSL

El script de instalación detecta automáticamente el entorno y selecciona el tipo de servicio apropiado:
- **Servicio de usuario**: Entorno de escritorio
- **Servicio del sistema**: Servidor sin cabeza (sesiones SSH, etc.)

#### Método 1: Servicio de usuario (Entorno de escritorio)

**1. Crear archivo de servicio systemd:**

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

**2. Registrar e iniciar servicio:**

```bash
# Cargar servicio de usuario
systemctl --user daemon-reload
systemctl --user enable summon.service
systemctl --user start summon.service

# Gestión del servicio
systemctl --user status summon    # Verificar estado
systemctl --user stop summon      # Detener
systemctl --user restart summon   # Reiniciar
systemctl --user disable summon   # Deshabilitar inicio automático
```

#### Método 2: Servicio del sistema (Servidor sin cabeza)

Para entornos sin sesiones de usuario D-Bus como sesiones SSH, use un servicio a nivel del sistema. **Requiere privilegios sudo.**

**1. Crear archivo de servicio systemd (requiere sudo):**

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

**2. Registrar e iniciar servicio (requiere sudo):**

```bash
# Cargar servicio del sistema
sudo systemctl daemon-reload
sudo systemctl enable summon.service
sudo systemctl start summon.service

# Gestión del servicio
sudo systemctl status summon    # Verificar estado
sudo systemctl stop summon      # Detener
sudo systemctl restart summon   # Reiniciar
sudo systemctl disable summon   # Deshabilitar inicio automático

# Ver registros
journalctl -u summon -f
```

> **Nota**: Para usar systemd en WSL2, es posible que necesites establecer `[boot] systemd=true` en `/etc/wsl.conf`.

## Características principales

- **Proxy transparente**: Claude Code no percibe la existencia del proxy
- **Enrutamiento basado en modelos**: Decisión de enrutamiento basada en el campo `model` en `/v1/messages` POST
- **Transmisión SSE**: Passthrough en tiempo real por fragmentos
- **Autenticación de suscripción concurrente**: Los tokens OAuth de Anthropic permanecen intactos, solo los proveedores externos usan claves de API
- **Seguridad**: Se enlaza solo a `127.0.0.1`, claves de API referenciadas desde variables de entorno

## ⚠️ Limitaciones conocidas

### No se pueden usar modelos de thinking de Anthropic después de cambiar a modelos externos

**Una vez que una conversación ha cambiado a un modelo de un proveedor externo (Kimi, Z.AI, etc.), no puedes continuar con modelos de thinking de Anthropic (Claude Opus, Sonnet, etc.) en la misma conversación.**

Esta es una limitación de la arquitectura del sistema que no se puede resolver:
- Los proveedores externos no son completamente compatibles con el formato de mensaje nativo de Anthropic
- Los modelos de thinking dependen de campos nativos específicos y estructuras de contexto
- Las respuestas de modelos externos no cumplen con el formato de contexto requerido por los modelos de thinking

**Uso recomendado:**
- Al cambiar modelos dentro de la misma sesión de conversación, cambia solo entre modelos externos ↔ modelos externos
- Si necesitas modelos de thinking de Anthropic, **inicia una nueva conversación**

## Hoja de ruta

- **v0.1** (actual): Passthrough + enrutamiento basado en modelos + transmisión SSE
- **v0.2**: Transformador (transformación de solicitud/respuesta — para proveedores incompatibles)
- **v0.3**: Registro, verificación de salud, recarga en caliente, tiempo de espera

## Licencia

MIT
