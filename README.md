# Pixiv AI 下载与智能检索平台

这是一个本地优先的个人 Pixiv 下载、索引、检索与图库管理项目。

当前项目采用 **spec-coding** 驱动方式：先把需求、状态、数据库、任务流、API、测试和实现路径写清楚，再让代码围绕这些规格推进。每个关键实现都尽量能追溯到 `REQ-*` 需求编号。

## 当前状态

当前阶段：**v1.0.0 Downloader-First Final** 已完成。

也就是：核心下载脚本已经跑通，下载结果已经进入 SQLite，并完成“文件 + 数据库”的双重去重、可追溯索引、任务状态持久化、确定性测试脚本结构、薄 Axum API wrapper、in-process Tokio 后台任务队列 / worker、Next.js 前端工作台、作者/收藏两个批量下载入口、Gallery 预览 / drawer / 多选删除、本地设置与 secret mask、自然语言智能解析标签后按 Pixiv 标签搜索并创建 smart 批量下载任务，以及 Home command center 化。按当前 downloader-first 产品边界，可以作为 **v1.0.0 最终形态**。

当前边界：

- 后端当前完成的是 downloader-first 的核心 vertical slice：单作品下载、DB 持久化、任务追踪、单作品下载 API、作者批量下载 API、收藏批量下载 API、smart 标签搜索批量下载 API、任务轮询 API、任务列表 API、图库 metadata 查询 API、图库安全文件预览 API、图库删除 API、settings public list/save API、settings-backed Pixiv cookie / 下载目录解析、Pixiv connection test API、DeepSeek connection test API、Smart Parse API、后台 worker。
- v1.0.0 不再把 Top10、Random、生成缩略图缓存、任务取消、图片编辑和 map API 作为阻塞项；这些进入 v1.x / v2 进化方向。
- 前端当前是可运行的 v1.0.0 工作台 UI：Home、Download、Tasks、Gallery、Settings 已对接现有 API；Home 展示最近 3 条任务、状态摘要、筛选后的最近 normal 横向候选图片 banner、Rust Core Driver 注解、Performance Watch、快速入口、Pixiv cookie / DeepSeek key / download_base_path 配置状态和本地图库提示；Settings 保存的 Pixiv cookie / 下载目录会影响后续下载并按配置分类展示；Download 已支持单作品、收藏批量、作者批量、Smart Retrieval 标签解析、chip 编辑标签、手动标签 smart 批量任务入队，并改为顶部工具 tabs 工作台；Gallery 可以显示本地下载图片预览、右侧详情 drawer 和多选删除；Tasks 默认展示 10 条 Recent Tasks，点击后用居中 modal 动态加载进度/items/logs。Top10 / Random 属于后续 discovery modes。

已经完成：

- downloader-first 架构与规格文档
- Rust 后端核心脚手架
- mock 单作品下载测试
- 真实 Pixiv 单作品下载冒烟测试，测试作品 `144920810`
- SQLite 初始迁移 `0001_init.sql`
- DB migration runner
- 图片 metadata / tag / source repository
- settings repository，并支持 secret mask
- DB-aware downloader：下载前 DB 查重、缺文件修复、已有文件索引、写入 images/image_tags/image_sources
- task repository / task logs / task items，并支持单作品下载任务生命周期
- 本地测试脚本、阶段测试脚本与 SQLite 集成测试脚本
- Axum API wrapper：`POST /api/download/single`、`GET /api/tasks/{task_id}`
- 后台任务队列：`POST /api/download/single` enqueue 后快速返回，worker 后台执行下载任务
- 前端 scaffold：Home / Download / Tasks / Gallery / Settings，下载页和任务页已对接现有 API
- Phase 4B 最小数据 API：`GET /api/images`、`GET /api/images/{image_id}`、`GET /api/settings`、`PUT /api/settings/{key}`、`GET /api/tasks`
- Phase 4C 前端配置下载闭环：worker 从 settings 读取 `pixiv_cookie` 和 `download_base_path`，并提供 `POST /api/settings/test/pixiv`
- Phase 4D Gallery 文件预览闭环：`GET /api/images/{image_id}/file` 安全读取本地文件，Gallery 卡片和详情可以显示真实下载图片
- Phase 4E Gallery 删除闭环：`DELETE /api/images/{image_id}`、`POST /api/images/delete-batch`，删除本地文件并清理 SQLite 索引
- Phase 5A Author 批量下载闭环：`POST /api/downloads/author`、作者作品发现、批量 task_items、顺序 worker、部分失败诊断、Download 作者表单
- Phase 5B Bookmarks 批量下载闭环：`POST /api/downloads/bookmarks`、当前用户收藏发现、批量 task_items、顺序 worker、部分失败诊断、Download 收藏表单
- Phase 6A Smart Retrieval Parse 闭环：`POST /api/smart/parse`、`POST /api/settings/test/deepseek`、DeepSeek key/base URL/model settings、Download 智能解析预览
- Phase 6B Smart Retrieval Download 闭环：`POST /api/smart/download`、Pixiv 标签搜索、`smart` task worker、`smart_retrievals` provenance、Download 智能下载入队
- Phase 7A Home Dashboard 首页真实化：复用 `GET /api/tasks`、`GET /api/images`、`GET /api/settings` 展示工作台状态，不暴露 secret
- Phase 7B UI Layout / Interaction Polish 第一版：Download tabs 工作台、Gallery 右侧详情 drawer、Tasks 详情 modal 与展开更多、Settings 分类面板、Home normal 最近图 banner
- Phase 7B follow-up UI Formatting / Interaction Repair：Home banner 前端优先筛选 normal 横向候选图，Recent Downloads / Configuration 底部动作对齐，并重构为 command center + Rust Core Driver + Performance Watch + capability slots；Smart 支持正/负 tag chip 手动输入并复用 `/api/smart/download`，API client 对空/非 JSON 响应给出可读错误；Gallery drawer 和 Tasks modal 支持遮罩/ESC 关闭；补移动端间距与确定性前端锚点
- `default_batch_count` 设置项：批量请求未传 limit 时默认使用 20，仍受 `max_request_count` 上限约束
- Home / Gallery / Settings / Tasks 前端页面已从占位切到真实 API 数据
- API smoke 测试脚本
- Phase 3B queue 测试脚本
- Phase 4B data API 测试脚本
- Phase 4C configured download 测试脚本
- Phase 4D gallery file API 测试脚本
- Phase 4E gallery delete 测试脚本
- Phase 5A author batch 测试脚本
- Phase 5B bookmark batch 测试脚本
- Phase 6A smart parse 测试脚本
- Phase 6B smart download 测试脚本
- Phase 7B frontend scaffold UI polish 锚点检查
- 文档地图与进度锚点

当前本地测试基线：

```text
./tests/run_local.sh
82 backend unit tests passed; Phase 2A checks passed; Phase 2C checks passed; backend SQLite integration checks passed; backend API smoke checks passed; Phase 3B queue checks passed; Phase 4B data API checks passed; Phase 4C configured download checks passed; Phase 4D gallery file API checks passed; Phase 4E gallery delete checks passed; Phase 5A author batch checks passed; Phase 5B bookmark batch checks passed; Phase 6A smart parse checks passed; Phase 6B smart download checks passed; frontend scaffold checks passed; 0 failed
```

手动浏览器锚点：

- 2026-05-22：已确认前端输入 Pixiv 作品 ID 可以成功下载，任务流完成并写入本地文件。
- 2026-05-22：已确认 Author Batch 和 Bookmarks Batch 都可以从前端触发并完成下载。
- 2026-05-23：Smart Retrieval 的 Parse -> 编辑标签 -> Enqueue smart download 流程已经过用户手动检查，当前没有明显问题。
- 2026-05-23：Home Dashboard 已在本地浏览器确认可通过真实 API 展示任务、图库预览和配置状态，控制台无错误。
- 2026-05-23：Phase 7B UI polish 第一版已完成确定性前端检查：Download tabs、Gallery drawer、Tasks modal、Settings 分类、Home normal banner。
- 2026-05-23：Phase 7B follow-up 已完成确定性前端检查：Home banner 候选筛选、Recent Tasks 收缩到 3 条、Home command center / Rust Core Driver / Performance Watch、Smart tag chips、Gallery drawer / Tasks modal 关闭交互和移动端布局锚点。
- Gallery 当前已经可以通过安全 file endpoint 显示真实图片预览。
- Gallery 当前已经支持多选删除，删除时会同步移除本地文件和 SQLite 索引。
- Smart Retrieval 当前已经可以把自然语言解析成标签计划，并从标签计划创建 smart 批量下载任务；用户已反馈 DeepSeek 转换偶发小错，当前策略改为日文 Pixiv 标签优先、英文兜底，且 R18 策略以用户选择为准。

v1.0.0 结论：

- 可以冻结为当前项目的第一个正式版本：本地优先、可追溯、可测试、可手动使用的 Pixiv downloader-first 平台。
- 该版本的成熟度来自稳定链路，而不是功能面面俱到：单作品 / 作者 / 收藏 / Smart tag search 下载、任务追踪、图库预览删除、Settings runtime 配置和 Home 工作台都已形成闭环。
- 后续工作建议以“高技术复杂度能力”或“v1.x 体验增强”立项，不再把它们视为 v1.0.0 未完成事项。

下一阶段推荐：

- **v1.x Evolution - Gallery Quality / Thumbnail Cache**：补缩略图缓存和 Gallery 浏览性能，降低批量/智能下载后图片变多带来的前端压力。
- **v1.x Discovery Modes**：继续做 Top10 / Random discovery modes。
- **v1.x Task Control**：补任务 cancel/retry、失败重试和更清晰的 worker 诊断。
- **v2 Research Track**：可探讨高复杂度能力，例如本地图像语义索引、CLIP/向量检索、自动聚类、相似图去重、智能标签回写、地图/画廊空间化浏览等。

## 快速入口

运行本地确定性测试：

```bash
./tests/run_local.sh
```

只运行 Phase 2A 的迁移与仓储测试：

```bash
./tests/stage/phase2a_repository.sh
```

只运行 Phase 2C 的任务持久化测试：

```bash
./tests/stage/phase2c_tasks.sh
```

运行确定性 SQLite 集成测试：

```bash
./tests/integration/backend_sqlite.sh
```

只运行 Axum API smoke 测试：

```bash
./tests/smoke/backend_api.sh
```

只运行 Phase 3B 后台队列测试：

```bash
./tests/stage/phase3b_queue.sh
```

只运行 Phase 4B 数据 API 测试：

```bash
./tests/stage/phase4b_data_api.sh
```

只运行 Phase 4C 前端配置下载测试：

```bash
./tests/stage/phase4c_configured_download.sh
```

只运行 Phase 4D Gallery 文件预览测试：

```bash
./tests/stage/phase4d_gallery_file_api.sh
```

只运行 Phase 4E Gallery 删除测试：

```bash
./tests/stage/phase4e_gallery_delete.sh
```

只运行 Phase 5A Author 批量下载测试：

```bash
./tests/stage/phase5a_author_batch.sh
```

只运行 Phase 5B Bookmarks 批量下载测试：

```bash
./tests/stage/phase5b_bookmark_batch.sh
```

只运行 Phase 6A Smart Retrieval 解析测试：

```bash
./tests/stage/phase6a_smart_parse.sh
```

只运行 Phase 6B Smart Retrieval 下载测试：

```bash
./tests/stage/phase6b_smart_download.sh
```

只运行前端 scaffold 检查：

```bash
./tests/stage/frontend_scaffold.sh
```

启动前端开发服务器：

```bash
cd src/frontend
npm run dev
```

运行真实 Pixiv 单作品 E2E 下载测试：

```bash
PIXIV_PHPSESSID=... ./tests/e2e/live_single_download.sh
```

该 E2E 脚本会走任务包装后的 DB-aware downloader，验证文件写入、task/items/logs、SQLite 索引、source history，以及第二次运行的 DB duplicate skip。

注意：不要把 Pixiv cookie、DeepSeek key、`.env` 文件提交到仓库。

## 项目结构

```text
docs/
  DOCUMENT_MAP.md        文档地图
  CONTEXT_HANDOFF.md     上下文接续说明
  maintenance.md         清理与维护策略
  progress.md            当前进度与下一阶段
  product_requirements.md
  specs/
src/
  backend/
    migrations/
    src/
  frontend/
    app/
    components/
    lib/
tests/
  unit/
  stage/
  smoke/
  e2e/
  live/
demo_B.html              当前选定主题 cyan-studio 的视觉参考
```

## 重要文档

- [文档地图](docs/DOCUMENT_MAP.md)
- [上下文接续说明](docs/CONTEXT_HANDOFF.md)
- [当前进度](docs/progress.md)
- [规格索引](docs/specs/README.md)
- [实现计划](docs/specs/implementation-plan.md)
- [追踪矩阵](docs/specs/traceability.md)
- [测试策略](docs/specs/testing-strategy.md)
- [清理与维护策略](docs/maintenance.md)

## 当前架构方向

下载器是项目内核。API、任务队列、图库、智能检索、前端页面都应该围绕下载器展开，而不是在各处重复下载逻辑。

当前后端模块：

- `db`：SQLite 初始化与迁移
- `downloads`：下载编排
- `images`：图片 metadata、tag、source history 仓储
- `pixiv`：Pixiv client 抽象与真实 HTTP client
- `settings`：本地配置仓储与 secret mask
- `storage`：本地路径规划与安全写入
- `tasks`：任务仓储 / worker 包装
- `api`：薄 Axum API wrapper 与 DTO

默认下载位置：

```text
output/originals/{pixiv_id}/{pixiv_id}_p{page}.{ext}
```

其中 `output/` 位于项目根目录。Settings 中的 `download_base_path` 仍可改成绝对路径或 `~/...`。

## 下一步

当前推荐方向：**v1.x / v2 evolution planning**。

1. 先围绕 v1.0.0 之后的进化方向做方案讨论，不急于直接扩 API。
2. v1.x 实用增强可优先考虑 Gallery thumbnail cache、浏览性能、Top10 / Random discovery、task cancel/retry。
3. v2 高技术复杂度方向可考虑本地图像语义索引、CLIP / vector search、自动聚类、相似图去重、智能标签回写、空间化 Gallery。
4. 选定方向后，先定义最小可验证切片，再读相关代码、补 specs/traceability/tests，保持 live Pixiv / live LLM 测试 opt-in。

当前平台已经从“单张能下载和预览”进入“v1.0.0 downloader-first 稳定闭环”阶段；下一步建议做进化方向评估，而不是继续把未实现能力视为 release blocker。
