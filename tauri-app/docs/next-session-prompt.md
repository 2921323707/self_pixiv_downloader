# 新会话首轮提示词

请在 `/Users/Admin/Downloads/pixiv_platform` 继续工作。本轮不要全局浏览仓库，采用渐进式披露检索。
第一轮只读：

1. `tauri-app/docs/progress.md`
2. `tauri-app/docs/implementation-plan.md`
3. `tauri-app/docs/architecture.md`
4. `tauri-app/docs/testing.md`
5. `tauri-app/docs/checklist.md`

如果要检查清理项，再读：

6. `docs/cleanup_candidates.md`

如果要改需求边界，再读：

7. `tauri-app/docs/requirements.md`

当前锚点：

- `tauri-app` 是桌面壳，不复制 `src/frontend` 或 `src/backend`。
- 前端仍来自 `src/frontend`；后端通过 Cargo path dependency 引用 `src/backend`。
- Desktop MVP、随机端口、`GET /api/health`、运行时 API base URL 注入、`.app` / 未签名 `.dmg`
  最小闭环、旧库自动恢复、基础菜单、启动诊断、Gallery 删除修复、Gallery 预览稳定性修复均已完成。
- 当前版本号锚定为 `v1.1.0`，`.app` / `.dmg` 构建产物由 `.gitignore` 忽略，不纳入提交。
- Pixiv 内置登录窗口刷新 `PHPSESSID` 已完成正式实现和用户 live 验证：
  Settings 的 Pixiv cookie 行在 Tauri 桌面端有 Refresh；
  点击后打开/聚焦 Pixiv 官方登录窗口；
  用户手动在 Pixiv 官方页面登录；
  Tauri command `refresh_pixiv_phpsessid` 使用 `WebviewWindow.cookies()` 全量读取 cookie store，
  按 `name == "PHPSESSID"` 和 Pixiv 域过滤；
  前端保存到现有 `pixiv_cookie` setting 并自动执行 Pixiv Test；
  成功后自动关闭 Pixiv 登录窗口，并在 Settings 主窗口弹出不含 secret 的成功提示。
- 不采集 Pixiv 账号密码；登录必须发生在 Pixiv 官方页面。
- 不在日志、文档、测试输出中打印完整 `PHPSESSID`；只允许输出是否存在、长度和非敏感元信息。
- 手动输入 Pixiv cookie 继续作为 fallback。
- 最新 build 已成功产出：
  `tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app`；
  `tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`。

建议下一阶段优先方向：

1. Gallery Quality / Thumbnail Cache：当前 Gallery 列表预览已稳定，但仍主要加载原图；建议生成真实小缩略图，
   降低 WebView 内存与网络压力，提升大图库滚动体验。
2. 清理收束：如用户确认，可删除 `.DS_Store`、`src/frontend/.next`、`src/frontend/out`、
   `tauri-app/src-tauri/target`、`tauri-app/node_modules` 等可再生成产物；未确认前不要删除。
3. P3a 小范围分发复核：手动打开最新 `.dmg`，拖入 Applications 后启动，确认测试用户安装体验。
4. Pixiv Refresh 体验微调：可选增加取消按钮、剩余等待时间、更细的错误提示；当前功能闭环已通过。
5. P1 后续数据正式化：如需更正式的数据分层，再评估 SQLite 是否从
   `~/Downloads/Pixiv Platform/` 迁移到 `~/Library/Application Support/Pixiv Platform/`。
6. P3 后续正式分发：签名、公证、自动更新仅作为未来路径；当前仍按未签名 `.dmg` 小范围分发。

建议如果进入 Gallery Thumbnail Cache：

1. 先读 `src/backend` 中图片下载、文件路径、`GET /api/images`、`GET /api/images/{id}/file` 相关代码。
2. 再读 `src/frontend/app/gallery/page.tsx` 的预览渲染和错峰加载逻辑。
3. 设计缩略图存储位置、生成时机、失败回退和迁移策略，不要先大改。
4. 验证优先跑相关后端测试、`cd src/frontend && npm run lint`、
   `cd src/frontend && NEXT_OUTPUT_EXPORT=1 npm run build`；需要完整产物时再跑
   `cd tauri-app && npm run build`。

渐进式披露规则：

- 只在当前任务需要时打开具体代码文件。
- 用 `rg` 精确搜索符号或路径。
- 不重复读取大文件，不重新总结整个仓库。
- 先给最小实现计划，再动代码。
- 遵循 spec-coding：实现后同步 `progress` / `checklist` / `testing`。

当前清理复核：

- `.DS_Store` 又出现在根目录、`static/` 和 Tauri bundle 目录。
- `src/frontend/.next`、`src/frontend/out`、`tauri-app/src-tauri/target`、`tauri-app/node_modules`
  因验证/build 重新生成。
- `src/frontend/.next/dev/lock` 曾存在；如确认没有 Next dev server 正在运行，可删除。
- 未获用户明确批准前不要删除这些文件。

默认不要 git commit、add、push，除非用户明确要求。
