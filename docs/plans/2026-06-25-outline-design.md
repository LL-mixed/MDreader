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

大纲展示数据与跳转定位分别用最合适的来源，并用数量校验保证一致：

| 用途 | 来源 | 理由 |
| --- | --- | --- |
| 展示文本 + 层级 + 顺序索引 | Kotlin 纯函数 `OutlineParser` 解析 markdown | 纯逻辑，可 JVM 单元测试，且不依赖渲染时机 |
| 跳转滚动定位 | DOM（JS） | 以最终渲染结果为准，定位最准 |
| 一致性兜底 | JS 回传 DOM 标题数量，Kotlin 取 `min(解析数, DOM数)` | 防御 marked 边缘规则导致索引错位 |

解析规则：支持 ATX（`#`~`######`）与 Setext（`===`→h1，`---`→h2），跳过围栏代码块（``` / ~~~）与缩进代码块（4 空格）；清理标题文本（去 `#` 尾缀、去强调/代码标记，保留纯文本）。

## 组件拆分

- `util/OutlineParser.kt`（新）：`parse(markdown): List<OutlineItem>`；`OutlineItem(index, level, text)`。纯逻辑。
- `render.js`：
  - `render()` 末尾：收集 `h1~h6` → 按顺序分配 `id="mdr-h-{index}"` → 通过 `mdreaderNative.onOutlineReady(count)` 回传数量。
  - 新增 `scrollToHeading(index)`：`document.getElementById('mdr-h-'+index).scrollIntoView()`。
  - 新增 IntersectionObserver：回传当前可见标题索引 `onActiveHeading(index)`。
  - 全部暴露到 `window.MDreader`。
- `render/MarkdownView.kt`：
  - `SourceBridge` 新增 `@JavascriptInterface onOutlineReady(count)` / `onActiveHeading(index)`，转发到 Kotlin 回调。
  - `MarkdownView` 新增参数 `onOutline: (Int) -> Unit`、`onActiveHeading: (Int) -> Unit`；提供内部跳转：`evaluateJavascript("window.MDreader.scrollToHeading(i)", null)`。
- `ui/OutlineDrawer.kt`（新）：渲染大纲树，接收 `List<OutlineItem>` + `activeIndex` + `onClick`。
- `ui/ReaderScreen.kt`：用 `OutlineParser` 解析标题；`Box` 内按窗口宽度分流 `ModalNavigationDrawer` / `PermanentNavigationDrawer`；TopAppBar 加目录图标与 `rememberDrawerState`；状态：大纲项列表、DOM 标题数（用于截断）、`activeIndex`。

## 测试策略

- **JVM 单元测试（`app/src/test`）**：
  - `OutlineParserTest`：ATX 各级、Setext、跨级、空标题、纯符号、带强调/代码文本清理、围栏代码块（``` 与 ~~~）跳过、缩进代码块跳过、嵌套列表内标题不算、文本与层级与顺序一致性。
- **手动验证清单（adb 装包后）**：
  1. 竖屏打开 `sample.md`，点目录图标 → 右侧抽屉滑出，标题按层级缩进显示。
  2. 点击某大纲项 → 抽屉关闭、正文滚动到对应标题。
  3. 手动上下滚动正文 → 大纲高亮项随之变化。
  4. 旋转横屏 → 大纲常驻左侧，点击即时跳转。
  5. 打开一个无标题的文档 → 目录图标可隐藏或抽屉为空提示。
  6. 打开含代码块内 `#` 开头行的文档 → 代码块内的「标题」不出现在大纲。
- **构建校验**：每次改动 `./gradlew assembleDebug` 与 `:app:testDebugUnitTest` 全绿。

## 响应式实现

- 用 `LocalConfiguration.current.screenWidthDp` 判断，阈值 `600`（Material 紧凑/中等窗口分界）。
- 竖屏抽屉宽度约 `320dp`；横屏沿用 `PermanentNavigationDrawer` 默认。

## 实现顺序（里程碑）

1. `OutlineParser` + 单测（纯逻辑先行，TDD）。
2. `render.js` 标题 id 分配 + `scrollToHeading` + `IntersectionObserver`。
3. `MarkdownView` 双向桥（回传 + 跳转）。
4. `OutlineDrawer` + `ReaderScreen` 竖屏抽屉。
5. 横屏常驻 + 当前项高亮。
6. 验证清单全过 + `assembleDebug`/单测全绿 + 提交。
