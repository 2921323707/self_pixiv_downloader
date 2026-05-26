"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
import {
  Download,
  GalleryHorizontalEnd,
  Home,
  ListChecks,
  Settings
} from "lucide-react";
import { fetchSettings } from "../lib/api";

const navItems = [
  { href: "/", label: "Home", icon: Home },
  { href: "/download", label: "Download", icon: Download },
  { href: "/gallery", label: "Gallery", icon: GalleryHorizontalEnd },
  { href: "/tasks", label: "Tasks", icon: ListChecks },
  { href: "/settings", label: "Settings", icon: Settings }
];

type ThemeId = "cyan-studio" | "sakura-light";

const themeLabels: Record<ThemeId, string> = {
  "cyan-studio": "Cyan Studio",
  "sakura-light": "Sakura Light"
};

export function AppShell({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const [theme, setTheme] = useState<ThemeId>("cyan-studio");

  useEffect(() => {
    let active = true;

    async function loadTheme() {
      try {
        const result = await fetchSettings();
        const setting = result.items.find((item) => item.key === "theme_id");
        const nextTheme = normalizeTheme(setting?.value);
        if (!active) return;
        setTheme(nextTheme);
        applyTheme(nextTheme);
      } catch {
        applyTheme("cyan-studio");
      }
    }

    function handleThemeChange(event: Event) {
      const nextTheme = normalizeTheme((event as CustomEvent).detail?.theme);
      setTheme(nextTheme);
      applyTheme(nextTheme);
    }

    loadTheme();
    window.addEventListener("pixiv-theme-change", handleThemeChange);

    return () => {
      active = false;
      window.removeEventListener("pixiv-theme-change", handleThemeChange);
    };
  }, []);

  return (
    <div className="app-shell">
      <header className="topbar">
        <Link className="brand" href="/" aria-label="Pixiv Platform home">
          <span className="brand-mark" aria-hidden="true">
            <img src="/app-icon.png" alt="" />
          </span>
          <span>
            <strong>Pixiv Platform</strong>
            <small>{themeLabels[theme]}</small>
          </span>
        </Link>
        <nav className="nav" aria-label="Primary navigation">
          {navItems.map((item) => {
            const active =
              item.href === "/" ? pathname === "/" : pathname.startsWith(item.href);
            const Icon = item.icon;

            return (
              <Link
                className={active ? "nav-link active" : "nav-link"}
                href={item.href}
                key={item.href}
              >
                <Icon size={17} aria-hidden="true" />
                <span>{item.label}</span>
              </Link>
            );
          })}
        </nav>
        <div className="task-pill" aria-label="Active task summary">
          <span className="pulse" aria-hidden="true" />
          Queue ready
        </div>
      </header>
      <main className="main">{children}</main>
    </div>
  );
}

function normalizeTheme(value: unknown): ThemeId {
  return value === "sakura-light" ? "sakura-light" : "cyan-studio";
}

function applyTheme(theme: ThemeId) {
  document.body.dataset.theme = theme;
}
