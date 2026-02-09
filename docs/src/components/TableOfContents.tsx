import { cn } from "@/lib/utils";

interface TocItem {
  id: string;
  label: string;
  level?: number;
}

interface TableOfContentsProps {
  items: TocItem[];
  className?: string;
}

export function TableOfContents({ items, className }: TableOfContentsProps) {
  return (
    <nav className={cn("space-y-1 text-sm", className)}>
      {items.map((item) => (
        <a
          key={item.id}
          href={`#${item.id}`}
          className={cn(
            "block rounded-md px-3 py-1.5 text-muted-foreground transition-colors hover:text-foreground",
            item.level === 2 && "pl-6"
          )}
        >
          {item.label}
        </a>
      ))}
    </nav>
  );
}
