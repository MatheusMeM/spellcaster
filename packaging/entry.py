# Entry dos dois executaveis congelados: o nome do exe escolhe o modo.
#   Spellcaster.exe (sem console) -> janela/GUI (navegador se faltar WebView2)
#   spell.exe       (com console) -> CLI `spell`
import os
import sys

if os.path.basename(sys.executable).lower().startswith("spellcaster"):
    from spellcaster import config
    from spellcaster.gui.window import window
    window(config.load()["port"])
else:
    from spellcaster.cli import main
    main()          # o retorno do comando nao e codigo de saida (sys.exit(dict) exporia o repr)
