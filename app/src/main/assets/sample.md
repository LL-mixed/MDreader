# MDreader 渲染示例

一个**只读**的 Android Markdown 阅读器，专注于把 `.md` 文件渲染得赏心悦目。

## 文本样式

支持 **加粗**、*斜体*、~~删除线~~、`行内代码`，以及 [超链接](https://example.com)。

> 引用块：好的阅读体验不让人思考——排版、间距、配色都应为内容服务。

## 数学公式（KaTeX）

行内公式：质能方程 $E = mc^2$，欧拉公式 $e^{i\pi} + 1 = 0$。

展示公式：

$$
\int_{-\infty}^{\infty} e^{-x^2}\, dx = \sqrt{\pi}
$$

$$
\sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}
$$

## 列表

无序列表：

- 第一项
- 第二项
  - 嵌套子项
  - 另一个子项
- 第三项

任务列表：

- [x] 渲染 Markdown 正文
- [x] 代码语法高亮
- [ ] 数学公式（KaTeX）
- [ ] 暗色主题适配

## 代码块

```kotlin
package com.mdreader

data class Document(
    val title: String,
    val content: String,
    val cachedAt: Long
)

fun main() {
    val doc = Document("Hello", "# Hi", System.currentTimeMillis())
    println("Loaded: ${doc.title}")
}
```

## 表格

| 功能 | 状态 | 说明 |
| --- | :---: | --- |
| 正文渲染 | ✅ | 标题/列表/引用 |
| 代码高亮 | ✅ | 多语言 |
| 表格 | ✅ | 含对齐 |
| 缓存管理 | 🚧 | 按日期/内容 |

## 流程图（Mermaid）

```mermaid
flowchart LR
    A[打开 .md 文件] --> B{已缓存?}
    B -->|是| C[渲染展示]
    B -->|否| D[读取并缓存]
    D --> C
```

## 其它

水平线：

---

行内 `code` 与 <kbd>Ctrl</kbd> + <kbd>K</kbd> 按键提示。
