#!/bin/sh
# Lite tarball for Raspberry Pi OS: Python package (pure stdlib) + shows + profiles + systemd + install.sh.
# Usage: sh packaging/build_lite.sh [output-folder]      (default: /tmp)
set -e
OUT=${1:-/tmp}

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

tar czf "$OUT/$NAME.tar.gz" -C "$OUT" "$NAME"
rm -rf "$STAGE"
ls -l "$OUT/$NAME.tar.gz"
