# MDreader Windows 端口设计（C# + WinUI 3 + WebView2）

日期：2026-07-01 ｜ 状态：已与用户确认，开始实现

## 目标与动机

为 MDreader 增加 Windows 原生端，**功能与 macOS / Linux 完全对等**。动机：作者在 Windows 上有实际（虽不频繁）的 Markdown 阅读需求，三端覆盖后补齐第四个主流平台。沿用项目既有架构原则：**每端 = 原生语言 + 原生工具包 + 原生 WebView，全部加载同一份 `shared/render/`（物理唯一来源，零改动）**。

## 技术决策（含 why）

| 决策点 | 选择 | 为什么 |
| --- | --- | --- |
| 语言/UI | C# / **WinUI 3**（Windows App SDK，.NET 8 LTS） | Windows 最原生栈；延续「每端选该平台最强原生栈」哲学（对应 macOS=SwiftUI、Linux=GTK4）。控件齐全（`NavigationView`/`CommandBar`），符合「美观是最高准则」 |
| WebView | **WebView2**（Edge Chromium，Win10 1809+ 预装 runtime） | 微软官方桌面 WebView 事实标准；与 WKWebView / WebKitGTK 并列，各端原生 WebView |
| 最低版本 | Win10 1809+ / .NET 8 LTS | WebView2 + WinUI 3 的底线；覆盖绝大多数现役 Windows |
| 渲染资源加载 | `SetVirtualHostNameToFolderMapping("app.local", ...)` | 把虚拟主机 `app.local` 映射到打包后的 `shared/render` 目录，页面走 `https://app.local/...` 同源加载；对应 macOS bundle / Linux `mdreader://` 自定义 scheme |
| 缓存元数据 | `Microsoft.Data.Sqlite` + 手写 SQL | 关系型查询/观察最合适（对应 Android Room / macOS SwiftData / Linux rusqlite）；手写 SQL 与 rusqlite 风格对应，便于对照移植 |
| 缓存正文 | `%LOCALAPPDATA%\MDreader\docs\<uuid>.md` | 正文体积可变，不入库；按 id 命名（对应 macOS App Support / Linux `$XDG_DATA_HOME`） |
| 内容去重 | 正文 SHA-256 | 四端一致 |
| 配置/会话 | `%LOCALAPPDATA%\MDreader\*.json` | Windows 应用数据规范；对应 macOS `~/.mdreader/*.json` / Linux `$XDG_CONFIG_HOME` |
| 发布形态 | **便携 zip**（`dotnet publish -r win-x64 --self-contained` 单文件夹） | 对齐 Linux `install.sh` 模式；解压即用、免装 runtime、免签名证书；贴合「个人工具、用得不多」定位。MSIX 签名成本对本项目不划算 |
| 工程 | `.sln` + `csproj`（SDK-style） | 原生构建工具；`dotnet build/test/publish` 统一命令行入口，对齐 gradle / xcodegen / cargo 取向 |

## 跨端渲染契约（零改动复用 `shared/render`）

`render.js` 只认 `window.mdreaderNative` 对象（见 `render.js:5/8/41/187/220`），方法：同步 `getMarkdown()/getDark()/getSvg(i)`，回调 `markRendered()/onOutline(json)/onActiveHeading(i)`；外加 `window.MDreader.render()` 与 `scrollToHeading(i)`。

**桥接手法（与 macOS/Linux 同构，仅换宿主 API）**：同步读转为读「预置 payload」`window.__mdrPayload = {md,dark,svgs}`（document-start 注入），异步回调走 `chrome.webview.postMessage(...)`（C# 侧 `WebMessageReceived` 接收），宿主→JS 走 `CoreWebView2.ExecuteScriptAsync(...)`。Windows 用 `AddScriptToExecuteOnDocumentCreatedAsync` 注入 shim，把上述机制包成 `window.mdreaderNative`——**`render.js` 一行不改**。

预处理流水线（native，在 `getMarkdown()` 前执行，对照 Rust/Swift 移植）：`resolve_images`（raster→base64 / svg→inline）→ `mermaid_fence` 归一化 → `svg_guard` 抽离。

## 功能对等映射

| macOS / Linux 能力 | Windows 实现 |
| --- | --- |
| 渲染内核 | WebView2 + `SetVirtualHostNameToFolderMapping`，`https://app.local/render/index.html` |
| JS 桥 / payload | `AddScriptToExecuteOnDocumentCreated`（document-start）+ payload + `WebMessageReceived` |
| 明暗主题 | `body.dark/light` + WinUI 3 系统主题跟随（`Application.RequestedTheme`） |
| 缩放 30–300% | `CoreWebView2.ZoomFactor` + 工具栏 −/百分比/+/重置 + Ctrl±0 + Ctrl 滚轮 |
| 大纲 TOC + 滚动高亮 | 同款 `onOutline/onActiveHeading/scrollToHeading` + `NavigationView` 大纲页 |
| 外部链接 | `Process.Start(url)` 交默认浏览器（对齐 Linux，非 mac webview 内加载） |
| 缓存元数据 | `Microsoft.Data.Sqlite` 表 `cached_docs`（镜像 `CachedDoc` / linux `doc_store` 字段） |
| SHA-256 去重 / 正文存储 | `System.Security.Cryptography.SHA256` + `<uuid>.md` 文件 |
| 内容管理 | `NavigationView`（库/大纲切换）+ `LibraryPage` 列表 + `DateBuckets` + 右键 `MenuFlyout`（新窗口/刷新/收藏/删除） |
| 文件关联 | `install.ps1` 写 `HKCU\Software\Classes` ProgId + `.md` 关联；命令行/`OnActivated` 处理打开 |
| 拖拽 | WinUI `DragOver`/`Drop`（窗口级） |
| 会话/缩放/设置 | `%LOCALAPPDATA%\MDreader\*.json`（session/zoom/settings） |
| 外部编辑器 | `Process.Start(editorCmd, file)`，argv 直起不经 shell（对齐 Linux「命令」语义，如 `code`/`notepad++`） |
| PDF 导出 | `CoreWebView2.PrintToPdfAsync(path, settings)`（直接生成 PDF，比 mac/linux 打印对话框更干净） |
| 关于窗口 | WinUI `ContentDialog` 或 `Window` + 构建时注入 git hash |
| 多文档 | WinUI 3 多窗口；库内点击载入当前窗口，右键「新窗口打开」（对应 mac `WindowTabber` / linux 新窗口） |

## 目录结构

```
windows/
├── MDreader.sln
├── MDreader/                    # WinUI 3 应用（Windows App SDK）
│   ├── App.xaml(.cs)            # 入口；OnActivated 处理 .md 打开
│   ├── MainWindow.xaml(.cs)     # NavigationView 主壳（库/大纲切换 + 工具栏）
│   ├── Pages/
│   │   ├── LibraryPage.xaml(.cs)   # 列表（日期分组）+ 搜索
│   │   └── OutlinePage.xaml(.cs)   # 大纲
│   ├── Render/
│   │   ├── MarkdownWebView.cs     # WebView2 包装（加载/缩放/打印/导航策略）
│   │   ├── BridgeShim.cs          # mdreaderNative shim + payload 注入
│   │   └── PayloadBuilder.cs      # resolve_images 预处理
│   ├── Store/
│   │   ├── DocStore.cs            # sqlite 连接 + schema
│   │   ├── DocRepository.cs       # CRUD（返回 DocInfo，避免离线对象）
│   │   ├── SessionStore.cs
│   │   ├── ZoomStore.cs
│   │   └── SettingsStore.cs
│   ├── Util/
│   │   ├── ContentHash.cs         # SHA-256
│   │   ├── SvgGuard.cs
│   │   ├── MermaidFenceNormalizer.cs
│   │   ├── Fence.cs
│   │   ├── DateBuckets.cs
│   │   └── Titles.cs
│   ├── Assets/                    # 图标（复用 ../shared 或 mac/linux PNG）
│   └── MDreader.csproj            # PackageReference：Microsoft.WindowsAppSDK + Microsoft.Data.Sqlite
├── MDreader.Tests/               # xUnit 单测（纯逻辑）
│   └── MDreader.Tests.csproj
├── scripts/
│   ├── install.ps1               # 用户级安装：解压 + 注册 .md ProgId + 图标 + --set-default + --uninstall
│   └── build.ps1                 # 本地打包：dotnet publish -r win-x64 --self-contained → zip
└── .gitignore                    # bin/ obj/ *.user publish/
```

## 发布形态与 install.ps1（含 Windows 平台限制）

**便携 zip**：`dotnet publish -c Release -r win-x64 --self-contained` 产出单文件夹（含 WebView2 runtime 之外的 .NET runtime，约 150MB；解压即用，不要求用户预装 .NET）。

**`install.ps1`**（对齐 linux `install.sh`，用户级 `HKCU`，免管理员）：
- 解压/复制到 `%LOCALAPPDATA%\MDreader`
- 写 `HKCU\Software\Classes\MDreader.md`（ProgId，含命令行 + 图标）+ `HKCU\Software\Classes\.md\OpenWithProgids` 关联
- 复制图标到 `%LOCALAPPDATA%\MDreader\icon.ico`
- `--set-default`：注册 ProgId 后**提示用户去「系统设置 > 应用 > 默认应用 > 按 .md 选择 MDreader」**
- `--uninstall`：清理上述注册表项 + 删除 `%LOCALAPPDATA%\MDreader`

**有意分歧（非缺陷）——Windows `UserChoice` 保护**：Win10/11 的「默认应用」`HKCU\...\Explorer\FileExts\.md\UserChoice` 有 hash 保护（`ProgIdHash`），脚本无法静默伪造，强写会被系统重置。因此 `--set-default` 不能像 Linux `xdg-mime` 那样静默设默认，只能注册 ProgId 并引导用户手动确认。这是平台限制，非实现缺陷。

## 增量里程碑（WM1–WM6，对齐 LM/MM 命名）

1. **WM1 脚手架**：WinUI 3 空壳 `Window` + WebView2 + `SetVirtualHostNameToFolderMapping` 加载 `shared/render` 渲染 `sample.md` + `mdreaderNative` shim 骨架；`dotnet build` 出 exe。
2. **WM2 渲染内核**：完整桥（payload + `WebMessageReceived` + `ExecuteScriptAsync`）+ payload 预处理；明暗主题；移植 `ContentHash`/`SvgGuard`/`MermaidFenceNormalizer`/`Fence` + xUnit 向量对齐 mac/linux。
3. **WM3 文件打开者**：`App.OnActivated`（FileActivated）/命令行参数处理 `.md`；`install.ps1` 注册 `.md` ProgId；窗口级拖放。
4. **WM4 缓存层**：`Microsoft.Data.Sqlite` 元数据 + `%LOCALAPPDATA%\MDreader\docs\<uuid>.md` 正文 + SHA-256 去重；移植 `DocStore`/`DocRepository` + 单测。
5. **WM5 内容管理**：`NavigationView` 库/大纲切换 + 日期分组列表 + 搜索 + 右键 `MenuFlyout`（新窗口/刷新/收藏/删除）+ 大纲（DOM 标题 + 滚动高亮）+ 缩放（按 content-hash 持久化）+ session restore。
6. **WM6 发布**：图标（复用 PNG 打进 `Assets/`）+ 外部编辑器（`Process.Start`，配置走 SettingsStore）+ PDF 导出（`PrintToPdfAsync`）+ 关于对话框（构建时注入 git hash）+ release zip + `install.ps1`/`build.ps1` + **CI 加 `build-windows` job**。

## CI 变更（`.github/workflows/release.yml`）

加 `build-windows` job：`windows-latest` runner → `actions/setup-dotnet@v5`（.NET 8）→ `dotnet publish windows/MDreader -c Release -r win-x64 --self-contained` → 压 zip → `actions/upload-artifact@v5`（name: `mdreader-windows`）。`release` job 的 `needs` 追加 `build-windows`，`files` glob 追加 `artifacts/mdreader-windows/*`。

## 有意分歧（非缺陷，对齐 linux 设计文档风格）

1. **默认应用设置**：`--set-default` 无法静默（`UserChoice` 保护，见上），引导用户手动确认；对应 Linux `xdg-mime` 可静默。
2. **外部链接**：交系统浏览器打开（对齐 Linux），不做 mac 的 webview 内加载 + 返回按钮。
3. **编辑器配置**：为「命令」语义（`code`/`notepad++`），argv 直起不经 shell；对应 Linux，非 mac 的 `open -a <app>`。
4. **不做 macOS `WindowTabber` 式标签合并**：Windows 用原生多窗口（每文档一窗口），与 GNOME 一致。

## 跨端影响（文档/工程需同步）

- `shared/render/`：**零改动**（核心红利）。
- `AGENT.md`：技术决策表加 Windows 行；新增「Windows 里程碑 WM1–WM6」段；目录结构图加 `windows/`。
- `README.md`：平台表加 Windows 一行；新增「Windows」一节（构建/安装命令）。
- `.gitignore`：加 `windows/**/bin/`、`windows/**/obj/`、`windows/**/*.user`、`windows/publish/`。
