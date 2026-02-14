import { useState, useEffect } from "react";
import { Link, useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Home,
  Download,
  Settings,
  Terminal,
  Puzzle,
  Server,
  HelpCircle,
  Menu,
  Moon,
  Sun,
  Github,
  FileText,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet";
import { LanguageSwitcher } from "@/components/LanguageSwitcher";

const navItems = [
  { path: "/", icon: Home, labelKey: "nav.home" },
  { path: "/installation", icon: Download, labelKey: "nav.installation" },
  { path: "/configuration", icon: Settings, labelKey: "nav.configuration" },
  { path: "/usage", icon: Terminal, labelKey: "nav.usage" },
  { path: "/providers", icon: Puzzle, labelKey: "nav.providers" },
  { path: "/service", icon: Server, labelKey: "nav.service" },
  { path: "/troubleshooting", icon: HelpCircle, labelKey: "nav.troubleshooting" },
  { path: "/changelog", icon: FileText, labelKey: "nav.changelog" },
];

function ThemeToggle() {
  const [dark, setDark] = useState(() => {
    if (typeof window !== "undefined") {
      return document.documentElement.classList.contains("dark");
    }
    return false;
  });

  useEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
    localStorage.setItem("theme", dark ? "dark" : "light");
  }, [dark]);

  useEffect(() => {
    const stored = localStorage.getItem("theme");
    if (stored === "dark" || (!stored && window.matchMedia("(prefers-color-scheme: dark)").matches)) {
      setDark(true);
    }
  }, []);

  return (
    <Button variant="ghost" size="icon" onClick={() => setDark(!dark)}>
      {dark ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
      <span className="sr-only">Toggle theme</span>
    </Button>
  );
}

function SidebarNav({ onNavigate }: { onNavigate?: () => void }) {
  const location = useLocation();
  const { t } = useTranslation();

  return (
    <nav className="space-y-1 px-2">
      {navItems.map((item) => {
        const Icon = item.icon;
        const isActive = location.pathname === item.path;
        return (
          <Link
            key={item.path}
            to={item.path}
            onClick={onNavigate}
            className={cn(
              "flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors",
              isActive
                ? "bg-primary text-primary-foreground"
                : "text-muted-foreground hover:bg-muted hover:text-foreground"
            )}
          >
            <Icon className="h-4 w-4 shrink-0" />
            {t(item.labelKey)}
          </Link>
        );
      })}
    </nav>
  );
}

export function Layout({ children }: { children: React.ReactNode }) {
  const [open, setOpen] = useState(false);

  return (
    <div className="flex min-h-screen">
      {/* Desktop Sidebar */}
      <aside className="hidden w-64 shrink-0 border-r bg-sidebar lg:block">
        <div className="flex h-full flex-col">
          <div className="flex h-14 items-center gap-2 border-b px-4">
            <span className="text-xl">🔮</span>
            <span className="font-bold">Summon</span>
            <span className="ml-auto rounded-md bg-muted px-2 py-0.5 text-xs text-muted-foreground">
              v0.2.8
            </span>
          </div>
          <ScrollArea className="flex-1 py-4">
            <SidebarNav />
          </ScrollArea>
        </div>
      </aside>

      {/* Main Content */}
      <div className="flex flex-1 flex-col">
        {/* Header */}
        <header className="sticky top-0 z-40 flex h-14 items-center gap-2 border-b bg-background/95 px-4 backdrop-blur supports-[backdrop-filter]:bg-background/60">
          {/* Mobile Menu */}
          <Sheet open={open} onOpenChange={setOpen}>
            <SheetTrigger asChild>
              <Button variant="ghost" size="icon" className="lg:hidden">
                <Menu className="h-5 w-5" />
                <span className="sr-only">Menu</span>
              </Button>
            </SheetTrigger>
            <SheetContent side="left" className="w-64 p-0 pt-10">
              <div className="flex items-center gap-2 px-4 pb-4">
                <span className="text-xl">🔮</span>
                <span className="font-bold">Summon</span>
              </div>
              <Separator />
              <ScrollArea className="h-[calc(100vh-8rem)] py-4">
                <SidebarNav onNavigate={() => setOpen(false)} />
              </ScrollArea>
            </SheetContent>
          </Sheet>

          <div className="lg:hidden font-bold flex items-center gap-2">
            <span>🔮</span> Summon
          </div>

          <div className="ml-auto flex items-center gap-1">
            <LanguageSwitcher />
            <ThemeToggle />
            <Button variant="ghost" size="icon" asChild>
              <a
                href="https://github.com/TheMagicTower/summon"
                target="_blank"
                rel="noopener noreferrer"
              >
                <Github className="h-4 w-4" />
                <span className="sr-only">GitHub</span>
              </a>
            </Button>
          </div>
        </header>

        {/* Page Content */}
        <main className="flex-1">
          <div className="mx-auto max-w-4xl px-4 py-8 sm:px-6 lg:px-8">
            {children}
          </div>
        </main>
      </div>
    </div>
  );
}
