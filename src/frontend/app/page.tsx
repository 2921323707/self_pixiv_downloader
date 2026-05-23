"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import {
  ArrowRight,
  Bot,
  Cpu,
  Database,
  Download,
  Folder,
  GalleryHorizontalEnd,
  Gauge,
  Heart,
  ImageOff,
  KeyRound,
  Layers,
  ListChecks,
  Loader2,
  Settings,
  ShieldCheck,
  Sparkles,
  UserRound
} from "lucide-react";
import {
  fetchImages,
  fetchSettings,
  fetchTasks,
  GalleryImage,
  PublicSetting,
  TaskSummary
} from "../lib/api";
import { StatusBadge } from "../components/StatusBadge";

const quickEntries = [
  { href: "/download", label: "Single", icon: Download },
  { href: "/download", label: "Author", icon: UserRound },
  { href: "/download", label: "Bookmarks", icon: Heart },
  { href: "/download", label: "Smart", icon: Sparkles },
  { href: "/gallery", label: "Gallery", icon: GalleryHorizontalEnd },
  { href: "/settings", label: "Settings", icon: Settings }
];

const roadmapEntries = [
  {
    title: "Thumbnail Cache",
    status: "Next",
    detail: "Generate small local previews so Gallery stays smooth after batch and smart downloads."
  },
  {
    title: "Top10 / Random",
    status: "Planned",
    detail: "Discovery modes can reuse the existing batch task worker once browsing ergonomics are ready."
  },
  {
    title: "Cancel / Retry",
    status: "Planned",
    detail: "Task controls need queue state transitions before exposing destructive worker actions."
  }
];

export default function HomePage() {
  const [tasks, setTasks] = useState<TaskSummary[]>([]);
  const [images, setImages] = useState<GalleryImage[]>([]);
  const [settings, setSettings] = useState<PublicSetting[]>([]);
  const [bannerIndex, setBannerIndex] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function loadDashboard() {
    setLoading(true);
    setError(null);
    try {
      const [taskResult, imageResult, settingsResult] = await Promise.all([
        fetchTasks({ limit: 3 }),
        fetchImages({ limit: 8, r18_visibility: "exclude" }),
        fetchSettings()
      ]);
      setTasks(taskResult.items);
      setImages(imageResult.items);
      setSettings(settingsResult.items);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Dashboard lookup failed");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    loadDashboard();
  }, []);

  const bannerImages = useMemo(() => selectBannerImages(images), [images]);
  const bannerImage = bannerImages[bannerIndex % Math.max(bannerImages.length, 1)];

  useEffect(() => {
    setBannerIndex(0);
  }, [bannerImages.length]);

  useEffect(() => {
    if (bannerImages.length < 2) return;
    const interval = window.setInterval(() => {
      setBannerIndex((current) => (current + 1) % bannerImages.length);
    }, 5200);

    return () => {
      window.clearInterval(interval);
    };
  }, [bannerImages.length]);

  const settingsByKey = useMemo(
    () => Object.fromEntries(settings.map((setting) => [setting.key, setting])),
    [settings]
  );
  const taskCounts = useMemo(() => summarizeTasks(tasks), [tasks]);
  const downloadBasePath = settingText(settingsByKey.download_base_path, "project:output");
  const defaultBatchCount = settingText(settingsByKey.default_batch_count, "20");
  const maxRequestCount = settingText(settingsByKey.max_request_count, "100");
  const performanceStats = [
    {
      label: "Dashboard fanout",
      value: "3 API",
      detail: "tasks, images, settings are fetched in parallel"
    },
    {
      label: "Preview budget",
      value: `${images.length}/8`,
      detail: "Home caps recent normal previews before Gallery cache lands"
    },
    {
      label: "Batch default",
      value: defaultBatchCount,
      detail: `bounded by max_request_count ${maxRequestCount}`
    },
    {
      label: "Task detail poll",
      value: "1.6s",
      detail: "live modal refresh interval on the Tasks page"
    }
  ];

  return (
    <div className="page-grid">
      <section className="page-heading">
        <div>
          <h1>Home Dashboard</h1>
          <p>Downloader-first command center for queue state, local images, and runtime health.</p>
        </div>
        <button className="button secondary" onClick={loadDashboard} type="button">
          {loading ? (
            <Loader2 className="spin" size={16} aria-hidden="true" />
          ) : (
            <ListChecks size={16} aria-hidden="true" />
          )}
          Refresh
        </button>
      </section>

      {error ? <div className="error-box">{error}</div> : null}

      <section className="home-hero-grid">
        <div className="home-image-banner" aria-label="Recent normal image banner">
          {bannerImage?.preview_url || bannerImage?.thumbnail_url ? (
            <img
              alt={bannerImage.title || `Pixiv ${bannerImage.pixiv_id} page ${bannerImage.page_index}`}
              src={bannerImage.preview_url || bannerImage.thumbnail_url || ""}
            />
          ) : (
            <ImageOff size={26} aria-hidden="true" />
          )}
          <div className="home-banner-copy">
            <span>Recent normal download</span>
            <h2>{bannerImage?.title || "Local library is ready"}</h2>
            <p>
              {bannerImage
                ? `#${bannerImage.pixiv_id}_p${bannerImage.page_index}`
                : "Downloads will appear here after the first indexed normal image."}
            </p>
          </div>
          {bannerImages.length > 1 ? (
            <div className="banner-dots" aria-label="Banner position">
              {bannerImages.map((image, index) => (
                <button
                  aria-label={`Show banner image ${index + 1}`}
                  className={image.image_id === bannerImage?.image_id ? "active" : ""}
                  key={image.image_id}
                  onClick={() => setBannerIndex(index)}
                  type="button"
                />
              ))}
            </div>
          ) : null}
        </div>

        <aside className="feature-panel home-core-card" aria-label="Rust core driver notes">
          <div className="panel-title">
            <Cpu size={18} aria-hidden="true" />
            <h2>Rust Core Driver</h2>
          </div>
          <div className="core-annotation-list">
            <CoreNote
              icon={Download}
              title="Downloader first"
              text="single, author, bookmarks, and smart flows all enqueue task records before worker execution."
            />
            <CoreNote
              icon={Database}
              title="SQLite trace"
              text="images, tags, sources, task_items, and task_logs remain queryable after each run."
            />
            <CoreNote
              icon={ShieldCheck}
              title="Runtime secrets"
              text="Pixiv cookie and DeepSeek key stay behind settings masks and are resolved only at runtime."
            />
          </div>
        </aside>
      </section>

      <section className="home-summary-grid" aria-label="Dashboard summary">
        <div className="summary-tile">
          <span>Active</span>
          <strong>{taskCounts.active}</strong>
          <small>recent pending or running tasks</small>
        </div>
        <div className="summary-tile">
          <span>Completed</span>
          <strong>{taskCounts.completed}</strong>
          <small>recent successful tasks</small>
        </div>
        <div className="summary-tile">
          <span>Warnings</span>
          <strong>{taskCounts.warning}</strong>
          <small>failed or partial tasks</small>
        </div>
        <div className="summary-tile">
          <span>Gallery</span>
          <strong>{images.length}</strong>
          <small>latest indexed previews loaded</small>
        </div>
      </section>

      <section className="dashboard-grid home-dashboard-grid">
        <div className="feature-panel">
          <div className="panel-title">
            <ListChecks size={18} aria-hidden="true" />
            <h2>Recent Tasks</h2>
          </div>
          {tasks.length > 0 ? (
            <div className="recent-task-list">
              {tasks.map((task) => (
                <Link
                  className="recent-task-row"
                  href={`/tasks?task=${encodeURIComponent(task.task_id)}`}
                  key={task.task_id}
                >
                  <div>
                    <strong>{task.type}</strong>
                    <span>{task.task_id}</span>
                  </div>
                  <StatusBadge status={task.status} />
                  <small>
                    {task.progress_done + task.progress_failed}/{task.progress_total || 1}
                  </small>
                </Link>
              ))}
            </div>
          ) : (
            <p className="quiet">No persisted tasks yet.</p>
          )}
          <div className="panel-footer">
            <Link className="inline-link panel-link" href="/tasks">
              Open task panel <ArrowRight size={15} aria-hidden="true" />
            </Link>
          </div>
        </div>

        <div className="feature-panel">
          <div className="panel-title">
            <GalleryHorizontalEnd size={18} aria-hidden="true" />
            <h2>Recent Downloads</h2>
          </div>
          {images.length > 0 ? (
            <div className="home-preview-grid" aria-label="Recent downloaded images">
              {images.slice(0, 6).map((image) => (
                <Link
                  className="home-preview"
                  href="/gallery"
                  key={image.image_id}
                  title={image.title || `Pixiv ${image.pixiv_id}`}
                >
                  {image.thumbnail_url || image.preview_url ? (
                    <img
                      alt={image.title || `Pixiv ${image.pixiv_id} page ${image.page_index}`}
                      src={image.thumbnail_url || image.preview_url || ""}
                    />
                  ) : (
                    <ImageOff size={20} aria-hidden="true" />
                  )}
                  <span>#{image.pixiv_id}_p{image.page_index}</span>
                </Link>
              ))}
            </div>
          ) : (
            <p className="quiet">No indexed images are available for preview yet.</p>
          )}
          <div className="panel-footer">
            <div className="library-note">
              <Folder size={16} aria-hidden="true" />
              <span>{downloadBasePath}</span>
            </div>
            <Link className="inline-link panel-link" href="/gallery">
              Browse gallery <ArrowRight size={15} aria-hidden="true" />
            </Link>
          </div>
        </div>

        <div className="feature-panel">
          <div className="panel-title">
            <ShieldCheck size={18} aria-hidden="true" />
            <h2>Configuration</h2>
          </div>
          <div className="config-status-list">
            <ConfigStatus
              icon={KeyRound}
              label="Pixiv cookie"
              setting={settingsByKey.pixiv_cookie}
            />
            <ConfigStatus
              icon={Bot}
              label="DeepSeek key"
              setting={settingsByKey.deepseek_api_key}
            />
            <ConfigStatus
              icon={Folder}
              label="Download path"
              setting={settingsByKey.download_base_path}
              visibleValue={downloadBasePath}
            />
          </div>
          <div className="panel-footer">
            <p className="quiet">
              Secrets stay masked in public settings responses and are never printed here.
            </p>
            <Link className="inline-link panel-link" href="/settings">
              Open settings <ArrowRight size={15} aria-hidden="true" />
            </Link>
          </div>
        </div>
      </section>

      <section className="home-insight-grid" aria-label="Operational insight panels">
        <div className="feature-panel performance-panel">
          <div className="panel-title">
            <Gauge size={18} aria-hidden="true" />
            <h2>Performance Watch</h2>
          </div>
          <div className="performance-metric-grid">
            {performanceStats.map((metric) => (
              <div className="performance-metric" key={metric.label}>
                <span>{metric.label}</span>
                <strong>{metric.value}</strong>
                <small>{metric.detail}</small>
              </div>
            ))}
          </div>
        </div>

        <div className="feature-panel roadmap-panel">
          <div className="panel-title">
            <Layers size={18} aria-hidden="true" />
            <h2>Next Capability Slots</h2>
          </div>
          <div className="roadmap-list">
            {roadmapEntries.map((entry) => (
              <div className="roadmap-row" key={entry.title}>
                <div>
                  <strong>{entry.title}</strong>
                  <span>{entry.detail}</span>
                </div>
                <em>{entry.status}</em>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="quick-entry-grid" aria-label="Quick entries">
        {quickEntries.map((entry) => {
          const Icon = entry.icon;
          return (
            <Link className="quick-entry" href={entry.href} key={entry.label}>
              <Icon size={18} aria-hidden="true" />
              <span>{entry.label}</span>
            </Link>
          );
        })}
      </section>
    </div>
  );
}

function CoreNote({
  icon: Icon,
  title,
  text
}: {
  icon: typeof Download;
  title: string;
  text: string;
}) {
  return (
    <div className="core-note">
      <Icon size={17} aria-hidden="true" />
      <div>
        <strong>{title}</strong>
        <span>{text}</span>
      </div>
    </div>
  );
}

function summarizeTasks(tasks: TaskSummary[]) {
  return tasks.reduce(
    (counts, task) => {
      if (task.status === "pending" || task.status === "running") counts.active += 1;
      if (task.status === "completed") counts.completed += 1;
      if (task.status === "failed" || task.status === "completed_with_errors") {
        counts.warning += 1;
      }
      return counts;
    },
    { active: 0, completed: 0, warning: 0 }
  );
}

function selectBannerImages(images: GalleryImage[]) {
  const normalImages = images.filter((image) => image.category === "normal");
  const wideNormalImages = normalImages.filter((image) => {
    if (!image.width || !image.height) return false;
    return image.width / image.height >= 1.2;
  });

  if (wideNormalImages.length > 0) return wideNormalImages.slice(0, 6);
  if (normalImages.length > 0) return normalImages.slice(0, 6);
  return images.slice(0, 6);
}

function settingText(setting: PublicSetting | undefined, fallback: string) {
  if (!setting) return fallback;
  if (typeof setting.value === "string" && setting.value.trim()) return setting.value;
  if (typeof setting.value === "number") return String(setting.value);
  return fallback;
}

function hasSettingValue(setting: PublicSetting | undefined) {
  if (!setting) return false;
  if (typeof setting.value === "string") return setting.value.trim().length > 0;
  return setting.value !== null && setting.value !== undefined;
}

function ConfigStatus({
  icon: Icon,
  label,
  setting,
  visibleValue
}: {
  icon: typeof KeyRound;
  label: string;
  setting?: PublicSetting;
  visibleValue?: string;
}) {
  const configured = hasSettingValue(setting);
  const status = configured ? "Configured" : "Missing";

  return (
    <div className="config-status-row">
      <Icon size={17} aria-hidden="true" />
      <div>
        <strong>{label}</strong>
        <span>{visibleValue && configured ? visibleValue : status}</span>
      </div>
      <em className={configured ? "ready" : ""}>{configured ? "ready" : "needed"}</em>
    </div>
  );
}
