import { useTranslation } from "react-i18next";
import { CodeBlock } from "@/components/CodeBlock";

export function Providers() {
  const { t } = useTranslation();

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold">{t("providers.title")}</h1>
        <p className="mt-2 text-muted-foreground">{t("providers.description")}</p>
      </div>

      {/* Kimi */}
      <section className="space-y-4" id="kimi">
        <h2 className="text-2xl font-semibold">Kimi</h2>
        <p className="text-sm text-muted-foreground">{t("providers.kimi.description")}</p>
        <CodeBlock language="yaml">{`routes:
  - match: "kimi"
    upstream:
      url: "https://api.kimi.com/coding"
      auth:
        header: "Authorization"
        value: "Bearer \${KIMI_API_KEY}"`}</CodeBlock>
      </section>

      {/* Z.AI */}
      <section className="space-y-4" id="zai">
        <h2 className="text-2xl font-semibold">Z.AI (GLM)</h2>
        <p className="text-sm text-muted-foreground">{t("providers.zai.description")}</p>
        <CodeBlock language="yaml">{`routes:
  - match: "glm"
    upstream:
      url: "https://api.z.ai/api/anthropic"
      auth:
        header: "x-api-key"
        value: "\${Z_AI_API_KEY}"`}</CodeBlock>
      </section>

      {/* Custom Provider */}
      <section className="space-y-4" id="custom">
        <h2 className="text-2xl font-semibold">{t("providers.custom.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("providers.custom.description")}</p>
        <CodeBlock language="yaml">{`routes:
  - match: "my-model"
    upstream:
      url: "https://api.example.com/v1"
      auth:
        header: "Authorization"
        value: "Bearer \${MY_API_KEY}"`}</CodeBlock>
        <div className="rounded-lg border border-yellow-500/20 bg-yellow-500/5 p-4">
          <p className="text-sm font-medium">{t("providers.custom.note.title")}</p>
          <p className="mt-1 text-sm text-muted-foreground">{t("providers.custom.note.description")}</p>
        </div>
      </section>

      {/* Routing Logic */}
      <section className="space-y-4" id="routing">
        <h2 className="text-2xl font-semibold">{t("providers.routing.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("providers.routing.description")}</p>
        <ul className="list-disc pl-6 space-y-1 text-sm text-muted-foreground">
          <li>{t("providers.routing.rule1")}</li>
          <li>{t("providers.routing.rule2")}</li>
          <li>{t("providers.routing.rule3")}</li>
          <li>{t("providers.routing.rule4")}</li>
        </ul>
      </section>
    </div>
  );
}
