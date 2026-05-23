# 上下文接续说明

用途：当 Codex 上下文重置、切换模型、或者很久以后重新打开项目时，用这份文件快速恢复对项目的掌控。

## 一句话项目状态

这是一个本地优先的 Pixiv AI 下载与智能检索平台。当前采用 downloader-first 路线，已跑通真实 Pixiv 单作品下载，并已完成 SQLite migration、图片仓储、settings 仓储、DB-aware downloader、任务状态持久化、确定性测试脚本结构、薄 Axum API wrapper、in-process Tokio 后台任务队列、Next.js 前端雏形、Phase 4B gallery/settings/task-list 最小数据 API 接线、Phase 4D Gallery 文件预览、Phase 4E Gallery 删除、Phase 5A 作者批量下载、Phase 5B 收藏批量下载、Phase 6A DeepSeek 智能解析、Phase 6B smart 标签搜索批量下载、Phase 7A Home Dashboard 首页真实化，以及 Phase 7B UI Layout / Interaction Polish 第一版。

## 当前阶段

当前阶段：**Phase 7B - UI Layout / Interaction Polish 第一版** 已完成。

2026-05-22 手动浏览器锚点：用户已确认在前端输入 Pixiv 作品 ID 可以成功下载，并确认 Author Batch 和 Bookmarks Batch 可用。2026-05-23：Smart Retrieval 已从“解析标签”推进到“解析后入队 smart 批量下载”，并经用户手动检查当前没有明显问题。同日 Home Dashboard 已在本地浏览器确认可通过真实 API 展示任务、图库预览和配置状态，控制台无错误。2026-05-23：Phase 7B UI polish 已完成确定性前端检查，未新增后端 API。

## 当前真实边界

后端：当前 single-download -> task -> indexed metadata -> Gallery preview/delete、author batch -> multi-item task -> DB-aware downloads、bookmark batch -> multi-item task -> DB-aware downloads、natural language -> DeepSeek -> tag plan preview -> Pixiv tag search -> smart batch task 已经可用，但不能说“全产品后端完成”。

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
- `GET /api/images/{image_id}/file`
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
- default download root resolves to repository `output/` through `project:output`

还未完成的是：

- 生成缩略图缓存 API
- top10 / random 批量下载
- 更完整的图库筛选、编辑与地图 API

前端：当前是可运行雏形，不是完整产品 UI。

- Download 页已真实对接单作品下载 API
- Download 页已真实对接 Bookmarks 批量下载 API
- Download 页已真实对接 Author 批量下载 API
- Download 页已真实对接 Smart Retrieval 解析 API，并可编辑解析出的 tags / negative tags 后入队 smart 批量下载任务
- Download 页已从左大栏/右堆叠改为 Single / Author / Bookmarks / Smart 顶部 tabs 工作台
- Download 页发起的单作品任务会使用 Settings 保存的 Pixiv cookie 和 download_base_path
- Tasks 页已真实对接 task-id 轮询 API 和任务列表 API，Recent Tasks 默认 10 条，可展开更多，点击后用 modal 动态加载进度/items/logs
- Gallery 页已真实对接图片 metadata 列表 API，可通过安全 file endpoint 显示本地图片预览，点击图片打开右侧详情 drawer，并支持多选删除本地文件和 SQLite 索引
- Settings 页已真实对接 public settings list/save API、Pixiv test、DeepSeek test，secret 显示保持 masked，并按通用/外观/Pixiv/DeepSeek/Storage 分类展示
- Home 页已真实复用 `GET /api/tasks`、`GET /api/images`、`GET /api/settings` 展示最近任务、状态摘要、最近 normal 图片 banner、快速入口、配置状态和本地图库提示
- Top10 / Random 仍未完成

已经完成：

1. 增加 Axum 依赖和最小 backend server 入口
2. 定义 app state 与单作品下载/task polling DTO
3. 实现 `POST /api/download/single`
4. 实现 `GET /api/tasks/{task_id}`
5. 增加 API smoke/integration 测试
6. 首页从占位改为真实 dashboard，API 层未新增后端端点
7. UI polish 第一版改造 Home / Download / Tasks / Gallery / Settings，API 层未新增后端端点

下一步候选：

1. 推荐进入 **Phase 7B follow-up - Gallery Quality / Thumbnail Cache Slice**：先让 Gallery 在批量/智能下载后更好浏览，再继续扩展 Top10 / Random。
2. Phase 7C 可做 Top10 / Random discovery modes，继续复用 Phase 5A/5B/6B 的统一数量/筛选策略：请求 limit/count、`default_batch_count=20`、`max_request_count=100`、settings `r18_policy`。
3. Phase 7D 可做 task cancel/retry 和更清晰的 worker 诊断。
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
| Axum API wrapper | `src/backend/src/api.rs` |
| DeepSeek / smart parse | `src/backend/src/ai.rs` |
| Pixiv tag search | `src/backend/src/pixiv/mod.rs` |
| 后端 server 入口 | `src/backend/src/bin/server.rs` |
| 前端 scaffold | `src/frontend` |

## 当前测试基线

默认质量门：

```text
./tests/run_local.sh
```

当前结果：

```text
82 backend unit tests passed; Phase 2A checks passed; Phase 2C checks passed; backend SQLite integration checks passed; backend API smoke checks passed; Phase 3B queue checks passed; Phase 4B data API checks passed; Phase 4C configured download checks passed; Phase 4D gallery file API checks passed; Phase 4E gallery delete checks passed; Phase 5A author batch checks passed; Phase 5B bookmark batch checks passed; Phase 6A smart parse checks passed; Phase 6B smart download checks passed; frontend scaffold checks passed; 0 failed
```

真实 Pixiv E2E：

```text
PIXIV_PHPSESSID=... ./tests/e2e/live_single_download.sh
```

注意：不要把 Pixiv cookie 写进仓库。

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
- 2026-05-22 用户手动浏览器验证已通过：前端输入 Pixiv 作品 ID 后下载成功
- 2026-05-22 用户手动浏览器验证已通过：Author Batch 下载成功
- 2026-05-22 用户手动浏览器验证已通过：Bookmarks Batch 下载成功
- Phase 3B 清理检查已完成：仅生成候选清单，没有删除文件
- 默认主题已选 `cyan-studio`
- 当前仓库保留 `demo_B.html` 作为视觉参考
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
1. README.md 的“当前状态/手动浏览器锚点/下一阶段推荐”
2. docs/CONTEXT_HANDOFF.md 的“当前阶段/当前真实边界/下一步候选/重要约束”
3. docs/progress.md 的“Current Anchor/Immediate Next Implementation Step”
4. docs/specs/traceability.md 里与本次任务相关的 REQ 行

当前锚点：
- Phase 7B - UI Layout / Interaction Polish 第一版已完成。
- 单图下载、Author Batch、Bookmarks Batch、Smart Retrieval Parse -> 编辑标签 -> Enqueue smart download 均已可用并经用户手动检查。
- Home、Download、Tasks、Gallery、Settings 已接入真实 API；Home 复用 tasks/images/settings API 展示工作台状态且不暴露 secret。
- 当前 UI polish：Download tabs 工作台、Gallery 右侧详情 drawer、Tasks 详情 modal 与展开更多、Settings 分类面板、Home normal 最近图 banner。
- 后端 downloader-first 核心稳定，但 Top10/Random、缩略图缓存、任务 cancel/retry、图片编辑/map 仍未完成。
- 默认下载目录是项目 output/，secret 只允许运行时配置，禁止写入 Pixiv cookie 或 DeepSeek key。
- live Pixiv / live LLM 测试保持 opt-in。

本次优先任务建议：Phase 7B follow-up - Gallery Quality / Thumbnail Cache Slice。
只在需要实现 Gallery thumbnail/cache 时再读取这些文件：
- 后端图片/图库：src/backend/src/images/mod.rs
- 后端 API：src/backend/src/api.rs
- 文件路径/安全：src/backend/src/storage/mod.rs
- DB migration：src/backend/migrations/0001_init.sql
- 前端 Gallery：src/frontend/app/gallery/page.tsx
- 前端 API client：src/frontend/lib/api.ts
- Gallery 样式：src/frontend/app/globals.css
- 测试入口：tests/run_local.sh、tests/stage/phase4d_gallery_file_api.sh、tests/stage/phase4e_gallery_delete.sh

工作方式：
- 先给 thumbnail/cache follow-up 的最小实现清单并确认是否有明显风险。
- 实现时 API 层保持薄封装，业务逻辑放 images/storage 或专门 query/cache 模块。
- 增加确定性测试和阶段脚本，最后运行相关测试，必要时再运行 ./tests/run_local.sh。
- 同步 README.md、docs/progress.md、docs/CONTEXT_HANDOFF.md、docs/specs/api-contract.md、docs/specs/traceability.md、tests/README.md。
- 清理文件前只生成候选清单，等待用户确认。
```
