import { useTranslation } from "react-i18next";
import { CodeBlock } from "@/components/CodeBlock";

export function Troubleshooting() {
  const { t } = useTranslation();

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold">{t("troubleshooting.title")}</h1>
        <p className="mt-2 text-muted-foreground">{t("troubleshooting.description")}</p>
      </div>

      {/* Known Limitations */}
      <section className="space-y-4" id="limitations">
        <h2 className="text-2xl font-semibold">{t("troubleshooting.limitations.title")}</h2>
        <div className="rounded-lg border border-red-500/20 bg-red-500/5 p-4 space-y-2">
          <p className="text-sm font-medium">{t("troubleshooting.limitations.thinking.title")}</p>
          <p className="text-sm text-muted-foreground">{t("troubleshooting.limitations.thinking.description")}</p>
          <ul className="list-disc pl-6 text-sm text-muted-foreground space-y-1">
            <li>{t("troubleshooting.limitations.thinking.reason1")}</li>
            <li>{t("troubleshooting.limitations.thinking.reason2")}</li>
            <li>{t("troubleshooting.limitations.thinking.reason3")}</li>
          </ul>
        </div>
        <div className="rounded-lg border border-blue-500/20 bg-blue-500/5 p-4 space-y-2">
          <p className="text-sm font-medium">{t("troubleshooting.limitations.recommendation.title")}</p>
          <ul className="list-disc pl-6 text-sm text-muted-foreground space-y-1">
            <li>{t("troubleshooting.limitations.recommendation.tip1")}</li>
            <li>{t("troubleshooting.limitations.recommendation.tip2")}</li>
          </ul>
        </div>
      </section>

      {/* Connection Issues */}
      <section className="space-y-4" id="connection">
        <h2 className="text-2xl font-semibold">{t("troubleshooting.connection.title")}</h2>

        <h3 className="text-lg font-medium">{t("troubleshooting.connection.refused.title")}</h3>
        <p className="text-sm text-muted-foreground">{t("troubleshooting.connection.refused.description")}</p>
        <CodeBlock language="bash">{`# Check if summon is running
ps aux | grep summon

# Check port
lsof -i :18081    # macOS/Linux
netstat -an | findstr 18081  # Windows`}</CodeBlock>

        <h3 className="text-lg font-medium mt-6">{t("troubleshooting.connection.timeout.title")}</h3>
        <p className="text-sm text-muted-foreground">{t("troubleshooting.connection.timeout.description")}</p>
      </section>

      {/* Config Issues */}
      <section className="space-y-4" id="config">
        <h2 className="text-2xl font-semibold">{t("troubleshooting.config.title")}</h2>

        <h3 className="text-lg font-medium">{t("troubleshooting.config.notFound.title")}</h3>
        <p className="text-sm text-muted-foreground">{t("troubleshooting.config.notFound.description")}</p>
        <CodeBlock language="bash">{`# Check config search order
ls -la ~/.config/summon/config.yaml
ls -la /etc/summon/config.yaml
ls -la ./config.yaml

# Or specify explicitly
summon --config /path/to/config.yaml`}</CodeBlock>

        <h3 className="text-lg font-medium mt-6">{t("troubleshooting.config.envVar.title")}</h3>
        <p className="text-sm text-muted-foreground">{t("troubleshooting.config.envVar.description")}</p>
        <CodeBlock language="bash">{`# Verify environment variables are set
echo $KIMI_API_KEY
echo $Z_AI_API_KEY

# Set them before starting summon
export KIMI_API_KEY="your-key"
summon`}</CodeBlock>
      </section>

      {/* Provider Issues */}
      <section className="space-y-4" id="provider">
        <h2 className="text-2xl font-semibold">{t("troubleshooting.provider.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("troubleshooting.provider.description")}</p>
        <CodeBlock language="bash">{`# Test direct connection to provider
curl -X POST https://api.kimi.com/coding/v1/messages \\
  -H "Authorization: Bearer $KIMI_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"model":"kimi-for-coding","max_tokens":10,"messages":[{"role":"user","content":"hi"}]}'`}</CodeBlock>
      </section>
    </div>
  );
}
