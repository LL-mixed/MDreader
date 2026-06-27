# MDreader Linux 端口设计（Rust + GTK4 + WebKitGTK）

日期：2026-06-27 ｜ 状态：已与用户确认，开始实现

## 目标

为 MDreader 增加 Linux 原生端，**功能与 macOS 版完全对等**。沿用项目既有架构原则：**每端 = 原生语言 + 原生工具包 + 原生 WebView，全部加载同一份 `shared/render/`（物理唯一来源，零改动）**。

## 技术决策（含 why）

| 决策点 | 选择 | 为什么 |
| --- | --- | --- |
| 语言 | Rust | 类型安全、零成本抽象、单二进制分发；`cargo` 统一 build/test，符合「命令行入口」准则；是新一代 Linux 原生应用的主流方向 |
| UI 工具包 | GTK4 | Linux 原生工具包；与 GNOME/系统主题融合，明暗主题原生支持 |
| WebView | **WebKitGTK 6**（webkitgtk-6.0） | 与 macOS WKWebView **同属 WebKit**，共享 `window.webkit.messageHandlers` 桥接 API——JS 桥可近乎逐字复刻 macOS 的 `bridgeScript`；是 Linux 桌面原生 WebView |
| 缓存元数据 | SQLite（rusqlite，bundled） | 关系型查询/观察最合适（对应 Android Room / macOS SwiftData）；bundled 省去运行时依赖 |
| 缓存正文 | `$XDG_DATA_HOME/MDreader/docs/<uuid>.md` | 正文体积可变，不入库；按 id 命名（对应 macOS App Support） |
| 内容去重 | 正文 SHA-256 | 三端一致 |
| 配置/会话 | `$XDG_CONFIG_HOME/mdreader/*.json` | XDG 规范，对应 macOS `~/.mdreader/*.json` |
| 资源打包 | GResource（`build.rs` 编译 `shared/render`） | GTK 原生资源系统；二进制自包含；`resource://` 供 WebKitGTK 同源加载 |
| 工程 | `cargo` + `build.rs`（+ Meson 仅用于系统安装，后期） | 原生构建工具，可复现，对齐 gradle/xcodegen 取向 |
| 窗口模型 | **每文档一个顶层窗口** | GNOME 原生多窗口行为（evince/gedit）；最贴近 macOS「每文档=一窗口」心智 |

## 跨端渲染契约（零改动复用 `shared/render`）

`render.js` 只认 `window.mdreaderNative` 对象，方法：同步 `getMarkdown()/getDark()/getSvg(i)`，回调 `markRendered()/onOutline(json)/onActiveHeading(i)`；外加 `window.MDreader.render()` 与 `scrollToHeading(i)`。

**macOS 的桥接手法（关键洞察，直接复刻）**：macOS 注入的 `bridgeScript` 把同步读转为读「预置 payload」`window.__mdrPayload = {md,dark,svgs}`，而异步回调走 `window.webkit.messageHandlers.mdreaderNative.postMessage(...)`。由于 WebKitGTK 与 WKWebView 同属 WebKit、共享 `window.webkit.messageHandlers`，**Linux 端用相同 shim + `webkit_user_content_manager_register_script_message_handler` 即可**，仅宿主 API 换名。

预处理流水线（native，在 `getMarkdown()` 前执行）：`resolve_images`（raster→base64 / svg→inline）→ `mermaid_fence` 归一化 → `svg_guard` 抽离。

## 功能对等映射

| macOS 能力 | Linux 实现 |
| --- | --- |
| 渲染内核 | WebKitGTK6 + GResource，`resource:///.../render/index.html` |
| JS 桥 / payload | `WebKitUserScript`（document-start）+ payload + `register_script_message_handler("mdreaderNative")` |
| 明暗主题 | `body.dark/light` + `GtkSettings` color-scheme 驱动 GTK 外壳同步 |
| 缩放 30–300% | `webkit_web_view_set_zoom_level` + Ctrl+/−/0 + Ctrl 滚轮 |
| 大纲 TOC + 滚动高亮 | 同款 `onOutline/onActiveHeading/scrollToHeading` + `GtkListView` |
| 链接处理 + 返回 | `decide-policy` 信号 + 返回按钮 |
| 相对图片解析 | 移植 `resolve_images` |
| 缓存元数据 | rusqlite 表 `cached_docs`（镜像 `CachedDoc` 字段） |
| SHA-256 去重 / 正文存储 | `sha2` crate + `<uuid>.md` 文件 |
| 内容管理 | `GtkPaned`/`GtkOverlaySplitView` + `GtkListView` + `DateBuckets` + context menu |
| 文件关联 | `.desktop` `MimeType=text/markdown;` + `GApplication::open` + `xdg-mime` |
| 拖拽 | 窗口 `GtkDropTarget` + 同款 webview drop script |
| 会话/缩放/设置 | `$XDG_CONFIG_HOME/mdreader/*.json` |
| 外部编辑器 | 设置命令 + 默认 `xdg-open`/`$EDITOR` |
| PDF 导出 | `WebKitPrintOperation`（打印对话框选 Print to File→PDF） |
| 关于窗口 | `GtkAboutDialog` + `build.rs` 注入 git hash |
| 多文档 | 每文档一个顶层窗口；库内点击载入当前窗口，右键「新窗口打开」 |

## 目录结构

```
linux/
├── Cargo.toml          # cargo = build + test 入口
├── build.rs            # 编译 render.gresource.xml 内嵌 ../shared/render；写 build-info.json(git hash)
├── .gitignore          # target/ *.gresource build-info.json
├── resources/
│   ├── render.gresource.xml   # 引 ../../shared/render/**（物理唯一来源）
│   ├── icons/                 # com.mdreader.MDreader.svg 等
│   └── style.css              # （可选）GTK 外壳样式
├── data/
│   ├── com.mdreader.MDreader.desktop       # MimeType=text/markdown; Exec=%U
│   └── com.mdreader.MDreader.appdata.xml
├── src/
│   ├── main.rs                 # gtk::Application + open 信号 + 窗口管理
│   ├── app.rs                  # 顶层窗口：headerbar + split(sidebar+webview) + actions
│   ├── config.rs               # XDG 路径 + build-info
│   ├── render/
│   │   ├── webview.rs          # WebKitWebView：加载/桥/zoom/nav/drop/print
│   │   ├── bridge.rs           # mdreaderNative shim + payload + 消息分发
│   │   ├── preprocess.rs       # resolve_images
│   │   ├── svg_guard.rs        # 移植 SvgGuard
│   │   ├── mermaid_fence.rs    # 移植 MermaidFenceNormalizer
│   │   ├── fence.rs            # 移植 Fence 正则
│   │   └── outline.rs          # OutlineItem {index,level,text}
│   ├── store/
│   │   ├── cache.rs            # DocRepository：rusqlite + SHA-256 去重 + refresh
│   │   ├── doc_store.rs        # 正文文件存储
│   │   ├── content_hash.rs     # SHA-256 hex（同测试向量）
│   │   ├── doc_info.rs         # DocInfo 值类型
│   │   ├── zoom_store.rs / session_store.rs / settings_store.rs
│   ├── ui/
│   │   ├── library_view.rs / outline_view.rs / sidebar.rs / about.rs
│   │   └── date_buckets.rs / titles.rs
│   └── util.rs                 # editor launch 等
└── tests/                       # 集成测试；单测内联 #[cfg(test)]
```

## 构建与验证

```
sudo apt-get install -y libgtk-4-dev libwebkitgtk-6.0-dev   # 一次性
cd linux && cargo build --release     # 出二进制 mdreader
cd linux && cargo test                # 全部对齐单测
cd linux && cargo run -- path/to.md   # CLI 入口：打开文件
```

工具链：Rust ≥ 1.74（本机 1.88）；GTK ≥ 4.6（本机 4.6.9）；WebKitGTK ≥ 2.42（本机 2.50.4）。

## 已识别风险

- **GTK 4.6 较旧** → gtk-rs crate 必须锁定到兼容 4.6 的版本组合（gtk4 0.6.x / webkit6 对应版本）。LM1 用真实编译验证并锁定。
- **无显示环境跑窗口** → 用 Xvfb 虚拟帧缓存做冒烟验证。
- **`resource://` 同源加载** → 若 WebKitGTK 对 gresource 的相对路径解析有问题，退到自定义 URI scheme（`mdreader://`）。

## 里程碑（对齐 MM1–MM6）

- **LM1 脚手架**：cargo crate + 空 GTK4 窗口 + WebKitGTK6 + GResource 内嵌 `shared/render`；`cargo build` 出二进制。✅ 目标
- **LM2 渲染内核**：桥 + payload + 主题 + 预处理流水线；渲染 sample.md。移植 `svg_guard/mermaid_fence/fence/content_hash` + 单测对齐。
- **LM3 文件打开者**：`GApplication::open` + `.desktop` + MimeType + 拖拽。
- **LM4 缓存层**：rusqlite + XDG 正文 + SHA-256 去重；打开/拖拽即缓存。移植 `cache/doc_store` + 单测。
- **LM5 内容管理**：sidebar（库/大纲）+ 日期分组列表 + 搜索 + 收藏/删除/刷新 + 大纲 + 缩放。
- **LM6 图标与发布**：图标 + `.desktop` + appdata + release。
