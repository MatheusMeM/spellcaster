#!/bin/sh
# Tarball Lite para Raspberry Pi OS: pacote Python (stdlib pura) + shows + profiles + systemd + install.sh.
# Uso: sh packaging/build_lite.sh [--no-gui] [pasta-de-saida]      (default: /tmp)
# --no-gui tira spellcaster/gui/web (os assets da GUI): o Pi por SSH so usa CLI, tui e MCP.
set -e
NOGUI=0
OUT=/tmp
for arg in "$@"; do
    case "$arg" in
        --no-gui) NOGUI=1 ;;
        *) OUT="$arg" ;;
    esac
done

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
VER=$(sed -n 's/^__version__ *= *"\(.*\)"/\1/p' "$ROOT/spellcaster/__init__.py")
NAME="spellcaster-lite-$VER"
STAGE="$OUT/$NAME"

rm -rf "$STAGE"
mkdir -p "$STAGE"
cp -r "$ROOT/spellcaster" "$ROOT/shows" "$ROOT/profiles" "$STAGE/"
cp "$ROOT/pyproject.toml" "$ROOT/packaging/spellcaster.service" "$ROOT/packaging/install.sh" "$STAGE/"
chmod +x "$STAGE/install.sh"
find "$STAGE" -name '__pycache__' -type d -prune -exec rm -rf {} +
[ "$NOGUI" = 1 ] && rm -rf "$STAGE/spellcaster/gui/web"

tar czf "$OUT/$NAME.tar.gz" -C "$OUT" "$NAME"
rm -rf "$STAGE"
ls -l "$OUT/$NAME.tar.gz"
