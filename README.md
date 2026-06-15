# MDreader

一个 Android 平台的 **Markdown 阅读器**（只读）。把 `.md` 渲染成排版精良的阅读界面，并注册为系统级 markdown 文件打开者——在微信、文件管理器等 app 中点开 `.md` 时可选择本 app 打开，打开的文件会自动缓存到 app 私有空间，可按日期/标题浏览、搜索、收藏、删除。

## 技术栈

- Kotlin + Jetpack Compose（Material 3，明暗主题 + 动态取色）
- Markdown 渲染：WebView + 本地 `marked.js` + `highlight.js` + 自研 CSS（GitHub 风格，明暗双主题）
- 缓存：Room（元数据 + SHA-256 内容去重）+ app 内部存储（正文文件）
- 构建：Gradle Kotlin DSL + Version Catalog；minSdk 24 / targetSdk 34；AGP 8.5.2 / Gradle 8.7 / Kotlin 1.9.24

## 环境要求

- JDK 17
- Android SDK（`platform-tools`、`platforms;android-34`、`build-tools;34.0.0`）
- 在 `local.properties` 里设置 `sdk.dir=<SDK 路径>`

## 常用命令

```bash
# 构建 debug APK
./gradlew :app:assembleDebug

# 构建签名 release APK（需在 local.properties 配置 mdreader.* 签名凭据，见下）
./gradlew :app:assembleRelease

# 运行单元测试
./gradlew :app:testDebugUnitTest

# 安装 debug APK 到已连接设备/模拟器
./gradlew :app:installDebug
# 或： adb install -r app/build/outputs/apk/debug/app-debug.apk

# 模拟「从外部打开 markdown」（验证 intent-filter + 缓存）
adb push README.md /sdcard/README.md
adb shell am start -a android.intent.action.VIEW -d "file:///sdcard/README.md" -t text/markdown com.mdreader/.MainActivity
```

## Release 签名

签名凭据放在 `local.properties`（已 gitignore，不进仓库）：

```properties
mdreader.storeFile=<绝对路径>/mdreader.jks
mdreader.storePassword=<密码>
mdreader.keyAlias=mdreader
mdreader.keyPassword=<密码>
```

用 keytool 生成一个开发用 keystore：

```bash
keytool -genkeypair -keystore keystore/mdreader.jks -alias mdreader \
  -keyalg RSA -keysize 2048 -validity 10000
```

未配置时 release 构建产出未签名 APK，仍可成功构建。

## 应用图标

图标由 `tools/gen_icon.py` 用 Pillow 生成（蓝渐变底 + 文档卡片 + markdown `#`），全 5 密度 + round 变体：

```bash
pip3 install --user Pillow
python3 tools/gen_icon.py
```

## 目录结构

见 [CLAUDE.md](CLAUDE.md)。
