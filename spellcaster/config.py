# config.json next to the executable. Key anyone reads: `port` (the GUI). Whatever else the
# user writes stays in the file and comes back on load; there is no default for what nobody reads.
import json

from .core.registry import command
from .paths import CONFIG

DEFAULTS = {"port": 8000}


def load():
    """Config from disk on top of the defaults. Missing or broken file = defaults."""
    try:
        return {**DEFAULTS, **json.loads(CONFIG.read_text(encoding="utf-8"))}
    except (OSError, ValueError):
        return dict(DEFAULTS)   # ponytail: unreadable config falls back to the default silently ; warn if someone hand-edits it and cannot tell


def save(**kw):
    d = {**load(), **kw}
    CONFIG.write_text(json.dumps(d, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    return d


@command
def config(key: str = "", value: str = ""):
    """Read/write config.json next to the executable. With no arguments, prints the configuration."""
    d = save(**{key: int(value) if value.isdigit() else value}) if key and value else load()
    print(json.dumps(d, indent=1, ensure_ascii=False))
    return d
