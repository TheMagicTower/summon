import { useTranslation } from "react-i18next";
import { CodeBlock } from "@/components/CodeBlock";

export function Configuration() {
  const { t } = useTranslation();

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold">{t("configuration.title")}</h1>
        <p className="mt-2 text-muted-foreground">{t("configuration.description")}</p>
      </div>

      {/* Config File Location */}
      <section className="space-y-4" id="location">
        <h2 className="text-2xl font-semibold">{t("configuration.location.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("configuration.location.description")}</p>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b">
                <th className="px-4 py-2 text-left font-medium">{t("configuration.location.priority")}</th>
                <th className="px-4 py-2 text-left font-medium">{t("configuration.location.path")}</th>
                <th className="px-4 py-2 text-left font-medium">{t("configuration.location.desc")}</th>
              </tr>
            </thead>
            <tbody>
              {[
                ["1", "--config <path>", "configuration.location.explicit"],
                ["2", "SUMMON_CONFIG", "configuration.location.envVar"],
                ["3", "~/.config/summon/config.yaml", "configuration.location.user"],
                ["4", "/etc/summon/config.yaml", "configuration.location.system"],
                ["5", "./config.yaml", "configuration.location.cwd"],
              ].map(([priority, path, descKey]) => (
                <tr key={priority} className="border-b">
                  <td className="px-4 py-2">{priority}</td>
                  <td className="px-4 py-2">
                    <code className="rounded bg-muted px-1.5 py-0.5 text-xs">{path}</code>
                  </td>
                  <td className="px-4 py-2 text-muted-foreground">{t(descKey)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Config Structure */}
      <section className="space-y-4" id="structure">
        <h2 className="text-2xl font-semibold">{t("configuration.structure.title")}</h2>
        <CodeBlock language="yaml" title="config.yaml">{`server:
  host: "127.0.0.1"    # Bind address (127.0.0.1 only for security)
  port: 18081          # Proxy port

# Default upstream (non-routed models + all non-message requests)
default:
  url: "https://api.anthropic.com"

# Model-based routing rules
routes:
  - match: "kimi"                          # Substring match on model field
    upstream:
      url: "https://api.kimi.com/coding"
      auth:
        header: "Authorization"
        value: "Bearer \${KIMI_API_KEY}"   # Environment variable reference

  - match: "glm"
    upstream:
      url: "https://api.z.ai/api/anthropic"
      auth:
        header: "x-api-key"
        value: "\${Z_AI_API_KEY}"`}</CodeBlock>
      </section>

      {/* Fields */}
      <section className="space-y-4" id="fields">
        <h2 className="text-2xl font-semibold">{t("configuration.fields.title")}</h2>

        <h3 className="text-lg font-medium mt-6">server</h3>
        <p className="text-sm text-muted-foreground">{t("configuration.fields.server")}</p>

        <h3 className="text-lg font-medium mt-6">default</h3>
        <p className="text-sm text-muted-foreground">{t("configuration.fields.default")}</p>

        <h3 className="text-lg font-medium mt-6">routes</h3>
        <p className="text-sm text-muted-foreground">{t("configuration.fields.routes")}</p>
        <ul className="list-disc pl-6 space-y-1 text-sm text-muted-foreground">
          <li><code className="text-foreground">match</code>: {t("configuration.fields.match")}</li>
          <li><code className="text-foreground">upstream.url</code>: {t("configuration.fields.upstreamUrl")}</li>
          <li><code className="text-foreground">upstream.auth.header</code>: {t("configuration.fields.authHeader")}</li>
          <li><code className="text-foreground">upstream.auth.value</code>: {t("configuration.fields.authValue")}</li>
        </ul>
      </section>

      {/* Environment Variables */}
      <section className="space-y-4" id="env">
        <h2 className="text-2xl font-semibold">{t("configuration.env.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("configuration.env.description")}</p>
        <CodeBlock language="bash">{`# Set API keys as environment variables
export KIMI_API_KEY="your-kimi-key"
export Z_AI_API_KEY="your-z-ai-key"

# Reference in config.yaml with \${VAR_NAME} syntax
# value: "Bearer \${KIMI_API_KEY}"`}</CodeBlock>
      </section>
    </div>
  );
}
