# Where the files live. Frozen (PyInstaller onedir): the executable's folder. Running from source: the repo root.
# Nothing under %APPDATA% -- the USB stick carries shows, profiles and config.json next to the exe.
import pathlib
import sys

FROZEN = bool(getattr(sys, "frozen", False))
ROOT = pathlib.Path(sys.executable).resolve().parent if FROZEN else pathlib.Path(__file__).resolve().parents[1]
BUNDLE = pathlib.Path(getattr(sys, "_MEIPASS", ROOT))     # bundled data (_internal/) or the repo itself


def _dir(name):
    """User folder next to the exe; falls back to the copy shipped in the bundle while it does not exist."""
    p = ROOT / name
    return p if p.is_dir() else BUNDLE / name


SHOWS = _dir("shows")
PROFILES = _dir("profiles")
WEB = BUNDLE / "spellcaster" / "gui" / "web"              # GUI assets: always the bundled ones
SKINS = WEB / "skins"
CONFIG = ROOT / "config.json"
