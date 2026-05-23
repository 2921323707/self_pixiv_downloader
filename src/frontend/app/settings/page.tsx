"use client";

import { FormEvent, useEffect, useMemo, useState } from "react";
import { EyeOff, Folder, KeyRound, Loader2, Palette, Save, Shield, Wifi } from "lucide-react";
import {
  fetchSettings,
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
    description: "Theme selection for the Cyan Studio shell.",
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

              return (
                <form
                  className="setting-row"
                  key={setting.key}
                  onSubmit={(event) => submit(event, setting)}
                >
                  <Icon size={19} aria-hidden="true" />
                  <div>
                    <h2>{meta.title}</h2>
                    <input
                      inputMode={
                        setting.key === "max_request_count" ||
                        setting.key === "default_batch_count"
                          ? "numeric"
                          : "text"
                      }
                      placeholder={setting.is_secret ? "masked" : setting.key}
                      value={drafts[setting.key] ?? ""}
                      onChange={(event) =>
                        setDrafts((current) => ({
                          ...current,
                          [setting.key]: event.target.value
                        }))
                      }
                    />
                  </div>
                  <div className="setting-actions">
                    {setting.key === "pixiv_cookie" ? (
                      <button
                        className="button secondary"
                        disabled={testingPixiv || isSaving}
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
                    <button
                      className="button secondary"
                      disabled={isSaving || (setting.is_secret && !drafts[setting.key])}
                      type="submit"
                    >
                      {isSaving ? (
                        <Loader2 className="spin" size={16} aria-hidden="true" />
                      ) : (
                        <Save size={16} aria-hidden="true" />
                      )}
                      {savedKey === setting.key ? "Saved" : "Save"}
                    </button>
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
