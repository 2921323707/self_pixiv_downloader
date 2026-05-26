# Tauri Desktop Progress

## 当前锚点

日期：2026-05-26

阶段：Desktop MVP dev 模式、`.app` 打包 MVP、随机端口、健康检查、正式图标、Gallery 删除修复、P2 启动诊断补强、P1 桌面默认下载目录迁移与旧库自动恢复、P2 基础菜单、P3a 未签名 `.dmg` 最小闭环、Tauri Gallery 预览稳定性修复均已跑通；用户已完成 P2 手动测试并确认正常；已完成一次热缓存测速、P3a `.dmg` 验证与残留清理。
P3a 已产出未签名、未公证 `.dmg`，并完成本机挂载验证；随后已清理可再生成的本地构建产物，并在版本号锚定为 `v1.1.0` 后重新执行最终 build。2026-05-26 修复桌面 Gallery 偶发预览空白后再次执行 Tauri release build，当前本地已有更新后的 `1.1.0` `.app` / `.dmg` 产物。

用户确认：

- 使用 Tauri。
- 根目录 `tauri-app` 作为桌面应用源码位置。
- 文档使用中文。
- 增加测试规范、上下文管理规范、省 Token 思考和提示词策略。
- `tauri-app` 引用现有 `src/frontend` 和 `src/backend`，不复制业务代码。
- 后端通过 Cargo path dependency 复用现有 Rust crate。
- 先跑通 dev 模式。
- 允许轻改前端 API client。
- MVP 沿用项目 `output/`；应用数据目录迁移只是待办，未得到用户确认不做。
- 桌面端初版曾固定使用 `127.0.0.1:3000` 后端端口，当前已升级为随机端口。
- 验证标准 OK。
- 用户已进入 `tauri-app` 手动运行验证，确认 Tauri 桌面端体验与网页端一致，当前没有明显问题。
- 用户确认 `.app` MVP 生产前端使用 Next 静态导出。
- 用户确认 `.app` MVP 初版可简化处理并固定 `127.0.0.1:3000`。
- 用户确认 `.app` MVP 暂时沿用项目 `output/`。
- 用户手动测试新 `.app`，确认功能一切正常。
- 用户确认后续每次启动使用随机端口，并拼接为前端 API base URL。
- 用户手动测试随机端口版本 `.app`，确认功能成功。
- 用户手动测试 P0 收尾版本 `.app`，确认通过。
- 用户确认今日任务到此结束。
- 用户手动打开最新 `.app` 直接运行并测试，确认没有问题。
- 用户确认本轮优先做 P2 日志位置与用户可见启动失败提示。
- 用户要求进行测速、更新文档锚点、规划下一阶段、设计新会话首轮提示词，并盘点开发生成杂项。
- 用户确认桌面端默认目录迁移到 `~/Downloads/Pixiv Platform/`。
- 用户确认 Settings 中平台目录配置通过点击选择文件夹完成，不再要求手动输入文件夹路径。
- 用户反馈迁移后 `.app` 看不到旧图片和旧配置，并出现
  `Pixiv cookie is required in settings or PIXIV_PHPSESSID`；确认原因为新默认 SQLite
  路径创建了空库，未自动恢复旧 `output/pixiv_platform.sqlite3`。
- 用户手动测试旧库自动恢复后的最新 `.app`，确认当前阶段功能正常。
- 用户手动测试 P2 基础菜单后的最新 `.app`，确认正常。
- 用户确认按没有 Apple Developer Program / Apple 开发者认证背景推进 P3a 未签名 `.dmg`
  最小闭环，不做签名、公证、自动更新。
- 用户要求总结 macOS 桌面端完成度、清理残留文件、锚定当前任务进度，并考虑 GitHub commit。
- 用户反馈桌面 app Gallery 页面偶发随机预览图片刷不出来，但点进详情后正常；已定位为列表并发加载原图导致 macOS WebView 偶发预览空白，并完成稳定性修复。
- 用户确认该 Gallery 预览修复有效，要求锚定进度、更新文档、add、commit、push，并生成下一阶段规划和新会话提示词。

## 已完成

- 建立 Tauri 桌面化文档骨架。
- 明确 MVP 范围、非目标、架构、测试规范和上下文管理规范。
- 建立 `tauri-app/package.json` 与 `src-tauri` Tauri 项目骨架。
- `src-tauri` 通过 Cargo path dependency 引用 `../../src/backend`。
- Tauri 启动时在后台线程内创建 Tokio runtime，并启动现有 Axum 后端。
- Tauri dev 加载现有 `src/frontend` Next dev server。
- 前端 API client 支持可选 `NEXT_PUBLIC_PIXIV_BACKEND_URL`，默认仍保持 `/api`。
- Home / Gallery 图片预览 URL 也接入同一 API base URL 解析。
- 添加 MVP 临时 RGBA 图标，满足 Tauri dev 编译要求。
- Tauri build 生产模式加载 Next 静态导出产物 `src/frontend/out`。
- `.app` 初版静态导出生产环境 API base URL 曾指向 `http://127.0.0.1:3000`。
- Tauri release `.app` 打包成功。
- 固定端口占用失败时输出明确诊断日志。
- 修复 `.app` 静态页面直连本地 API 时的 CORS 问题。
- 新 `.app` 已通过用户手动功能测试。
- 后端启动改为绑定 `127.0.0.1:0` 随机可用端口。
- Tauri 改为程序化创建主窗口，并注入 `window.__PIXIV_PLATFORM_BACKEND_URL__`。
- 前端 `apiUrl()` 优先读取运行时注入的 API base URL，再回退环境变量与 `/api`。
- 静态导出产物不再写死 `127.0.0.1:3000`。
- 随机端口版本 `.app` 已通过用户手动功能测试。
- 后端增加轻量 `GET /api/health`。
- Tauri 启动随机端口后先轮询 `GET /api/health`，健康后再创建主窗口并注入运行时 API base URL。
- 健康检查超时时会向 stderr 输出包含 health URL、8 秒超时和最后一次错误的诊断。
- 使用用户提供图片替换 MVP 临时 App 图标。
- 主页面左上角品牌 logo 已替换为同一图标资源。
- P0 收尾版本 `.app` 已通过用户手动测试。
- 修复 Gallery 删除交互：Tauri `.app` 内选择图片后点击 Delete 不再依赖浏览器原生
  `window.confirm()`，改为应用内确认弹窗，确认后调用 `POST /api/images/delete-batch`。
- 最新 `.app` 可直接打开运行，用户手动测试通过。
- 桌面启动日志追加写入 `~/Library/Logs/Pixiv Platform/desktop.log`。
- 后端启动失败或健康检查超时时，Tauri 创建启动失败窗口，显示错误原因和日志文件路径。
- 完成一次热缓存测速和开发杂项盘点；未执行清理或删除。
- Tauri 桌面端默认下载根目录改为 `~/Downloads/Pixiv Platform/`，除非显式设置
  `PIXIV_DOWNLOAD_ROOT`。
- Tauri 桌面端默认 SQLite 路径改为
  `~/Downloads/Pixiv Platform/pixiv_platform.sqlite3`，除非显式设置
  `PIXIV_PLATFORM_DB_PATH`。
- Settings 页面 `download_base_path` 改为只读路径展示和桌面文件夹选择按钮；
  选择文件夹后自动保存到后端设置。
- 新增 Tauri 自定义 command `select_download_directory` 和对应 permission manifest。
- 修复 P1 迁移回归：Tauri 桌面端启动时如发现新桌面 SQLite 不存在，或新库为空且旧
  `output/pixiv_platform.sqlite3` 包含用户配置/图库，会自动恢复旧库。
- 自动迁移旧 `output/` 中除 SQLite 以外的下载文件到 `~/Downloads/Pixiv Platform/`，
  并把迁移后 SQLite 中旧 `output` 绝对图片路径改写到新下载目录。
- 新增 Tauri 原生基础菜单：About / Quit / Reload，Developer Tools 仅 debug 构建显示，
  同时保留常见 File / Edit / Window 菜单项。
- Tauri bundle targets 扩展为 `app` 和 `dmg`。
- 产出未签名、未公证 `.dmg`：
  `tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`。
- 新增 `tauri-app/docs/distribution.md`，记录 GitHub Release 小范围分发说明、Gatekeeper
  风险和测试用户无需安装 Rust、Cargo、Node、npm、TypeScript。
- 清理可再生成的本地残留：全部 `.DS_Store`、`src/frontend/.next`、`src/frontend/out`、
  `src/frontend/tsconfig.tsbuildinfo`、`tauri-app/src-tauri/target`、`tauri-app/node_modules`。
- 根 `.gitignore` 已补充忽略 `tauri-app/src-tauri/gen/`，避免提交 Tauri 自动生成的 schema 文件。
- 推荐下一版本号为 `v1.1.0`，并已同步 Tauri 桌面壳版本：
  `tauri-app/package.json`、`tauri-app/package-lock.json`、`tauri-app/src-tauri/Cargo.toml`、
  `tauri-app/src-tauri/Cargo.lock`、`tauri-app/src-tauri/tauri.conf.json`。
- 执行最终 release build 后，当前本地产物为
  `tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app` 和
  `tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`。
- 修复 Tauri Gallery 偶发预览空白：
  Gallery 列表预览改为错峰加载、`loading="lazy"`、`decoding="async"`，失败后最多重试两次；
  后端 `GET /api/images/{image_id}/file` 响应补充 `Content-Length`，并新增测试断言。
- Gallery 预览稳定性修复后重新执行 `cd tauri-app && npm run build`，成功产出更新后的
  `tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app` 和
  `tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`。

## 当前状态

- Desktop MVP P0 收尾、主页面 logo 替换、Gallery 删除交互修复、P2 启动诊断补强、
  P1 桌面默认下载目录迁移与旧库自动恢复、P2 基础菜单、P3a 未签名 `.dmg` 最小闭环、Tauri Gallery 预览稳定性修复完成并锚定；
  P2 `.app` 用户手动测试正常，P3a `.dmg` 本机挂载验证通过，热缓存测速已记录。当前版本号
  锚定为 `v1.1.0`，本地已有更新后的 `.app` / `.dmg` 构建产物，但这些构建产物由 `.gitignore`
  忽略，不纳入提交。
  当前无进行中的实现任务。

## 下一步

建议按低风险到高正式化程度推进：

1. P3a 人工分发复核：用户可手动打开最新 `.dmg`，拖入 Applications 后启动，确认小范围分发体验。
2. Gallery Quality / Thumbnail Cache：当前修复已稳定原图预览链路；下一阶段建议生成真实小缩略图，避免列表长期加载原图。
3. P1 后续数据正式化：如需更正式的数据分层，再评估 SQLite 是否从
   `~/Downloads/Pixiv Platform/` 迁移到 `~/Library/Application Support/Pixiv Platform/`。
4. P3 后续正式分发评估：签名、公证、自动更新仅作为未来路径；当前仍按未签名 `.dmg` 小范围分发。

新会话首轮提示词已整理到 `tauri-app/docs/next-session-prompt.md`。

## 验证记录

已通过：

```text
cd src/backend && cargo check
cd src/backend && cargo test api_health_returns_ok
cd src/frontend && npm run lint
cd src/frontend && npm run build
cd tauri-app && npm install
cd tauri-app/src-tauri && cargo check
cd tauri-app && npm run dev
cd src/frontend && NEXT_OUTPUT_EXPORT=1 npm run build
cd src/backend && cargo check
cd tauri-app && npm run build
test -f src/frontend/out/startup-error.html
curl -i http://127.0.0.1:3000/api/settings
curl -i http://127.0.0.1:3001/
```

验证结果：

- Next dev server 成功监听 `127.0.0.1:3001`。
- Tauri dev 成功编译并启动桌面进程。
- Tauri 进程内启动的 Rust 后端成功监听 `127.0.0.1:3000`。
- `GET /api/settings` 返回 `200 OK`，secret 仍以 `***` 遮罩。
- `GET /` 返回 `200 OK`。
- 用户手动验证 Tauri 桌面端体验与网页端一致，未发现明显问题。
- Next 静态导出成功生成 Home / Download / Gallery / Tasks / Settings。
- Tauri build 成功产出：
  `tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app`。
- `.app` 初版静态产物曾嵌入 `http://127.0.0.1:3000` API base URL。
- 用户复测旧 `.app` 时出现静态页面 `load failed`。
- 检查发现后端已监听 `127.0.0.1:3000`，`GET /api/settings` 可返回 `200 OK`，
  但旧后端未返回 CORS 响应头，`OPTIONS` 预检返回 `405`。
- 增加后端 CORS layer 后，备用端口验证通过：
  `GET /api/settings` 返回 `Access-Control-Allow-Origin: *`；
  `PUT /api/settings/theme_id` 预检返回 `200 OK`。
- 重新执行 `cd tauri-app && npm run build`，新的 `.app` 打包成功。
- 直接启动新 `.app` 内部可执行文件验证通过：
  后端监听 `127.0.0.1:3000`，`GET /api/settings` 返回 `200 OK` 和 CORS 响应头，
  `PUT /api/settings/theme_id` 预检返回 `200 OK`。
- 用户手动打开新 `.app` 并测试功能，确认一切正常。
- 随机端口版本验证通过：
  新 `.app` 内部可执行文件启动后监听 `127.0.0.1:52098`，
  `GET /api/settings` 返回 `200 OK` 和 CORS 响应头。
- 验证期间发现旧实例仍监听 `127.0.0.1:3000`，新随机端口实例可正常避开冲突。
- 静态导出产物检查确认不再包含 `127.0.0.1:3000` 字面量。
- 用户手动打开随机端口版本 `.app` 并测试功能，确认成功。
- 本轮 P0/P2 收尾验证通过：
  `cd src/frontend && npm run lint`；
  `cd src/backend && cargo check`；
  `cd tauri-app/src-tauri && cargo check`；
  `cd src/backend && cargo test api_health_returns_ok`；
  `cd src/frontend && NEXT_OUTPUT_EXPORT=1 npm run build`；
  `cd tauri-app && npm run build`。
- 最新 `.app` 产物：
  `tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app`。
- `tauri-app/src-tauri/icons/icon.png` 已替换为用户提供图片生成的 256x256 RGBA PNG。
- 用户手动测试 P0 收尾版本 `.app`，确认通过。
- 主页面左上角 logo 已替换为同源图标资源，`cd src/frontend && npm run lint` 通过。
- 用户打开旧 `.app` 仍看到默认 logo，经检查原因为前端源码改动后尚未重新静态导出和打包；
  已重新执行 `cd tauri-app && npm run build`，新的 `src/frontend/out/index.html` 已包含
  `<img src="/app-icon.png">`。
- Gallery 删除问题排查与修复：
  `cd src/backend && cargo test req_img_007` 通过，确认后端单张/批量删除接口与安全校验正常；
  前端将删除确认从 `window.confirm()` 改为应用内 modal；
  `cd src/frontend && npm run lint`、`cd src/frontend && NEXT_OUTPUT_EXPORT=1 npm run build`、
  `cd tauri-app && npm run build` 均通过；
  静态产物已包含 `delete-confirm-modal`。
- P2 启动诊断补强验证通过：
  `cd tauri-app/src-tauri && cargo check`；
  `cd src/frontend && npm run lint`；
  `cd src/frontend && NEXT_OUTPUT_EXPORT=1 npm run build`；
  `test -f src/frontend/out/startup-error.html`；
  `cd tauri-app && npm run build`。
- 验证期间曾并行触发 Tauri build 与独立 Next build，导致一次 `.next/lock` 竞争失败；
  随后单独重跑 `cd tauri-app && npm run build` 通过。
- 热缓存测速记录：
  `cd tauri-app/src-tauri && /usr/bin/time -p cargo check`：real 0.78s；
  `cd src/frontend && /usr/bin/time -p npm run lint`：real 1.07s；
  `cd src/frontend && /usr/bin/time -p env NEXT_OUTPUT_EXPORT=1 npm run build`：real 4.92s；
  `cd tauri-app && /usr/bin/time -p npm run build`：real 26.58s。
- P1 桌面默认下载目录迁移验证通过：
  `cd tauri-app/src-tauri && cargo check`；
  `cd src/frontend && npm run lint`；
  `cd src/frontend && env NEXT_OUTPUT_EXPORT=1 npm run build`；
  `cd tauri-app && npm run build`。
  首次把自定义 command 直接加入 capability 时，Tauri build 因缺少 app permission manifest
  失败；补充 `permissions/select-download-directory.toml` 后重跑通过。
- P1 旧库自动恢复修复验证通过：
  确认旧 `output/pixiv_platform.sqlite3` 存在且包含 `pixiv_cookie` 与 22 条图库记录；
  确认当前新桌面库存在但没有 `pixiv_cookie` 且图库为 0；
  `cd tauri-app/src-tauri && cargo check`；
  `cd tauri-app && npm run build`。
- 用户手动测试最新 `.app`，确认旧库自动恢复后当前阶段功能正常。
- P2 基础菜单验证通过：
  `cd tauri-app/src-tauri && cargo check`；
  `cd tauri-app/src-tauri && cargo check --release`；
  `cd tauri-app && npm run build`。
  新 `.app` 产物：
  `tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app`。
- 用户手动测试 P2 基础菜单后的最新 `.app`，确认正常。
- P3a 未签名 `.dmg` 最小闭环验证通过：
  `cd tauri-app/src-tauri && cargo check`；
  `cd tauri-app && npm run build`。
  首次在沙箱内执行 `npm run build` 时，`.app` 已产出，但 `bundle_dmg.sh` 因无法完成 macOS
  磁盘映像挂载/打包而失败；随后按权限规则在沙箱外重跑同一命令通过。
  新 `.dmg` 产物：
  `tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`。
  本机挂载验证通过：`.dmg` CRC 校验通过，挂载卷中存在 `Pixiv Platform.app` 和
  `Applications -> /Applications` 拖拽链接；验证后已卸载。
- 残留清理完成：
  删除全部 `.DS_Store`；
  删除 `src/frontend/.next`、`src/frontend/out`、`src/frontend/tsconfig.tsbuildinfo`、
  `tauri-app/src-tauri/target`、`tauri-app/node_modules`。
  清理后 `find . -name .DS_Store -print` 无输出；相关构建目录已不存在。
- `v1.1.0` 提交前最终验证：
  `cd tauri-app && npm install` 通过；
  `cd tauri-app && npm run build` 通过，产出
  `tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`。
  最终复核 `find . -name .DS_Store -print` 无输出。
- Tauri Gallery 预览稳定性修复验证通过：
  `cd src/frontend && npm run build`；
  `cd src/frontend && NEXT_OUTPUT_EXPORT=1 npm run build`；
  `cd src/backend && cargo test images::tests::req_img_004 --lib`；
  `cd src/backend && cargo test api::tests::req_img_002_req_ui_005_get_images_returns_gallery_metadata --lib`；
  `cd src/backend && cargo test api::tests::req_img_004 --lib`；
  `cd tauri-app && npm run build`。
  新 `.app` / `.dmg` 产物：
  `tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app`；
  `tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`。

观察到的非阻塞提示：

- `tauri info` 显示 Xcode Command Line Tools 已安装，完整 Xcode 未安装；MVP 不阻塞。
- Next dev 输出 `allowedDevOrigins` 未来版本提示；当前不阻塞，后续可在 dev 配置中处理。
- `baseline-browser-mapping` 提示数据较旧；当前不阻塞。
- shell 启动 npm 时提示 `pyenv: cannot rehash: /Users/Admin/.pyenv/shims isn't writable`；
  当前不阻塞构建。
- 本轮首次测速静态导出时，`/usr/bin/time -p NEXT_OUTPUT_EXPORT=1 npm run build` 因
  环境变量前缀写法不适配 `time` 直接执行而失败，实际测速使用
  `/usr/bin/time -p env NEXT_OUTPUT_EXPORT=1 npm run build`。

## 注意事项

- 当前工作区已有用户既有改动：`src/frontend/app/gallery/page.tsx`、
  `src/frontend/app/page.tsx`、`src/frontend/lib/api.ts` 等。不要覆盖或回退。
- `tauri-app/.DS_Store` 已存在，暂不删除。
- 清理项已复核：正式图标已替换，`target/` 与 `node_modules/` 仍作为本地构建/依赖产物处理。
- 前端新增 `src/frontend/public/app-icon.png`，用于主页面左上角品牌 logo。
- 前端新增 `src/frontend/public/startup-error.html`，用于 Tauri 后端启动失败或健康检查失败时显示可见错误。
- Gallery 删除修复已重新打包进最新 `.app`，用户已手动打开直接运行并确认无问题。
- 默认不做 git commit。
- Web / 后端独立运行默认仍沿用项目 `output/`；Tauri 桌面端默认目录已迁移到
  `~/Downloads/Pixiv Platform/`。
- 若用户已打开过上一版 `.app` 产生空的新桌面库，最新 `.app` 下次启动会先备份该空库，
  再从旧项目 `output/` 自动恢复用户配置和图库。

## 清理盘点

本轮只盘点，未删除：

- `.DS_Store` 分布在根目录、`tauri-app/`、`tauri-app/src-tauri/`、`tauri-app/src-tauri/target/`、
  `docs/`、`static/`、`tests/`、`src/`、`src/backend/`、`output/` 等位置；根 `.gitignore` 已忽略。
- `src/frontend/.next` 约 49M，是 Next 构建缓存；可清理，下一次构建会重建。
- `src/frontend/out` 约 1.2M，是当前 Tauri 生产静态前端产物；清理后需重新 build。
- `tauri-app/src-tauri/target` 约 3.0G，是 Rust/Tauri 构建产物；可清理但会显著增加下次 build 时间。
- `tauri-app/node_modules` 约 14M，是 Tauri 壳依赖；可清理但需重新 `npm install`。
- `src/frontend/.next/dev/lock` 存在；如确认没有 Next dev server 正在运行，可删除。
- `src/frontend/tsconfig.tsbuildinfo` 存在且已被 `.gitignore` 覆盖；可清理，下一次 TypeScript 检查会重建。
