# .spell show file (JSON): load, save and version migration.
# Minimum schema:
#   {"version": 1, "name": "...", "fps": 30, "duration": 85.9,
#    "outputs": [{"type": "sacn", "universes": [1]}],
#    "tracks":  [{"type": "pyfx", "file": "medgrupo.py", "universe": 1}],
#    "cues": []}
# load() adds "_dir" (the file's folder, to resolve relative paths); save() drops keys starting with "_".
import json
import os

VERSION = 1


def migrate(sh):
    v = int(sh.get("version", 0))
    if v > VERSION:
        raise ValueError(f".spell version {v}: newer than this player (v{VERSION})")
    if v < 1:                       # v0 = draft without a version field; the rest of the schema is the same
        sh["version"] = VERSION
    return sh


def load(path):
    with open(path, encoding="utf-8") as f:
        sh = migrate(json.load(f))
    sh["_dir"] = os.path.dirname(os.path.abspath(path))
    return sh


def save(path, sh):
    out = {k: v for k, v in sh.items() if not k.startswith("_")}
    out["version"] = VERSION
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(out, f, indent=1, ensure_ascii=False)
        f.write("\n")
    return path
