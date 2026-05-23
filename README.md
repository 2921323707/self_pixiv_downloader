# Pixiv AI 下载与智能检索平台

<p align="center">
  <img src="docs/assets/readme-hero.svg" alt="Pixiv AI Downloader Platform v1.0.0" width="100%" />
</p>

这是一个 **本地优先** 的个人 Pixiv 下载、索引、检索与图库管理平台。项目已经进入 **v1.0.0 Downloader-First Final**：核心下载、SQLite 索引、任务追踪、图库预览、设置管理、作者/收藏批量下载、Smart Retrieval 智能标签检索和 Next.js 工作台已经形成完整闭环。

如果你是来体验或帮忙提 issue 的朋友，可以先从这三件事开始：

1. 按 [快速入口](#快速入口) 跑起来。
2. 在 Settings 配置运行时 Pixiv cookie 和下载目录。
3. 试用 Single / Author / Bookmarks / Smart / Gallery / Tasks，并把问题或建议提到 GitHub Issues。

## v1.0.0 已完成

当前版本的目标不是做一个“大而全”的 Pixiv 客户端，而是先做一个稳定、可追溯、可测试、能真实使用的 downloader-first 本地平台。

| 能力 | 状态 |
| --- | --- |
| 单作品下载 | 已完成，支持任务追踪和 SQLite 入库 |
| 作者批量下载 | 已完成，复用批量 task worker |
| 收藏批量下载 | 已完成，支持默认数量和上限控制 |
| Smart Retrieval | 已完成，支持 DeepSeek parse、手动 tag chips、Pixiv tag search 批量下载 |
| Gallery | 已完成 metadata 列表、安全文件预览、右侧 drawer、多选删除 |
| Tasks | 已完成任务列表、任务详情 modal、items/logs/progress |
| Settings | 已完成 public settings、secret mask、Pixiv/DeepSeek connection test |
| Home | 已完成 command center、最近任务、normal banner、Rust Core Driver 注解、Performance Watch |
| 测试 | 已完成 backend unit、stage、integration、smoke、frontend scaffold gates |

## 产品形态

### Home Command Center

Home 不再是占位页，而是项目控制台：

- 最近 3 条任务和状态摘要
- 最近 normal 横向候选图片 banner
- Pixiv cookie / DeepSeek key / download path 配置状态
- Rust Core Driver 注解
- Performance Watch
- v1.x / v2 后续能力槽

### Download Workbench

Download 是一个真实工具台：

- Single：输入 Pixiv ID 入队下载
- Author：按作者 UID 批量下载
- Bookmarks：按当前用户收藏批量下载
- Smart：自然语言解析 tags，或手动添加正/负 tag chips 后入队下载

### Gallery / Tasks / Settings

- Gallery 使用后端安全 file endpoint 展示本地图片，不暴露原始路径。
- Tasks 可以查看进度、items、logs 和失败诊断。
- Settings 中的 secret 只显示 masked 状态，真实 Pixiv cookie / DeepSeek key 只允许运行时配置。

## 技术栈

| 层 | 技术 |
| --- | --- |
| Backend | Rust, Axum, Tokio |
| Database | SQLite, migrations, repository modules |
| Frontend | Next.js, React, TypeScript |
| Downloader | Pixiv client abstraction, DB-aware storage |
| AI | DeepSeek-compatible Smart Parse, Pixiv tag search |
| Quality | deterministic stage scripts, integration tests, smoke tests |

项目采用 **spec-coding** 驱动方式：需求、状态、数据库、任务流、API、测试和实现路径都写在 `docs/` 下，关键实现尽量追溯到 `REQ-*`。

## 当前边界

v1.0.0 已经完成 downloader-first 本地平台闭环。以下内容不是 v1.0.0 blocker，而是后续进化方向：

- Top10 / Random discovery modes
- Gallery thumbnail cache
- task cancel / retry
- 更完整的图片编辑和 map API
- 本地图像语义索引、向量检索、相似图去重、自动聚类等高复杂度能力

## 测试基线

当前本地基线：

```text
./tests/run_local.sh
82 backend unit tests passed; Phase 2A checks passed; Phase 2C checks passed; backend SQLite integration checks passed; backend API smoke checks passed; Phase 3B queue checks passed; Phase 4B data API checks passed; Phase 4C configured download checks passed; Phase 4D gallery file API checks passed; Phase 4E gallery delete checks passed; Phase 5A author batch checks passed; Phase 5B bookmark batch checks passed; Phase 6A smart parse checks passed; Phase 6B smart download checks passed; frontend scaffold checks passed; 0 failed
```

已手动验证：

- 前端输入 Pixiv 作品 ID 可以成功下载并写入本地文件。
- Author Batch 和 Bookmarks Batch 可从前端触发并完成下载。
- Smart Retrieval 的 Parse -> 编辑 tags/chips -> Enqueue smart download 流程可用。
- Home / Download / Tasks / Gallery / Settings 均已接入真实 API。

## 欢迎提 Issue

如果你是 pull 下来试用的朋友，最希望你帮忙反馈：

- 安装/启动过程是否顺畅
- Pixiv cookie / 下载目录 / DeepSeek key 配置是否容易理解
- Single / Author / Bookmarks / Smart 哪个流程最容易出错
- Gallery 在图片较多时是否卡顿
- UI 上是否有文字溢出、按钮难点、drawer/modal 不好关闭等问题
- 你最希望 v1.x 或 v2 加什么高级功能

提 issue 时最好附上：

```text
系统：
运行命令：
出问题的页面：
期望行为：
实际行为：
终端错误或浏览器控制台错误：
是否使用 live Pixiv / live LLM：
```

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
