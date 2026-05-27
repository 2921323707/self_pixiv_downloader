<p align="center">
  <img src="./static/innerFiles/app-icon.png" alt="Pixiv Platform logo" width="128" />
</p>

<h1 align="center">Pixiv Platform</h1>

<p align="center">
  本地优先的 Pixiv 下载、索引、图库与智能检索工作台，支持 Web 工作台、Windows 桌面端与 macOS 桌面端源码路径。
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
- Windows 桌面端：Tauri / NSIS `.exe` 安装包，内嵌启动现有 Rust 后端，`v1.2.0` 已手动验证。
- macOS 桌面端：Tauri `.app` / ad-hoc signed `.dmg` 源码路径保留，历史 `v1.1.1` 构建已验证。

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
tauri-app/            Tauri 桌面壳与打包配置，当前默认本地构建目标为 Windows NSIS
docs/                 产品、架构、接口、测试和交付文档
docs/design/          视觉主题参考与设计归档
tests/                单元、集成、阶段、smoke 和 opt-in live 测试脚本
static/               README 图片和展示素材
```

## 环境要求

- Rust stable with Cargo
- Node.js 20+ 和 npm
- Windows 本地构建需要 Visual Studio 2022 Build Tools / MSVC C++ linker
- macOS 构建 `.dmg` 需要在 macOS 上执行，并使用 macOS 构建配置
- 有效的 Pixiv `PHPSESSID` cookie，真实下载时需要
- DeepSeek API key，只有 Smart Retrieval 解析或连接测试时需要

小范围使用已打包桌面安装包的用户不需要安装 Rust、Cargo、Node、npm 或 TypeScript。

## 快速启动

### 1. 安装前端依赖

```bash
cd src/frontend
npm install
```

Windows PowerShell 如遇到 `npm.ps1` 执行策略限制，使用 `npm.cmd install`。

### 2. 启动后端

后端默认监听 `127.0.0.1:3000`。Web / 后端独立运行和桌面端默认共享同一套本地目录：

```text
~/Downloads/Pixiv Platform/
~/Downloads/Pixiv Platform/pixiv_platform.sqlite3
```

```bash
cd src/backend
cargo run --bin server
```

Windows 本地快速启动：

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\dev_backend_windows.ps1
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

Windows 本地快速启动：

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\dev_frontend_windows.ps1
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

### Windows 桌面端构建

当前仓库默认 Tauri 构建目标为 Windows NSIS 安装包。Tauri 桌面端复用现有
`src/frontend` 和 `src/backend`，不会复制业务代码。Tauri 启动时会在进程内启动 Axum
后端，并为前端注入运行时 API 地址。

```powershell
cd tauri-app
npm.cmd install
npm.cmd run build
```

构建产物：

```text
tauri-app/src-tauri/target/release/pixiv_platform_tauri_app.exe
tauri-app/src-tauri/target/release/bundle/nsis/Pixiv Platform_1.2.0_x64-setup.exe
```

最新 Windows 本地锚点（2026-05-27）：用户手动确认 Web 正常、Windows Tauri App 正常；
Settings -> Pixiv 连接 -> Refresh 可正常弹出 Pixiv 登录窗口并完成刷新；`npm.cmd run build`
成功生成 `v1.2.0` NSIS 安装包。

### macOS 桌面端构建

v1.1.1 已作为成熟交付第一版本发布。Tauri macOS 桌面端复用现有 `src/frontend` 和 `src/backend`，
不会复制业务代码。Tauri 启动时会在进程内启动 Axum 后端，并为前端注入运行时 API 地址。

注意：当前 `tauri-app/src-tauri/tauri.conf.json` 的默认构建命令和 bundle target 已切到
Windows / NSIS。macOS 代码分支仍保留，但要重新构建 `.app` / `.dmg`，需要在 macOS 上把
前端导出命令切回 `NEXT_OUTPUT_EXPORT=1 npm --prefix ../src/frontend run build`，并把
`bundle.targets` 切回 `["app", "dmg"]`，或使用后续新增的 macOS 专用 Tauri 配置。

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

说明：当前 Tauri 配置默认面向 Windows，并已把 Windows 包版本同步为 `1.2.0`；macOS 历史
release 锚点仍为 `v1.1.1`。

当前 `.dmg` 为 ad-hoc signed、未 Developer ID 签名、未公证版本，可用于 GitHub Release
小范围分发。首次打开时，macOS Gatekeeper 可能拦截，需要用户在系统安全设置中手动允许。

本机验证命令：

```bash
codesign --verify --deep --strict --verbose=2 "tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app"
hdiutil verify "tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg"
```

最新本地验证：`./tests/run_local.sh` 通过，Tauri `.app` codesign 校验通过，`.dmg` checksum 校验通过。

### 桌面包源码关系

Windows `.exe` / NSIS installer 与 macOS `.app` / `.dmg` 不是两套应用源码。

```text
src/frontend/                唯一前端源码，Next.js 静态导出后进入 Tauri bundle
src/backend/                 唯一 Rust 后端源码，被 tauri-app 通过 Cargo path dependency 复用
tauri-app/src-tauri/src/     唯一 Tauri 桌面壳源码，少量代码用 cfg(target_os) 分支适配系统差异
tauri-app/src-tauri/target/  本地构建产物目录，不是源码
```

平台差异主要存在于打包目标、系统文件夹选择器、日志路径和安装包格式：

- Windows：`bundle/nsis/Pixiv Platform_1.2.0_x64-setup.exe`
- macOS：`bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`

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

v1.2.0 已完成稳定的本地下载、索引、Web 工作台和 Windows Tauri 桌面端交付闭环；macOS
历史成熟交付锚点为 v1.1.1。
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
- `docs/releases/v1.2.0.md`：v1.2.0 Release notes
- `docs/specs/architecture.md`：架构说明
- `docs/specs/api-contract.md`：API 合约
- `docs/specs/testing-strategy.md`：测试策略
- `tauri-app/docs/progress.md`：桌面端进度与验证记录
- `tauri-app/docs/next-session-prompt.md`：下一轮新会话提示词

## 安全说明

- secret 只允许运行时配置或通过 Settings 保存到本地 SQLite。
- API 返回 Settings 时会遮罩 secret。
- live 测试必须手动 opt-in。
- Web / 后端独立运行和桌面端默认共享 `~/Downloads/Pixiv Platform/`。
- 默认 SQLite 路径为 `~/Downloads/Pixiv Platform/pixiv_platform.sqlite3`。
- 可通过 Settings 或环境变量改为其它绝对路径。
