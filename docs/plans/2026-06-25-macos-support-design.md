# macOS 平台支持设计

日期：2026-06-25
状态：已确认，实施中

## 目标

在现有 Android 实现基础上，新增 macOS 原生 GUI 阅读器，最大复用已实现的 common 组件。repo 重组为 monorepo。

## 关键决策（含 why）

| 决策点 | 选择 | 为什么 |
| --- | --- | --- |
| macOS 技术栈 | Swift + SwiftUI + WKWebView | 项目灵魂是「WebView + 一套精修 CSS = 美观」。macOS 用 WKWebView 加载同一套渲染资源，零成本复用整个渲染核心；原生体验最佳；系统级注册 `.md` 打开(UTI)原生支持。纯 Kotlin 工具逻辑量极小，用 Swift 重写成本可忽略 |
| 目录结构 | `android/` + `macos/` + `shared/` 彻底 monorepo | 对称清晰；shared 作为跨端 common 唯一来源 |
| 工程管理 | xcodegen 声明式生成 `.xcodeproj` | project.yml 可读、可 diff、可复现，贴合 monorepo 与「命令行入口」准则 |

## 复用边界（核心）

`shared/` 是跨端 common 的唯一来源，**只放平台无关的渲染资源**：
- `shared/render/`：`index.html`、`render.js`、`render.css`、`marked.min.js`、`highlight.min.js`、`katex/`、`mermaid.min.js`
- `shared/sample.md`：内置样例文档

Kotlin 纯工具逻辑（`ContentHash`/`DateBuckets`/`Titles`/`SvgGuard`/`MermaidFenceNormalizer`/`OutlineItem`）**不进 shared/**（跨语言无意义），由 Swift 在 macOS 端重写。

WebView 双向桥是平台特定的：Android 走 `@JavascriptInterface`，macOS 走 `WKScriptMessageHandler`，两端各自实现，但 `render.js` 共享。

## 目标目录结构

```
MDreader/
├── android/                 # 完整 Android Gradle 工程（整组迁入）
│   ├── settings.gradle.kts  ├── gradlew / .bat / gradle.properties
│   ├── build.gradle.kts     ├── gradle/{libs.versions.toml,wrapper/}
│   └── app/
│       ├── build.gradle.kts # assets.srcDir 指向 ../../shared
│       └── src/main/{java,res,AndroidManifest.xml}
├── macos/                   # xcodegen 工程
│   ├── project.yml          # 声明式工程（唯一来源）
│   └── MDreader/            # Swift 源码（.xcodeproj 为生成物，gitignore）
│       ├── App/  ContentView/  render/  data/  util/
│       └── Tests/
├── shared/                  # 跨端 common，唯一来源
│   ├── render/              # index.html render.js render.css marked/highlight/katex/mermaid
│   └── sample.md
├── docs/  tools/  .gitignore  CLAUDE.md
```

## 资源共享机制（解决 DRY）

- `shared/render/` 是唯一来源。
- **Android**：`app/build.gradle.kts` 加 `assets.srcDir`（解析自 `rootProject.projectDir.parentFile/shared`）→ `shared/render/*` 落到 APK 的 `assets/render/`，`shared/sample.md` 落到 `assets/sample.md`，与现状逐字节一致。
- **macOS**：`project.yml` 用 folder reference 引 `../shared/render`，构建时整体拷进 bundle；WKWebView 用 `loadFileURL` 加载 `index.html`。

## Kotlin → Swift 工具逻辑映射

| Kotlin | Swift | 说明 |
| --- | --- | --- |
| `ContentHash` (SHA-256) | CryptoKit `SHA256` | 1:1 |
| `DateBuckets` | Foundation `Calendar` | 日期分组 |
| `Titles` | Swift | 标题提取 |
| `SvgGuard` / `MermaidFenceNormalizer` | `NSRegularExpression` | 正则逻辑平移 |
| `OutlineItem` | `struct` | 数据模型 |

## 迁移步骤（git mv 保留历史）

1. 建 `android/`，git mv 根级 Gradle 文件入内：`settings.gradle.kts`、`build.gradle.kts`、`gradle.properties`、`gradlew(.bat)`、`gradle/`。
2. `git mv app/ android/app/`（`:app` 模块路径不变，settings 无需改 include）。
3. 建 `shared/`，`git mv android/app/src/main/assets/render → shared/render`；`git mv .../assets/sample.md → shared/sample.md`；删空 `assets/`。
4. `android/app/build.gradle.kts` 加 `assets.srcDir` 指向 shared。
5. 建 `macos/`：`project.yml` + Swift 骨架。

## .gitignore 调整（根级集中，加路径前缀）

- Android 产物：`android/**/build/`、`android/.gradle/`、`android/local.properties`、`*.apk`、`*.aab`
- macOS 产物：`macos/**/*.xcodeproj`（xcodegen 可再生）、`**/xcuserdata/`、`**/DerivedData/`
- 通用：`.DS_Store`

## 构建与验证命令

- macOS（一次性 `brew install xcodegen`）：
  `cd macos && xcodegen generate && xcodebuild -project MDreader.xcodeproj -scheme MDreader -configuration Debug build`
- Android（`JAVA_HOME=/opt/homebrew/opt/openjdk@17`）：
  `cd android && ./gradlew assembleDebug && ./gradlew :app:testDebugUnitTest`

## 本次交付范围

- macOS **MM1**（工程骨架可构建）+ **MM2**（WKWebView 加载 shared/render 渲染 sample.md）——MM2 验证复用架构真成立。
- Swift 工具逻辑配 `XCTest` 单测（对应 CLAUDE.md「任何功能都要有测试」）。
- MM3–MM6（文件打开者/缓存层/内容管理/图标发布）作为后续里程碑。

## CLAUDE.md 同步更新

目录结构改 monorepo、技术栈表加 macOS 行、构建命令分平台、新增 macOS 里程碑。
