import { useTranslation } from "react-i18next";
import { CodeBlock } from "@/components/CodeBlock";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

export function Service() {
  const { t } = useTranslation();

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold">{t("service.title")}</h1>
        <p className="mt-2 text-muted-foreground">{t("service.description")}</p>
      </div>

      <Tabs defaultValue="macos">
        <TabsList>
          <TabsTrigger value="macos">macOS (launchd)</TabsTrigger>
          <TabsTrigger value="linux">Linux (systemd)</TabsTrigger>
          <TabsTrigger value="windows">Windows</TabsTrigger>
        </TabsList>

        {/* macOS */}
        <TabsContent value="macos" className="space-y-4 mt-4">
          <h2 className="text-xl font-semibold">{t("service.macos.title")}</h2>

          <h3 className="text-lg font-medium">{t("service.macos.step1")}</h3>
          <CodeBlock language="bash">{`cat > ~/Library/LaunchAgents/com.themagictower.summon.plist << 'EOF'
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
</dict>
</plist>
EOF`}</CodeBlock>

          <h3 className="text-lg font-medium">{t("service.macos.step2")}</h3>
          <CodeBlock language="bash">{`mkdir -p ~/.local/share/summon
launchctl load ~/Library/LaunchAgents/com.themagictower.summon.plist
launchctl start com.themagictower.summon`}</CodeBlock>

          <h3 className="text-lg font-medium">{t("service.macos.manage")}</h3>
          <CodeBlock language="bash">{`# Status
launchctl list | grep com.themagictower.summon

# Stop
launchctl stop com.themagictower.summon

# Restart
launchctl stop com.themagictower.summon && launchctl start com.themagictower.summon

# Unload
launchctl unload ~/Library/LaunchAgents/com.themagictower.summon.plist`}</CodeBlock>
        </TabsContent>

        {/* Linux */}
        <TabsContent value="linux" className="space-y-4 mt-4">
          <h2 className="text-xl font-semibold">{t("service.linux.title")}</h2>

          <h3 className="text-lg font-medium">{t("service.linux.step1")}</h3>
          <CodeBlock language="bash">{`mkdir -p ~/.config/systemd/user

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
EOF`}</CodeBlock>

          <h3 className="text-lg font-medium">{t("service.linux.step2")}</h3>
          <CodeBlock language="bash">{`systemctl --user daemon-reload
systemctl --user enable summon.service
systemctl --user start summon.service`}</CodeBlock>

          <h3 className="text-lg font-medium">{t("service.linux.manage")}</h3>
          <CodeBlock language="bash">{`systemctl --user status summon    # Status
systemctl --user stop summon      # Stop
systemctl --user restart summon   # Restart
systemctl --user disable summon   # Disable autostart`}</CodeBlock>

          <div className="rounded-lg border border-blue-500/20 bg-blue-500/5 p-4">
            <p className="text-sm">{t("service.linux.wslNote")}</p>
          </div>
        </TabsContent>

        {/* Windows */}
        <TabsContent value="windows" className="space-y-4 mt-4">
          <h2 className="text-xl font-semibold">{t("service.windows.title")}</h2>
          <p className="text-sm text-muted-foreground">{t("service.windows.description")}</p>

          <h3 className="text-lg font-medium">{t("service.windows.nssm")}</h3>
          <CodeBlock language="powershell">{`# Install nssm
winget install nssm

# Register service
nssm install Summon "$env:LOCALAPPDATA\\summon\\bin\\summon.exe"
nssm set Summon AppParameters "--config \`"$env:APPDATA\\summon\\config.yaml\`""
nssm set Summon DisplayName "Summon LLM Proxy"
nssm set Summon Start SERVICE_AUTO_START

# Start
Start-Service Summon`}</CodeBlock>

          <h3 className="text-lg font-medium">{t("service.windows.scheduler")}</h3>
          <CodeBlock language="powershell">{`# Task Scheduler (auto-start at logon)
$Action = New-ScheduledTaskAction -Execute "$env:LOCALAPPDATA\\summon\\bin\\summon.exe" -Argument "--config \`"$env:USERPROFILE\\.config\\summon\\config.yaml\`""
$Trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
Register-ScheduledTask -TaskName "Summon LLM Proxy" -Action $Action -Trigger $Trigger

# Manage
schtasks /run /tn "Summon LLM Proxy"      # Start
schtasks /end /tn "Summon LLM Proxy"      # Stop
schtasks /query /tn "Summon LLM Proxy"    # Status
schtasks /delete /tn "Summon LLM Proxy"   # Remove`}</CodeBlock>
        </TabsContent>
      </Tabs>
    </div>
  );
}
