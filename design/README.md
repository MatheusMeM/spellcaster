# design/ — departamento de design do Spellcaster

- `PRINCIPIOS.md` — cinco princípios com o que cada um proíbe; checklist anti-slop antes de qualquer tela.
- `DECISOES.md` — decisões datadas (cores, tipografia, grade, famílias de nó). Nova decisão = nova linha, nunca edição silenciosa.
- `tokens/spellcaster.css` — tokens `--sc-*`. Fonte única de cor, fonte, escala e grade para Theme, Face e Graph.
- `canvas/` — artboards `.dc.html` do Claude Design (uma Face por arquivo, um Theme por variante) e `canvas.json` com o layout.

Fluxo: proposta como artboard em `canvas/` → revisão no canvas publicado → decisão registrada em `DECISOES.md` → token ou componente entra em `tokens/` ou em `spellgui/`.

Regenerar o canvas (base da skill `design` do Claude Code):

```bash
node "<base>/seed-canvas.mjs" --template "<base>/payload.template.html" --out "%TEMP%/spellcaster-faces.html" --title "Spellcaster Faces" --artboard Main.dc.html --artboard ThemeHeadspace.dc.html --artboard ThemeSignal.dc.html --artboard Editor.dc.html --canvas canvas.json
```

Sync com claude.ai/design (projeto de design system): `/design-login` uma vez em sessão interativa, depois `/design-sync` na pasta `design/`.
