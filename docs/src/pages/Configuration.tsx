import { useTranslation } from "react-i18next";
import { CodeBlock } from "@/components/CodeBlock";
import { TableOfContents } from "@/components/TableOfContents";

export function Configuration() {
  const { t } = useTranslation();

  const tocItems = [
    { id: "location", label: t("configuration.location.title") },
    { id: "multi-user", label: t("configuration.multiUser.title") },
    { id: "approach1", label: t("configuration.approach1.title") },
    { id: "approach2", label: t("configuration.approach2.title") },
    { id: "fields", label: t("configuration.fields.title") },
    { id: "key-pool", label: t("configuration.keyPool.title") },
    { id: "env", label: t("configuration.env.title") },
  ];

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold">{t("configuration.title")}</h1>
        <p className="mt-2 text-muted-foreground">{t("configuration.description")}</p>
      </div>

      {/* Table of Contents */}
      <div className="rounded-lg border bg-muted/30 p-4">
        <h3 className="mb-2 text-sm font-medium">{t("configuration.toc")}</h3>
        <TableOfContents items={tocItems} />
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

      {/* Multi-user Environment */}
      <section className="space-y-4" id="multi-user">
        <h2 className="text-2xl font-semibold">{t("configuration.multiUser.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("configuration.multiUser.perUser")}</p>
        <CodeBlock language="bash">{`mkdir -p ~/.config/summon
cp /path/to/config.yaml ~/.config/summon/`}</CodeBlock>
        <p className="text-sm text-muted-foreground">{t("configuration.multiUser.system")}</p>
        <CodeBlock language="bash">{`sudo mkdir -p /etc/summon
sudo cp config.yaml /etc/summon/`}</CodeBlock>
      </section>

      {/* Approach 1: Compatible Providers */}
      <section className="space-y-4" id="approach1">
        <h2 className="text-2xl font-semibold">{t("configuration.approach1.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("configuration.approach1.description")}</p>
        <CodeBlock language="yaml" title="config.yaml">{`server:
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
        value: "\${Z_AI_API_KEY}"

  - match: "claude-sonnet"
    upstream:
      url: "https://api.kimi.com/coding"
      auth:
        header: "Authorization"
        value: "Bearer \${KIMI_API_KEY}"`}</CodeBlock>
        <ul className="list-disc pl-6 space-y-1 text-sm text-muted-foreground">
          <li>{t("configuration.approach1.point1")}</li>
          <li>{t("configuration.approach1.point2")}</li>
        </ul>
      </section>

      {/* Approach 2: Custom Model Binding */}
      <section className="space-y-4" id="approach2">
        <h2 className="text-2xl font-semibold">{t("configuration.approach2.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("configuration.approach2.description")}</p>

        <h3 className="text-lg font-medium">{t("configuration.approach2.step1")}</h3>
        <p className="text-sm text-muted-foreground">{t("configuration.approach2.step1Desc")}</p>
        <CodeBlock language="json" title="~/.claude/settings.json">{`{
  "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:18081",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "glm-4.7",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "kimi-for-coding"
  }
}`}</CodeBlock>

        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b">
                <th className="px-4 py-2 text-left font-medium">{t("configuration.approach2.envVar")}</th>
                <th className="px-4 py-2 text-left font-medium">{t("configuration.approach2.envDesc")}</th>
              </tr>
            </thead>
            <tbody>
              {[
                ["ANTHROPIC_BASE_URL", "configuration.approach2.envBaseUrl"],
                ["ANTHROPIC_DEFAULT_HAIKU_MODEL", "configuration.approach2.envHaiku"],
                ["ANTHROPIC_DEFAULT_SONNET_MODEL", "configuration.approach2.envSonnet"],
                ["ANTHROPIC_DEFAULT_OPUS_MODEL", "configuration.approach2.envOpus"],
              ].map(([envVar, descKey]) => (
                <tr key={envVar} className="border-b">
                  <td className="px-4 py-2">
                    <code className="rounded bg-muted px-1.5 py-0.5 text-xs">{envVar}</code>
                  </td>
                  <td className="px-4 py-2 text-muted-foreground">{t(descKey)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <h3 className="text-lg font-medium mt-4">{t("configuration.approach2.step2")}</h3>
        <p className="text-sm text-muted-foreground">{t("configuration.approach2.step2Desc")}</p>
        <CodeBlock language="yaml" title="config.yaml">{`server:
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
        value: "\${Z_AI_API_KEY}"

  - match: "kimi"
    upstream:
      url: "https://api.kimi.com/coding"
      auth:
        header: "Authorization"
        value: "Bearer \${KIMI_API_KEY}"`}</CodeBlock>
        <ul className="list-disc pl-6 space-y-1 text-sm text-muted-foreground">
          <li>{t("configuration.approach2.point1")}</li>
          <li>{t("configuration.approach2.point2")}</li>
          <li>{t("configuration.approach2.point3")}</li>
        </ul>
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
          <li><code className="text-foreground">upstream.auth.pool</code>: {t("configuration.fields.authPool")}</li>
          <li><code className="text-foreground">concurrency</code>: {t("configuration.fields.concurrency")}</li>
          <li><code className="text-foreground">fallback</code>: {t("configuration.fields.fallback")}</li>
        </ul>
      </section>

      {/* Key Pool */}
      <section className="space-y-4" id="key-pool">
        <h2 className="text-2xl font-semibold">{t("configuration.keyPool.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("configuration.keyPool.description")}</p>

        <CodeBlock language="yaml" title="config.yaml">{`routes:
  - match: "glm-5"
    concurrency: 1           # per-key concurrent request limit
    upstream:
      url: "https://open.bigmodel.cn/api/paas/v4"
      auth:
        header: "Authorization"
        value: "Bearer \${GLM_KEY_1}"
        pool:                 # additional keys
          - "Bearer \${GLM_KEY_2}"
          - "Bearer \${GLM_KEY_3}"
    transformer: "openai"
    model_map: "glm-5"`}</CodeBlock>

        <h3 className="text-lg font-medium">{t("configuration.keyPool.howItWorks")}</h3>
        <ul className="list-disc pl-6 space-y-1 text-sm text-muted-foreground">
          <li>{t("configuration.keyPool.rule1")}</li>
          <li>{t("configuration.keyPool.rule2")}</li>
          <li>{t("configuration.keyPool.rule3")}</li>
          <li>{t("configuration.keyPool.rule4")}</li>
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
