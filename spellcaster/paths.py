# Onde ficam os arquivos. Congelado (PyInstaller onedir): pasta do executavel. Rodando do fonte: raiz do repo.
# Nada em %APPDATA% -- o pendrive leva shows, profiles e config.json junto do exe.
import pathlib
import sys

FROZEN = bool(getattr(sys, "frozen", False))
ROOT = pathlib.Path(sys.executable).resolve().parent if FROZEN else pathlib.Path(__file__).resolve().parents[1]
BUNDLE = pathlib.Path(getattr(sys, "_MEIPASS", ROOT))     # dados empacotados (_internal/) ou o proprio repo


def _dir(name):
    """Pasta do usuario ao lado do exe; cai na copia que veio no bundle enquanto ela nao existir."""
    p = ROOT / name
    return p if p.is_dir() else BUNDLE / name


SHOWS = _dir("shows")
PROFILES = _dir("profiles")
WEB = BUNDLE / "spellcaster" / "gui" / "web"              # assets da GUI: sempre os do bundle
SKINS = WEB / "skins"
CONFIG = ROOT / "config.json"
