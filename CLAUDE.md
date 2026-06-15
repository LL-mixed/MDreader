# MDreader — Android Markdown Reader

## 项目目标

一个 Android 平台的 **Markdown 阅读器**（只读，不做编辑）。核心价值：

1. **美观渲染**：把 `.md` 渲染成排版精良、支持代码高亮、表格、数学公式、任务列表、明暗主题的阅读界面。
2. **系统级文件打开者**：注册为 markdown 文件的默认打开方式之一。在微信、文件管理器等 app 中点开 `.md` 时，可选择本 app 打开。
3. **缓存与内容管理**：通过外部 app 打开的 md 会**自动缓存**到本 app 私有空间（持久化），并可按「日期 / 内容」浏览、搜索、删除、收藏。

## 技术栈与关键决策（含 why）

| 决策点 | 选择 | 为什么 |
| --- | --- | --- |
| 语言 | Kotlin | Android 官方首选，生态成熟 |
| UI 框架 | Jetpack Compose | 现代、声明式、默认美观、样板代码少；列表/导航/界面外壳全部用 Compose |
| Markdown 渲染 | **WebView + 本地 JS 引擎**（marked.js + highlight.js + KaTeX）+ 精修 CSS | 「美观」是最高准则。WebView 方案能稳定呈现表格/公式/代码高亮/任务列表，且一套 CSS 即可做到 GitHub 级排版；纯原生（Markwon）在表格与公式上成本高 |
| 最低 SDK | 24（Android 7.0） | 覆盖 ~98% 设备，且拥有现代 API |
| Target/Compile SDK | 34 | 当前主流 |
| 构建 | Gradle Kotlin DSL + Version Catalog（libs.versions.toml） | 现代、可复现、依赖集中管理 |
| 缓存元数据 | Room | 需按日期/内容查询与观察，关系型数据库最合适 |
| 缓存正文 | App 内部存储文件（每条一份 `.md`） | 正文体积可变，不适合塞进 DB；文件按 id 命名 |
| 内容去重 | 正文 SHA-256 作为唯一键 | 同一文件多次打开不重复占用空间 |

> 这些决策是默认方案，若有更好的第一性路径直接提出来改文档、再改实践。

## 目录结构约定

```
MDreader/
├── CLAUDE.md                      # 本文件：项目约定
├── settings.gradle.kts            # 工程设置
├── build.gradle.kts               # 根构建脚本
├── gradle/libs.versions.toml      # 依赖版本目录
├── gradle/wrapper/                # Gradle Wrapper
├── gradlew, gradlew.bat
├── app/
│   ├── build.gradle.kts
│   ├── proguard-rules.pro
│   └── src/
│       ├── main/
│       │   ├── AndroidManifest.xml
│       │   ├── java/com/mdreader/   # 源码，包名 com.mdreader
│       │   │   ├── MainActivity.kt
│       │   │   ├── ui/              # Compose 界面（主题、组件、屏幕）
│       │   │   ├── data/            # Room 实体、DAO、数据库、仓库
│       │   │   ├── render/          # WebView 渲染器与资源装载
│       │   │   └── util/            # 工具（哈希、时间格式化等）
│       │   └── res/                 # 图标、字符串、主题等资源
│       └── test/                    # 单元测试（JVM）
│           └── java/com/mdreader/
└── docs/                           # 设计文档、截图（可选）
```

命名约定：包 `com.mdreader`；类名 PascalCase；资源 snake_case；代码与变量英文，注释/文档/commit message 之外的面向用户文本中文。

## 构建与验证流程

- **每次功能改动后必须能通过构建**：`./gradlew assembleDebug`（或至少 `./gradlew :app:compileDebugKotlin`）成功。
- **任何功能都必须有命令行入口 + 测试用例**（见全局准则）：纯逻辑（哈希、文件名、模板拼装等）走单元测试；UI/Intent 行为留 instrumentation 测试或手动验证清单。
- **每次改动后完整通过所有测试用例**，再提交。
- 工具链：JDK 17 + Android SDK（platform-tools、platforms;android-34、build-tools;34.0.0），环境变量 `ANDROID_HOME` 指向 SDK 根。

## 增量交付里程碑

每个里程碑：实现 → build 通过 → 测试通过 → git 提交 → 继续。

1. **M1 脚手架**：Gradle 工程、空 Activity、能 `assembleDebug` 出 APK、能装、能跑。
2. **M2 渲染内核**：WebView + 本地 JS/CSS，能渲染内置样例 md，明暗主题。
3. **M3 文件打开者**：Manifest 注册 intent-filter，能从外部打开 `.md`/`text/markdown`。
4. **M4 缓存层**：Room 元数据 + 内部存储正文 + SHA-256 去重，打开即缓存。
5. **M5 内容管理**：列表（按日期分组）、搜索、详情、删除、收藏。
6. **M6 图标与发布**：应用图标、名称、Release APK。

## 编码约定

- 不使用 shell 脚本改动代码；单次少量修改。
- 行末不留空字符；源码统一 Unix（LF）换行，**禁止 DOS**。
- 不为让代码跑起来而注释掉报错，找根因。
- 密钥/token/密码不进代码。
- commit message 英文、简洁描述意图；**不**加 `Co-Authored-By`。
- `git push` 仅用于跨设备同步，不自动执行，等用户指示。
