# Tauri Desktop Architecture

## 模块职责

```text
pixiv_platform/
├─ src/frontend   Next.js / React UI
├─ src/backend    Rust Axum API, downloads, SQLite, local files
└─ tauri-app      Tauri desktop shell and packaging orchestration
```

`tauri-app` 只负责编排桌面运行时，不复制业务代码。

当前本地 checkout 默认构建 Windows NSIS 安装包；macOS `.app` / `.dmg` 源码分支仍保留。两个平台的安装包不是两套源码，都是同一个前端、后端和 Tauri 壳在不同系统上的构建产物。

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

桌面端生产模式采用 Next 静态导出作为优先方案。Tauri build 时加载
`src/frontend` 生成的静态资源，而不是在 `.app` 内额外启动 Next standalone/server。

该方案保持桌面壳职责简单：

- `tauri-app` 负责编排构建与加载静态资源。
- `src/frontend` 仍是唯一前端源码位置。
- `src/backend` 仍由 Tauri Rust 进程通过 path dependency 复用。

除非后续发现前端强依赖 Next server runtime，否则不引入 Node/Next server 进程。

当前 Windows 构建通过 `tauri-app/scripts/build-frontend-windows.cmd` 设置
`NEXT_OUTPUT_EXPORT=1` 并执行前端 build。macOS 构建需要使用 macOS shell 写法设置
同一环境变量，或使用独立 macOS Tauri config。

## 数据目录

Web / 后端独立运行和 Tauri 桌面端默认共享同一个本地目录：

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

旧项目 `output/` 不再作为默认目录，也不再由桌面端启动时自动迁移。需要保留旧数据时应在清理前手动备份或显式配置 `PIXIV_DOWNLOAD_ROOT` / `PIXIV_PLATFORM_DB_PATH`。

## 桌面启动诊断

桌面壳启动时会把关键启动事件追加到平台日志文件：

```text
macOS:   ~/Library/Logs/Pixiv Platform/desktop.log
Windows: %LOCALAPPDATA%\Pixiv Platform\desktop.log
```

如果内嵌 Axum 后端启动失败，或 `GET /api/health` 在 8 秒内未变为健康，Tauri 会创建
启动失败窗口。该窗口显示失败原因和日志文件路径，方便用户在双击 `.app` 启动失败时获得
可见反馈。

## 桌面菜单

Tauri 桌面壳提供原生基础菜单。菜单只承载安全的窗口和 WebView 操作，不接入业务逻辑：

- 应用菜单：About / Services / Hide / Quit。
- File / Edit / Window：使用系统预定义的关闭、编辑和窗口操作。
- View：Reload；Developer Tools 仅 debug 构建显示；Fullscreen 使用系统预定义项。

## Pixiv 登录态刷新

当前 Pixiv 下载链路依赖后端运行时设置中的 `pixiv_cookie`，其值为 Pixiv Web 登录态
`PHPSESSID`。为降低用户定期手动获取 cookie 的成本，桌面端已实现内置 Pixiv 登录窗口：

```text
Settings Pixiv Login/Refresh
  ├─ frontend invokes a Tauri command
  ├─ Tauri opens a separate WebViewWindow at https://www.pixiv.net/
  ├─ user signs in on Pixiv's official page
  ├─ Tauri polls the login window cookie store
  ├─ finds PHPSESSID from the Pixiv domain
  ├─ returns the cookie value to Settings
  ├─ frontend saves it through the existing PUT /api/settings/pixiv_cookie path
  ├─ frontend runs POST /api/settings/test/pixiv for validation
  ├─ Tauri closes the Pixiv login window after the cookie is captured
  └─ Settings shows a non-sensitive success alert
```

设计约束：

- 不采集 Pixiv 账号密码；登录发生在 Pixiv 官方页面。
- 不把完整 `PHPSESSID` 写入日志、文档或测试输出。
- 不绕过现有 settings repository；保存后仍由 `pixiv_cookie` secret masking 保护。
- 手动输入 `pixiv_cookie` 继续保留为 fallback。
- Web 端浏览器页面不能直接跨域读取 Pixiv cookie；该能力仅面向 Tauri 桌面端。
- 获取成功后自动关闭 Pixiv 登录窗口；提示只说明刷新成功，不显示 cookie 值。

2026-05-26 小规模验证结论：

- 当前 `tauri 2.11.2` 支持 `WebviewWindow.cookies()` / `cookies_for_url()`。
- 本地 Tauri 探针确认 WebView 响应写入的 HttpOnly `PHPSESSID` 可被 Rust 侧读取。
- 本地探针中 `cookies_for_url()` 对 `127.0.0.1` 返回为空，但 `cookies()` 能读到完整 store。
  正式实现优先使用 `cookies()`，再按 `name == "PHPSESSID"` 和 Pixiv 域过滤。

2026-05-26 正式实现与 live 验证结论：

- Tauri command `refresh_pixiv_phpsessid` 已实现并加入 capability permission。
- Settings 中 `pixiv_cookie` 行在 Tauri 桌面端显示 Refresh；非 Tauri 环境保留手动输入和 Test。
- 用户手动在官方 Pixiv 登录窗口登录后，应用成功刷新 `pixiv_cookie`、自动执行 Pixiv Test、
  自动关闭登录窗口，并在 Settings 主窗口弹出成功提示。

## 桌面分发模型

Windows 当前默认构建：

- Tauri build 产出 release `.exe` 和 NSIS installer。
- 当前产物路径：`tauri-app/src-tauri/target/release/bundle/nsis/Pixiv Platform_1.2.0_x64-setup.exe`。
- 用户手动确认 Windows Web、Windows Tauri App 和 Settings Pixiv Refresh 弹窗正常。

macOS 历史 release 构建：

在没有 Apple Developer Program / Apple 开发者认证背景的当前阶段，macOS 分发只做 ad-hoc signed、
未公证 `.dmg` 的最小闭环：

- Tauri build 同时产出 `.app` 与 `.dmg`。
- `.app` bundle 显式使用 `bundle.macOS.signingIdentity = "-"` 进行 ad-hoc signing，确保 bundle 有完整 sealed resources。
- `.dmg` 可上传到 GitHub Release，供小范围测试用户下载。
- 测试用户运行 `.dmg` 内的应用不需要安装 Rust、Cargo、Node、npm 或 TypeScript。
- 因为产物未 Developer ID 签名、未公证，其他 macOS 用户首次打开时可能遇到 Gatekeeper 拦截，需要按
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
