# 上下文接续说明

用途：当 Codex 上下文重置、切换模型、或者很久以后重新打开项目时，用这份文件快速恢复对项目的掌控。

## 一句话项目状态

这是一个本地优先的 Pixiv AI 下载与智能检索平台。当前采用 downloader-first 路线，已跑通真实 Pixiv 单作品下载，并已完成 SQLite migration、图片仓储、settings 仓储、DB-aware downloader、任务状态持久化、确定性测试脚本结构、薄 Axum API wrapper、in-process Tokio 后台任务队列、Next.js 前端工作台、Gallery/Settings/Tasks/Home 真实数据接线、作者批量下载、收藏批量下载、DeepSeek 智能解析、smart 标签搜索批量下载、UI polish、Tauri 桌面壳、Web/App 默认共享下载目录、Gallery 预览稳定性修复、Pixiv 内置登录刷新 `PHPSESSID`、macOS ad-hoc signed `.app` / `.dmg` 分发验证，以及 Windows Web / Tauri NSIS 安装包适配。GitHub `v1.1.1` 是成熟 macOS 交付锚点；当前本地 checkout 已提升到 `v1.2.0` Windows 桌面发布锚点。

## 当前阶段

当前阶段：**v1.2.0 Windows Desktop Release**。后续以缺陷修复、安装体验、小范围分发反馈和可选功能演进为主。

关键锚点：2026-05-22 用户确认前端单作品、Author Batch、Bookmarks Batch 可用。2026-05-23 Smart Retrieval、Home Dashboard、Phase 7B UI polish 完成。2026-05-26 修复 Tauri Gallery 偶发预览空白，原因是 macOS WebView 内并发加载多张原图；修复为错峰/lazy/async decode/失败重试，并给图片文件响应补 `Content-Length`。2026-05-26 至 2026-05-27 完成 Pixiv 内置登录刷新 `PHPSESSID`、`.dmg` 损坏提示复查、macOS ad-hoc signing、codesign 和 hdiutil 验证。当前策略：以 `v1.1.1` 为成熟交付基线，文档采用“顶部锚点覆写 + 关键 debug 信息保留 + 阶段结束压缩”。

最新维护锚点：2026-05-27 已完成 `src/backend/src/api.rs` 拆分，现为 `src/backend/src/api/` 模块树；外部 `api::{AppState, router, serve, serve_listener}` 入口保持兼容。同日重新运行 `./tests/unit/backend_unit.sh`、`./tests/run_local.sh`、`cd tauri-app && npm run build`、`codesign --verify --deep --strict` 和 `hdiutil verify` 均通过。Live Pixiv E2E 因当前 shell 未提供 `PIXIV_PHPSESSID` 未运行，仍保持 opt-in。

Windows 发布锚点：2026-05-27 在 Windows 本地确认 Web 和 Tauri App 正常。已安装 Visual Studio 2022 Build Tools / MSVC C++ linker；后端修复 Windows 绝对路径和 `USERPROFILE` home fallback；Settings Pixiv Refresh 已修复 Tauri invoke bridge 判断并经用户手动确认可弹出 Pixiv 登录窗口；`cargo test` 86 tests passed；前端 `npm.cmd run lint` passed；Web 联通验证 `/api/health` ok 且首页 HTTP 200；`cd tauri-app && npm.cmd run build` 生成 `tauri-app/src-tauri/target/release/bundle/nsis/Pixiv Platform_1.2.0_x64-setup.exe`。当前 `tauri.conf.json` 默认构建目标为 Windows NSIS。

macOS 兼容边界：`tauri-app/src-tauri/src/main.rs` 中 macOS 文件夹选择、日志路径、Pixiv 登录窗口等源码分支仍保留；`.exe` 和 `.dmg` 不是两套源码。当前不兼容的是默认构建配置：Windows 适配把 `beforeBuildCommand` 和 `bundle.targets` 切到 Windows。要在 macOS 再构建 `.app` / `.dmg`，需要把 Tauri 构建命令/targets 切回 macOS，或新增 macOS 专用 config。

## 当前真实边界

后端：当前 single-download -> task -> indexed metadata -> Gallery preview/delete、author batch -> multi-item task -> DB-aware downloads、bookmark batch -> multi-item task -> DB-aware downloads、natural language -> DeepSeek -> tag plan preview -> Pixiv tag search -> smart batch task 已经可用。对 v1.0.0 而言，这是完整的 downloader-first 后端边界；Top10 / Random / thumbnail cache / cancel-retry / image edit-map 属于后续进化，不再阻塞 v1.0.0。

已完成的是：

- 单作品下载核心链路
- DB-aware downloader
- 图片 / tag / source 持久化
- task / task_items / task_logs 持久化
- `POST /api/download/single`
- `POST /api/downloads/bookmarks`
- `POST /api/downloads/author`
- `GET /api/tasks/{task_id}`
- `GET /api/tasks`
- `GET /api/images`
- `GET /api/images/{image_id}`
- `GET /api/images/{image_id}/file`，响应带 `Content-Length`
- `DELETE /api/images/{image_id}`
- `POST /api/images/delete-batch`
- `GET /api/settings`
- `PUT /api/settings/{key}`
- `POST /api/settings/test/pixiv`
- `POST /api/settings/test/deepseek`
- `POST /api/smart/parse`
- `POST /api/smart/download`
- 后台队列 worker
- Pixiv tag search through `search_works_by_tags`
- `smart_retrievals` provenance persistence
- settings-backed `pixiv_cookie` / `download_base_path` runtime resolution for single-download tasks
- `default_batch_count` 默认批量数量设置，默认 `20`
- default download root resolves to `~/Downloads/Pixiv Platform/` for both Web/backend standalone and macOS App

v1.x / v2 候选，不属于 v1.0.0 阻塞：

- 生成缩略图缓存 API
- top10 / random 批量下载
- 更完整的图库筛选、编辑与地图 API

前端：当前是可运行的 v1.0.0 工作台 UI，不是终局型全产品 UI。

- Download 页已真实对接单作品下载 API
- Download 页已真实对接 Bookmarks 批量下载 API
- Download 页已真实对接 Author 批量下载 API
- Download 页已真实对接 Smart Retrieval 解析 API，并可编辑解析出的 tags / negative tags 后入队 smart 批量下载任务
- Download 页 Smart tab 已支持不调用 DeepSeek 的手动正/负 tag chip 输入，仍复用 `POST /api/smart/download`
- Download 页已从左大栏/右堆叠改为 Single / Author / Bookmarks / Smart 顶部 tabs 工作台
- Download 页发起的单作品任务会使用 Settings 保存的 Pixiv cookie 和 download_base_path
- Tasks 页已真实对接 task-id 轮询 API 和任务列表 API，Recent Tasks 默认 10 条，可展开更多，点击后用 modal 动态加载进度/items/logs
- Gallery 页已真实对接图片 metadata 列表 API，可通过安全 file endpoint 显示本地图片预览；Tauri 桌面列表预览已错峰/lazy/async decode 并支持失败重试；点击图片打开右侧详情 drawer，drawer 支持关闭按钮、遮罩和 ESC 关闭，并支持多选删除本地文件和 SQLite 索引
- Settings 页已真实对接 public settings list/save API、Pixiv test、DeepSeek test，secret 显示保持 masked，并按通用/外观/Pixiv/DeepSeek/Storage 分类展示
- Home 页已真实复用 `GET /api/tasks`、`GET /api/images`、`GET /api/settings` 展示最近 3 条任务、状态摘要、优先 normal 且横向候选的最近图片 banner、快速入口、配置状态、本地图库提示、Rust 核心驱动注解、性能观察和后续能力槽
- Top10 / Random 仍未完成，但已从 v1.0.0 阻塞项转为后续 discovery modes

已经完成：

1. 增加 Axum 依赖和最小 backend server 入口
2. 定义 app state 与单作品下载/task polling DTO
3. 实现 `POST /api/download/single`
4. 实现 `GET /api/tasks/{task_id}`
5. 增加 API smoke/integration 测试
6. 首页从占位改为真实 dashboard，API 层未新增后端端点
7. UI polish 第一版改造 Home / Download / Tasks / Gallery / Settings，API 层未新增后端端点
8. UI formatting follow-up 修补 Home banner / panel spacing、Home command center、Smart tag chips、API client empty response guard、drawer/modal 可关闭性和移动端间距，API 层未新增后端端点
9. v1.0.0 downloader-first final 状态确认：当前稳定闭环可作为正式版本，后续进入 v1.x / v2 进化方向讨论

交付后维护策略：

1. `main` 作为 v1.0.0 final delivery 主分支，优先接受文档、安装、验证、缺陷修复和小范围稳定性改动。
2. Gallery Quality / Thumbnail Cache、Top10 / Random discovery、task cancel/retry、图片编辑/map、语义检索等不再作为 v1.0.0 缺口，只作为交付后的可选演进。
3. 如开启新能力，先定义最小可验证切片，再补 specs、traceability、tests。
4. 继续保持 API 层薄封装，不复制 `tasks` / `downloads` / `images` 仓储逻辑。

## 必读文件顺序

如果上下文重置，请按这个顺序读：

1. `README.md`
2. `docs/DOCUMENT_MAP.md`
3. `docs/progress.md`
4. `docs/specs/implementation-plan.md`
5. `docs/specs/traceability.md`
6. 当前要改的代码模块

## 当前关键代码

| 作用 | 路径 |
| --- | --- |
| 下载编排 | `src/backend/src/downloads/mod.rs` |
| Pixiv client | `src/backend/src/pixiv/mod.rs` |
| 文件写入/路径规划 | `src/backend/src/storage/mod.rs` |
| DB 初始化/迁移 | `src/backend/src/db/mod.rs` |
| 图片仓储 | `src/backend/src/images/mod.rs` |
| Settings 仓储 | `src/backend/src/settings/mod.rs` |
| 任务仓储 / worker 包装 | `src/backend/src/tasks/mod.rs` |
| Axum API wrapper | `src/backend/src/api/` |
| API routes / server helpers | `src/backend/src/api/routes.rs` |
| API DTO / envelope / errors | `src/backend/src/api/dto.rs`, `src/backend/src/api/error.rs` |
| API runtime settings / queue worker | `src/backend/src/api/runtime.rs`, `src/backend/src/api/worker.rs` |
| API handlers | `src/backend/src/api/handlers/` |
| DeepSeek / smart parse | `src/backend/src/ai.rs` |
| Pixiv tag search | `src/backend/src/pixiv/mod.rs` |
| 后端 server 入口 | `src/backend/src/bin/server.rs` |
| 前端 scaffold | `src/frontend` |
| Tauri 桌面壳 | `tauri-app/src-tauri` |
| Windows Web 启动脚本 | `tools/dev_backend_windows.ps1`, `tools/dev_frontend_windows.ps1` |
| Windows 前端静态导出脚本 | `tauri-app/scripts/build-frontend-windows.cmd` |

## 当前测试基线

默认质量门：

```text
./tests/run_local.sh
```

当前结果：

```text
86 backend unit tests passed; Phase 2A checks passed; Phase 2C checks passed; backend SQLite integration checks passed; backend API smoke checks passed; Phase 3B queue checks passed; Phase 4B data API checks passed; Phase 4C configured download checks passed; Phase 4D gallery file API checks passed; Phase 4E gallery delete checks passed; Phase 5A author batch checks passed; Phase 5B bookmark batch checks passed; Phase 6A smart parse checks passed; Phase 6B smart download checks passed; frontend scaffold checks passed; 0 failed
```

最新 macOS 桌面构建验证：

```text
cd tauri-app && npm run build
codesign --verify --deep --strict --verbose=2 "tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app"
hdiutil verify "tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg"
```

当前产物：`tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`，本地大小约 8.2M；`.app` ad-hoc signed 且 codesign 校验通过；`.dmg` checksum valid。

最新 Windows 桌面构建验证：

```text
cd src/backend && cargo test
cd src/frontend && npm.cmd run lint
cd tauri-app && npm.cmd run build
```

当前产物：`tauri-app/src-tauri/target/release/bundle/nsis/Pixiv Platform_1.2.0_x64-setup.exe`。用户已手动确认 Web 正常、Windows App 正常，Pixiv Refresh 弹窗正常。

真实 Pixiv E2E：

```text
PIXIV_PHPSESSID=... ./tests/e2e/live_single_download.sh
```

注意：不要把 Pixiv cookie 写进仓库。
最新状态：2026-05-27 release verification shell 未设置 `PIXIV_PHPSESSID`，所以未运行 live E2E。

## 已验证事实

- 真实 Pixiv 作品 `144920810` 可以通过当前 client 下载
- DB-aware downloader 已覆盖首次下载、DB duplicate skip、missing-file repair、existing-file indexing
- 任务状态持久化已覆盖 task/items/logs、状态流转、成功完成和失败诊断
- Phase 2D 测试脚本结构已完成：`phase2c_tasks.sh`、`backend_sqlite.sh`、expanded `run_local.sh`
- Phase 3A API wrapper 已完成：`POST /api/download/single`、`GET /api/tasks/{task_id}`、`tests/smoke/backend_api.sh`
- Phase 3B 后台任务队列已完成：`POST /api/download/single` enqueue 后快速返回，worker 后台执行，`tests/stage/phase3b_queue.sh`
- Phase 4A 前端雏形已完成：Next.js app shell，Home/Download/Tasks/Gallery/Settings，下载页和任务页对接现有 API
- Phase 4B 最小数据接线已完成：gallery/settings/task-list API，前端 Gallery/Settings/Tasks 切到真实数据
- Phase 4C 前端配置下载闭环已完成：Settings 保存的 `pixiv_cookie` 和 `download_base_path` 会驱动后续单作品下载，且 Pixiv test API 不回显 secret
- Phase 4D Gallery 文件预览闭环已完成：`GET /api/images/{image_id}/file` 安全返回本地图片 bytes，Gallery 卡片和详情使用真实预览 URL
- Phase 4E Gallery 删除闭环已完成：`DELETE /api/images/{image_id}`、`POST /api/images/delete-batch` 会安全删除本地文件并清理 SQLite 索引
- Phase 5A Author 批量下载闭环已完成：`POST /api/downloads/author`、作者发现、批量 task_items、顺序 worker、部分失败诊断、Download 作者表单
- Phase 5B Bookmarks 批量下载闭环已完成：`POST /api/downloads/bookmarks`、当前用户收藏发现、批量 task_items、顺序 worker、部分失败诊断、Download 收藏表单
- Phase 6A Smart Retrieval 解析闭环已完成：`POST /api/smart/parse`、`POST /api/settings/test/deepseek`、`deepseek_model=deepseek-v4-flash`、Download 智能解析预览
- Phase 6B Smart Retrieval 下载闭环已完成：`POST /api/smart/download`、Pixiv 标签搜索、`smart` task worker、`smart_retrievals` provenance、Download 智能下载入队
- Phase 7A Home Dashboard 首页真实化已完成：真实 task/image/settings 工作台，不暴露 secret
- Phase 7B UI polish 第一版已完成：Download tabs、Gallery drawer、Tasks modal、Settings 分类、Home normal banner
- 2026-05-26 Tauri Gallery 预览稳定性修复已完成：列表预览不再瞬间并发解码全部原图，图片文件端点返回 `Content-Length`
- 2026-05-22 用户手动浏览器验证已通过：前端输入 Pixiv 作品 ID 后下载成功
- 2026-05-22 用户手动浏览器验证已通过：Author Batch 下载成功
- 2026-05-22 用户手动浏览器验证已通过：Bookmarks Batch 下载成功
- Phase 3B 清理检查已完成：仅生成候选清单，没有删除文件
- 2026-05-27 用户确认清理已执行：构建缓存删除，旧项目 `output/` 删除，未引用静态资源删除，主题 demo 移至 `docs/design/theme-reference/`
- 2026-05-27 默认存储路径统一：Web/backend standalone 与 macOS App 默认共享 `~/Downloads/Pixiv Platform/`，旧 `output/` 自动迁移逻辑已删除
- 2026-05-27 Windows v1.2.0 发布锚点完成：Web 本地启动正常，Windows Tauri App 手动验证正常，Pixiv Refresh 弹窗正常，NSIS 安装包构建成功
- `.exe` / `.dmg` 共享同一套源码：`src/frontend`、`src/backend`、`tauri-app/src-tauri`；差异只在构建配置和少量 `cfg(target_os)` 系统能力分支
- 默认主题已选 `cyan-studio`
- 当前仓库保留 `docs/design/theme-reference/demo_B.html` 作为视觉参考
- `src/backend/target/` 是构建产物，已忽略
- `Cargo.lock` 应保留，用于应用级 Rust 项目可复现构建

## 重要约束

- 不要绕过 spec-coding 状态文档推进大改
- 不要自动删除代码文件
- 任何清理都先生成清单并说明原因
- secret 只能运行时提供，不写入文档、代码或测试 fixture
- 新测试尽量带 `REQ-*` 编号

## 新会话低 Token 提示词

复制下面这段给新会话。它会让模型先读少量状态文件，再按目标只打开相关代码，避免全局浏览。

```text
你在 /Users/Admin/Downloads/pixiv_platform 继续 Pixiv AI 下载与智能检索平台。

请节省 token，不要全局浏览仓库。先只阅读：
1. README.md 的“交付定位/交付范围/分支与发布策略”
2. docs/CONTEXT_HANDOFF.md 的“当前阶段/当前真实边界/下一步候选/重要约束”
3. docs/progress.md 的“Current Anchor/Delivery Status”
4. docs/specs/traceability.md 里与本次任务相关的 REQ 行

当前锚点：
- 当前项目已 release 为 v1.1.1 Mature First Delivery。
- 单图下载、Author Batch、Bookmarks Batch、Smart Retrieval Parse -> 编辑标签/chips -> Enqueue smart download 均已可用并经用户手动检查或确定性检查。
- Home、Download、Tasks、Gallery、Settings 已接入真实 API；Home 是 command center，包含最近 3 条任务、normal 横向候选 banner、Rust Core Driver 注解、Performance Watch 和后续能力槽。
- 当前 UI polish：Download tabs 工作台、Gallery 右侧详情 drawer、Tasks 详情 modal 与展开更多、Settings 分类面板、Home command center。
- 后端 downloader-first 核心和 macOS 桌面端交付链路稳定；Top10/Random、缩略图缓存、任务 cancel/retry、图片编辑/map、语义检索等已经转为 v1.x / v2 进化方向，不再阻塞 v1.1.1。
- 默认下载目录是 `~/Downloads/Pixiv Platform/`，secret 只允许运行时配置，禁止写入 Pixiv cookie 或 DeepSeek key。
- live Pixiv / live LLM 测试保持 opt-in。

本轮目标：围绕 v1.0.0 之后的进化与优化方向做方案讨论，优先选择一个高技术复杂度、产品收益明显、符合本地优先路线的功能方向。

候选方向可以包括但不限于：
- Gallery Quality / Thumbnail Cache：缩略图缓存、懒加载、批量图片浏览性能。
- Discovery Modes：Top10 / Random，复用现有 batch task substrate。
- Task Control：cancel/retry、失败重试、worker 诊断。
- Semantic Retrieval：本地图像语义索引、CLIP/vector search、相似图去重、自动聚类、智能标签回写。
- Spatial Gallery：图片地图、二维空间浏览、收藏/标签组织视图。

只在确定要实现某个方向后，再读取相关代码文件。常见入口：
- 后端图片/图库：src/backend/src/images/mod.rs
- 后端任务：src/backend/src/tasks/mod.rs
- 后端 Pixiv/API：src/backend/src/pixiv/mod.rs、src/backend/src/api/
- 文件路径/安全：src/backend/src/storage/mod.rs
- DB migration：src/backend/migrations/0001_init.sql
- 前端 Home/Gallery/Download/Tasks：src/frontend/app/page.tsx、src/frontend/app/gallery/page.tsx、src/frontend/app/download/page.tsx、src/frontend/app/tasks/page.tsx
- 前端 API client/style：src/frontend/lib/api.ts、src/frontend/app/globals.css
- 测试入口：tests/run_local.sh、tests/stage/*.sh

工作方式：
- 先判断候选方向是否适合进入 v1.x 还是 v2 research track。
- 讨论阶段不要直接大改代码；先输出目标、用户价值、技术难点、风险、最小可验证切片。
- 一旦用户选定要实现的方向，再给最小实现清单并开始实现。
- 实现时 API 层保持薄封装，业务逻辑放领域模块；新增测试尽量关联 REQ-*。
- 增加确定性测试和阶段脚本，最后运行相关测试，必要时再运行 ./tests/run_local.sh。
- 同步 README.md、docs/progress.md、docs/CONTEXT_HANDOFF.md、docs/specs/api-contract.md、docs/specs/traceability.md、tests/README.md。
- 清理文件前只生成候选清单，等待用户确认。
```
