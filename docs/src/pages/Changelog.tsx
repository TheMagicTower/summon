import { useTranslation } from "react-i18next";

export function Changelog() {
  const { t } = useTranslation();

  const versions = [
    { version: "0.3.0", date: "2026-02-16" },
    { version: "0.2.8", date: "2026-02-14" },
    { version: "0.2.7", date: "2026-02-13" },
    { version: "0.2.6", date: "2026-02-12" },
    { version: "0.2.0", date: "2026-02-10" },
    { version: "0.1.0", date: "2026-02-08" },
  ];

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold">{t("changelog.title")}</h1>
        <p className="mt-2 text-muted-foreground">{t("changelog.description")}</p>
      </div>

      {versions.map(({ version, date }) => (
        <section key={version} className="space-y-4" id={`v${version}`}>
          <h2 className="text-2xl font-semibold">
            v{version}
            <span className="ml-2 text-sm font-normal text-muted-foreground">{date}</span>
          </h2>
          <div className="text-sm text-muted-foreground whitespace-pre-line">
            {t(`changelog.versions.v${version.replace(/\./g, "_")}`)}
          </div>
        </section>
      ))}
    </div>
  );
}
