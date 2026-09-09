# spellgui/web — as páginas do Spellcaster

HTML e JS puro, sem build, sem framework, sem npm. Cada página é uma pasta rasa de arquivos que
o `spellcore serve` (ou qualquer servidor estático) entrega. Cor e tipografia vêm só de
`design/tokens/spellcaster.css`; nenhuma página escreve cor literal.

| Arquivo | O que é |
|---|---|
| `bus.js` | cliente do barramento: WS `{"id","cmd","args"}` com promessa por id, reconexão, eventos (`transport`, `show`, `log`, `widget`), frames binários `topic\|universe\|512` decodificados em `{topic, universe, data}` (topic 1 = saída, topic 2 = entrada). Modo offline embutido |
| `widgets.js` | parâmetro tipado → widget (`WG.kindOf`), campos → `args` do request (`WG.args`), formulário de um comando do registry (`WG.form`). Classes para a página estilizar: `.wg`, `.wg-<tipo>`, `.wg-lab`, `.wg-num`, `.wg-form`, `.wg-doc`, `.wg-go`, `.wg-out` |
| `face.js` + `face.html` | runtime da Face: `faces/<nome>.face.json` vira grade de widgets em modo kiosk |
| `canvaskit.js` | pan, zoom, hit-test, marquee, DPR, dirty-flag; compartilhado por timeline e graph |
| `timeline.js` + `index.html` | timeline em canvas. `R` (botão do cabeçalho, tecla e botão da barra) chama `rec_arm` no engine e o estado do arme vem de `rec_state`; `+Track` abre menu de tipo (`dmx`, `laser` com a lista de `laser_files`, `fx`); `Alt+M` mostra o universo de saída da lane focada, ou o de ENTRADA quando ela está armada |
| `dev/commands.json` | `Registry::schema()` congelado, usado no modo offline |
| `test/*.test.js` | `node --test spellgui/web/test/*.test.js` |

## Como abrir

As páginas leem `../../design/tokens/spellcaster.css`, `../../faces/` e `../../shows/` (é o que o
`index.html` já fazia): **sirva a raiz do repo**, não `spellgui/web`.

Com engine (o normal — é o único processo que toca hardware):

```
spellcore serve --port 8000 --dir . --show shows/medgrupo.spell
# http://127.0.0.1:8000/spellgui/web/face.html?face=quatro
```

Sem engine (só para desenhar a página; nada de saída DMX):

```
python -m http.server 8000
# http://127.0.0.1:8000/spellgui/web/face.html?face=quatro&offline=1
```

No modo offline não há engine: `bus.call` ecoa `{offline, cmd, args}` e escreve no log o que
faria. A página monta, os botões respondem, nada sai pela rede.

## Face

`faces/<nome>.face.json` (PRD §10). O widget declara **onde** fica e **o que dispara**; o
comportamento mora no Graph, dentro do `.spell`:

```json
{"id": "go", "type": "button", "label": "GO", "at": [0, 0, 2, 2], "cmd": "cue_go", "args": {}}
{"id": "blackout", "type": "button", "label": "BLACK", "at": [3, 1, 1, 1], "input": "widget:blackout"}
```

- `cmd` + `args` → `Registry::call` pelo barramento.
- `input` → comando `input {key, value}`; um nó `in.widget` com `"widget": "blackout"` no graph
  do show é quem decide o que isso faz. Sem esse nó, o `input` não tem efeito — é de propósito:
  a página não implementa comportamento.
- Volta pelo evento `widget` (`out.widget` do Graph): `{"id","prop","value"}`. `prop` vira classe
  de mesmo nome no widget, ligada quando o valor não é zero; `face.html` pinta `glow`/`on`,
  `alert`/`live` e `off`, e o resto fica para a página estilizar.
- `views` remanejam os mesmos widgets (`grid` `[colunas, linhas]`, lista de `widgets`, `at` por
  id); `?view=compact`. `views` é obrigatório.
- Tipos hoje: `button`, `toggle`, `fader`, `label`. O resto do catálogo do PRD §10 entra com o
  editor de Face (R9).

Atalhos da Face (design/SHORTCUTS.md): `Enter` = cue GO, `Esc` segurado 0,5 s = blackout,
`Shift+F` = tela cheia.

## Regenerar `dev/commands.json`

Sai do registry, nunca escrito à mão:

```
spellcore commands > spellgui/web/dev/commands.json
```

`spellcore/cli/tests/commands_json.rs` falha quando um comando sai do registry ou muda de schema.
