# Tauri Desktop Implementation Plan

## Phase D0: 文档与边界

- [x] `v1.1.1` 已作为成熟交付第一版本 release。
- [x] 状态文档已从流水追加改为“顶部锚点覆写 + 关键 debug 信息保留 + 阶段结束压缩”。
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
- [x] 确认 `.app` 默认数据目录统一为 `~/Downloads/Pixiv Platform/`。
- [x] 设计 Tauri build 加载静态前端产物的最小实现。
- [x] 定义固定端口失败时的最小诊断策略。
- [x] 执行 `.app` 最小打包验证。

## Phase D5: 后续优化候选

- [x] P0 随机可用端口与运行时 API base URL 注入，替代固定 `127.0.0.1:3000`。
- [x] P0 更完整的健康检查和 stderr 启动失败诊断：实现轻量 `/api/health`，再由 Tauri 启动流程轮询通过后创建主窗口。
- [x] P0 桌面 MVP 稳定化：最新 `.app` 可直接打开运行，用户手动测试通过；Gallery 删除交互已修复。
- [x] P1 桌面默认目录迁移到 `~/Downloads/Pixiv Platform/`，用于下载与默认 SQLite。
- [x] P1 Settings 下载目录改为系统文件夹选择器配置。
- [x] P1 默认目录统一为 `~/Downloads/Pixiv Platform/`；旧 `output/` 自动迁移逻辑已移除。
- [x] P2 基础菜单。
- [x] P2 正式替换当前临时 RGBA 图标：使用用户提供图片生成 Tauri bundle 图标。
- [x] P2 启动失败提示与日志位置：增加桌面启动日志文件，并在后端启动或健康检查失败时显示应用内错误窗口。
- [x] P3a `.dmg` 最小闭环：以没有 Apple 开发者认证为前提，产出可上传 GitHub Release 的 ad-hoc signed `.dmg`。
- [x] P3 分发说明：记录未 Developer ID 签名、未公证 Gatekeeper 风险，并说明他人不需要 Rust、Cargo、Node、npm、TypeScript。
- [ ] P3 正式分发评估：未来如需公开分发，再评估 Apple 账号、证书、签名、公证和更新渠道。
- [x] P3 `.dmg` 最小产物验证。
- [x] P3a `.dmg` 损坏提示复查：显式启用 macOS ad-hoc signing，重新 build 后 `.app` codesign、
  `.dmg` checksum 和挂载结构验证通过。
- [ ] P3 签名、公证。
- [ ] P3 自动更新。

## Phase D6: 下一阶段建议

- [x] 以 `v1.1.1` 作为成熟交付第一版维护基线。
- [x] P2 基础菜单：先做 macOS 常见菜单项与安全的窗口操作，不扩大业务逻辑面。
  已加入 About / Quit / Reload，Developer Tools 仅在 debug 构建显示；不接入业务逻辑。
- [x] P3 分发评估：先按没有 Apple 开发者认证的现实条件推进 ad-hoc signed `.dmg`，Developer ID 签名/公证只做后续说明。
- [x] P3 `.dmg` 最小闭环：优先产出 ad-hoc signed `.dmg`，记录产物路径和手动安装测试点。
- [ ] P4 清理收束：清理项必须先确认，不删除用户未确认的本地产物。

## Phase D7: Pixiv 登录态刷新

目标：在 Tauri 桌面端提供内置 Pixiv 登录窗口，一键刷新 `PHPSESSID`，并复用现有
`pixiv_cookie` 设置、secret masking 和 Pixiv connection test。

- [x] 方案比较：App 内置登录窗口优先于浏览器扩展、OAuth 长期方案和后端模拟登录。
- [x] 小规模可行性验证：Tauri 2.11.2 可读取 WebView cookie store；HttpOnly cookie 可读；
  本地探针不输出完整 secret。
- [x] 设计锚点：正式实现优先调用 `window.cookies()`，按 `PHPSESSID` 与 Pixiv 域名过滤；
  `cookies_for_url()` 可作为辅助但不作为唯一路径。
- [x] Tauri command：打开或聚焦 Pixiv 登录窗口，轮询 cookie store，找到 `PHPSESSID` 后返回
  `{ value, domain, path, http_only, secure }` 中的非敏感元信息和值；日志只记录长度和状态。
- [x] Settings UI：在 `pixiv_cookie` 行增加桌面端可见的 Login/Refresh 按钮；非 Tauri Web 环境显示
  手动输入和 Test 作为 fallback。
- [x] Web 端范围锚定：不实现网页端自动读取 Pixiv `PHPSESSID`；网页端继续让用户手动填写
  `pixiv_cookie` 并使用 Test 验证。
- [x] 保存闭环：前端拿到 cookie 后调用现有 `saveSetting("pixiv_cookie", value)`，随后自动调用
  `testPixivConnection("144920810")` 或无作品 ID 的配置验证。
- [x] 安全处理：获取成功后自动关闭 Pixiv 登录窗口；主 Settings 窗口弹出成功提示；不保留完整 cookie
  到前端状态以外的可见文本；错误状态不泄漏 cookie。
- [x] 验证：`cargo check`、前端 lint/build、Tauri build；live 验证由用户手动登录 Pixiv 完成并确认成功。

实施顺序：

实现结果：

1. Tauri command 不直接写 SQLite，只返回 cookie 给前端。
2. Settings 按钮保存到既有 `pixiv_cookie` setting，保留手动输入。
3. 成功后自动 Pixiv Test、关闭登录窗口并弹出非敏感成功提示。
4. 网页端不追加自动 cookie 获取能力，保持手动填写 `pixiv_cookie` 的 fallback 路径。
