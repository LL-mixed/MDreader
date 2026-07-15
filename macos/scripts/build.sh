#!/usr/bin/env bash
#
# build.sh — 构建 MDreader for macOS 并归拢产物到 macos/outputs/。
#
# 流程：
#   1. 校验 xcodegen 可用
#   2. xcodegen generate（按 project.yml 生成 .xcodeproj）
#   3. xcodebuild -configuration Release（用 -derivedDataPath build 锁定产物路径，
#      避免 DerivedData 哈希目录漂移）
#   4. 清空并重建 macos/outputs/
#   5. 拷贝 .app + .dSYM（调试符号，便于崩溃排查）
#   6. hdiutil 打一个 DMG（macOS 分发惯例，对齐 AGENT.md MM6「Release .app / DMG」）
#
# 产物：
#   macos/outputs/MDreader.app
#   macos/outputs/MDreader.app.dSYM
#   macos/outputs/MDreader-{version}-macos-{arch}.dmg
#
# 用法：
#   ./scripts/build.sh            # Release 构建 + DMG
#   ./scripts/build.sh --no-dmg   # 跳过 DMG，只出 .app
#   ./scripts/build.sh --debug    # Debug 构建（默认 Release）
#   ./scripts/build.sh --clean    # 构建前 clean
#
set -euo pipefail

# macos/scripts/ -> macos/
MACOS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="MDreader"
SCHEME="MDreader"
DERIVED_DATA="$MACOS_DIR/build"
OUTPUTS_DIR="$MACOS_DIR/outputs"

CONFIGURATION="Release"
MAKE_DMG=1
DO_CLEAN=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --no-dmg)  MAKE_DMG=0; shift ;;
        --debug)   CONFIGURATION="Debug"; shift ;;
        --clean)   DO_CLEAN=1; shift ;;
        -h|--help)
            sed -n '2,/^$/p' "$0" | sed 's/^# \?//'
            exit 0 ;;
        *) echo "未知参数: $1" >&2; exit 2 ;;
    esac
done

echo "▶ 配置: $CONFIGURATION  DMG: $([ $MAKE_DMG = 1 ] && echo yes || echo no)"

# --- 1. 校验 xcodegen -------------------------------------------------------
if ! command -v xcodegen >/dev/null 2>&1; then
    echo "✗ 未找到 xcodegen。请先安装：brew install xcodegen" >&2
    exit 1
fi

# --- 2. 生成工程 ------------------------------------------------------------
echo "▶ xcodegen generate"
cd "$MACOS_DIR"
xcodegen generate >/dev/null

# --- 3. 构建 ----------------------------------------------------------------
XCODEBUILD_ARGS=(
    -project "$APP_NAME.xcodeproj"
    -scheme "$SCHEME"
    -configuration "$CONFIGURATION"
    -destination 'platform=macOS'
    -derivedDataPath "$DERIVED_DATA"
)
if [[ $DO_CLEAN = 1 ]]; then
    echo "▶ xcodebuild clean"
    xcodebuild "${XCODEBUILD_ARGS[@]}" clean >/dev/null
fi
echo "▶ xcodebuild build ($CONFIGURATION)"
# xcodebuild 往 stderr 打印大量进度；只保留错误/警告摘要到终端，完整日志丢 build.log
if ! xcodebuild "${XCODEBUILD_ARGS[@]}" build > "$OUTPUTS_DIR.build.log" 2>&1; then
    echo "✗ 构建失败，完整日志见 $OUTPUTS_DIR.build.log" >&2
    tail -30 "$OUTPUTS_DIR.build.log" >&2
    exit 1
fi

# --- 4. 定位产物 ------------------------------------------------------------
BUILT_APP="$DERIVED_DATA/Build/Products/$CONFIGURATION/$APP_NAME.app"
BUILT_DSYM="$BUILT_APP.dSYM"
if [[ ! -d "$BUILT_APP" ]]; then
    echo "✗ 构建产物未找到: $BUILT_APP" >&2
    exit 1
fi

# 从构建出的 Info.plist 读版本（最可靠，反映实际打进 bundle 的值）
VERSION="$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$BUILT_APP/Contents/Info.plist" 2>/dev/null || echo "0.0.0")"
ARCH="$(uname -m)"
APP_SIZE="$(du -sh "$BUILT_APP" | cut -f1)"
echo "▶ 产物: $BUILT_APP  (v$VERSION, $APP_SIZE)"

# --- 5. 归拢到 outputs/ -----------------------------------------------------
echo "▶ 归拢到 $OUTPUTS_DIR/"
rm -rf "$OUTPUTS_DIR"
mkdir -p "$OUTPUTS_DIR"
cp -R "$BUILT_APP" "$OUTPUTS_DIR/"
[[ -d "$BUILT_DSYM" ]] && cp -R "$BUILT_DSYM" "$OUTPUTS_DIR/"

# 把构建日志也留一份，方便回溯
mv "$OUTPUTS_DIR.build.log" "$OUTPUTS_DIR/build.log"

# --- 6. 打 DMG --------------------------------------------------------------
if [[ $MAKE_DMG = 1 ]]; then
    DMG_NAME="$APP_NAME-$VERSION-macos-$ARCH.dmg"
    DMG_PATH="$OUTPUTS_DIR/$DMG_NAME"
    echo "▶ 打 DMG: $DMG_NAME"

    STAGING="$(mktemp -d)/$APP_NAME"
    mkdir -p "$STAGING"
    cp -R "$BUILT_APP" "$STAGING/"
    # 指向 /Applications 的软链，便于用户拖拽安装
    ln -s /Applications "$STAGING/Applications"

    # 先创一个可读写的 UDZO 压缩 DMG。hdiutil 的 -srcfolder 方式最稳，无需手动挂载。
    hdiutil create -volname "$APP_NAME" -srcfolder "$STAGING" \
        -fs HFS+ -format UDZO -imagekey zlib-level=9 "$DMG_PATH" >/dev/null

    rm -rf "$(dirname "$STAGING")"
    DMG_SIZE="$(du -h "$DMG_PATH" | cut -f1)"
    echo "  → $DMG_PATH ($DMG_SIZE)"
fi

# --- 汇总 -------------------------------------------------------------------
echo
echo "✔ 完成。outputs/ 内容："
( cd "$OUTPUTS_DIR" && ls -lh )

if [[ $MAKE_DMG = 1 ]]; then
    echo
    echo "安装：双击 $(ls "$OUTPUTS_DIR"/*.dmg | xargs -n1 basename)，把 MDreader 拖到 Applications。"
fi
