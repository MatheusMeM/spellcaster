# Arquivo de show .spell (JSON): carga, gravacao e migracao de versao.
# Esquema minimo:
#   {"version": 1, "name": "...", "fps": 30, "duration": 85.9,
#    "outputs": [{"type": "sacn", "universes": [1]}],
#    "tracks":  [{"type": "pyfx", "file": "medgrupo.py", "universe": 1}],
#    "cues": []}
# load() acrescenta "_dir" (pasta do arquivo, para resolver caminhos relativos); save() descarta chaves com "_".
import json
import os

VERSION = 1


def migrate(sh):
    v = int(sh.get("version", 0))
    if v > VERSION:
        raise ValueError(f".spell versao {v}: mais novo que este player (v{VERSION})")
    if v < 1:                       # v0 = rascunho sem campo version; o resto do esquema e igual
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
