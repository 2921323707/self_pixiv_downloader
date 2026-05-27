<p align="center">
  <img src="./static/innerFiles/app-icon.png" alt="Pixiv Platform logo" width="128" />
</p>

<h1 align="center">Pixiv Platform</h1>

<p align="center">
  本地优先的 Pixiv 下载、索引、图库与智能检索工作台，支持 Web 工作台与 macOS 桌面端。
</p>

## 功能概览

- Pixiv 单作品下载：通过作品 ID 下载并写入本地目录。
- 作者批量下载：按作者 UID 发现作品并创建批量任务。
- 收藏批量下载：读取当前用户收藏并创建批量任务。
- Smart Retrieval：用 DeepSeek 将自然语言需求解析为 Pixiv tags，再发起标签搜索下载。
- 本地 SQLite 索引：保存图片、来源、标签、任务、任务项和任务日志。
- Gallery：查看已下载作品，读取本地图片预览，支持单张和批量删除。
- Tasks：查看任务状态、进度、任务项和日志。
- Settings：配置 Pixiv cookie、DeepSeek key、下载目录、默认批量数量、R18 策略和主题。
- Next.js 工作台：Home、Download、Tasks、Gallery、Settings 五个主要页面。
- macOS 桌面端：Tauri `.app` / ad-hoc signed `.dmg` 打包，内嵌启动现有 Rust 后端。

## 部分页面展示
### 首页
![](./static/demoDisplay/index.png)

### 智能检索批量下载
![](./static/demoDisplay/download.png)

### 任务监控面板
![](./static/demoDisplay/taskReview.png)

## 技术栈

- Backend: Rust 2024, Axum, Tokio, rusqlite, reqwest
- Frontend: Next.js 16, React 19, TypeScript
- Storage: local filesystem + SQLite
- AI provider: DeepSeek-compatible chat API

## 目录结构

```text
src/backend/          Rust 后端、API、任务队列、Pixiv/DeepSeek 客户端、SQLite 仓储
src/backend/src/api/  Axum routes、DTO/envelope、handlers、runtime settings 和 queue worker glue
src/frontend/         Next.js 前端工作台
tauri-app/            macOS Tauri 桌面壳与打包配置
docs/                 产品、架构、接口、测试和交付文档
docs/design/          视觉主题参考与设计归档
tests/                单元、集成、阶段、smoke 和 opt-in live 测试脚本
static/               README 图片和展示素材
```

## 环境要求

- Rust stable with Cargo
- Node.js 20+ 和 npm
- macOS / Linux shell 环境
- 有效的 Pixiv `PHPSESSID` cookie，真实下载时需要
- DeepSeek API key，只有 Smart Retrieval 解析或连接测试时需要

macOS 桌面端打包需要在 macOS 上执行。小范围使用已打包 `.dmg` 的用户不需要安装
Rust、Cargo、Node、npm 或 TypeScript。

## 快速启动

### 1. 安装前端依赖

```bash
cd src/frontend
npm install
```

### 2. 启动后端

后端默认监听 `127.0.0.1:3000`。Web / 后端独立运行和 macOS 桌面端默认共享同一套本地目录：

```text
~/Downloads/Pixiv Platform/
~/Downloads/Pixiv Platform/pixiv_platform.sqlite3
```

```bash
cd src/backend
cargo run --bin server
```

可选环境变量：

```bash
PIXIV_PLATFORM_BIND=127.0.0.1:3000
PIXIV_DOWNLOAD_ROOT=/absolute/path/to/output
PIXIV_PLATFORM_DB_PATH=/absolute/path/to/pixiv_platform.sqlite3
PIXIV_PHPSESSID=your_pixiv_cookie
DEEPSEEK_API_KEY=your_deepseek_key
```

也可以在前端 Settings 页面保存 `pixiv_cookie`、`deepseek_api_key` 和
`download_base_path`。secret 会被后端遮罩返回，不会在 API 响应中明文回显。

### 3. 启动前端

前端默认监听 `127.0.0.1:3001`。

```bash
cd src/frontend
npm run dev
```

前端 API 代理默认指向 `http://127.0.0.1:3000`。如果后端使用了其它端口：

```bash
PIXIV_BACKEND_URL=http://127.0.0.1:3002 npm run dev
```

打开：

```text
http://127.0.0.1:3001
```

### 4. 使用流程

1. 进入 Settings，保存 Pixiv cookie。
2. 可选：保存下载目录，默认值为 `~/Downloads/Pixiv Platform`。
3. 进入 Download，选择 Single、Author、Bookmarks 或 Smart。
4. 提交下载任务后进入 Tasks 查看进度。
5. 进入 Gallery 查看本地图片预览和详情。

## 生产构建

### 后端 release 构建

```bash
cd src/backend
cargo build --release
```

构建产物：

```text
src/backend/target/release/server
src/backend/target/release/live_single
```

运行 release 后端：

```bash
PIXIV_PLATFORM_BIND=127.0.0.1:3000 \
PIXIV_DOWNLOAD_ROOT=/absolute/path/to/output \
PIXIV_PLATFORM_DB_PATH=/absolute/path/to/pixiv_platform.sqlite3 \
src/backend/target/release/server
```

### 前端生产构建

```bash
cd src/frontend
npm run build
```

### macOS 桌面端构建

v1.1.1 已作为成熟交付第一版本发布。Tauri macOS 桌面端复用现有 `src/frontend` 和 `src/backend`，
不会复制业务代码。Tauri 启动时会在进程内启动 Axum 后端，并为前端注入运行时 API 地址。

```bash
cd tauri-app
npm install
npm run build
```

构建产物：

```text
tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app
tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg
```

说明：当前 Tauri 配置和产物文件名仍保留 `1.1.0`，公开 release 锚点为 `v1.1.1`。

当前 `.dmg` 为 ad-hoc signed、未 Developer ID 签名、未公证版本，可用于 GitHub Release
小范围分发。首次打开时，macOS Gatekeeper 可能拦截，需要用户在系统安全设置中手动允许。

本机验证命令：

```bash
codesign --verify --deep --strict --verbose=2 "tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app"
hdiutil verify "tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg"
```

最新本地验证：`./tests/run_local.sh` 通过，Tauri `.app` codesign 校验通过，`.dmg` checksum 校验通过。

## 测试

默认本地质量门：

```bash
./tests/run_local.sh
```

该脚本会运行后端单测、SQLite 集成、API smoke、阶段脚本和前端 typecheck/build。
当前基线为 86 个后端单测通过，完整本地质量门通过。

真实 Pixiv E2E 是 opt-in 测试，需要运行时提供 cookie：

```bash
PIXIV_PHPSESSID=your_pixiv_cookie ./tests/e2e/live_single_download.sh
```

最近一次发布验证 shell 未设置 `PIXIV_PHPSESSID`，因此未运行 live Pixiv E2E。
不要把 Pixiv cookie、DeepSeek API key 或其它 secret 写入仓库文件。

## API 入口

常用端点：

- `POST /api/download/single`
- `POST /api/downloads/author`
- `POST /api/downloads/bookmarks`
- `POST /api/smart/parse`
- `POST /api/smart/download`
- `GET /api/tasks`
- `GET /api/tasks/{task_id}`
- `GET /api/images`
- `GET /api/images/{image_id}`
- `GET /api/images/{image_id}/file`
- `DELETE /api/images/{image_id}`
- `POST /api/images/delete-batch`
- `GET /api/settings`
- `PUT /api/settings/{key}`
- `POST /api/settings/test/pixiv`
- `POST /api/settings/test/deepseek`

完整接口说明见 `docs/specs/api-contract.md`。

## 当前边界

v1.1.1 已完成稳定的本地下载、索引、Web 工作台和 macOS Tauri 桌面端交付闭环，可作为成熟交付第一版本。
以下能力仍属于后续 v1.x / v2 演进方向：

- 缩略图缓存和大图库性能优化
- Top10 / Random discovery modes
- 任务取消、重试和更细 worker 诊断
- 图片编辑、地图视图和更复杂的图库组织
- 本地语义检索、相似图聚类和向量索引
- macOS Developer ID 签名、公证和自动更新

## 文档入口

- `docs/CONTEXT_HANDOFF.md`：新会话恢复项目上下文
- `docs/progress.md`：当前阶段、完成项和验证基线
- `docs/DOCUMENT_MAP.md`：文档地图
- `docs/releases/v1.1.1.md`：v1.1.1 Release notes
- `docs/specs/architecture.md`：架构说明
- `docs/specs/api-contract.md`：API 合约
- `docs/specs/testing-strategy.md`：测试策略
- `tauri-app/docs/progress.md`：macOS 桌面端进度与验证记录
- `tauri-app/docs/next-session-prompt.md`：下一轮新会话提示词

## 安全说明

- secret 只允许运行时配置或通过 Settings 保存到本地 SQLite。
- API 返回 Settings 时会遮罩 secret。
- live 测试必须手动 opt-in。
- Web / 后端独立运行和 macOS 桌面端默认共享 `~/Downloads/Pixiv Platform/`。
- 默认 SQLite 路径为 `~/Downloads/Pixiv Platform/pixiv_platform.sqlite3`。
- 可通过 Settings 或环境变量改为其它绝对路径。
