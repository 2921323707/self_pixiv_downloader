# Tauri Desktop Implementation Plan

## Phase D0: 文档与边界

- [x] 明确 MVP 范围。
- [x] 明确非目标。
- [x] 建立测试规范、上下文管理规范和进度文档。

## Phase D1: Tauri 开发壳

- [x] 在 `tauri-app` 建立 Tauri 项目骨架。
- [x] 配置 Tauri dev 加载 `http://127.0.0.1:3001`。
- [x] 配置 `beforeDevCommand` 启动现有 Next dev server。
- [x] 通过 Cargo path dependency 引用 `src/backend`。
- [x] Tauri 启动时在后台线程内创建 Tokio runtime，并启动现有 Axum 后端。

## Phase D2: API 地址兼容

- [x] 调整前端 API client 支持 `NEXT_PUBLIC_PIXIV_BACKEND_URL`。
- [x] 默认未配置时继续使用 `/api`，保持 Web 开发体验。
- [x] 图片预览 URL 复用同一 API base URL 解析。
- [x] Tauri dev 初版通过 Next rewrites 访问 `http://127.0.0.1:3000`。
- [x] Tauri dev 当前通过运行时注入的 API base URL 访问随机端口后端。

## Phase D3: 验证

- [x] 运行前端 typecheck。
- [x] 运行前端 build。
- [x] 运行后端相关 Rust 检查。
- [x] 尝试启动 Tauri dev。
- [x] 同步 `progress.md` 和 `checklist.md`。

## Phase D4: `.app` 打包 MVP

- [x] `.app` 打包前置规划。
- [x] 确认生产模式使用 Next 静态导出。
- [x] `.app` MVP 初版继续固定 `127.0.0.1:3000`。
- [x] 确认 `.app` MVP 继续沿用项目 `output/`。
- [x] 设计 Tauri build 加载静态前端产物的最小实现。
- [x] 定义固定端口失败时的最小诊断策略。
- [x] 执行 `.app` 最小打包验证。

## Phase D5: 后续优化候选

- [x] P0 随机可用端口与运行时 API base URL 注入，替代固定 `127.0.0.1:3000`。
- [x] P0 更完整的健康检查和 stderr 启动失败诊断：实现轻量 `/api/health`，再由 Tauri 启动流程轮询通过后创建主窗口。
- [x] P0 桌面 MVP 稳定化：最新 `.app` 可直接打开运行，用户手动测试通过；Gallery 删除交互已修复。
- [x] P1 桌面默认目录迁移到 `~/Downloads/Pixiv Platform/`，用于下载与默认 SQLite。
- [x] P1 Settings 下载目录改为系统文件夹选择器配置。
- [x] P1 旧 `output/` 数据自动恢复到桌面默认目录。
- [x] P2 基础菜单。
- [x] P2 正式替换当前临时 RGBA 图标：使用用户提供图片生成 Tauri bundle 图标。
- [x] P2 启动失败提示与日志位置：增加桌面启动日志文件，并在后端启动或健康检查失败时显示应用内错误窗口。
- [x] P3a 未签名 `.dmg` 最小闭环：以没有 Apple 开发者认证为前提，产出可上传 GitHub Release 的未签名 `.dmg`。
- [x] P3 分发说明：记录未签名/未公证 Gatekeeper 风险，并说明他人不需要 Rust、Cargo、Node、npm、TypeScript。
- [ ] P3 正式分发评估：未来如需公开分发，再评估 Apple 账号、证书、签名、公证和更新渠道。
- [x] P3 未签名 `.dmg` 最小产物验证。
- [ ] P3 签名、公证。
- [ ] P3 自动更新。

## Phase D6: 下一阶段建议

- [x] P2 基础菜单：先做 macOS 常见菜单项与安全的窗口操作，不扩大业务逻辑面。
  已加入 About / Quit / Reload，Developer Tools 仅在 debug 构建显示；不接入业务逻辑。
- [x] P3 分发评估：先按没有 Apple 开发者认证的现实条件推进未签名 `.dmg`，签名/公证只做后续说明。
- [x] P3 `.dmg` 最小闭环：优先尝试未签名 `.dmg` 产物，记录产物路径和手动安装测试点。
- [ ] P4 清理收束：清理项必须先确认，不删除用户未确认的本地产物。
