🌐 [English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [中文](README.zh.md) | [Español](README.es.md) | [Tiếng Việt](README.vi.md)

# Summon

ein leichtgewichtiger Reverse-Proxy in Rust, der API-Anfragen von Claude Code basierend auf dem Modellnamen an verschiedene LLM-Anbieter weiterleitet.

Behält Ihre bestehende Anthropic-Abonnement (OAuth) Authentifizierung bei, während spezifische Modelle an externe Anbieter (Z.AI, Kimi, etc.) verzweigt werden.

## Architektur

```
Claude Code CLI
  │ ANTHROPIC_BASE_URL=http://127.0.0.1:18081
  ▼
Proxy (axum-Server)
  ├─ /v1/messages POST → Parsen des Modell-Felds → Routing-Entscheidung
  │   ├─ Treffer → Externer Anbieter (Header/Auth-Ersetzung)
  │   └─ Kein Treffer → Anthropic API (Passthrough)
  └─ Andere Anfragen → Anthropic API (Passthrough)
```

## Installation

### Ein-Zeilen-Installation (Empfohlen)

**Linux/macOS/WSL:**
```bash
curl -fsSL https://raw.githubusercontent.com/TheMagicTower/summon/master/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/TheMagicTower/summon/master/install.ps1 | iex
```

> 💡 **WSL-Benutzer**: Sie können Claude Code sowohl von WSL- als auch von Windows-Seite verwenden. Siehe Abschnitt [WSL-Nutzung](#wsl-nutzung) unten für Details.

### Binär-Download

Laden Sie das Binary für Ihre Plattform von der [Releases](https://github.com/TheMagicTower/summon/releases) Seite herunter.

| Plattform | Datei |
|-----------|-------|
| Linux x86_64 | `summon-linux-amd64.tar.gz` |
| Linux ARM64 | `summon-linux-arm64.tar.gz` |
| macOS Intel | `summon-darwin-amd64.tar.gz` |
| macOS Apple Silicon | `summon-darwin-arm64.tar.gz` |
| Windows x86_64 | `summon-windows-amd64.zip` |
| Windows ARM64 | `summon-windows-arm64.zip` |

```bash
# Beispiel: macOS Apple Silicon
tar xzf summon-darwin-arm64.tar.gz
chmod +x summon-darwin-arm64
sudo mv summon-darwin-arm64 /usr/local/bin/summon
```

### Aus Quellcode kompilieren

```bash
cargo build --release
```

## Konfiguration

### Speicherort der Konfigurationsdatei

summon sucht in folgender Prioritätsreihenfolge nach Konfigurationsdateien:

| Priorität | Speicherort | Beschreibung |
|-----------|-------------|--------------|
| 1 | `--config <Pfad>` | Explizite Angabe |
| 2 | `SUMMON_CONFIG` Umgebungsvariable | Pfad der Umgebungsvariable |
| 3 | `~/.config/summon/config.yaml` | Benutzerspezifische Konfiguration (XDG) |
| 4 | `/etc/summon/config.yaml` | Systemweite Konfiguration |
| 5 | `./config.yaml` | Aktuelles Verzeichnis |

### Multi-Benutzer-Umgebung

Damit jeder Benutzer seine eigene Konfiguration verwenden kann:
```bash
mkdir -p ~/.config/summon
cp /path/to/config.yaml ~/.config/summon/
```

Für Systemadministratoren zur Bereitstellung einer Standardkonfiguration:
```bash
sudo mkdir -p /etc/summon
sudo cp config.yaml /etc/summon/
```

### Konfigurationsansätze

Es gibt zwei Ansätze, je nach Anbieter und Anwendungsfall.

#### Ansatz 1: Kompatible Anbieter (Modellname unverändert weiterleiten)

Für Anbieter, die Anthropic-Modellnamen nativ verstehen (z.B. Z.AI, Kimi). Der ursprüngliche Modellname von Claude Code wird unverändert weitergeleitet.

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

- Claude Code sendet `model: "claude-haiku-4-5-20251001"` → stimmt mit `"claude-haiku"` überein → wird zu Z.AI weitergeleitet
- Der Anbieter entscheidet, welches tatsächliche Modell für den Anthropic-Modellnamen verwendet wird
- Einfache Einrichtung, keine zusätzliche Claude Code-Konfiguration erforderlich

#### Ansatz 2: Benutzerdefinierte Modellbindung (Spezifische Modellauswahl)

Wenn Sie ein bestimmtes Upstream-Modell verwenden möchten (z.B. `glm-4.7` statt dem, was der Anbieter `claude-haiku` zuordnet). Überschreiben Sie die Modellnamen in Claude Code's `settings.json`:

**Schritt 1.** Konfigurieren Sie Claude Code, um benutzerdefinierte Modellnamen zu senden:

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

| Umgebungsvariable | Beschreibung |
|-------------------|--------------|
| `ANTHROPIC_BASE_URL` | Proxy-Adresse (auch die Angabe bei jedem Start entbehrlich) |
| `ANTHROPIC_DEFAULT_HAIKU_MODEL` | Modellname, der gesendet wird, wenn Haiku-Stufe ausgewählt ist |
| `ANTHROPIC_DEFAULT_SONNET_MODEL` | Modellname, der gesendet wird, wenn Sonnet-Stufe ausgewählt ist |
| `ANTHROPIC_DEFAULT_OPUS_MODEL` | Modellname, der gesendet wird, wenn Opus-Stufe ausgewählt ist |

**Schritt 2.** Passen Sie `config.yaml` an die überschriebenen Modellnamen an:

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

- Claude Code sendet `model: "glm-4.7"` (überschrieben) → stimmt mit `"glm"` überein → wird zu Z.AI mit exaktem Modell weitergeleitet
- Sie kontrollieren genau, welches Modell der Anbieter verwendet
- `ANTHROPIC_BASE_URL` in `settings.json` bedeutet, dass Sie einfach `claude` ohne zusätzliche Umgebungsvariablen ausführen können

### Konfigurationsreferenz

- `match`: Stimmt überein, wenn diese Zeichenfolge im Modellnamen enthalten ist (von oben nach unten, erste Übereinstimmung wird angewendet)
- `${ENV_VAR}`: Umgebungsvariablen-Referenz (API-Schlüssel werden nicht direkt in die Konfigurationsdatei geschrieben)
- `upstream.auth.pool`: Zusätzliche API-Schlüsselwerte für Lastverteilung (verwendet denselben Header wie `auth.header`)
- `concurrency`: Gleichzeitige Anfragen-Limit pro Schlüssel (bei Überschreitung Fallback zu Anthropic oder 429 zurückgeben)
- `fallback`: Fallback-Verhalten bei Anbieter-Ausfall (Standard: `true`)
  - `false`: Kein Fallback, Fehler unverändert zurückgeben
  - `true`: Fallback zur Anthropic API mit dem ursprünglichen Modellnamen
  - `"Modellname"`: Fallback zur Anthropic API mit dem angegebenen Modellnamen (empfohlen für nicht-Anthropic-Modellnamen)
- Modelle ohne Übereinstimmung werden an `default.url` (Anthropic API) weitergeleitet

### API-Schlüssel-Pool (Gleichzeitigkeits-Begrenzung)

Einige Anbieter begrenzen die gleichzeitigen Anfragen pro API-Schlüssel (z.B. erlaubt GLM-5 nur 1 gleichzeitige Anfrage pro Schlüssel). Sie können mehrere API-Schlüssel als Pool registrieren, um die Gesamt-Gleichzeitigkeit zu erhöhen:

```yaml
routes:
  - match: "glm-5"
    concurrency: 1           # Gleichzeitige Anfragen-Limit pro Schlüssel
    upstream:
      url: "https://open.bigmodel.cn/api/paas/v4"
      auth:
        header: "Authorization"
        value: "Bearer ${GLM_KEY_1}"
        pool:                 # Zusätzliche Schlüssel (gleicher Header)
          - "Bearer ${GLM_KEY_2}"
          - "Bearer ${GLM_KEY_3}"
    transformer: "openai"
    model_map: "glm-5"
```

**Funktionsweise:**

- Anfragen werden an den Schlüssel mit den wenigsten aktiven Verbindungen verteilt (**Least-Connections**)
- Die gleichzeitige Nutzung jedes Schlüssels wird durch die `concurrency`-Einstellung verfolgt und begrenzt
- Wenn alle Schlüssel ihr Limit erreichen: Fallback zu Anthropic (wenn `fallback` aktiviert ist) oder HTTP 429 zurückgeben. Verwenden Sie `fallback: "claude-sonnet-4-5-20250929"` für einen sicheren Fallback mit einem kompatiblen Modellnamen
- Streaming-Antworten geben den Schlüssel automatisch frei, wenn der Stream endet

## Ausführung

```bash
# Umgebungsvariablen setzen
export Z_AI_API_KEY="your-z-ai-key"
export KIMI_API_KEY="your-kimi-key"

# Proxy starten (Konfigurationsdatei automatisch erkannt)
summon

# Oder Konfigurationsdatei direkt angeben
summon --config /path/to/config.yaml
```

### Claude Code verbinden

**Option A: Manuell (pro Sitzung)**
```bash
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

**Option B: Automatisch (empfohlen)**

Fügen Sie `~/.claude/settings.json` hinzu, sodass Sie die URL nicht mehr angeben müssen:
```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:18081"
  }
}
```

Dann einfach ausführen:
```bash
claude
```

## CLI-Verwaltung

### Selbst-Update

Auf neue Versionen prüfen und das Binary direkt aktualisieren:

```bash
summon update
```

Der Update-Befehl:
1. Vergleicht die aktuelle Version mit dem neuesten GitHub-Release
2. Fragt bei Bedarf nach Bestätigung, wenn eine neuere Version verfügbar ist
3. Lädt das Binary herunter und ersetzt es automatisch

> Windows: Selbst-Update wird nicht unterstützt. Verwenden Sie stattdessen `install.ps1`.

### Direkte Befehle

Alle Verwaltungsbefehle sind Top-Level-Befehle:

```bash
summon status          # Aktuellen Status anzeigen
summon enable          # Proxy aktivieren (settings.json ändern + starten)
summon disable         # Proxy deaktivieren (stoppen + settings.json wiederherstellen)
summon start           # Proxy im Hintergrund starten
summon stop            # Proxy stoppen
summon add             # Eine Anbieter-Route hinzufügen
summon remove          # Eine Anbieter-Route entfernen
summon restore         # settings.json aus Backup wiederherstellen
```

### Interaktive Konfiguration

Durch Ausführen von `summon configure` wird ein interaktives Menü mit allen verfügbaren Aktionen geöffnet:

```bash
summon configure
```

## WSL-Nutzung

Sie können summon auch von WSL (Windows Subsystem for Linux) aus verwenden.

### Claude Code von WSL-Seite verwenden

```bash
# In WSL-Terminal (unter der Annahme, dass die Konfigurationsdatei unter ~/.config/summon/config.yaml liegt)
summon

# In einem anderen WSL-Terminal
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude
```

### Claude Code von Windows-Seite verwenden (summon läuft in WSL)

```bash
# summon in WSL ausführen (an 0.0.0.0 binden, damit es von Windows aus zugänglich ist)
summon

# In Windows-Terminal (PowerShell/CMD)
# WSL-IP prüfen: ip addr show eth0 | grep 'inet '
ANTHROPIC_BASE_URL=http://$(wsl hostname -I | awk '{print $1}'):18081 claude
```

Alternativ können Sie `server.host` in `config.yaml` auf `"0.0.0.0"` setzen, damit es von Windows aus zugänglich ist.

## Als Hintergrunddienst registrieren

### macOS (launchd)

**1. LaunchAgent plist-Datei erstellen:**

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

**2. Protokollverzeichnis erstellen und Dienst registrieren:**

```bash
mkdir -p ~/.local/share/summon
launchctl load ~/Library/LaunchAgents/com.themagictower.summon.plist
launchctl start com.themagictower.summon
```

**3. Dienstverwaltung:**

```bash
# Status prüfen
launchctl list | grep com.themagictower.summon

# Stoppen
launchctl stop com.themagictower.summon

# Neustart
launchctl stop com.themagictower.summon && launchctl start com.themagictower.summon

# Entfernen
launchctl unload ~/Library/LaunchAgents/com.themagictower.summon.plist
rm ~/Library/LaunchAgents/com.themagictower.summon.plist
```

### Windows (Windows-Dienst)

**PowerShell (erfordert Administratorrechte):**

```powershell
# 1. summon als Windows-Dienst registrieren (nssm empfohlen)
# nssm installieren: winget install nssm

# Dienst registrieren
nssm install Summon "$env:LOCALAPPDATA\summon\bin\summon.exe"
nssm set Summon AppParameters "--config `"$env:APPDATA\summon\config.yaml`""
nssm set Summon DisplayName "Summon LLM Proxy"
nssm set Summon Start SERVICE_AUTO_START

# Dienst starten
Start-Service Summon

# Dienstverwaltung
Get-Service Summon      # Status prüfen
Stop-Service Summon     # Stoppen
Restart-Service Summon  # Neustart
sc delete Summon        # Entfernen
```

**Oder WinSW verwenden:**

```powershell
# WinSW herunterladen und konfigurieren
# https://github.com/winsw/winsw/releases

# summon-service.xml erstellen:
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

# Dienst registrieren und starten
winsw install $env:LOCALAPPDATA\summon\bin\summon-service.xml
winsw start $env:LOCALAPPDATA\summon\bin\summon-service.xml
```

### Linux (systemd) - Inklusive WSL

Das Installationsskript erkennt automatisch die Umgebung und wählt den geeigneten Diensttyp:
- **Benutzerdienst**: Desktop-Umgebung
- **Systemdienst**: Headless-Server (SSH-Sitzungen, etc.)

#### Methode 1: Benutzerdienst (Desktop-Umgebung)

**1. systemd-Dienstdatei erstellen:**

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

**2. Dienst registrieren und starten:**

```bash
# Benutzerdienst laden
systemctl --user daemon-reload
systemctl --user enable summon.service
systemctl --user start summon.service

# Dienstverwaltung
systemctl --user status summon    # Status prüfen
systemctl --user stop summon      # Stoppen
systemctl --user restart summon   # Neustart
systemctl --user disable summon   # Autostart deaktivieren
```

#### Methode 2: Systemdienst (Headless-Server)

Für Umgebungen ohne D-Bus-Benutzersitzungen wie SSH-Sitzungen verwenden Sie einen systemweiten Dienst. **Erfordert sudo-Rechte.**

**1. systemd-Dienstdatei erstellen (erfordert sudo):**

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

**2. Dienst registrieren und starten (erfordert sudo):**

```bash
# Systemdienst laden
sudo systemctl daemon-reload
sudo systemctl enable summon.service
sudo systemctl start summon.service

# Dienstverwaltung
sudo systemctl status summon    # Status prüfen
sudo systemctl stop summon      # Stoppen
sudo systemctl restart summon   # Neustart
sudo systemctl disable summon   # Autostart deaktivieren

# Protokolle anzeigen
journalctl -u summon -f
```

> **Hinweis**: Um systemd in WSL2 zu verwenden, müssen Sie möglicherweise `[boot] systemd=true` in `/etc/wsl.conf` setzen.

## Hauptfunktionen

- **Transparenter Proxy**: Claude Code bemerkt nicht die Existenz des Proxies
- **Modellbasiertes Routing**: Routing-Entscheidung basierend auf dem `model`-Feld in `/v1/messages` POST
- **SSE-Streaming**: Echtzeit-Passthrough in Blöcken
- **Gleichzeitige Abonnement-Authentifizierung**: Anthropic-OAuth-Tokens bleiben intakt, nur externe Anbieter verwenden API-Schlüssel
- **API-Schlüssel-Pool**: Unterstützung mehrerer API-Schlüssel pro Route mit Least-Connections-Verteilung für Anbieter mit Pro-Schlüssel-Gleichzeitigkeits-Begrenzungen
- **Fallback-Modellname**: Geben Sie einen kompatiblen Anthropic-Modellnamen für sicheren Fallback an, wenn nicht-Anthropic-Modellnamen verwendet werden
- **Sicherheit**: Bindet nur an `127.0.0.1`, API-Schlüssel aus Umgebungsvariablen referenziert

## ⚠️ Bekannte Einschränkungen

### Anthropic Thinking-Modelle nicht nutzbar nach Wechsel zu externen Modellen

**Sobald ein Gespräch zu einem Modell eines externen Anbieters (Kimi, Z.AI, etc.) gewechselt wurde, können Sie nicht mit Anthropic's Thinking-Modellen (Claude Opus, Sonnet, etc.) im selben Gespräch fortfahren.**

Dies ist eine Systemarchitektur-Begrenzung, die nicht gelöst werden kann:
- Externe Anbieter sind nicht vollständig kompatibel mit Anthropic's nativem Nachrichtenformat
- Thinking-Modelle hängen von bestimmten nativen Feldern und Kontextstrukturen ab
- Antworten externer Modelle erfüllen nicht das von Thinking-Modellen erforderliche Kontextformat

**Empfohlene Verwendung:**
- Wenn Sie innerhalb derselben Gesprächssitzung Modelle wechseln müssen, wechseln Sie nur zwischen externe Modelle ↔ externe Modelle
- Wenn Sie Anthropic Thinking-Modelle benötigen, **starten Sie ein neues Gespräch**

## Fahrplan

- **v0.1**: Passthrough + modellbasiertes Routing + SSE-Streaming
- **v0.2** (aktuell): Transformator, API-Schlüssel-Pool, Fallback-Modellname, interaktive CLI, Selbst-Update
- **v0.3**: Protokollierung, Gesundheitsprüfung, Hot-Reload, Timeout

## Lizenz

MIT
