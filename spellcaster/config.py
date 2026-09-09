# config.json ao lado do executavel: porta da GUI, skin, interface preferida, ultimo show.
import json

from .core.registry import command
from .paths import CONFIG

DEFAULTS = {"port": 8000, "skin": "feiticaria", "interface": None, "last_show": None}


def load():
    """Config do disco sobre os defaults. Arquivo ausente ou quebrado = defaults."""
    try:
        return {**DEFAULTS, **json.loads(CONFIG.read_text(encoding="utf-8"))}
    except (OSError, ValueError):
        return dict(DEFAULTS)   # ponytail: config ilegivel cai no default calado ; avisar se alguem editar a mao e nao entender


def save(**kw):
    d = {**load(), **kw}
    CONFIG.write_text(json.dumps(d, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    return d


@command
def config(key: str = "", value: str = ""):
    """Le/grava config.json ao lado do executavel. Sem argumentos, imprime a configuracao."""
    d = save(**{key: int(value) if value.isdigit() else value}) if key and value else load()
    print(json.dumps(d, indent=1, ensure_ascii=False))
    return d
