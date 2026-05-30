"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
import {
  Download,
  GalleryHorizontalEnd,
  Home,
  ListChecks,
  Settings,
  UserRound
} from "lucide-react";
import { fetchPixivAccounts, fetchSettings, fetchTasks } from "../lib/api";

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
  const [activePixivUid, setActivePixivUid] = useState<string | null>(null);
  const [activeTaskRunning, setActiveTaskRunning] = useState(false);

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

  useEffect(() => {
    let active = true;

    async function loadActiveAccount() {
      try {
        const result = await fetchPixivAccounts();
        if (!active) return;
        setActivePixivUid(result.active?.user_uid || null);
      } catch {
        if (active) setActivePixivUid(null);
      }
    }

    function handlePixivAccountChange() {
      loadActiveAccount();
    }

    loadActiveAccount();
    window.addEventListener("pixiv-account-change", handlePixivAccountChange);

    return () => {
      active = false;
      window.removeEventListener("pixiv-account-change", handlePixivAccountChange);
    };
  }, []);

  useEffect(() => {
    let active = true;

    async function loadActiveTasks() {
      try {
        const result = await fetchTasks({ limit: 5 });
        if (!active) return;
        setActiveTaskRunning(
          result.items.some((task) => task.status === "pending" || task.status === "running")
        );
      } catch {
        if (active) setActiveTaskRunning(false);
      }
    }

    loadActiveTasks();
    const interval = window.setInterval(loadActiveTasks, 4000);

    return () => {
      active = false;
      window.clearInterval(interval);
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
                <Icon
                  className={
                    item.href === "/download" && activeTaskRunning ? "nav-download-active" : ""
                  }
                  size={17}
                  aria-hidden="true"
                />
                <span>{item.label}</span>
              </Link>
            );
          })}
        </nav>
        <div
          className={activePixivUid ? "task-pill uid-pill" : "task-pill uid-pill muted"}
          aria-label="Active Pixiv account"
          title={activePixivUid ? `Active Pixiv UID ${activePixivUid}` : "Pixiv account not bound"}
        >
          <UserRound size={15} aria-hidden="true" />
          UID: {activePixivUid || "Not bound"}
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
