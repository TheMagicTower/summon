import { useTranslation } from "react-i18next";
import { CodeBlock } from "@/components/CodeBlock";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

export function Installation() {
  const { t } = useTranslation();

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-3xl font-bold">{t("installation.title")}</h1>
        <p className="mt-2 text-muted-foreground">{t("installation.description")}</p>
      </div>

      {/* One-line Install */}
      <section className="space-y-4" id="one-line">
        <h2 className="text-2xl font-semibold">{t("installation.oneLine.title")}</h2>
        <Tabs defaultValue="linux">
          <TabsList>
            <TabsTrigger value="linux">Linux / macOS</TabsTrigger>
            <TabsTrigger value="windows">Windows</TabsTrigger>
            <TabsTrigger value="wsl">WSL</TabsTrigger>
          </TabsList>
          <TabsContent value="linux">
            <CodeBlock language="bash">{`curl -fsSL https://raw.githubusercontent.com/TheMagicTower/summon/master/install.sh | bash`}</CodeBlock>
          </TabsContent>
          <TabsContent value="windows">
            <CodeBlock language="powershell">{`irm https://raw.githubusercontent.com/TheMagicTower/summon/master/install.ps1 | iex`}</CodeBlock>
          </TabsContent>
          <TabsContent value="wsl">
            <p className="mb-3 text-sm text-muted-foreground">{t("installation.oneLine.wslNote")}</p>
            <CodeBlock language="bash">{`curl -fsSL https://raw.githubusercontent.com/TheMagicTower/summon/master/install.sh | bash`}</CodeBlock>
          </TabsContent>
        </Tabs>
      </section>

      {/* Binary Download */}
      <section className="space-y-4" id="binary">
        <h2 className="text-2xl font-semibold">{t("installation.binary.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("installation.binary.description")}</p>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b">
                <th className="px-4 py-2 text-left font-medium">{t("installation.binary.platform")}</th>
                <th className="px-4 py-2 text-left font-medium">{t("installation.binary.file")}</th>
              </tr>
            </thead>
            <tbody>
              {[
                ["Linux x86_64", "summon-linux-amd64.tar.gz"],
                ["Linux ARM64", "summon-linux-arm64.tar.gz"],
                ["macOS Intel", "summon-darwin-amd64.tar.gz"],
                ["macOS Apple Silicon", "summon-darwin-arm64.tar.gz"],
                ["Windows x86_64", "summon-windows-amd64.zip"],
                ["Windows ARM64", "summon-windows-arm64.zip"],
              ].map(([platform, file]) => (
                <tr key={file} className="border-b">
                  <td className="px-4 py-2">{platform}</td>
                  <td className="px-4 py-2">
                    <code className="rounded bg-muted px-1.5 py-0.5 text-xs">{file}</code>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <CodeBlock language="bash" title={t("installation.binary.example")}>{`# Example: macOS Apple Silicon
tar xzf summon-darwin-arm64.tar.gz
chmod +x summon-darwin-arm64
sudo mv summon-darwin-arm64 /usr/local/bin/summon`}</CodeBlock>
      </section>

      {/* Build from Source */}
      <section className="space-y-4" id="source">
        <h2 className="text-2xl font-semibold">{t("installation.source.title")}</h2>
        <p className="text-sm text-muted-foreground">{t("installation.source.description")}</p>
        <CodeBlock language="bash">{`git clone https://github.com/TheMagicTower/summon.git
cd summon
cargo build --release
sudo cp target/release/summon /usr/local/bin/`}</CodeBlock>
      </section>
    </div>
  );
}
