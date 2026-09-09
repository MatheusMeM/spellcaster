# design/ — departamento de design do Spellcaster

## O que tem aqui

| | O que é |
|---|---|
| `PRINCIPIOS.md` | cinco princípios e o que cada um proíbe; checklist anti-slop antes de qualquer tela. |
| `DECISOES.md` | decisões datadas. Nova decisão = linha nova; para mudar, linha que revoga a antiga. Nunca edição silenciosa. |
| `TEMAS.md` | as cinco funções e o material (tema) que cada uma veste: LASER, FÓSFORO, PATCHBAY, TEATRO DE PAPEL, Pino. |
| `SHORTCUTS.md` | mapa de atalhos, gramática Premiere/Resolve. |
| `FUNCOES/` | a função antes da UI: um arquivo por função (`ilda-player.md`, `ndi-ilda.md`, `orquestrador.md`, `cenas-cues-dmx.md`, `cenario-interativo.md`, `aprendiz-menu.md`, `integracao-laser.md`), 14 regras transversais no `README.md` e as auditorias com `path:linha` em `fontes/`. |
| `laser/` | a rodada viva: o projetor 10 W em 3D (`app.html` + módulos `.js`), design system da ferramenta (`tokens.css`, `SISTEMA.md`, `sistema.html`), `PEDIDOS.md`, `module.json` e `graph.json`. |
| `tokens/spellcaster.css` | tokens `--sc-*` do produto (cor, fonte, escala, grade). |
| `canvas/` | artboards `.dc.html` do Claude Design e `canvas.json`. |
| `build.py` | inlina `<script src>` e `<link>` locais num HTML só, para publicar como artifact. |

Rodadas 2, 3 e 4 saíram da árvore (a 2 foi reprovada no voto, a 3 e a 4 foram absorvidas por `laser/`). O que elas decidiram está em `DECISOES.md`; os arquivos continuam no git.

## Fluxo de rodada

Função em `FUNCOES/` → protótipo funcional (não prancha) → publicar como artifact → **voto dentro do protótipo** (`db`, coleção `moodboard/roundN`) → o voto vira linha em `DECISOES.md` → a próxima rodada parte daí. Pedido do Matheus entra literal em `laser/PEDIDOS.md`.

Artifact atual: https://claude.ai/code/artifact/8a913f8b-8ea7-4621-b57a-88d7738dbafd

## Buildar e verificar

```
C:/Python313/python.exe design/build.py design/laser/app.html <scratchpad>/spellcaster-laser.html
node <scratchpad>/jscheck.js <scratchpad>/spellcaster-laser.html
```

`jscheck.js` passa cada `<script>` sem `src` por `new Function` e conta erros. Runtime, uma passada só (não é loop):

```
"C:/Program Files/Google/Chrome/Application/chrome.exe" --headless=new --no-first-run
  --user-data-dir=<tmp> --use-angle=swiftshader --enable-unsafe-swiftshader
  --window-size=1240,1200 --virtual-time-budget=12000 --screenshot=<png>
  "file:///<scratchpad>/spellcaster-laser.html#tras"
```

Hashes `#tras`, `#dentro`, `#laser` pulam a splash. Contrato do módulo: `C:/Python313/python.exe -m unittest tests.test_laser_graph`.

HTML e PNG ficam no scratchpad; binário não entra nesta pasta (Drive sync).

Canvas do Claude Design (regenerar):

```
node "<base>/seed-canvas.mjs" --template "<base>/payload.template.html" --out "%TEMP%/spellcaster-faces.html" --title "Spellcaster Faces" --artboard Main.dc.html --artboard ThemeHeadspace.dc.html --artboard ThemeSignal.dc.html --artboard Editor.dc.html --canvas canvas.json
```
