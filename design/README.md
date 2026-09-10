# design/ — Spellcaster design department

## What lives here

| | What it is |
|---|---|
| `PRINCIPIOS.md` | five principles and what each one forbids; anti-slop checklist before any screen. |
| `DECISOES.md` | dated decisions. New decision = new line; to change one, a line that revokes the old one. Never a silent edit. |
| `TEMAS.md` | the five functions and the material (theme) each one wears: LASER, FÓSFORO, PATCHBAY, PAPER THEATER, Pino. |
| `SHORTCUTS.md` | shortcut map, Premiere/Resolve grammar. |
| `FUNCOES/` | function before UI: one file per function (`ilda-player.md`, `ndi-ilda.md`, `orquestrador.md`, `cenas-cues-dmx.md`, `cenario-interativo.md`, `aprendiz-menu.md`, `integracao-laser.md`), 14 cross-cutting rules in `README.md` and the audits with `path:line` in `fontes/`. |
| `laser/` | the live round: the 10 W projector in 3D (`app.html` + `.js` modules), the tool's design system (`tokens.css`, `SISTEMA.md`, `sistema.html`), `PEDIDOS.md`, `module.json` and `graph.json`. |
| `tokens/spellcaster.css` | the product's `--sc-*` tokens (color, font, scale, grid). |
| `canvas/` | Claude Design `.dc.html` artboards and `canvas.json`. |
| `build.py` | inlines local `<script src>` and `<link>` into a single HTML, to publish as an artifact. |

Rounds 2, 3 and 4 left the tree (2 was rejected in the vote, 3 and 4 were absorbed by `laser/`). What they decided is in `DECISOES.md`; the files remain in git.

## Round flow

Function in `FUNCOES/` → working prototype (not a drawing sheet) → publish as an artifact → **vote inside the prototype** (`db`, collection `moodboard/roundN`) → the vote becomes a line in `DECISOES.md` → the next round starts from there. A request from Matheus goes in verbatim in `laser/PEDIDOS.md`.

Current artifact: https://claude.ai/code/artifact/8a913f8b-8ea7-4621-b57a-88d7738dbafd

## Build and verify

```
C:/Python313/python.exe design/build.py design/laser/app.html <scratchpad>/spellcaster-laser.html
node <scratchpad>/jscheck.js <scratchpad>/spellcaster-laser.html
```

`jscheck.js` runs every `<script>` without `src` through `new Function` and counts the errors. Runtime, a single pass (not a loop):

```
"C:/Program Files/Google/Chrome/Application/chrome.exe" --headless=new --no-first-run
  --user-data-dir=<tmp> --use-angle=swiftshader --enable-unsafe-swiftshader
  --window-size=1240,1200 --virtual-time-budget=12000 --screenshot=<png>
  "file:///<scratchpad>/spellcaster-laser.html#tras"
```

The `#tras`, `#dentro`, `#laser` hashes skip the splash. Module contract: `C:/Python313/python.exe -m unittest tests.test_laser_graph`.

HTML and PNG stay in the scratchpad; no binary goes into this folder (Drive sync).

Claude Design canvas (regenerate):

```
node "<base>/seed-canvas.mjs" --template "<base>/payload.template.html" --out "%TEMP%/spellcaster-faces.html" --title "Spellcaster Faces" --artboard Main.dc.html --artboard ThemeHeadspace.dc.html --artboard ThemeSignal.dc.html --artboard Editor.dc.html --canvas canvas.json
```
