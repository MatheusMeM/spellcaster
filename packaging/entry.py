# Entry point of both frozen executables: the exe name picks the mode.
#   Spellcaster.exe (no console) -> window/GUI (browser if WebView2 is missing)
#   spell.exe       (with console) -> `spell` CLI
import os
import sys

if os.path.basename(sys.executable).lower().startswith("spellcaster"):
    from spellcaster import config
    from spellcaster.gui.window import window
    window(config.load()["port"])
else:
    from spellcaster.cli import main
    main()          # the command return value is not an exit code (sys.exit(dict) would expose the repr)
