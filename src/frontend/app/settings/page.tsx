"use client";

import { FormEvent, useEffect, useMemo, useState } from "react";
import { EyeOff, Folder, FolderOpen, KeyRound, Loader2, Palette, Save, Shield, Wifi } from "lucide-react";
import {
  fetchSettings,
  getTauriInvoke,
  isTauriDesktopRuntime,
  PublicSetting,
  saveSetting,
  testDeepSeekConnection,
  testPixivConnection
} from "../../lib/api";

const labels: Record<string, { title: string; icon: typeof KeyRound }> = {
  pixiv_cookie: { title: "Pixiv cookie", icon: KeyRound },
  deepseek_api_key: { title: "DeepSeek API key", icon: KeyRound },
  deepseek_base_url: { title: "DeepSeek base URL", icon: KeyRound },
  deepseek_model: { title: "DeepSeek model", icon: KeyRound },
  download_base_path: { title: "Download base path", icon: Folder },
  default_batch_count: { title: "Default batch count", icon: Shield },
  max_request_count: { title: "Max request count", icon: Shield },
  r18_policy: { title: "R18 visibility", icon: EyeOff },
  theme_id: { title: "Theme", icon: Palette }
};

type ThemeId = "cyan-studio" | "sakura-light";

type PixivSessionCookie = {
  value: string;
  domain: string | null;
  path: string | null;
  http_only: boolean | null;
  secure: boolean | null;
};

const themeOptions = [
  { value: "cyan-studio", label: "Cyan Studio" },
  { value: "sakura-light", label: "Sakura Light" }
] as const;

const settingsCategories = [
  {
    id: "general",
    title: "通用配置",
    description: "Default request behavior and library visibility.",
    icon: Shield,
    keys: ["default_batch_count", "max_request_count", "r18_policy"]
  },
  {
    id: "appearance",
    title: "外观与主题",
    description: "Switch between the default shell and Sakura Light.",
    icon: Palette,
    keys: ["theme_id"]
  },
  {
    id: "pixiv",
    title: "Pixiv 连接",
    description: "Runtime cookie used by downloader workers.",
    icon: KeyRound,
    keys: ["pixiv_cookie"]
  },
  {
    id: "deepseek",
    title: "DeepSeek / Smart Retrieval",
    description: "Smart prompt parsing model and endpoint.",
    icon: KeyRound,
    keys: ["deepseek_base_url", "deepseek_model", "deepseek_api_key"]
  },
  {
    id: "storage",
    title: "Storage / Download",
    description: "Local output root for downloaded files.",
    icon: Folder,
    keys: ["download_base_path"]
  }
] as const;

type SettingsCategory = (typeof settingsCategories)[number]["id"];

export default function SettingsPage() {
  const [settings, setSettings] = useState<PublicSetting[]>([]);
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [activeCategory, setActiveCategory] = useState<SettingsCategory>("general");
  const [savingKey, setSavingKey] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [savedKey, setSavedKey] = useState<string | null>(null);
  const [testingPixiv, setTestingPixiv] = useState(false);
  const [testingDeepSeek, setTestingDeepSeek] = useState(false);
  const [selectingDownloadDirectory, setSelectingDownloadDirectory] = useState(false);
  const [refreshingPixivLogin, setRefreshingPixivLogin] = useState(false);
  const [tauriDesktopReady, setTauriDesktopReady] = useState(false);
  const [pixivTestResult, setPixivTestResult] = useState<string | null>(null);
  const [deepseekTestResult, setDeepseekTestResult] = useState<string | null>(null);

  async function load() {
    setLoading(true);
    setError(null);
    try {
      const result = await fetchSettings();
      setSettings(result.items);
      setDrafts(
        Object.fromEntries(
          result.items.map((setting) => [
            setting.key,
            setting.is_secret ? "" : settingValueToInput(setting.value)
          ])
        )
      );
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Settings lookup failed");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
  }, []);

  useEffect(() => {
    setTauriDesktopReady(isTauriDesktopRuntime());

    const timeout = window.setTimeout(() => {
      setTauriDesktopReady(isTauriDesktopRuntime());
    }, 500);

    return () => window.clearTimeout(timeout);
  }, []);

  const settingsByKey = useMemo(
    () => Object.fromEntries(settings.map((setting) => [setting.key, setting])),
    [settings]
  );

  const activeCategoryMeta = settingsCategories.find((category) => category.id === activeCategory);
  const visibleSettings = activeCategoryMeta
    ? activeCategoryMeta.keys
        .map((key) => settingsByKey[key])
        .filter((setting): setting is PublicSetting => Boolean(setting))
    : [];

  async function submit(event: FormEvent<HTMLFormElement>, setting: PublicSetting) {
    event.preventDefault();
    const raw = drafts[setting.key] ?? "";
    if (setting.is_secret && raw.trim() === "") {
      return;
    }

    await saveSettingDraft(setting, raw);
  }

  async function saveSettingDraft(setting: PublicSetting, raw: string) {
    setSavingKey(setting.key);
    setSavedKey(null);
    setPixivTestResult(null);
    setDeepseekTestResult(null);
    setError(null);
    try {
      const saved = await saveSetting(setting.key, coerceSettingValue(setting.key, raw));
      setSettings((current) =>
        current.map((item) => (item.key === saved.key ? saved : item))
      );
      setDrafts((current) => ({
        ...current,
        [saved.key]: saved.is_secret ? "" : settingValueToInput(saved.value)
      }));
      setSavedKey(saved.key);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Setting save failed");
    } finally {
      setSavingKey(null);
    }
  }

  async function chooseDownloadDirectory(setting: PublicSetting) {
    const invoke = getTauriInvoke();
    if (!invoke) {
      setError("Folder picker is only available in the Tauri desktop app.");
      return;
    }

    setSelectingDownloadDirectory(true);
    setError(null);
    setSavedKey(null);
    try {
      const selectedPath = await invoke<string | null>("select_download_directory");
      if (!selectedPath) {
        return;
      }
      setDrafts((current) => ({
        ...current,
        [setting.key]: selectedPath
      }));
      await saveSettingDraft(setting, selectedPath);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Folder selection failed");
    } finally {
      setSelectingDownloadDirectory(false);
    }
  }

  async function refreshPixivLogin(setting: PublicSetting) {
    const invoke = getTauriInvoke();
    if (!invoke) {
      setError(
        "Pixiv login refresh could not reach the Tauri desktop bridge. Restart the desktop app and try again."
      );
      return;
    }

    setRefreshingPixivLogin(true);
    setSavingKey(setting.key);
    setSavedKey(null);
    setPixivTestResult(null);
    setDeepseekTestResult(null);
    setError(null);
    try {
      const cookie = await invoke<PixivSessionCookie>("refresh_pixiv_phpsessid");
      const saved = await saveSetting(setting.key, cookie.value);
      setSettings((current) =>
        current.map((item) => (item.key === saved.key ? saved : item))
      );
      setDrafts((current) => ({
        ...current,
        [saved.key]: saved.is_secret ? "" : settingValueToInput(saved.value)
      }));
      setSavedKey(saved.key);

      const result = await testPixivConnection("144920810");
      setPixivTestResult(
        result.title
          ? `Pixiv login refreshed: ${result.title}`
          : `Pixiv login refreshed${cookie.domain ? ` for ${cookie.domain}` : ""}`
      );
      window.alert("Pixiv login refreshed successfully. The login window has been closed.");
    } catch (caught) {
      const message = caught instanceof Error ? caught.message : String(caught);
      setError(`Pixiv login refresh failed: ${message}`);
    } finally {
      setRefreshingPixivLogin(false);
      setSavingKey(null);
    }
  }

  async function switchTheme(setting: PublicSetting, theme: ThemeId) {
    const currentTheme = normalizeTheme(drafts[setting.key]);
    if (currentTheme === theme) {
      window.dispatchEvent(new CustomEvent("pixiv-theme-change", { detail: { theme } }));
      return;
    }

    setSavingKey(setting.key);
    setSavedKey(null);
    setPixivTestResult(null);
    setDeepseekTestResult(null);
    setError(null);
    try {
      const saved = await saveSetting(setting.key, theme);
      setSettings((current) =>
        current.map((item) => (item.key === saved.key ? saved : item))
      );
      setDrafts((current) => ({
        ...current,
        [saved.key]: settingValueToInput(saved.value)
      }));
      window.dispatchEvent(new CustomEvent("pixiv-theme-change", { detail: { theme } }));
      setSavedKey(saved.key);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Theme save failed");
    } finally {
      setSavingKey(null);
    }
  }

  async function testPixiv() {
    setTestingPixiv(true);
    setPixivTestResult(null);
    setError(null);
    try {
      const result = await testPixivConnection("144920810");
      setPixivTestResult(
        result.title ? `Pixiv connection ok: ${result.title}` : "Pixiv cookie is configured"
      );
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Pixiv connection test failed");
    } finally {
      setTestingPixiv(false);
    }
  }

  async function testDeepSeek() {
    setTestingDeepSeek(true);
    setDeepseekTestResult(null);
    setError(null);
    try {
      const result = await testDeepSeekConnection();
      setDeepseekTestResult(`DeepSeek ${result.status}: ${result.model}`);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "DeepSeek connection test failed");
    } finally {
      setTestingDeepSeek(false);
    }
  }

  return (
    <div className="page-grid">
      <section className="page-heading">
        <div>
          <h1>Settings</h1>
          <p>{settings.length} settings loaded from the backend repository.</p>
        </div>
        <span className="mode-chip">
          {loading ? (
            <Loader2 className="spin" size={15} aria-hidden="true" />
          ) : (
            <Shield size={15} aria-hidden="true" />
          )}
          Secrets masked
        </span>
      </section>

      {error ? <div className="error-box">{error}</div> : null}
      {pixivTestResult ? <div className="success-box">{pixivTestResult}</div> : null}
      {deepseekTestResult ? <div className="success-box">{deepseekTestResult}</div> : null}

      <section className="settings-workbench">
        <nav className="settings-category-list" aria-label="Settings categories">
          {settingsCategories.map((category) => {
            const Icon = category.icon;
            return (
              <button
                className={activeCategory === category.id ? "active" : ""}
                key={category.id}
                onClick={() => setActiveCategory(category.id)}
                type="button"
              >
                <Icon size={17} aria-hidden="true" />
                <span>{category.title}</span>
              </button>
            );
          })}
        </nav>

        <div className="settings-panel">
          <div className="settings-panel-head">
            <h2>{activeCategoryMeta?.title || "Settings"}</h2>
            <p>{activeCategoryMeta?.description}</p>
          </div>

          <div className="settings-grid">
            {visibleSettings.map((setting) => {
              const meta = labels[setting.key] || { title: setting.key, icon: Shield };
              const Icon = meta.icon;
              const isSaving = savingKey === setting.key;
              const isDownloadPath = setting.key === "download_base_path";
              const isPickingDirectory = isDownloadPath && selectingDownloadDirectory;
              const isPixivCookie = setting.key === "pixiv_cookie";
              const currentTheme = normalizeTheme(drafts[setting.key]);

              return (
                <form
                  className="setting-row"
                  key={setting.key}
                  onSubmit={(event) => submit(event, setting)}
                >
                  <Icon size={19} aria-hidden="true" />
                  <div>
                    <h2>{meta.title}</h2>
                    {setting.key === "theme_id" ? (
                      <div className="theme-switcher" aria-label={meta.title}>
                        {themeOptions.map((theme) => (
                          <button
                            className={currentTheme === theme.value ? "active" : ""}
                            disabled={isSaving}
                            key={theme.value}
                            onClick={() => switchTheme(setting, theme.value)}
                            type="button"
                          >
                            {theme.label}
                          </button>
                        ))}
                      </div>
                    ) : (
                      <div className={isDownloadPath ? "path-picker-field" : undefined}>
                        <input
                          inputMode={
                            setting.key === "max_request_count" ||
                            setting.key === "default_batch_count"
                              ? "numeric"
                              : "text"
                          }
                          placeholder={setting.is_secret ? "masked" : setting.key}
                          readOnly={isDownloadPath}
                          value={drafts[setting.key] ?? ""}
                          onChange={(event) =>
                            setDrafts((current) => ({
                              ...current,
                              [setting.key]: event.target.value
                            }))
                          }
                        />
                      </div>
                    )}
                  </div>
                  <div className="setting-actions">
                    {setting.key === "theme_id" && isSaving ? (
                      <Loader2 className="spin" size={16} aria-hidden="true" />
                    ) : null}
                    {isPixivCookie && tauriDesktopReady ? (
                      <button
                        className="button secondary"
                        disabled={refreshingPixivLogin || testingPixiv || isSaving}
                        onClick={() => refreshPixivLogin(setting)}
                        type="button"
                      >
                        {refreshingPixivLogin ? (
                          <Loader2 className="spin" size={16} aria-hidden="true" />
                        ) : (
                          <KeyRound size={16} aria-hidden="true" />
                        )}
                        Refresh
                      </button>
                    ) : null}
                    {isPixivCookie ? (
                      <button
                        className="button secondary"
                        disabled={testingPixiv || refreshingPixivLogin || isSaving}
                        onClick={testPixiv}
                        type="button"
                      >
                        {testingPixiv ? (
                          <Loader2 className="spin" size={16} aria-hidden="true" />
                        ) : (
                          <Wifi size={16} aria-hidden="true" />
                        )}
                        Test
                      </button>
                    ) : null}
                    {setting.key === "deepseek_api_key" ? (
                      <button
                        className="button secondary"
                        disabled={testingDeepSeek || isSaving}
                        onClick={testDeepSeek}
                        type="button"
                      >
                        {testingDeepSeek ? (
                          <Loader2 className="spin" size={16} aria-hidden="true" />
                        ) : (
                          <Wifi size={16} aria-hidden="true" />
                        )}
                        Test
                      </button>
                    ) : null}
                    {isDownloadPath ? (
                      <button
                        className="button secondary"
                        disabled={isSaving || selectingDownloadDirectory}
                        onClick={() => chooseDownloadDirectory(setting)}
                        type="button"
                      >
                        {isPickingDirectory ? (
                          <Loader2 className="spin" size={16} aria-hidden="true" />
                        ) : (
                          <FolderOpen size={16} aria-hidden="true" />
                        )}
                        {savedKey === setting.key ? "Selected" : "Choose"}
                      </button>
                    ) : null}
                    {setting.key !== "theme_id" && !isDownloadPath ? (
                      <button
                        className="button secondary"
                        disabled={
                          isSaving ||
                          refreshingPixivLogin ||
                          (setting.is_secret && !drafts[setting.key])
                        }
                        type="submit"
                      >
                        {isSaving ? (
                          <Loader2 className="spin" size={16} aria-hidden="true" />
                        ) : (
                          <Save size={16} aria-hidden="true" />
                        )}
                        {savedKey === setting.key ? "Saved" : "Save"}
                      </button>
                    ) : null}
                  </div>
                </form>
              );
            })}
            {visibleSettings.length === 0 ? (
              <p className="quiet">No backend settings are available for this category.</p>
            ) : null}
          </div>
        </div>
      </section>
    </div>
  );
}

function settingValueToInput(value: unknown): string {
  if (typeof value === "string") return value;
  if (typeof value === "number") return String(value);
  return JSON.stringify(value);
}

function coerceSettingValue(key: string, raw: string): unknown {
  if (key === "max_request_count" || key === "default_batch_count") return Number(raw);
  return raw;
}

function normalizeTheme(value: unknown): ThemeId {
  return value === "sakura-light" ? "sakura-light" : "cyan-studio";
}
