export type TaskLog = {
  log_id: string;
  level: string;
  phase: string;
  message: string;
  context: unknown;
  created_at: string;
};

export type TaskItem = {
  item_id: string;
  pixiv_id: string | null;
  page_index: number | null;
  status: string;
  image_id: string | null;
  error_code: string | null;
  error_message: string | null;
  created_at: string;
  updated_at: string;
};

export type TaskSnapshot = {
  task_id: string;
  type: string;
  status: string;
  progress_total: number | null;
  progress_done: number;
  progress_failed: number;
  current_item: string | null;
  error_code: string | null;
  error_message: string | null;
  created_at: string;
  started_at: string | null;
  finished_at: string | null;
  updated_at: string;
  items: TaskItem[];
  logs: TaskLog[];
};

export type TaskSummary = Omit<TaskSnapshot, "items" | "logs">;

export type TaskListResult = {
  items: TaskSummary[];
  next_cursor: string | null;
};

export type SingleDownloadResult = {
  task_id: string;
  image_id: string | null;
  download_status: string;
};

export type BatchDownloadResult = {
  task_id: string;
  download_status: string;
};

export type GalleryImage = {
  image_id: string;
  pixiv_id: string;
  page_index: number;
  title: string | null;
  author_uid: string | null;
  tags: string[];
  category: string;
  thumbnail_url: string | null;
  preview_url: string | null;
  width: number | null;
  height: number | null;
  downloaded_at: string;
  created_at: string;
};

export type GalleryImageSource = {
  source: string;
  task_id: string | null;
  created_at: string;
};

export type GalleryImageDetail = GalleryImage & {
  sources: GalleryImageSource[];
  map_x: number | null;
  map_y: number | null;
  updated_at: string;
};

export type GalleryResult = {
  items: GalleryImage[];
  next_cursor: string | null;
};

export type ImageDeleteItem = {
  image_id: string;
  status: string;
  pixiv_id: string | null;
  page_index: number | null;
  file_deleted: boolean;
  file_missing: boolean;
  error_code: string | null;
  error_message: string | null;
};

export type ImageDeleteResult = {
  items: ImageDeleteItem[];
  deleted_count: number;
  failed_count: number;
};

export type PublicSetting = {
  key: string;
  value: unknown;
  is_secret: boolean;
  updated_at: string;
};

export type SettingsResult = {
  items: PublicSetting[];
};

export type PixivConnectionTestResult = {
  configured: boolean;
  status: string;
  pixiv_id: string | null;
  title: string | null;
  user_uid: string | null;
  user_name: string | null;
  bound: boolean;
};

export type DeepSeekConnectionTestResult = {
  configured: boolean;
  status: string;
  model: string;
};

export type RuntimeReadinessAction = {
  label: string;
  href: string | null;
  action: string | null;
};

export type RuntimeReadinessCheck = {
  ok: boolean;
  status: string;
  message: string;
  recommendation: string | null;
  action: RuntimeReadinessAction | null;
  error_code: string | null;
  latency_ms: number | null;
};

export type PixivAccount = {
  user_uid: string;
  user_name: string | null;
  is_active: boolean;
  last_verified_at: string;
  created_at: string;
  updated_at: string;
};

export type PixivAccountsResult = {
  items: PixivAccount[];
  active: PixivAccount | null;
};

export type PixivAccountDeleteResult = {
  deleted: boolean;
};

export type RuntimePixivAccountReadiness = RuntimeReadinessCheck & {
  account: PixivAccount | null;
};

export type RuntimeReadinessResult = {
  backend: RuntimeReadinessCheck;
  pixiv_network: RuntimeReadinessCheck;
  pixiv_account: RuntimePixivAccountReadiness;
  deepseek: RuntimeReadinessCheck;
};

export type SmartParseResult = {
  tags: string[];
  negative_tags: string[];
  count_recommend: number;
  r18_policy: string;
  confidence: number;
  model: string;
};

type ApiEnvelope<T> = {
  data: T;
};

type ApiErrorEnvelope = {
  error: {
    code: string;
    message: string;
    details: unknown;
  };
};

declare global {
  interface Window {
    __PIXIV_PLATFORM_BACKEND_URL__?: string;
    __TAURI_INTERNALS__?: {
      invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
    };
    __TAURI__?: {
      invoke?<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
      core?: {
        invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
      };
      tauri?: {
        invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
      };
    };
  }
}

export type TauriInvoke = <T>(
  cmd: string,
  args?: Record<string, unknown>,
) => Promise<T>;

export function getTauriInvoke(): TauriInvoke | null {
  if (typeof window === "undefined") return null;

  const candidates: Array<[unknown, unknown]> = [
    [window.__TAURI_INTERNALS__, window.__TAURI_INTERNALS__?.invoke],
    [window.__TAURI__?.core, window.__TAURI__?.core?.invoke],
    [window.__TAURI__?.tauri, window.__TAURI__?.tauri?.invoke],
    [window.__TAURI__, window.__TAURI__?.invoke]
  ];

  for (const [owner, invoke] of candidates) {
    if (typeof invoke === "function") {
      return invoke.bind(owner) as TauriInvoke;
    }
  }

  return null;
}

export function isTauriDesktopRuntime(): boolean {
  if (typeof window === "undefined") return false;
  return Boolean(getTauriInvoke() || window.__PIXIV_PLATFORM_BACKEND_URL__);
}

export function apiUrl(path: string): string {
  const runtimeBaseUrl =
    typeof window === "undefined" ? undefined : window.__PIXIV_PLATFORM_BACKEND_URL__;
  const baseUrl = (runtimeBaseUrl || process.env.NEXT_PUBLIC_PIXIV_BACKEND_URL)?.replace(
    /\/+$/,
    "",
  );
  return baseUrl ? `${baseUrl}${path}` : path;
}

export async function startSingleDownload(input: {
  pixiv_id: string;
  page_index?: number;
  r18_policy: string;
}): Promise<SingleDownloadResult> {
  const response = await fetch(apiUrl("/api/download/single"), {
    method: "POST",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify(input)
  });

  return unwrap<SingleDownloadResult>(response);
}

export async function startAuthorDownload(input: {
  author_uid: string;
  limit?: number;
  r18_policy?: string;
}): Promise<BatchDownloadResult> {
  const response = await fetch(apiUrl("/api/downloads/author"), {
    method: "POST",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify(input)
  });

  return unwrap<BatchDownloadResult>(response);
}

export async function startBookmarkDownload(input: {
  limit?: number;
  r18_policy?: string;
}): Promise<BatchDownloadResult> {
  const response = await fetch(apiUrl("/api/downloads/bookmarks"), {
    method: "POST",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify(input)
  });

  return unwrap<BatchDownloadResult>(response);
}

export async function parseSmartPrompt(input: {
  prompt: string;
  count?: number;
  r18_policy?: string;
}): Promise<SmartParseResult> {
  const response = await fetch(apiUrl("/api/smart/parse"), {
    method: "POST",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify(input)
  });

  return unwrap<SmartParseResult>(response);
}

export async function startSmartDownload(input: {
  prompt: string;
  tags: string[];
  negative_tags?: string[];
  count?: number;
  r18_policy?: string;
  model?: string;
}): Promise<BatchDownloadResult> {
  const response = await fetch(apiUrl("/api/smart/download"), {
    method: "POST",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify(input)
  });

  return unwrap<BatchDownloadResult>(response);
}

export async function fetchTask(taskId: string): Promise<TaskSnapshot> {
  const response = await fetch(apiUrl(`/api/tasks/${encodeURIComponent(taskId)}`), {
    cache: "no-store"
  });

  return unwrap<TaskSnapshot>(response);
}

export async function fetchTasks(input: {
  status?: string;
  type?: string;
  limit?: number;
  cursor?: string | null;
} = {}): Promise<TaskListResult> {
  const params = new URLSearchParams();
  if (input.status) params.set("status", input.status);
  if (input.type) params.set("type", input.type);
  if (input.limit) params.set("limit", String(input.limit));
  if (input.cursor) params.set("cursor", input.cursor);

  const response = await fetch(apiUrl(`/api/tasks?${params.toString()}`), {
    cache: "no-store"
  });

  return unwrap<TaskListResult>(response);
}

export async function fetchImages(input: {
  tag?: string;
  category?: string;
  source?: string;
  r18_visibility?: string;
  limit?: number;
  cursor?: string | null;
} = {}): Promise<GalleryResult> {
  const params = new URLSearchParams();
  if (input.tag) params.set("tag", input.tag);
  if (input.category) params.set("category", input.category);
  if (input.source) params.set("source", input.source);
  if (input.r18_visibility) params.set("r18_visibility", input.r18_visibility);
  if (input.limit) params.set("limit", String(input.limit));
  if (input.cursor) params.set("cursor", input.cursor);

  const response = await fetch(apiUrl(`/api/images?${params.toString()}`), {
    cache: "no-store"
  });

  return unwrap<GalleryResult>(response);
}

export async function fetchImage(imageId: string): Promise<GalleryImageDetail> {
  const response = await fetch(apiUrl(`/api/images/${encodeURIComponent(imageId)}`), {
    cache: "no-store"
  });

  return unwrap<GalleryImageDetail>(response);
}

export async function deleteImages(imageIds: string[]): Promise<ImageDeleteResult> {
  const response = await fetch(apiUrl("/api/images/delete-batch"), {
    method: "POST",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify({ image_ids: imageIds })
  });

  return unwrap<ImageDeleteResult>(response);
}

export async function fetchSettings(): Promise<SettingsResult> {
  const response = await fetch(apiUrl("/api/settings"), {
    cache: "no-store"
  });

  return unwrap<SettingsResult>(response);
}

export async function fetchRuntimeReadiness(): Promise<RuntimeReadinessResult> {
  const response = await fetch(apiUrl("/api/runtime/readiness"), {
    cache: "no-store"
  });

  return unwrap<RuntimeReadinessResult>(response);
}

export async function saveSetting(key: string, value: unknown): Promise<PublicSetting> {
  const response = await fetch(apiUrl(`/api/settings/${encodeURIComponent(key)}`), {
    method: "PUT",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify({ value })
  });

  return unwrap<PublicSetting>(response);
}

export async function testPixivConnection(pixivId?: string): Promise<PixivConnectionTestResult> {
  const response = await fetch(apiUrl("/api/settings/test/pixiv"), {
    method: "POST",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify({ pixiv_id: pixivId || null })
  });

  return unwrap<PixivConnectionTestResult>(response);
}

export async function testDeepSeekConnection(): Promise<DeepSeekConnectionTestResult> {
  const response = await fetch(apiUrl("/api/settings/test/deepseek"), {
    method: "POST"
  });

  return unwrap<DeepSeekConnectionTestResult>(response);
}

export async function fetchPixivAccounts(): Promise<PixivAccountsResult> {
  const response = await fetch(apiUrl("/api/pixiv/accounts"), {
    cache: "no-store"
  });

  return unwrap<PixivAccountsResult>(response);
}

export async function activatePixivAccount(userUid: string): Promise<PixivAccount> {
  const response = await fetch(apiUrl("/api/pixiv/accounts/active"), {
    method: "POST",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify({ user_uid: userUid })
  });

  return unwrap<PixivAccount>(response);
}

export async function deletePixivAccount(userUid: string): Promise<PixivAccountDeleteResult> {
  const response = await fetch(apiUrl(`/api/pixiv/accounts/${encodeURIComponent(userUid)}`), {
    method: "DELETE"
  });

  return unwrap<PixivAccountDeleteResult>(response);
}

async function unwrap<T>(response: Response): Promise<T> {
  const raw = await response.text();
  if (!raw.trim()) {
    throw new Error(
      response.ok ? "API returned an empty response" : response.statusText || "Request failed"
    );
  }

  let payload: ApiEnvelope<T> | ApiErrorEnvelope;
  try {
    payload = JSON.parse(raw) as ApiEnvelope<T> | ApiErrorEnvelope;
  } catch {
    const fallback = raw.trim().slice(0, 280) || response.statusText || "Request failed";
    throw new Error(response.ok ? "API returned invalid JSON" : fallback);
  }

  if (!response.ok || "error" in payload) {
    const message = "error" in payload ? payload.error.message : response.statusText;
    throw new Error(message || "Request failed");
  }

  return payload.data;
}
