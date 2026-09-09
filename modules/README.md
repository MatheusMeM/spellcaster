# `modules/` — apps declarados

Um `module.json` por app. O app diz o que tem; o core só guarda a tabela dos módulos vivos
(`module_add`, `module_list`, `module_get`, `module_del`) e o PATCHBAY monta o nó a partir dela,
sem conhecer o app. Formato de `design/FUNCOES/orquestrador.md §5` (o `module.json` do Chataigne).

- `parameters` — o que se manda para o app. Endereço `grupo/nome`, sem espaço (regra 2: o endereço
  é a identidade). `type` decide o widget (regra 1): `float`, `int`, `bool`, `trigger`, `color`,
  `string`, `enum` (com `options`). `min`/`max` é clamp físico; `norm` é a faixa útil do slider.
- `values` — o que o app devolve, só leitura; chega pelo comando `input` com chave `module:<nome>/<path>`.
- `commands` — `context`: `action` (só dispara), `mapping` (só valor), `both`.

Valide antes de carregar: `module_check {"file": "laser"}` (pelo MCP, pela GUI ou por
`spellcore serve`; a lista de comandos sai em `spellcore commands`).

```json
{ "name": "laser", "type": "laser", "version": "0.1.0",
  "parameters": { "geo/scale": { "type": "float", "default": 1, "min": 0, "max": 4, "norm": [0, 2] } },
  "values":     { "stat/fps": { "type": "float" } },
  "commands":   { "play": { "context": "action", "args": { "file": "string" } } } }
```
