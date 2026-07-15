# Outline（大纲）功能设计

日期：2026-06-25
状态：已确认，待实现

## 目标

在 Markdown 阅读界面展示文档大纲（基于 `h1~h6` 标题层级），并支持点击大纲项跳转到正文对应位置；长文档滚动时高亮当前所在标题。

## 交互设计

- TopAppBar 右上角新增「目录」图标（`Icons.AutoMirrored.Filled.List`）。
- **竖屏（`screenWidthDp < 600`）**：点击图标从右侧滑出 `ModalNavigationDrawer`；点击大纲项 → 关闭抽屉并跳转到对应标题。
- **横屏 / 大屏（`screenWidthDp ≥ 600`）**：使用 `PermanentNavigationDrawer`，大纲常驻，点击即跳转。
- **当前位置高亮**：JS 侧用 `IntersectionObserver` 监听标题可见性，回传当前最靠上的可见标题索引，大纲对应项高亮（竖屏抽屉打开时同样生效）。
- 大纲项样式：按 `level` 缩进，标题文本 `maxLines = 2` + `TextOverflow.Ellipsis`。

## 数据与跳转来源（关键决策）

**大纲数据（索引 + 层级 + 文本）与跳转定位均以渲染后的 DOM 为唯一权威来源。**

| 用途 | 来源 | 理由 |
| --- | --- | --- |
| 大纲数据（index + level + text）+ 跳转定位 | DOM（JS），唯一权威 | 与最终渲染结果天然一致，索引零错位 |

`render.js` 在 `marked.parse` 后遍历 `#content` 下的 `h1~h6`，按出现顺序分配 `id="mdr-h-{index}"`，并一次性把 `[{ index, level, text }]` 回传给 Kotlin（`textContent` 取纯文本）。点击大纲项 → 调 `window.MDreader.scrollToHeading(index)` → `getElementById('mdr-h-'+index).scrollIntoView()`。数据与跳转共用同一套 DOM 索引，永不错位。

**为何不用 Kotlin 解析 markdown 标题（原方案）**：跳转要求大纲索引与 DOM 标题序列严格一一对齐。若 Kotlin 只解析 ATX，遇到 Setext 标题（`===`/`---`）或 HTML `<h2>` 标题就会与 marked 输出的 DOM 序列错位；`min()` 兜底只能截断数量、不能重排，会导致「点 A 跳到 B」。在 Kotlin 精确复刻 marked 的段落/标题解析成本高，且会与 marked 版本脱节（违反 DRY），故改由 DOM 为唯一权威。

**代价**：标题提取属渲染行为，归入手动验证清单（AGENT.md 允许渲染/UI 行为走此路径），无 JVM 单测。

## 组件拆分

- `data class OutlineItem(val index: Int, val level: Int, val text: String)`（新，放 `render` 包）：大纲项模型。
- `render.js`：
  - `render()` 末尾：`indexHeadings()` 遍历 `#content` 下 `h1~h6`，按顺序分配 `id="mdr-h-{index}"`，组装 `[{index, level, text}]`（`textContent`）→ 经 `mdreaderNative.onOutline(json)` 一次性回传。
  - 新增 `scrollToHeading(index)`：`getElementById('mdr-h-'+index).scrollIntoView({block:'start'})`。
  - 新增 IntersectionObserver：只回传变化的当前可见标题索引（记录 `lastActive` 去抖），`mdreaderNative.onActiveHeading(index)`。
  - 全部暴露到 `window.MDreader`。
- `render/MarkdownView.kt`：
  - `SourceBridge` 新增 `@JavascriptInterface onOutline(json)` / `onActiveHeading(index)`；回调通过主线程 `Handler` 转发（JS 线程非主线程）。
  - `MarkdownView` 新增参数 `onOutline: (List<OutlineItem>) -> Unit`、`onActiveHeading: (Int) -> Unit`；并暴露跳转：返回一个 `remember` 的 `OutlineController`（弱持 `WebView`），`controller.scrollToHeading(i)` 内部 `evaluateJavascript("window.MDreader.scrollToHeading(i)", null)`。
- `ui/OutlineDrawer.kt`（新）：渲染大纲树（`level` 缩进、`maxLines=2`+省略号、`activeIndex` 项高亮、点击回调）。
- `ui/ReaderScreen.kt`：`Box` 内按 `screenWidthDp` 分流 `ModalNavigationDrawer`（右侧）/ `PermanentNavigationDrawer`（常驻）；TopAppBar 加目录图标 + `rememberDrawerState`；状态：`outline: List<OutlineItem>`、`activeIndex: Int?`、`OutlineController`。无标题时隐藏目录图标并展示空提示。

## 测试策略

大纲数据来自 DOM（渲染行为），无 JVM 单测，走详细手动验证清单：

- **手动验证清单（adb 装包后）**：
  1. 竖屏打开 `sample.md`，点目录图标 → 右侧抽屉滑出，标题按层级缩进显示。
  2. 点击某大纲项 → 抽屉关闭、正文滚动到对应标题。
  3. 手动上下滚动正文 → 大纲高亮项随之变化。
  4. 旋转横屏 → 大纲常驻左侧，点击即时跳转。
  5. 打开一个无标题的文档 → 目录图标隐藏（或抽屉为空提示）。
  6. 打开含代码块内 `#` 开头行的文档 → 代码块内的「标题」不出现在大纲。
  7. Setext 标题（`===`/`---`）与 HTML `<h2>` 标题也出现在大纲，且点击跳转正确（验证索引对齐）。
- **构建校验**：每次改动 `./gradlew assembleDebug` 与 `:app:testDebugUnitTest` 全绿（确保新代码不破坏既有纯逻辑测试）。

## 响应式实现

- 用 `LocalConfiguration.current.screenWidthDp` 判断，阈值 `600`（Material 紧凑/中等窗口分界）。
- 竖屏抽屉宽度约 `320dp`；横屏沿用 `PermanentNavigationDrawer` 默认。

## 实现顺序（里程碑）

1. `OutlineItem` 模型 + `render.js` 标题 id 分配 + outline 回传 + `scrollToHeading` + IntersectionObserver。
2. `MarkdownView` 双向桥（`onOutline`/`onActiveHeading` 回传 + `OutlineController` 跳转）。
3. `OutlineDrawer` Composable。
4. `ReaderScreen` 竖屏抽屉 + 横屏常驻 + 当前项高亮。
5. 验证清单全过 + `assembleDebug`/单测全绿 + 提交。
