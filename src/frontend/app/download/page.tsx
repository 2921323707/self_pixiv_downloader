"use client";

import Link from "next/link";
import { FormEvent, KeyboardEvent, useState } from "react";
import { Download, Heart, Loader2, Plus, Send, Sparkles, UserRound, X } from "lucide-react";
import {
  parseSmartPrompt,
  startSmartDownload,
  SmartParseResult,
  startAuthorDownload,
  startBookmarkDownload,
  startSingleDownload
} from "../../lib/api";

const r18Policies = [
  { value: "exclude", label: "Exclude" },
  { value: "include_blurred", label: "Blurred" },
  { value: "include_visible", label: "Visible" },
  { value: "only_r18", label: "Only R18" }
];

const downloadTools = [
  { value: "single", label: "Single", icon: Download },
  { value: "author", label: "Author", icon: UserRound },
  { value: "bookmarks", label: "Bookmarks", icon: Heart },
  { value: "smart", label: "Smart", icon: Sparkles }
] as const;

type DownloadTool = (typeof downloadTools)[number]["value"];

export default function DownloadPage() {
  const [activeTool, setActiveTool] = useState<DownloadTool>("single");
  const [pixivId, setPixivId] = useState("");
  const [pageIndex, setPageIndex] = useState("0");
  const [policy, setPolicy] = useState("exclude");
  const [taskId, setTaskId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [authorUid, setAuthorUid] = useState("");
  const [authorLimit, setAuthorLimit] = useState("20");
  const [authorPolicy, setAuthorPolicy] = useState("exclude");
  const [authorTaskId, setAuthorTaskId] = useState<string | null>(null);
  const [authorError, setAuthorError] = useState<string | null>(null);
  const [authorSubmitting, setAuthorSubmitting] = useState(false);
  const [bookmarkLimit, setBookmarkLimit] = useState("20");
  const [bookmarkPolicy, setBookmarkPolicy] = useState("exclude");
  const [bookmarkTaskId, setBookmarkTaskId] = useState<string | null>(null);
  const [bookmarkError, setBookmarkError] = useState<string | null>(null);
  const [bookmarkSubmitting, setBookmarkSubmitting] = useState(false);
  const [smartPrompt, setSmartPrompt] = useState("");
  const [smartCount, setSmartCount] = useState("20");
  const [smartPolicy, setSmartPolicy] = useState("exclude");
  const [smartPlan, setSmartPlan] = useState<SmartParseResult | null>(null);
  const [smartTags, setSmartTags] = useState<string[]>([]);
  const [smartNegativeTags, setSmartNegativeTags] = useState<string[]>([]);
  const [smartTagDraft, setSmartTagDraft] = useState("");
  const [smartNegativeTagDraft, setSmartNegativeTagDraft] = useState("");
  const [smartTaskId, setSmartTaskId] = useState<string | null>(null);
  const [smartError, setSmartError] = useState<string | null>(null);
  const [smartSubmitting, setSmartSubmitting] = useState(false);
  const [smartDownloadSubmitting, setSmartDownloadSubmitting] = useState(false);

  async function submitSingle(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setTaskId(null);
    setSubmitting(true);

    try {
      const result = await startSingleDownload({
        pixiv_id: pixivId.trim(),
        page_index: Number(pageIndex || 0),
        r18_policy: policy
      });
      setTaskId(result.task_id);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Download failed");
    } finally {
      setSubmitting(false);
    }
  }

  async function submitAuthor(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setAuthorError(null);
    setAuthorTaskId(null);
    setAuthorSubmitting(true);

    try {
      const result = await startAuthorDownload({
        author_uid: authorUid.trim(),
        limit: authorLimit.trim() ? Number(authorLimit) : undefined,
        r18_policy: authorPolicy
      });
      setAuthorTaskId(result.task_id);
    } catch (caught) {
      setAuthorError(caught instanceof Error ? caught.message : "Author batch failed");
    } finally {
      setAuthorSubmitting(false);
    }
  }

  async function submitBookmarks(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBookmarkError(null);
    setBookmarkTaskId(null);
    setBookmarkSubmitting(true);

    try {
      const result = await startBookmarkDownload({
        limit: bookmarkLimit.trim() ? Number(bookmarkLimit) : undefined,
        r18_policy: bookmarkPolicy
      });
      setBookmarkTaskId(result.task_id);
    } catch (caught) {
      setBookmarkError(caught instanceof Error ? caught.message : "Bookmark batch failed");
    } finally {
      setBookmarkSubmitting(false);
    }
  }

  async function submitSmartParse(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSmartError(null);
    setSmartPlan(null);
    setSmartTaskId(null);
    setSmartSubmitting(true);

    try {
      const result = await parseSmartPrompt({
        prompt: smartPrompt.trim(),
        count: smartCount.trim() ? Number(smartCount) : undefined,
        r18_policy: smartPolicy
      });
      setSmartPlan(result);
      setSmartTags(result.tags);
      setSmartNegativeTags(result.negative_tags);
      setSmartTagDraft("");
      setSmartNegativeTagDraft("");
    } catch (caught) {
      setSmartError(caught instanceof Error ? caught.message : "Smart parse failed");
    } finally {
      setSmartSubmitting(false);
    }
  }

  async function submitSmartDownload() {
    const tags = mergeTags(smartTags, tagTokens(smartTagDraft));
    const negativeTags = mergeTags(smartNegativeTags, tagTokens(smartNegativeTagDraft));
    if (tags.length === 0) {
      setSmartError("Add at least one tag before enqueueing smart download.");
      return;
    }

    setSmartError(null);
    setSmartTaskId(null);
    setSmartDownloadSubmitting(true);
    setSmartTags(tags);
    setSmartNegativeTags(negativeTags);
    setSmartTagDraft("");
    setSmartNegativeTagDraft("");

    try {
      const result = await startSmartDownload({
        prompt: smartPrompt.trim() || tags.join(" "),
        tags,
        negative_tags: negativeTags,
        count: smartPlan?.count_recommend || (smartCount.trim() ? Number(smartCount) : undefined),
        r18_policy: smartPolicy,
        model: smartPlan?.model
      });
      setSmartTaskId(result.task_id);
    } catch (caught) {
      setSmartError(caught instanceof Error ? caught.message : "Smart download failed");
    } finally {
      setSmartDownloadSubmitting(false);
    }
  }

  return (
    <div className="page-grid">
      <section className="page-heading">
        <div>
          <h1>Download Center</h1>
          <p>Start with the proven single-work backend, then expand outward.</p>
        </div>
        <span className="mode-chip">Downloader APIs live</span>
      </section>

      <section className="download-workbench">
        <div className="download-tool-tabs" role="tablist" aria-label="Download modes">
          {downloadTools.map((tool) => {
            const Icon = tool.icon;
            return (
              <button
                aria-selected={activeTool === tool.value}
                className={activeTool === tool.value ? "active" : ""}
                key={tool.value}
                onClick={() => setActiveTool(tool.value)}
                role="tab"
                type="button"
              >
                <Icon size={17} aria-hidden="true" />
                <span>{tool.label}</span>
              </button>
            );
          })}
        </div>

        <div className="download-tool-shell">
          {activeTool === "single" ? (
            <form className="tool-panel download-tool-panel" onSubmit={submitSingle}>
          <div className="panel-title">
            <Download size={18} aria-hidden="true" />
            <h2>Single Work</h2>
          </div>

          <label>
            Pixiv ID
            <input
              inputMode="numeric"
              pattern="[0-9]*"
              placeholder="144920810"
              value={pixivId}
              onChange={(event) => setPixivId(event.target.value)}
              required
            />
          </label>

          <label>
            Page index
            <input
              min="0"
              type="number"
              value={pageIndex}
              onChange={(event) => setPageIndex(event.target.value)}
            />
          </label>

          <fieldset className="segmented">
            <legend>R18 policy</legend>
            {r18Policies.map((item) => (
              <button
                aria-pressed={policy === item.value}
                className={policy === item.value ? "active" : ""}
                key={item.value}
                onClick={() => setPolicy(item.value)}
                type="button"
              >
                {item.label}
              </button>
            ))}
          </fieldset>

          <button className="button primary" disabled={submitting} type="submit">
            {submitting ? (
              <Loader2 className="spin" size={17} aria-hidden="true" />
            ) : (
              <Send size={17} aria-hidden="true" />
            )}
            Enqueue download
          </button>

          {taskId ? (
            <div className="success-box">
              <strong>Task queued</strong>
              <Link href={`/tasks?task=${encodeURIComponent(taskId)}`}>{taskId}</Link>
            </div>
          ) : null}

          {error ? <div className="error-box">{error}</div> : null}
            </form>
          ) : null}

          {activeTool === "bookmarks" ? (
            <form className="tool-panel download-tool-panel" onSubmit={submitBookmarks}>
            <div className="panel-title">
              <Heart size={18} aria-hidden="true" />
              <h2>Bookmarks</h2>
            </div>

            <label>
              Limit
              <input
                min="1"
                type="number"
                value={bookmarkLimit}
                onChange={(event) => setBookmarkLimit(event.target.value)}
              />
            </label>

            <fieldset className="segmented">
              <legend>R18 policy</legend>
              {r18Policies.map((item) => (
                <button
                  aria-pressed={bookmarkPolicy === item.value}
                  className={bookmarkPolicy === item.value ? "active" : ""}
                  key={item.value}
                  onClick={() => setBookmarkPolicy(item.value)}
                  type="button"
                >
                  {item.label}
                </button>
              ))}
            </fieldset>

            <button className="button primary" disabled={bookmarkSubmitting} type="submit">
              {bookmarkSubmitting ? (
                <Loader2 className="spin" size={17} aria-hidden="true" />
              ) : (
                <Send size={17} aria-hidden="true" />
              )}
              Enqueue bookmarks
            </button>

            {bookmarkTaskId ? (
              <div className="success-box">
                <strong>Bookmark task queued</strong>
                <Link href={`/tasks?task=${encodeURIComponent(bookmarkTaskId)}`}>
                  {bookmarkTaskId}
                </Link>
              </div>
            ) : null}

            {bookmarkError ? <div className="error-box">{bookmarkError}</div> : null}
            </form>
          ) : null}

          {activeTool === "author" ? (
            <form className="tool-panel download-tool-panel" onSubmit={submitAuthor}>
            <div className="panel-title">
              <UserRound size={18} aria-hidden="true" />
              <h2>Author Batch</h2>
            </div>

            <label>
              Author UID
              <input
                inputMode="numeric"
                pattern="[0-9]*"
                placeholder="98765"
                value={authorUid}
                onChange={(event) => setAuthorUid(event.target.value)}
                required
              />
            </label>

            <label>
              Limit
              <input
                min="1"
                type="number"
                value={authorLimit}
                onChange={(event) => setAuthorLimit(event.target.value)}
              />
            </label>

            <fieldset className="segmented">
              <legend>R18 policy</legend>
              {r18Policies.map((item) => (
                <button
                  aria-pressed={authorPolicy === item.value}
                  className={authorPolicy === item.value ? "active" : ""}
                  key={item.value}
                  onClick={() => setAuthorPolicy(item.value)}
                  type="button"
                >
                  {item.label}
                </button>
              ))}
            </fieldset>

            <button className="button primary" disabled={authorSubmitting} type="submit">
              {authorSubmitting ? (
                <Loader2 className="spin" size={17} aria-hidden="true" />
              ) : (
                <Send size={17} aria-hidden="true" />
              )}
              Enqueue author batch
            </button>

            {authorTaskId ? (
              <div className="success-box">
                <strong>Author task queued</strong>
                <Link href={`/tasks?task=${encodeURIComponent(authorTaskId)}`}>
                  {authorTaskId}
                </Link>
              </div>
            ) : null}

            {authorError ? <div className="error-box">{authorError}</div> : null}
            </form>
          ) : null}

          {activeTool === "smart" ? (
            <form className="tool-panel download-tool-panel" onSubmit={submitSmartParse}>
            <div className="panel-title">
              <Sparkles size={18} aria-hidden="true" />
              <h2>Smart Retrieval</h2>
            </div>

            <label>
              Prompt
              <textarea
                placeholder="下载一些蓝色头发、赛博朋克风格的少女插画"
                value={smartPrompt}
                onChange={(event) => setSmartPrompt(event.target.value)}
              />
            </label>

            <label>
              Count
              <input
                min="1"
                type="number"
                value={smartCount}
                onChange={(event) => setSmartCount(event.target.value)}
              />
            </label>

            <fieldset className="segmented">
              <legend>R18 policy</legend>
              {r18Policies.map((item) => (
                <button
                  aria-pressed={smartPolicy === item.value}
                  className={smartPolicy === item.value ? "active" : ""}
                  key={item.value}
                  onClick={() => setSmartPolicy(item.value)}
                  type="button"
                >
                  {item.label}
                </button>
              ))}
            </fieldset>

            <button className="button primary" disabled={smartSubmitting} type="submit">
              {smartSubmitting ? (
                <Loader2 className="spin" size={17} aria-hidden="true" />
              ) : (
                <Sparkles size={17} aria-hidden="true" />
              )}
              Parse tags
            </button>

            <section className="smart-tag-editor" aria-label="Smart tag chip editor">
              <div>
                <strong>{smartPlan ? "Tag plan ready" : "Manual tag download"}</strong>
                <span>
                  Count {smartPlan?.count_recommend || smartCount || "default"} · {smartPolicy}
                  {smartPlan?.model ? ` · ${smartPlan.model}` : ""}
                </span>
              </div>
              <TagChipInput
                label="Tags"
                placeholder="blue hair"
                value={smartTagDraft}
                tags={smartTags}
                onAdd={(value) => {
                  setSmartTags((current) => mergeTags(current, tagTokens(value)));
                  setSmartTagDraft("");
                }}
                onChange={setSmartTagDraft}
                onRemove={(tag) =>
                  setSmartTags((current) => current.filter((item) => item !== tag))
                }
              />
              <TagChipInput
                label="Negative tags"
                placeholder="low quality"
                value={smartNegativeTagDraft}
                tags={smartNegativeTags}
                onAdd={(value) => {
                  setSmartNegativeTags((current) => mergeTags(current, tagTokens(value)));
                  setSmartNegativeTagDraft("");
                }}
                onChange={setSmartNegativeTagDraft}
                onRemove={(tag) =>
                  setSmartNegativeTags((current) => current.filter((item) => item !== tag))
                }
              />
              <button
                className="button secondary"
                disabled={smartDownloadSubmitting}
                onClick={submitSmartDownload}
                type="button"
              >
                {smartDownloadSubmitting ? (
                  <Loader2 className="spin" size={17} aria-hidden="true" />
                ) : (
                  <Send size={17} aria-hidden="true" />
                )}
                Enqueue smart download
              </button>
            </section>

            {smartTaskId ? (
              <div className="success-box">
                <strong>Smart task queued</strong>
                <Link href={`/tasks?task=${encodeURIComponent(smartTaskId)}`}>
                  {smartTaskId}
                </Link>
              </div>
            ) : null}

            {smartError ? <div className="error-box">{smartError}</div> : null}
            </form>
          ) : null}

          <div className="download-backlog-strip" aria-label="Pending download modes">
          {["Top10", "Random"].map((name) => (
            <section className="placeholder-row" key={name}>
              <strong>{name}</strong>
              <span>Backend mode pending</span>
            </section>
          ))}
          </div>
        </div>
      </section>
    </div>
  );
}

function tagTokens(value: string): string[] {
  return value
    .split(/[,\n]/)
    .map((tag) => tag.trim())
    .filter(Boolean);
}

function mergeTags(current: string[], incoming: string[]) {
  return Array.from(new Set([...current, ...incoming]));
}

function TagChipInput({
  label,
  placeholder,
  tags,
  value,
  onAdd,
  onChange,
  onRemove
}: {
  label: string;
  placeholder: string;
  tags: string[];
  value: string;
  onAdd: (value: string) => void;
  onChange: (value: string) => void;
  onRemove: (tag: string) => void;
}) {
  function handleKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key !== "Enter") return;
    event.preventDefault();
    onAdd(value);
  }

  return (
    <label className="tag-chip-field">
      {label}
      <div className="tag-chip-box">
        {tags.map((tag) => (
          <span className="tag-chip" key={tag}>
            {tag}
            <button aria-label={`Remove ${tag}`} onClick={() => onRemove(tag)} type="button">
              <X size={13} aria-hidden="true" />
            </button>
          </span>
        ))}
        <input
          placeholder={placeholder}
          value={value}
          onChange={(event) => onChange(event.target.value)}
          onKeyDown={handleKeyDown}
        />
        <button
          className="icon-button"
          onClick={() => onAdd(value)}
          type="button"
          aria-label={`Add ${label.toLowerCase()}`}
        >
          <Plus size={15} aria-hidden="true" />
        </button>
      </div>
    </label>
  );
}
