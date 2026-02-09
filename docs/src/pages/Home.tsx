import { useTranslation } from "react-i18next";
import { Link } from "react-router-dom";
import { ArrowRight, Zap, Shield, Router, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { CodeBlock } from "@/components/CodeBlock";

export function Home() {
  const { t } = useTranslation();

  const features = [
    { icon: Router, titleKey: "home.feature.routing.title", descKey: "home.feature.routing.desc" },
    { icon: Shield, titleKey: "home.feature.auth.title", descKey: "home.feature.auth.desc" },
    { icon: Zap, titleKey: "home.feature.streaming.title", descKey: "home.feature.streaming.desc" },
    { icon: RefreshCw, titleKey: "home.feature.transparent.title", descKey: "home.feature.transparent.desc" },
  ];

  return (
    <div className="space-y-12">
      {/* Hero */}
      <section className="space-y-4">
        <h1 className="text-4xl font-bold tracking-tight">{t("home.title")}</h1>
        <p className="text-lg text-muted-foreground max-w-2xl">
          {t("home.description")}
        </p>
        <div className="flex gap-3 pt-2">
          <Button asChild>
            <Link to="/installation">
              {t("home.getStarted")} <ArrowRight className="ml-1 h-4 w-4" />
            </Link>
          </Button>
          <Button variant="outline" asChild>
            <a
              href="https://github.com/TheMagicTower/summon"
              target="_blank"
              rel="noopener noreferrer"
            >
              GitHub
            </a>
          </Button>
        </div>
      </section>

      {/* Architecture */}
      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">{t("home.architecture")}</h2>
        <CodeBlock language="text">{`Claude Code CLI
  │ ANTHROPIC_BASE_URL=http://127.0.0.1:18081
  ▼
Summon (Reverse Proxy)
  ├─ /v1/messages POST → model field → routing
  │   ├─ match → External Provider (header/auth swap)
  │   └─ no match → Anthropic API (passthrough)
  └─ Other requests → Anthropic API (passthrough)`}</CodeBlock>
      </section>

      {/* Quick Start */}
      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">{t("home.quickStart")}</h2>
        <CodeBlock language="bash" title={t("home.install")}>{`curl -fsSL https://raw.githubusercontent.com/TheMagicTower/summon/master/install.sh | bash`}</CodeBlock>
        <CodeBlock language="bash" title={t("home.run")}>{`# Start proxy
summon

# Connect Claude Code
ANTHROPIC_BASE_URL=http://127.0.0.1:18081 claude`}</CodeBlock>
      </section>

      {/* Features */}
      <section className="space-y-6">
        <h2 className="text-2xl font-semibold">{t("home.features")}</h2>
        <div className="grid gap-4 sm:grid-cols-2">
          {features.map((f) => {
            const Icon = f.icon;
            return (
              <div key={f.titleKey} className="rounded-lg border p-4 space-y-2">
                <div className="flex items-center gap-2">
                  <Icon className="h-5 w-5 text-primary" />
                  <h3 className="font-medium">{t(f.titleKey)}</h3>
                </div>
                <p className="text-sm text-muted-foreground">{t(f.descKey)}</p>
              </div>
            );
          })}
        </div>
      </section>
    </div>
  );
}
