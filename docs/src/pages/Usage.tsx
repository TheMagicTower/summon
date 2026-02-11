import { useTranslation } from "react-i18next";
import { CodeBlock } from "@/components/CodeBlock";

export function Usage() {
  const { t } = useTranslation();

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold">{t("usage.title")}</h1>
        <p className="mt-2 text-muted-foreground">{t("usage.description")}</p>
      </div>

      {/* Basic Usage */}
      <section className="space-y-4" id="basic">
        <h2 className="text-2xl font-semibold">{t("usage.basic.title")}</h2>
        <CodeBlock language="bash">{`# Start with auto-detected config
summon

# Start with explicit config path
summon --config /path/to/config.yaml

# Start with environment variable
SUMMON_CONFIG=/path/to/config.yaml summon`}</CodeBlock>
      </section>

      {/* Claude Code Integration */}
      <section className="space-y-4" id="claude-code">
        <h2 className="text-2xl font-semibold">{t("usage.claudeCode.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("usage.claudeCode.description")}</p>
        <CodeBlock language="bash" title={t("usage.claudeCode.manual")}>{`# Terminal 1: Start proxy
summon

# Terminal 2: Connect Claude Code
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude`}</CodeBlock>

        <h3 className="text-lg font-medium mt-6">{t("usage.claudeCode.auto.title")}</h3>
        <p className="text-sm text-muted-foreground">{t("usage.claudeCode.auto.description")}</p>
        <CodeBlock language="json" title="~/.claude/settings.json">{`{
  "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:18081"
  }
}`}</CodeBlock>
      </section>

      {/* Model Binding */}
      <section className="space-y-4" id="model-binding">
        <h2 className="text-2xl font-semibold">{t("usage.modelBinding.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("usage.modelBinding.description")}</p>
        <CodeBlock language="json" title="~/.claude/settings.json">{`{
  "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:18081",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "glm-4.7",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "kimi-for-coding"
  }
}`}</CodeBlock>
      </section>

      {/* CLI Management */}
      <section className="space-y-4" id="cli">
        <h2 className="text-2xl font-semibold">{t("usage.cli.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("usage.cli.description")}</p>

        <h3 className="text-lg font-medium">{t("usage.cli.update.title")}</h3>
        <p className="text-sm text-muted-foreground">{t("usage.cli.update.description")}</p>
        <CodeBlock language="bash">{`summon update`}</CodeBlock>

        <h3 className="text-lg font-medium mt-6">{t("usage.cli.commands.title")}</h3>
        <p className="text-sm text-muted-foreground">{t("usage.cli.commands.description")}</p>
        <CodeBlock language="bash">{`summon status          # Show current status
summon enable          # Enable proxy
summon disable         # Disable proxy
summon start           # Start proxy in background
summon stop            # Stop proxy
summon add             # Add a provider route
summon remove          # Remove a provider route
summon restore         # Restore settings backup`}</CodeBlock>

        <h3 className="text-lg font-medium mt-6">{t("usage.cli.configure.title")}</h3>
        <p className="text-sm text-muted-foreground">{t("usage.cli.configure.description")}</p>
        <CodeBlock language="bash">{`summon configure`}</CodeBlock>
      </section>

      {/* WSL */}
      <section className="space-y-4" id="wsl">
        <h2 className="text-2xl font-semibold">{t("usage.wsl.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("usage.wsl.description")}</p>

        <h3 className="text-lg font-medium">{t("usage.wsl.inside.title")}</h3>
        <CodeBlock language="bash">{`# WSL terminal 1
summon

# WSL terminal 2
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude`}</CodeBlock>

        <h3 className="text-lg font-medium mt-4">{t("usage.wsl.outside.title")}</h3>
        <CodeBlock language="bash">{`# WSL: start summon
summon

# Windows PowerShell:
$env:ANTHROPIC_BASE_URL="http://$(wsl hostname -I | ForEach-Object { $_.Trim() }):18081"; claude`}</CodeBlock>
      </section>
    </div>
  );
}
