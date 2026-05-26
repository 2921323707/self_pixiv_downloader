# Tauri Desktop Architecture

## 模块职责

```text
pixiv_platform/
├─ src/frontend   Next.js / React UI
├─ src/backend    Rust Axum API, downloads, SQLite, local files
└─ tauri-app      Tauri desktop shell and packaging orchestration
```

`tauri-app` 只负责编排桌面运行时，不复制业务代码。

## MVP 运行模型

```text
Tauri dev
  ├─ binds Axum backend to a random 127.0.0.1 port
  ├─ injects runtime API base URL into the desktop WebView
  ├─ opens desktop WebView
  ├─ loads http://127.0.0.1:3001 from existing Next dev server
  ├─ references pixiv_platform_backend through Cargo path dependency
  └─ starts Axum backend inside a background Tokio runtime thread

Next frontend
  └─ fetches API through injected runtime base URL, env base URL, or /api fallback

Rust backend
  └─ serves Axum API on 127.0.0.1:<random-port>
```

## 后端接入

MVP 使用 Cargo path dependency：

```toml
pixiv_platform_backend = { path = "../../src/backend" }
```

这保留现有 HTTP API 和测试资产。第一阶段不把后端二进制作为外部子进程打包，
也不把 API 改写成 Tauri commands。

## 前端接入

开发模式继续使用现有 `src/frontend`。Tauri dev 窗口加载 Next dev server：

```text
http://127.0.0.1:3001
```

前端 API 层支持可选的 public backend base URL。没有配置时，保留 `/api`
相对路径行为，使 Web 版继续依赖 Next rewrites。

生产静态前端从 Tauri WebView 直接访问 Tauri 注入的运行时 API base URL。由于这不是
Next rewrites 同源代理，后端需要返回 CORS 响应头。MVP 后端统一 router 已允许
本地桌面静态页面跨源访问 API。

## 生产打包前端

`.app` MVP 生产模式采用 Next 静态导出作为优先方案。Tauri build 时加载
`src/frontend` 生成的静态资源，而不是在 `.app` 内额外启动 Next standalone/server。

该方案保持桌面壳职责简单：

- `tauri-app` 负责编排构建与加载静态资源。
- `src/frontend` 仍是唯一前端源码位置。
- `src/backend` 仍由 Tauri Rust 进程通过 path dependency 复用。

除非后续发现前端强依赖 Next server runtime，否则不引入 Node/Next server 进程。

## 数据目录

Web / 后端独立运行默认仍沿用项目 `output/`，保持既有开发体验。

Tauri 桌面端启动时使用桌面专属默认目录：

```text
~/Downloads/Pixiv Platform/
```

在没有显式设置 `PIXIV_DOWNLOAD_ROOT` 时，该目录作为下载根目录；在没有显式设置
`PIXIV_PLATFORM_DB_PATH` 时，SQLite 默认放在：

```text
~/Downloads/Pixiv Platform/pixiv_platform.sqlite3
```

Settings 页面中的下载目录配置通过桌面系统文件夹选择器选择。非 Tauri Web 环境仍可显示
并保存后端返回的路径值。

为避免既有用户升级后丢失配置或图库，Tauri 桌面端启动时会执行一次保守迁移：

- 如果新桌面 SQLite 不存在，直接从旧 `output/pixiv_platform.sqlite3` 复制。
- 如果新桌面 SQLite 已存在但没有 `pixiv_cookie` 且没有图库记录，而旧库包含用户数据，
  会先备份新空库，再用旧库恢复。
- 迁移时复制旧 `output/` 中除 SQLite 以外的本地下载文件到
  `~/Downloads/Pixiv Platform/`，并把 DB 中旧 `output` 绝对路径改写为新目录路径。

## 桌面启动诊断

桌面壳启动时会把关键启动事件追加到 macOS 日志文件：

```text
~/Library/Logs/Pixiv Platform/desktop.log
```

如果内嵌 Axum 后端启动失败，或 `GET /api/health` 在 8 秒内未变为健康，Tauri 会创建
启动失败窗口。该窗口显示失败原因和日志文件路径，方便用户在双击 `.app` 启动失败时获得
可见反馈。

## 桌面菜单

Tauri 桌面壳提供原生基础菜单。菜单只承载安全的窗口和 WebView 操作，不接入业务逻辑：

- 应用菜单：About / Services / Hide / Quit。
- File / Edit / Window：使用系统预定义的关闭、编辑和窗口操作。
- View：Reload；Developer Tools 仅 debug 构建显示；Fullscreen 使用系统预定义项。

## P3a 未签名 `.dmg` 分发模型

在没有 Apple Developer Program / Apple 开发者认证背景的当前阶段，macOS 分发只做未签名、
未公证 `.dmg` 的最小闭环：

- Tauri build 同时产出 `.app` 与 `.dmg`。
- `.dmg` 可上传到 GitHub Release，供小范围测试用户下载。
- 测试用户运行 `.dmg` 内的应用不需要安装 Rust、Cargo、Node、npm 或 TypeScript。
- 因为产物未签名、未公证，其他 macOS 用户首次打开时可能遇到 Gatekeeper 拦截，需要按
  macOS 安全提示手动允许。
- Apple 账号、证书、签名、公证和自动更新只作为未来正式分发路径评估，不写入仓库配置或文档
  secret。

## 运行时 API 地址

桌面端每次启动先绑定随机可用端口：

```text
127.0.0.1:<random-port>
```

随后 Tauri 用初始化脚本注入：

```text
window.__PIXIV_PLATFORM_BACKEND_URL__ = "http://127.0.0.1:<random-port>"
```

前端 `apiUrl()` 优先读取该运行时值；没有运行时值时再使用
`NEXT_PUBLIC_PIXIV_BACKEND_URL`；仍没有配置时回退 `/api`，保持 Web 开发兼容。
