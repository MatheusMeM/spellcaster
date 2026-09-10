#!/bin/sh
# Installs Spellcaster Lite on Raspberry Pi OS: copies to /opt/spellcaster, creates the `spell`
# command and enables the systemd service. No pip, no venv, no dependency: it is the system
# python3 stdlib.
# Usage: sudo ./install.sh [destination]
set -e
DEST=${1:-/opt/spellcaster}
[ "$(id -u)" = 0 ] || exec sudo sh "$0" "$@"
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

mkdir -p "$DEST"
cp -r "$HERE/spellcaster" "$HERE/shows" "$HERE/profiles" "$HERE/pyproject.toml" "$DEST/"

printf '#!/bin/sh\nexec env PYTHONPATH=%s python3 -m spellcaster.cli "$@"\n' "$DEST" > /usr/local/bin/spell
chmod +x /usr/local/bin/spell

sed "s#/opt/spellcaster#$DEST#g" "$HERE/spellcaster.service" > /etc/systemd/system/spellcaster.service
systemctl daemon-reload
systemctl enable --now spellcaster

echo "ok: $DEST ; GUI em http://$(hostname).local:8000 ; CLI: spell net | spell tui | spell play_show medgrupo.spell"
