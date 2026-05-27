# Tauri Desktop Checklist

## 当前 MVP 清单

- [x] 用户确认使用 Tauri。
- [x] 用户确认 `tauri-app` 目录作为桌面源码位置。
- [x] 用户确认复用现有前端和后端。
- [x] 用户确认中文文档。
- [x] 用户确认 dev 模式优先。
- [x] 用户确认允许轻改前端 API client。
- [x] 早期 MVP 曾沿用 `output/`；当前默认已统一为 `~/Downloads/Pixiv Platform/`。
- [x] 用户曾确认固定 `127.0.0.1:3000` 作为简化 MVP。
- [x] 用户确认升级为每次启动随机端口并拼接为 API base URL。
- [x] 建立 Tauri 项目骨架。
- [x] 引用现有 backend crate。
- [x] 配置加载现有 frontend dev server。
- [x] 前端 API base URL 兼容桌面模式。
- [x] 完成最小验证。
- [x] Windows Web 本地启动验证通过。
- [x] Windows Tauri App 手动验证通过。
- [x] Windows NSIS 安装包构建通过。
- [x] 记录 Windows `.exe` / macOS `.dmg` 共享同一套源码，不是两套应用代码。

## `.app` 打包前置清单

- [x] 用户确认生产前端使用 Next 静态导出。
- [x] `.app` MVP 初版曾简化处理，继续固定 `127.0.0.1:3000`。
- [x] `.app` 默认数据目录已统一到 `~/Downloads/Pixiv Platform/`。
- [x] 更新 Tauri build 配置以加载静态前端产物。
- [x] `.app` 初版曾确认静态导出下 API base URL 指向 `http://127.0.0.1:3000`。
- [x] 确认 `.app` 启动时自动启动内嵌 Axum 后端。
- [x] 确认静态 `.app` 直连本地 API 时后端返回 CORS 响应头。
- [x] 确认端口被占用时有可诊断日志或清晰错误。
- [x] 确认 `npm run build` 产出 macOS `.app`。
- [x] 确认双击 `.app` 后 Home / Download / Gallery / Tasks / Settings 可访问。
- [x] 用户手动测试新 `.app` 功能正常。
- [x] 随机可用端口与运行时 API base URL 注入。
- [x] 新 `.app` 在随机端口上返回 `GET /api/settings`。
- [x] 新 `.app` 不再依赖 `127.0.0.1:3000`。
- [x] 用户手动测试随机端口版本 `.app` 功能正常。
- [x] 后端提供轻量 `GET /api/health`。
- [x] Tauri 创建主窗口前先轮询 `GET /api/health`。
- [x] 健康检查失败时 stderr 输出清晰诊断。
- [x] 使用用户提供图片替换正式 App 图标资源。
- [x] P0 收尾后重新执行前端 lint、后端 cargo check、Tauri cargo check、Next 静态导出和 Tauri build。
- [x] 用户手动测试 P0 收尾版本 `.app` 通过。
- [x] 清理项复核：`.DS_Store` 暂不删除，图标占位已替换，构建/依赖产物仍按忽略规则处理。
- [x] 主页面左上角品牌 logo 替换为同一图标资源。
- [x] 今日任务收尾前重新执行 `cd src/frontend && npm run lint`。
- [x] 主页面 logo 替换后重新执行 `cd tauri-app && npm run build`，刷新 `.app` 静态产物。
- [x] Gallery 删除接口测试：`cd src/backend && cargo test req_img_007` 通过。
- [x] Gallery 删除交互修复：移除删除流程对 `window.confirm()` 的依赖，改为应用内确认弹窗。
- [x] Gallery 删除修复后重新执行前端 lint、Next 静态导出和 Tauri build。
- [x] 用户手动打开最新 `.app` 直接运行并测试通过。

## 下一阶段候选清单

- [x] 随机可用端口与基础连通验证。
- [x] 更完整的健康检查和 stderr 启动失败诊断。
- [x] 用户可见启动失败提示。
- [x] 桌面启动日志写入固定 macOS 日志目录。
- [x] 记录热缓存测速结果。
- [x] 盘点开发生成杂项，不执行清理。
- [x] 设计新会话首轮提示词。
- [x] 桌面默认下载目录迁移到 `~/Downloads/Pixiv Platform/`。
- [x] 桌面默认 SQLite 迁移到 `~/Downloads/Pixiv Platform/pixiv_platform.sqlite3`。
- [x] Settings 下载目录通过系统文件夹选择器配置。
- [x] Web / 后端独立运行和桌面端默认共享 `~/Downloads/Pixiv Platform/`。
- [x] 旧 `output/` 自动迁移逻辑已删除。
- [x] 用户手动测试最新 `.app`，确认当前阶段功能正常。
- [x] 正式 App 图标。
- [x] 基础菜单：About / Quit / Reload，Developer Tools 仅 debug。
- [x] P3a ad-hoc signed `.dmg` 最小闭环：以没有 Apple 开发者认证为前提，产出可上传 GitHub Release 的 `.dmg`。
- [x] P3 分发说明：记录 Gatekeeper 风险，并说明他人不需要 Rust、Cargo、Node、npm、TypeScript。
- [x] P3 `.dmg` 最小产物验证。
- [x] P4 清理收束：已清理 `.DS_Store` 和可再生成的本地构建产物；发布产物需重新 build 生成。
- [x] `.gitignore` 覆盖 Tauri 自动生成的 `tauri-app/src-tauri/gen/` schema 目录。
- [x] GitHub `v1.1.1` 已 release，作为成熟交付第一版本。
- [x] `v1.1.1` release 前最终 build 验证通过；当前本地产物文件名仍可能保留
  `Pixiv Platform_1.1.0_aarch64.dmg`。
- [x] Pixiv 登录态刷新可行性验证：Tauri WebView cookie store 可读取 HttpOnly `PHPSESSID`；
  正式实现建议用 `cookies()` 全量读取后按 Pixiv 域过滤。
- [x] Pixiv 登录态刷新 Tauri command。
- [x] Settings Pixiv Login/Refresh 按钮。
- [x] 获取后复用现有 `pixiv_cookie` 保存和 Pixiv Test 验证。
- [x] Pixiv 登录态刷新成功后自动关闭登录窗口并弹出非敏感成功提示。
- [x] 用户手动 live 验证 Pixiv 登录刷新成功。
- [x] 显式启用 macOS ad-hoc signing：`bundle.macOS.signingIdentity = "-"`。
- [x] `.app` codesign、`.dmg` checksum 和挂载结构验证通过。
- [x] Windows v1.2.0 发布：安装 MSVC Build Tools，修复 Windows download path / home fallback，修复 Settings Pixiv Refresh 弹窗，构建 `Pixiv Platform_1.2.0_x64-setup.exe`。
- [ ] 如需长期双平台维护，拆分 Windows/macOS Tauri build config，避免默认 `tauri.conf.json` 在平台间来回切换。
- [ ] P3 签名、公证。
- [ ] P3 自动更新。
- [ ] P1 后续：如需更正式的数据分层，再评估 macOS Application Support。

## 不做清单

- [x] 未经用户确认，不迁移到 macOS Application Support 数据目录。
- [x] 未经用户确认，不删除或清理现有代码。
- [ ] 未经用户确认，不做 git commit。
- [ ] 不写入任何 secret。

## 清理候选

- `.DS_Store` 已清理；根 `.gitignore` 已覆盖。
- `tauri-app/src-tauri/icons/icon.png` 已替换为用户提供图片生成的 256x256 RGBA PNG。
- `src/frontend/public/app-icon.png` 是前端品牌 logo 使用的图标副本。
- `tauri-app/src-tauri/target/` 和 `tauri-app/node_modules/` 是本地构建/依赖产物，由根 `.gitignore` 覆盖；最终 build 验证后当前本地已重新生成，但不纳入提交。
- 本轮已按用户要求清理残留；仍不做 git commit，除非用户明确确认。
- 2026-05-26 复核：`.DS_Store` 已再次出现在根目录、`static/` 和 Tauri bundle 目录；
  `src/frontend/.next`、`src/frontend/out`、`tauri-app/src-tauri/target`、`tauri-app/node_modules`
  也因验证/build 重新生成；`src/frontend/.next/dev/lock` 存在。未获用户明确批准前不删除。
