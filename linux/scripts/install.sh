#!/usr/bin/env bash
# Install MDreader into the user's desktop environment (GNOME Activities/dash, .md handler) —
# user-scope only, no root, fully reversible. Re-runnable; overwrites with the current release
# build.
#
#   ./scripts/install.sh                 # install (binary + .desktop + icons + metainfo)
#   ./scripts/install.sh --set-default   # also make it the default app for text/markdown (.md)
#   ./scripts/install.sh --uninstall     # remove everything this script installed
#
# Why a script (not done at runtime): the .desktop/icon-theme/mime registration must live on disk
# under standard freedesktop paths; GTK4/GNOME do not read the in-process GResource for these.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LINUX="$HERE/.."
APP_ID="com.mdreader.MDreader"
DATA="${XDG_DATA_HOME:-$HOME/.local/share}"
BIN_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"

uninstall() {
  rm -f "$BIN_DIR/mdreader" \
        "$DATA/applications/$APP_ID.desktop" \
        "$DATA/metainfo/$APP_ID.metainfo.xml"
  for s in 128x128 256x256 512x512; do
    rm -f "$DATA/icons/hicolor/$s/apps/$APP_ID.png"
  done
  update-desktop-database "$DATA/applications" 2>/dev/null || true
  gtk-update-icon-cache -f "$DATA/icons/hicolor" >/dev/null 2>&1 || true
  echo "Removed MDreader (user-scope)."
  echo "If you'd set it as the .md default, pick another with:  xdg-mime default <id>.desktop text/markdown"
  exit 0
}

case "${1:-}" in
  --uninstall) uninstall ;;
  --set-default) SET_DEFAULT=1 ;;
  "") SET_DEFAULT=0 ;;
  *) echo "Unknown option: $1" >&2; echo "Usage: $0 [--set-default|--uninstall]" >&2; exit 2 ;;
esac

BIN="$(find "$LINUX/target" -path '*/release/mdreader' -type f 2>/dev/null | head -1)"
if [ -z "$BIN" ]; then
  echo "Release binary not found. Build it first:  (cd $LINUX && cargo build --release)" >&2
  exit 1
fi

install -v -Dm755 "$BIN"                                 "$BIN_DIR/mdreader"
install -v -Dm644 "$LINUX/data/$APP_ID.desktop"          "$DATA/applications/$APP_ID.desktop"
install -v -Dm644 "$LINUX/data/$APP_ID.metainfo.xml"     "$DATA/metainfo/$APP_ID.metainfo.xml"
for s in 128x128 256x256 512x512; do
  install -v -Dm644 "$LINUX/resources/icons/hicolor/$s/apps/$APP_ID.png" \
                   "$DATA/icons/hicolor/$s/apps/$APP_ID.png"
done

update-desktop-database   "$DATA/applications" 2>/dev/null || true
gtk-update-icon-cache -f  "$DATA/icons/hicolor" >/dev/null 2>&1 || true

if [ "$SET_DEFAULT" = "1" ]; then
  xdg-mime default "$APP_ID.desktop" text/markdown text/x-markdown
  echo "Default app for text/markdown -> MDreader"
else
  echo "Tip: make it the default for .md:  $0 --set-default"
fi

echo
echo "Ensure $BIN_DIR is on \$PATH (Ubuntu's ~/.profile adds it by default)."
echo "If MDreader doesn't appear in Activities immediately, log out/in once."
