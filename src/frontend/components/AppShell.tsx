"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  Download,
  GalleryHorizontalEnd,
  Home,
  ListChecks,
  Settings,
  Sparkles
} from "lucide-react";

const navItems = [
  { href: "/", label: "Home", icon: Home },
  { href: "/download", label: "Download", icon: Download },
  { href: "/gallery", label: "Gallery", icon: GalleryHorizontalEnd },
  { href: "/tasks", label: "Tasks", icon: ListChecks },
  { href: "/settings", label: "Settings", icon: Settings }
];

export function AppShell({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();

  return (
    <div className="app-shell">
      <header className="topbar">
        <Link className="brand" href="/" aria-label="Pixiv Platform home">
          <span className="brand-mark" aria-hidden="true">
            <Sparkles size={18} />
          </span>
          <span>
            <strong>Pixiv Platform</strong>
            <small>Cyan Studio</small>
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
