import { useState } from "react";
import { Check, Copy } from "lucide-react";
import { cn } from "@/lib/utils";

interface CodeBlockProps {
  children: string;
  language?: string;
  title?: string;
  className?: string;
}

export function CodeBlock({ children, language, title, className }: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(children.trim());
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className={cn("group relative my-4 rounded-lg border bg-muted/50", className)}>
      {title && (
        <div className="flex items-center justify-between border-b px-4 py-2 text-sm text-muted-foreground">
          <span>{title}</span>
          {language && (
            <span className="rounded bg-muted px-2 py-0.5 text-xs">{language}</span>
          )}
        </div>
      )}
      <div className="relative">
        <pre className="overflow-x-auto p-4 text-sm leading-relaxed">
          <code>{children.trim()}</code>
        </pre>
        <button
          onClick={handleCopy}
          className="absolute right-2 top-2 rounded-md border bg-background p-1.5 opacity-0 transition-opacity group-hover:opacity-100 hover:bg-muted"
          aria-label="Copy code"
        >
          {copied ? (
            <Check className="h-4 w-4 text-green-500" />
          ) : (
            <Copy className="h-4 w-4 text-muted-foreground" />
          )}
        </button>
      </div>
    </div>
  );
}
