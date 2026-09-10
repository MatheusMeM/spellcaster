# -*- mode: python ; coding: utf-8 -*-
# PyInstaller onedir: Spellcaster.exe (GUI, no console) and spell.exe (CLI, console) from the SAME Analysis.
# Build: packaging/build_win.ps1. pyinstaller is a development dependency (never a runtime one).
import glob
import os

ROOT = os.path.dirname(SPECPATH)


def files(sub, *pats):
    return [(f, sub) for p in pats for f in glob.glob(os.path.join(ROOT, sub, p))]


a = Analysis(
    [os.path.join(SPECPATH, "entry.py")],
    pathex=[ROOT],
    datas=[(os.path.join(ROOT, "spellcaster", "gui", "web"), os.path.join("spellcaster", "gui", "web"))]
          + files("profiles", "*.json") + files("shows", "*.spell", "*.py", "*.ild"),
    hiddenimports=["spellcaster.cli", "spellcaster.tui", "spellcaster.config", "spellcaster.gui.window"],
    excludes=["tkinter", "test", "idlelib", "pydoc_data"],   # ponytail: only the obvious ones ; measure again if it goes past 40 MB
    noarchive=False,
)
pyz = PYZ(a.pure)

gui = EXE(pyz, a.scripts, [], exclude_binaries=True, name="Spellcaster", console=False,
          debug=False, strip=False, upx=False)
cli = EXE(pyz, a.scripts, [], exclude_binaries=True, name="spell", console=True,
          debug=False, strip=False, upx=False)
COLLECT(gui, cli, a.binaries, a.datas, strip=False, upx=False, name="Spellcaster")
