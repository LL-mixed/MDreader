# MDreader

只读的 Markdown 阅读器，把 `.md` 文件渲染成排版精良的阅读界面，并注册为系统级 `.md` 文件打开者——在微信、文件管理器、Finder、GNOME 文件等处点开 `.md` 时可选择本应用打开。打开过的文件自动缓存到应用私有空间，可按日期/标题浏览、搜索、收藏、删除。

同一套渲染资源（marked + highlight.js + KaTeX + Mermaid + 自研 GitHub 风格 CSS），四个原生壳：

| 平台 | 语言 / UI | WebView | 缓存 |
| --- | --- | --- | --- |
| Android | Kotlin / Jetpack Compose | Android WebView | Room + 内部存储 |
| macOS | Swift / SwiftUI | WKWebView | SwiftData + App Support |
| Linux | Rust / GTK4 | WebKitGTK 6 | SQLite (rusqlite) + `$XDG_DATA_HOME` |
| Windows | C# / WinUI 3 | WebView2 | SQLite + `%LOCALAPPDATA%` |

四端共用 `shared/render/`（渲染资源唯一来源），各自打包；纯工具逻辑（哈希、SVG/Mermaid 预处理、日期分桶等）按端重写。

## 功能

- **渲染**：GitHub 风格排版，代码高亮、表格、数学公式、任务列表、Mermaid 图、内联 SVG，明暗主题
- **文件打开者**：注册 `.md` / `text/markdown`，从外部点开即用本应用阅读
- **缓存**：打开即落盘，SHA-256 正文去重，元数据入库
- **内容管理**：按日期分组的列表、标题/正文搜索、详情、收藏、删除
- **大纲**：从 DOM 标题抽取，点击跳转、滚动高亮当前章节
- **缩放**：30%–300%，按文件持久化

## Android

```bash
cd android
./gradlew :app:assembleDebug          # debug APK
./gradlew :app:testDebugUnitTest      # JVM 单测
./gradlew :app:installDebug           # 装到已连接设备
```

要求 JDK 17，`local.properties` 里设 `sdk.dir`。模拟「从外部打开」：

```bash
adb shell am start -a android.intent.action.VIEW \
  -d "file:///sdcard/README.md" -t text/markdown com.mdreader/.MainActivity
```

Release 签名凭据放 `local.properties`（已 gitignore）：

```properties
mdreader.storeFile=<绝对路径>/mdreader.jks
mdreader.storePassword=<密码>
mdreader.keyAlias=mdreader
mdreader.keyPassword=<密码>
```

## macOS

```bash
cd macos
xcodegen generate
xcodebuild -project MDreader.xcodeproj -scheme MDreader \
  -configuration Debug -destination 'platform=macOS' build
xcodebuild -project MDreader.xcodeproj -scheme MDreader \
  -destination 'platform=macOS' test
```

需要 `brew install xcodegen`，最低 macOS 14。`.md` UTI 在 `Info.plist` 声明，Finder「打开方式」、双击、拖拽均可。

## Linux

```bash
cd linux
cargo build --release          # 出二进制 mdreader
cargo test                     # 纯逻辑单测
cargo run -- path/to/file.md   # CLI 打开 .md
./scripts/install.sh           # 用户级安装：binary + .desktop + 图标 + metainfo
./scripts/install.sh --set-default   # 同时设为 .md 默认打开方式
```

依赖 `libgtk-4-dev` 与 `libwebkitgtk-6.0-dev`，Rust ≥ 1.74。`.desktop` 已声明 `MimeType=text/markdown;`，外部 http(s) 链接交系统浏览器，编辑器可配置（`code` / `typora` 等命令，argv 直起不经 shell）。

## Windows

```powershell
cd windows
dotnet build MDreader.sln -c Release          # 构建
dotnet test MDreader.sln                       # xUnit 单测
.\scripts\build.ps1                            # 出 self-contained 便携 zip
.\scripts\install.ps1                          # 用户级安装：解压 + 注册 .md 关联
.\scripts\install.ps1 -SetDefault              # 注册后引导系统设置选默认
```

需 .NET 8 SDK，最低 Win10 1809（WebView2 运行时 Win10/11 预装）。便携 zip 解压即用、免装 runtime、免签名。编辑器可配置（`code` / `typora` 等命令，argv 直起不经 shell）；外部链接交默认浏览器。

## 目录结构

```
MDreader/
├── android/    # Gradle 工程（独立根）
├── macos/      # xcodegen 工程（project.yml 为唯一来源）
├── linux/      # cargo + GTK4 + WebKitGTK6
├── windows/    # dotnet + WinUI 3 + WebView2
├── shared/     # 跨端渲染资源（render/ + sample.md）
├── tools/      # 辅助脚本（图标生成等）
└── docs/       # 设计文档
```

详细约定见 [CLAUDE.md](CLAUDE.md)。
