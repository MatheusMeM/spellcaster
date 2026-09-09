# Blender 4.5 — auditoria de organização de editor

Leitura direta dos arquivos instalados em `C:\Program Files\Blender Foundation\Blender 4.5\4.5\scripts\`.
Tudo abaixo tem `path:linha`. O que não foi lido não está aqui.

Duas raízes de caminho, abreviadas no resto do documento:

- `bl_ui/` = `scripts/startup/bl_ui/`
- `keymap/` = `scripts/presets/keyconfig/keymap_data/blender_default.py`

## Fontes lidas

| Arquivo | Linhas | O que foi lido |
|---|---|---|
| `bl_ui/space_sequencer.py` | 3243 | 131-242 (headers), 388-560 (View), 955-1010, 1100-1197 (Strip), 1461-1500 (mix-ins), 1586-1614 (painel) |
| `bl_ui/space_dopesheet.py` | 1024 | 31-130 (filtros), 373-396 (menus), 534-572 (Channel), 591-641 (Key), 813-864 (contexto de canal) |
| `bl_ui/space_graph.py` | 578 | 30-64 (header), 155-168 (menus) |
| `bl_ui/space_nla.py` | 425 | 26-55 (header), 97-111 (menus), 384-399 (contexto de track) |
| `bl_ui/space_node.py` | 1234 | 66-95 (header), 257-300 (menus/Add), 369-425 (Node), 577-599 (Show/Hide) |
| `bl_ui/space_outliner.py` | 559 | 39-68 (header), 99-110 (menus), 219-238 (Visibility), 400-495 (Filter) |
| `bl_ui/space_time.py` | 347 | 1-120 (transporte e menus), 232-280 (Playback) |
| `bl_ui/space_topbar.py` | 856 | 20-60 (topbar), 106-126 (menus), 495-520 (Edit), 632-652 (workspace) |
| `bl_ui/space_statusbar.py` | 41 | inteiro; `bl_ui/utils.py` (66) inteiro |
| `bl_ui/space_info.py`, `space_text.py`, `space_console.py`, `space_spreadsheet.py` | — | blocos `MT_editor_menus`; `INFO_MT_area` em `space_info.py:20-107` |
| `bl_ui/space_properties.py`, `space_userpref.py`, `__init__.py` | — | 10-134; 314-319, 588-628, 1095-1133, 1780-1810; 68-96 |
| `bl_ui/properties_data_bone.py`, `properties_grease_pencil_common.py`, `properties_output.py`, `properties_render.py`, `asset_shelf.py` | — | 268-300 (solo); 675-690; 10-45 (presets); linhas de `use_property_decorate` |
| `bl_operators/presets.py`, `bl_operators/wm.py` | — | 73-130 (`AddPresetBase`); 32-92 (busca de RNA path) |
| `keymap/` (`blender_default.py`) | 6000+ | 316-360, 410-470, 562-575, 676-900, 1002-1050, 1132-1170, 1831-1990, 2121-2260, 2503-2640, 2668-2700, 2950-3110, 3624-3790 |
| `scripts/presets/interface_theme/Blender_Light.xml` | 1675 | 322-341 (State), 830-868 (Sequencer) |

Nota: `Blender_Dark.xml` tem 6 linhas e está vazio (`presets/interface_theme/Blender_Dark.xml:1-6`) — o tema escuro é o built-in em C, não um preset. Só o Light expõe os valores.

## Anatomia de um editor

Um editor (`Space`) é um conjunto de regiões. A região é escolhida por `bl_region_type` na classe:

- `HEADER` — cabeçalho. É o default quando `Header` só declara `bl_space_type` (`bl_ui/space_sequencer.py:163-164`).
- `TOOL_HEADER` — segunda barra, só com as opções da ferramenta ativa (`bl_ui/space_sequencer.py:137-139`).
- `UI` — sidebar, tecla N (`bl_ui/space_sequencer.py:131-134`).
- `TOOLS` — toolbar, tecla T.
- `CHANNELS` — coluna de canais/tracks à esquerda da timeline (`keymap/:3262` define o keymap "Sequencer Channels").
- `WINDOW` — a região principal.
- `HUD` — o rodapé flutuante "Adjust Last Operation" (`bl_ui/space_sequencer.py:471`).
- `NAVIGATION_BAR` — a coluna de ícones do Properties (`bl_ui/space_properties.py:62-64`).
- `PREVIEW` — a área de preview quando o editor tem duas vistas.

Regras que se repetem em todo editor:

1. **O menu View começa ligando/desligando as regiões, nesta ordem**: toolbar, sidebar, tool header, HUD, channels (`bl_ui/space_sequencer.py:467-473`). O usuário sempre acha "onde sumiu a barra" no mesmo lugar.
2. **O menu View termina com `INFO_MT_area`** (`bl_ui/space_sequencer.py:546`), que é o mesmo em todos: quadview, Horizontal Split, Vertical Split, separador, screen_full_area, Toggle Fullscreen Area, area_dupli, separador, area_close (`bl_ui/space_info.py:67-89`).
3. **O header segue sempre a mesma sequência**: `layout.template_header()` → seletor de modo do espaço (`st.view_type`) → `MT_editor_menus.draw_collapsible` → `separator_spacer()` → tool settings → `separator_spacer()` → toggles de exibição com popover (`bl_ui/space_sequencer.py:166-215`).
4. **Idioma toggle+popover**: um `row` com o booleano e um `sub` cujo `sub.active` é amarrado ao booleano, contendo o `popover` com os detalhes (`bl_ui/space_sequencer.py:206-215`; `bl_ui/space_time.py:21-28`; `bl_ui/space_graph.py:36-40`). O botão liga; a setinha ao lado abre o painel de ajuste. Nunca dois botões separados.
5. **Sidebar é dividida em abas** por `bl_category`. No sequencer: Strip, View, Tool, Cache, Proxy, Modifiers. No node editor: Item, Tool, Node, Group, Options.

### Ordem dos menus por editor

| Editor | Menus, na ordem em que são desenhados | Fonte |
|---|---|---|
| Sequencer | View, Select, [Marker], [Add], Strip, [Image] | `bl_ui/space_sequencer.py:225-238` |
| Dope Sheet | View, Select, [Marker], Channel, Key, [Action] | `bl_ui/space_dopesheet.py:381-394` |
| Graph | View, Select, [Marker], Channel, Key | `bl_ui/space_graph.py:162-167` |
| NLA | View, Select, [Marker], Add, Tracks, Strips | `bl_ui/space_nla.py:104-110` |
| Node | View, Select, Add, Node | `bl_ui/space_node.py:263-266` |
| Text | View, Text, [Edit, Select, Format], Templates | `bl_ui/space_text.py:96-104` |
| Console | View, Console | `bl_ui/space_console.py:25-26` |
| Info | View, Info | `bl_ui/space_info.py:26-27` |
| Spreadsheet | View | `bl_ui/space_spreadsheet.py:51` |
| Outliner | nenhum (só em modo DATA_API) | `bl_ui/space_outliner.py:107-108` |
| Topbar | Blender, File, Edit, Render, Window, Help | `bl_ui/space_topbar.py:115-125` |

A lei: **View primeiro, Select segundo, Add terceiro, e por último o menu com o nome do objeto do editor** (Strip, Node, Key/Channel, Tracks/Strips). Menus entre colchetes só aparecem se o estado permite — Marker some se `st.show_markers` é falso (`bl_ui/space_sequencer.py:232-233`), Action some se não há action (`bl_ui/space_dopesheet.py:393-394`).

### O Timeline como caso de barra de transporte

O header do Timeline é um único `row(align=True)` com seis botões e nada mais: rewind, keyframe anterior, play reverso, play, keyframe seguinte, fast-forward (`bl_ui/space_time.py:31-49`). Quando toca, os dois botões de play viram um Pause com `row.scale_x = 2` — o botão que importa fica fisicamente maior (`bl_ui/space_time.py:44-47`). Depois vem o campo de frame atual e o par Start/End, que troca para os campos de preview range quando o toggle está ligado (`bl_ui/space_time.py:53-70`).

Os menus do Timeline não são menus: Playback e Keying são popovers, e só depois vêm View e Marker (`bl_ui/space_time.py:88-105`).

## Objetos e verbos

Cada editor tem UM substantivo e um menu com o nome dele. Os verbos são operadores com nome `dominio.verbo`.

### Sequencer — objeto `strip`

`SEQUENCER_MT_strip` (`bl_ui/space_sequencer.py:1100-1197`), na ordem: Transform, [Duplicate/Show-Hide/Text no preview], Retiming, Split e Hold Split, Copy/Paste/Duplicate, Delete, Add Modifier / Copy Modifiers, submenu por tipo (Effect, Movie, Image, Meta), Color Tag, Lock/Mute, Connect/Disconnect, Inputs.

Sub-blocos que importam:

- **Lock/Mute** (`bl_ui/space_sequencer.py:990-1004`): lock, unlock, separador, mute, unmute, "Mute Unselected Strips", "Unmute Deselected Strips". Lock e mute são coisas diferentes e ficam em blocos separados.
- **Show/Hide** no preview (`bl_ui/space_sequencer.py:955-964`): "Show Hidden Strips" primeiro, depois "Hide Selected" e "Hide Unselected". Revelar vem antes de esconder.
- **Meta** = agrupar strips num container navegável: `meta_make`, `meta_separate`, `meta_toggle` (`bl_ui/space_sequencer.py:1177-1183`). Toggle entra e sai do grupo com Tab (`keymap/:3055`).
- **Connect/Disconnect** (`bl_ui/space_sequencer.py:1191-1192`): vínculo entre strips que faz seleção e movimento andarem juntos, e que pode ser ignorado segurando Alt (`keymap/:3005-3009`).

### Dope Sheet / Graph — objetos `channel` e `keyframe`

`DOPESHEET_MT_channel` (`bl_ui/space_dopesheet.py:534-572`): delete, clean, group/ungroup, `channels_setting_toggle` / `_enable` / `_disable` (todos como `operator_menu_enum`, ou seja mute e protect são *valores de enum* de um mesmo verbo, não três botões), editable_toggle, extrapolation, expand/collapse, move, fcurves_enable, bake, view_selected.

O menu de contexto do canal nomeia esses enums em texto humano (`bl_ui/space_dopesheet.py:827-831`):
`Mute Channels`/`Unmute Channels` = `type='MUTE'`; `Protect Channels`/`Unprotect Channels` = `type='PROTECT'`. Ou seja, o vocabulário de estado do canal é **mute** e **protect** (lock), e não existe solo aqui.

`DOPESHEET_MT_key` (`bl_ui/space_dopesheet.py:591-629`): Transform, Snap, Mirror, Keyframe Insert, Frame Jump, Copy/Paste/Paste Flipped/Duplicate/Delete, e então **quatro enums separados**: Keyframe Type, Handle Type, Interpolation Mode, Easing Mode. Depois Clean, Bake, Euler Filter.

O submenu Transform do keyframe tem quatro modos distintos de mover no tempo: Move (`TIME_TRANSLATE`), Extend (`TIME_EXTEND`), Slide (`TIME_SLIDE`), Scale (`TIME_SCALE`) (`bl_ui/space_dopesheet.py:638-641`).

Filtros do canal aparecem em dois níveis: três toggles ícone-only direto no header — `show_only_selected`, `show_hidden`, `show_only_errors` (`bl_ui/space_dopesheet.py:36-45`) — e o conjunto completo num popover com campo de texto e uma grade "Filter by Type" (`bl_ui/space_dopesheet.py:54-130`).

### Node editor — objetos `node`, `socket`, `link`, `frame`, `group`, `reroute`

`NODE_MT_node` (`bl_ui/space_node.py:369-425`), na ordem: transform, copy/paste/duplicate/duplicate linked, delete e `delete_reconnect`, "Join in New Frame" / "Remove from Frame", Rename (via `wm.call_panel` do `TOPBAR_PT_name`, `bl_ui/space_node.py:401-403`), bloco de links (`link_make`, "Make and Replace Links", `links_cut`, `links_detach`, `links_mute`), bloco de grupo (`group_make`, "Insert Into Group", `group_edit`, `group_ungroup`), Show/Hide.

`Show/Hide` do nó (`bl_ui/space_node.py:577-599`): Mute, Node Preview, Node Options, separador, Unconnected Sockets, Collapse, Collapse and Hide Unused. Ou seja o nó tem quatro níveis de "esconder": mudo (passa direto), sem preview, sem opções, colapsado.

`node.links_mute` é o verbo que **desliga um cabo sem apagar** (`bl_ui/space_node.py:410`, atalho Ctrl+Alt+arraste com botão direito em `keymap/:2196`). É a coisa mais próxima de um bypass de rota.

### Outliner — objeto `collection` e as colunas de restrição

As colunas de toggle são configuráveis e a lista muda com o modo (`bl_ui/space_outliner.py:412-429`):

- modo View Layer: `enable`, `select`, `hide`, `viewport`, `render`, `holdout`, `indirect_only` — sete colunas, nesta ordem.
- modo Scenes: só `select`, `hide`, `viewport`, `render`.

Menu Visibility da collection (`bl_ui/space_outliner.py:219-238`): **Isolate** primeiro, sozinho; depois Show / Show All Inside / Hide / Hide All Inside; depois "Enable in Viewports" / "Disable in Viewports". Isolate é o solo do Outliner.

### Solo, e o que ele faz com os outros toggles

O único solo escrito em Python está nas bone collections (`bl_ui/properties_data_bone.py:286-295`):

```python
sub_visible = row.row(align=True)
sub_visible.active = (not is_solo_active) and bcoll.is_visible_ancestors
sub_visible.prop(bcoll, "is_visible", text="", icon='HIDE_OFF' if bcoll.is_visible else 'HIDE_ON')
row.prop(bcoll, "is_solo", text="", icon='SOLO_ON' if bcoll.is_solo else 'SOLO_OFF')
```

Três coisas: (a) o ícone do toggle troca conforme o estado, não é um checkbox; (b) enquanto o solo está ativo o toggle de visibilidade fica **apagado mas presente** — não some, não é removido; (c) solo vem depois de visibilidade na linha, e a ação destrutiva (unassign, ícone X) fica separada por um espaço (`bl_ui/properties_data_bone.py:298-300`).

## Estados

### O tema tem um bloco só para estado

`USERPREF_PT_theme_interface_state` (`bl_ui/space_userpref.py:1095-1133`) lista, nesta ordem: `error`, `warning`, `info`, `success`, depois `inner_anim`, `inner_driven`, `inner_key`, `inner_overridden`, `inner_changed` — cada um com um par `_sel` para quando o item está selecionado — e por fim `blend`.

Valores reais (`presets/interface_theme/Blender_Light.xml:322-340`):

| Estado | Cor | Significado |
|---|---|---|
| `error` | `#771111` | operação falhou |
| `warning` | `#ac8737` | vai dar problema |
| `info` | `#28487d` | aviso neutro |
| `success` | `#188625` | terminou bem |
| `inner_anim` | `#73be4c` verde | campo é animado, mas o frame atual não tem key |
| `inner_key` | `#f0eb64` amarelo | há keyframe exatamente neste frame |
| `inner_driven` | `#b400ff` roxo | valor vem de driver, não do usuário |
| `inner_overridden` | `#6bf3cc` ciano | override de library |
| `inner_changed` | `#cc7529` laranja | diferente do default |
| `blend` | `0.5` | fator de mistura |

O detalhe que mais importa: existe `blend`. **A cor de estado não é borda nem ícone — é uma tinta misturada no fundo do próprio campo.** O widget continua sendo o widget; só muda de cor. E `_sel` existe porque a cor precisa continuar legível quando o item está selecionado.

### O decorador por propriedade

Cada linha de propriedade pode ter, na direita, um widget de animação. Liga-se com `layout.use_property_decorate`, e painéis de dado não-animável desligam com um comentário explícito: `layout.use_property_decorate = False  # No animation.` (`bl_ui/properties_render.py:64`; `bl_ui/asset_shelf.py:20`; `bl_ui/space_time.py:240`). Ou seja: o affordance de "isso pode ser animado" é estrutural, aparece em toda propriedade animável, e a exceção é que precisa ser declarada.

O companheiro é `layout.use_property_split = True` (`bl_ui/space_sequencer.py:1604`): rótulo à esquerda, valor à direita, coluna do decorador no extremo direito. Grade fixa, não fluida.

### Apagado, desabilitado e alarmado

Três níveis distintos, todos usados no mesmo arquivo:

- `layout.active = not strip.mute` (`bl_ui/space_sequencer.py:1605`) — o painel inteiro de um strip mutado fica **apagado mas clicável**. Continua editável; só não está no ar.
- `row.enabled = has_material_slots` (`bl_ui/space_node.py:82`) — cinza e **não clicável**.
- `row.alert = True` (`bl_ui/space_text.py:28-31`) — a linha fica vermelha. Usado quando o arquivo de texto foi alterado fora do Blender e o botão vira "resolve_conflict". Também em campos inválidos de preferências (`bl_ui/space_userpref.py:2266`, `:2294`).

### Cor na timeline: identidade, não estado

O tema do sequencer (`presets/interface_theme/Blender_Light.xml:830-868`) dá uma cor **por tipo de strip** — movie `#4d6890`, image `#8f744b`, scene `#828f50`, audio `#4c8f8f`, effect `#4c456c`, meta `#5b4d91`, text `#824c8f` — e reserva só duas cores para estado: `active_strip` branco e `selected_strip` laranja `#ff6a00`. O playhead é uma única linha `frame_current` `#5680c2`.

Keyframes têm sua própria família de estados com cor: `keyframe`, `keyframe_breakdown`, `keyframe_movehold`, `keyframe_generated`, cada um com par `_selected` e uma borda comum (`presets/interface_theme/Blender_Light.xml:850-859`).

### A status bar

Quatro coisas, na ordem, e nada mais (`bl_ui/space_statusbar.py:12-30`):

1. `template_input_status()` — o que os botões do mouse fazem AGORA, no contexto atual. É a linha de aprendizado do software.
2. `template_reports_banner()` — a última mensagem de erro/aviso.
3. `template_running_jobs()` — barra de progresso do que está rodando.
4. `template_status_info()` — estatísticas.

O conteúdo do item 4 é preferência do usuário, não obrigação: scene stats, scene duration, system memory, video memory, versão (`bl_ui/space_userpref.py:314-319`). E se a status bar estiver escondida, o banner de reports e o de jobs migram para o topbar (`bl_ui/space_topbar.py:47-50`) — a mensagem nunca some, só muda de lugar.

## Atalhos

### Gramática de modificadores

Extraída dos templates, que existem justamente para não deixar cada editor inventar:

| Padrão | Regra | Fonte |
|---|---|---|
| Selecionar tudo / nada / inverter | `A` / `Alt+A` / `Ctrl+I` | `keymap/:410-431` |
| Esconder / esconder o resto / revelar | `H` / `Shift+H` / `Alt+H` | `keymap/:451-456` |
| Menu de contexto | tecla primária + `APP` (tecla de menu do teclado) | `keymap/:316-321` |
| Toolbar / sidebar / channels | `T` / `N` / (por editor) via `wm.context_toggle` em `show_region_*` | `keymap/:330-357` |
| Mover playhead com o mouse | `Shift+botão direito` quando a seleção é botão esquerdo | `keymap/:562-570` |

A regra por trás: **sem modificador = a ação; Shift = a mesma ação sobre o complemento ou estendendo; Alt = o inverso ou o "limpar"; Ctrl = a variante forte.** `H`/`Shift+H`/`Alt+H` (esconder / esconder não-selecionados / revelar) e `H`/`Ctrl+H`/`Ctrl+Alt+H` no sequencer (mute / lock / unlock) são o mesmo esquema (`keymap/:3028-3037`).

### Janela e tela

| Ação | Tecla | Fonte |
|---|---|---|
| Busca de menu (paleta de comandos) | `F3` | `keymap/:758` |
| Renomear item ativo / em lote | `F2` / `Ctrl+F2` | `keymap/:756-757` |
| Menu de favoritos do usuário | `Q` | `keymap/:716` |
| Novo / Abrir / Recentes / Salvar / Salvar como / Sair | `Ctrl+N` / `Ctrl+O` / `Shift+Ctrl+O` / `Ctrl+S` / `Shift+Ctrl+S` / `Ctrl+Q` | `keymap/:705-713` |
| Trocar o tipo de editor da área | `Shift+F1`..`Shift+F12` | `keymap/:719-738` |
| Desfazer / refazer | `Ctrl+Z` / `Shift+Ctrl+Z` | `keymap/:822-823` |
| Maximizar área | `Ctrl+Space` | `keymap/:836` |
| Maximizar sem painéis (fullscreen) | `Ctrl+Alt+Space` | `keymap/:837` |
| Ajustar última operação | `F9` | `keymap/:839` |
| Repetir última operação | `Shift+R` | `keymap/:813` |
| Ciclar contexto do editor | `Ctrl+Tab` / `Shift+Ctrl+Tab` | `keymap/:802-805` |
| Ciclar workspace | `Ctrl+PageDown` / `Ctrl+PageUp` | `keymap/:806-809` |
| Preferências | `Ctrl+,` | `keymap/:864` |

`Shift+F8` = Video Sequencer, `Shift+F12` = Dope Sheet, `Shift+F6` = Graph Editor, `Shift+F3` = Node Editor (`keymap/:725-737`). Cada editor tem um número fixo, e o número não muda com o layout.

### Frames e reprodução (keymap "Frames", vale em toda janela)

| Ação | Tecla | Fonte |
|---|---|---|
| Frame anterior / próximo | `←` / `→` (com repeat) | `keymap/:3634-3637` |
| Ir ao início / fim do range | `Shift+←` / `Shift+→` | `keymap/:3638-3641` |
| Keyframe anterior / próximo | `↓` / `↑` | `keymap/:3642-3645` |
| Play / pause | `Space` (ou `Shift+Space` se Space for busca/ferramenta) | `keymap/:3658-3665` |
| Play reverso | `Shift+Ctrl+Space` | `keymap/:3670-3671` |
| Cancelar reprodução (volta ao frame de origem) | `Esc` | `keymap/:3690` |
| Play/stop em teclado de mídia | `MEDIA_PLAY` / `MEDIA_STOP` | `keymap/:3691-3692` |
| Avançar/recuar frame na roda | `Alt+roda` | `keymap/:3650-3653` |

Detalhe estrutural: **Space é configurável** entre TOOL, SEARCH e PLAY, e o resto do keymap se remaneja em volta (`keymap/:774-788` e `:3656-3665`). O default de fábrica é Space = play, com busca em F3.

Keymap "Animation" (`keymap/:3698-3715`): definir preview range `P`, limpar `Alt+P`, `Ctrl+Home` define frame inicial, `Ctrl+End` define final, `Ctrl+T` alterna frames/segundos.

### Keyframe e driver a partir de qualquer campo (keymap "User Interface")

| Ação | Tecla | Fonte |
|---|---|---|
| Inserir keyframe no campo sob o mouse | `I` | `keymap/:1024` |
| Apagar keyframe do campo | `Alt+I` | `keymap/:1026` |
| Limpar toda a animação do campo | `Shift+Alt+I` | `keymap/:1028` |
| Adicionar driver | `Ctrl+D` | `keymap/:1030` |
| Remover driver | `Ctrl+Alt+D` | `keymap/:1031` |
| Adicionar/remover do keying set | `K` / `Alt+K` | `keymap/:1032-1033` |
| Voltar ao valor default | `Backspace` | `keymap/:1034` |
| Copiar o data path RNA do campo | `Shift+Ctrl+C` | `keymap/:1020` |
| Filtrar a lista sob o cursor | `Ctrl+F` | `keymap/:1036-1037` |

Esse keymap é o que faz o Blender parecer coeso: as teclas valem sobre **qualquer** campo de **qualquer** editor, porque estão no keymap da UI e não no do editor.

### Sequencer

| Ação | Tecla | Fonte |
|---|---|---|
| Cortar (soft) / cortar duro | `K` / `Shift+K` | `keymap/:3024-3027` |
| Mute selecionados / mute o resto | `H` / `Shift+H` | `keymap/:3028-3031` |
| Unmute / unmute o resto | `Alt+H` / `Shift+Alt+H` | `keymap/:3032-3035` |
| Lock / unlock | `Ctrl+H` / `Ctrl+Alt+H` | `keymap/:3036-3037` |
| Duplicar | `Shift+D` | `keymap/:3045` |
| Apagar | `X` ou `Del` | `keymap/:3048-3049` |
| Copiar / colar / colar mantendo offset | `Ctrl+C` / `Ctrl+V` / `Shift+Ctrl+V` | `keymap/:3050-3053` |
| Criar meta / entrar e sair dela | `Ctrl+G` / `Tab` | `keymap/:3055-3056` |
| Enquadrar tudo / seleção / playhead | `Home` / `NumPad .` / `NumPad 0` | `keymap/:3058-3061` |
| Strip anterior / próximo | `PageUp` / `PageDown` | `keymap/:3062-3069` |
| Remover gap / remover todos os gaps | `Backspace` / `Shift+Backspace` | `keymap/:3074-3078` |
| Snap ao playhead / slip / mover no tempo | `Shift+S` / `S` / `G` | `keymap/:3079`, `:3091`, `:3094` |
| Menu Add / pie de View / marcador no playhead | `Shift+A` / `` ` `` / `M` | `keymap/:3088`, `:3090`, `:3104` |
| Toolbar / sidebar | `T` / `N` | `keymap/:2959-2963` |
| Alternar Sequencer/Preview | `Ctrl+Tab` | `keymap/:2966-2967` |
| Snapping liga/desliga | `Shift+Tab` | `keymap/:2968-2969` |

### Dope Sheet

| Ação | Tecla | Fonte |
|---|---|---|
| Inserir keyframe | `I` | `keymap/:2605` |
| Duplicar / apagar | `Shift+D` / `X` ou `Del` | `keymap/:2603-2604` |
| Copiar / colar / colar espelhado | `Ctrl+C` / `Ctrl+V` / `Shift+Ctrl+V` | `keymap/:2606-2609` |
| Handle type / Interpolation / Easing / Keyframe type | `V` / `T` / `Ctrl+E` / `R` | `keymap/:2595-2599` |
| Extrapolation / pie de snap / espelhar | `Shift+E` / `Shift+S` / `Ctrl+M` | `keymap/:2597`, `:2590-2593`, `:2594` |
| Selecionar coluna (keys/frame atual/marcadores) | `K` / `Ctrl+K` / `Shift+K` | `keymap/:2577-2586` |
| Enquadrar tudo / seleção / playhead | `Home` / `NumPad .` / `NumPad 0` | `keymap/:2611-2614` |
| Bloquear edição dos canais / filtrar por nome | `Tab` / `Ctrl+F` | `keymap/:2616-2617` |
| Mover / esticar / escalar / deslizar no tempo | `G` / `E` / `S` / `Shift+T` | `keymap/:2618-2627` |
| Marcador / preview range da seleção | `M` / `Ctrl+Alt+P` | `keymap/:2630`, `:2610` |
| Ir para o Graph Editor | `Ctrl+Tab` | `keymap/:2516-2517` |

### Graph Editor

Idêntico ao Dope Sheet onde faz sentido (`I`, `V`, `T`, `Ctrl+E`, `Home`, `NumPad .`, `Tab`, `Ctrl+F` — `keymap/:1922-1947`), mais:

| Ação | Tecla | Fonte |
|---|---|---|
| Inserir keyframe clicando na curva | `Ctrl+clique` | `keymap/:1932` |
| Suavizar | `Alt+O` | `keymap/:1925` |
| Adicionar F-modifier | `Shift+Ctrl+M` | `keymap/:1845-1846` |
| Esconder curvas / revelar | `H` / `Shift+H` / `Alt+H` | `keymap/:1848` |
| Menu de suavização / blending | `Alt+S` / `Alt+D` | `keymap/:1940-1941` |
| Ir para o Dope Sheet | `Ctrl+Tab` | `keymap/:1849-1850` |

### Canais de animação (keymap "Animation Channels")

| Ação | Tecla | Fonte |
|---|---|---|
| Alternar / ligar / desligar um setting (mute, protect…) | `Shift+W` / `Shift+Ctrl+W` / `Alt+W` | `keymap/:3758-3760` |
| Bloquear/desbloquear edição | `Tab` | `keymap/:3761` |
| Expandir / colapsar | `NumPad +` / `NumPad -` | `keymap/:3763-3764` |
| Mover um acima/abaixo, ou para topo/fundo | `PageUp`/`PageDown`, com `Shift` | `keymap/:3770-3777` |
| Agrupar / desagrupar | `Ctrl+G` / `Ctrl+Alt+G` | `keymap/:3779-3780` |
| Apagar canal / renomear | `X` ou `Del` / duplo clique | `keymap/:3755-3756`, `:3738` |
| Filtrar por nome / enquadrar selecionados | `Ctrl+F` / `NumPad .` | `keymap/:3744`, `:3785` |

### Node Editor

| Ação | Tecla | Fonte |
|---|---|---|
| Fazer link / fazer e substituir | `J` / `Shift+J` | `keymap/:2213-2216` |
| Cortar / mutar links / reroute (arraste com botão direito) | `Ctrl` / `Ctrl+Alt` / `Shift` | `keymap/:2192-2196` |
| Mutar nó / colapsar / esconder sockets soltos / preview | `M` / `H` / `Ctrl+H` / `Shift+H` | `keymap/:2227-2230` |
| Agrupar / desagrupar / separar | `Ctrl+G` / `Ctrl+Alt+G` / `P` | `keymap/:2250-2252` |
| Entrar/sair do grupo | `Tab` / `Ctrl+Tab` | `keymap/:2253-2256` |
| Frame nomeado na seleção / buscar nó pelo nome | `F` / `Ctrl+F` | `keymap/:2226`, `:2249` |
| Apagar / apagar reconectando | `X` / `Ctrl+X` | `keymap/:2235-2238` |
| Enquadrar tudo / seleção | `Home` / `NumPad .` | `keymap/:2231-2233` |
| Toolbar / sidebar | `T` / `N` | `keymap/:2129-2133` |

`node.delete_reconnect` (`Ctrl+X`) apaga o nó e **religa o cabo por cima dele**. Não existe equivalente no sequencer.

### Marcadores (keymap "Markers", vale em toda timeline)

Um keymap só, compartilhado por sequencer, dope sheet, graph, NLA e timeline: criar no playhead `M` (`keymap/:1141`), duplicar `Shift+D` (`:1145`), mover `G` ou arraste (`:1164`, `:1142`), apagar `X`/`Del` (`:1159-1160`), renomear `F2` ou duplo clique (`:1161-1162`). Os mesmos verbos com as mesmas teclas de todo o resto — marcador não ganhou vocabulário próprio.

## Paleta de comandos

`wm.search_menu` é a paleta, ligada em `F3` (`keymap/:758`). Aparece no menu Edit como "Menu Search..." com ícone de lupa, e ao lado dela um "Operator Search..." que **só aparece se o modo desenvolvedor está ligado** (`bl_ui/space_topbar.py:505-507`):

```python
layout.operator("wm.search_menu", text="Menu Search...", icon='VIEWZOOM')
if show_developer:
    layout.operator("wm.search_operator", text="Operator Search...")
```

Isso é a decisão de design inteira em duas linhas: **a busca padrão indexa os menus, não os operadores.** O que o usuário digita é o rótulo que ele viu no menu, e o resultado mostra o caminho do menu em que aquilo mora. A busca por nome interno de operador é ferramenta de desenvolvedor.

Consequência prática para quem escreve UI: um comando só é encontrável se ele estiver declarado em algum `Menu.draw` como `layout.operator("dominio.verbo", text="Rótulo")`. O `text=` é a chave de busca. Exemplos reais: `layout.operator("sequencer.split", text="Split")` (`bl_ui/space_sequencer.py:1133`), `layout.operator("node.link_make", text="Make and Replace Links")` (`bl_ui/space_node.py:407`).

Duas variantes:

- `WM_OT_search_single_menu` — busca dentro de UM menu só. Usada no menu Add do node editor e do 3D View, que são grandes demais para navegar (`bl_ui/space_node.py:282`; `bl_ui/space_view3d.py:2655`).
- `bl_options = {'SEARCH_ON_KEY_PRESS'}` na classe do menu (`bl_ui/space_node.py:269-274`) — o menu vira campo de busca assim que o usuário digita uma letra, sem precisar clicar em "Search...".

Complementos que não são a paleta mas resolvem o mesmo problema:

- `SCREEN_MT_user_menu` em `Q` (`keymap/:716`): favoritos que o usuário monta ele mesmo, botão a botão.
- `screen.redo_last` em `F9` (`keymap/:839`) e o painel HUD: a última operação vira um painel editável em vez de exigir desfazer-e-refazer.
- `ui.copy_data_path_button` em `Shift+Ctrl+C` (`keymap/:1020`): copia o caminho RNA do campo sob o mouse, que é literalmente o nome que o script usa. A GUI ensina a API.
- Presets: qualquer painel pode virar um menu de presets herdando `PresetPanel`, que só precisa de `preset_subdir` e `preset_operator` (`bl_ui/utils.py:9-40`; uso real em `bl_ui/properties_output.py:15-20`). O arquivo do preset é gerado a partir de uma lista de data paths (`bl_operators/presets.py:73-78`).

## Painel declarativo (exemplo real)

Copiado inteiro de `bl_ui/space_sequencer.py:1586-1612`:

```python
class SEQUENCER_PT_adjust_crop(SequencerButtonsPanel, Panel):
    bl_label = "Crop"
    bl_options = {'DEFAULT_CLOSED'}
    bl_category = "Strip"

    @classmethod
    def poll(cls, context):
        if not cls.has_sequencer(context):
            return False

        strip = context.active_strip
        if not strip:
            return False

        return strip.type != 'SOUND'

    def draw(self, context):
        strip = context.active_strip
        layout = self.layout
        layout.use_property_split = True
        layout.active = not strip.mute

        col = layout.column(align=True)
        col.prop(strip.crop, "min_x")
        col.prop(strip.crop, "max_x")
        col.prop(strip.crop, "max_y")
        col.prop(strip.crop, "min_y")
```

O que cada peça faz:

- `bl_space_type` e `bl_region_type` vêm do mix-in `SequencerButtonsPanel` (`bl_ui/space_sequencer.py:1461-1471`), que também traz um `poll` base. O painel só declara o que é próprio dele.
- `bl_label` é o título e, junto com `bl_category`, é o que a busca de menu e a aba da sidebar usam.
- `bl_options = {'DEFAULT_CLOSED'}` — nasce fechado. Existe também `{'HIDE_HEADER'}` para painel sem título (`bl_ui/space_time.py:277`).
- `poll` decide se o painel existe neste contexto. Retornar `False` faz o painel sumir inteiro; não há painel vazio.
- `draw` é chamado a cada redraw e não guarda estado. Todo estado está no dado (`strip.crop.min_x`), nunca no widget.

- Subpainel: mesma classe mais `bl_parent_id = "SEQUENCER_PT_effect"` (`bl_ui/space_sequencer.py:1777-1780`); o aninhamento chega a três níveis (`SEQUENCER_PT_effect` → `_effect_text_style` → `_effect_text_shadow`, `bl_ui/space_sequencer.py:1851-1855`).
- Checkbox no cabeçalho do painel: um `draw_header` separado com um único `layout.prop(strip, "use_shadow", text="")` (`bl_ui/space_sequencer.py:1862-1865`).

## Arquivo

O `.blend` guarda a UI — workspaces, screens, áreas e o estado das regiões — e isso é opcional na abertura. A evidência no lado Python é a preferência `use_load_ui`, listada sob o cabeçalho "Default To" junto com caminhos relativos e compressão (`bl_ui/space_userpref.py:1799-1803`). Ou seja: o arquivo sempre carrega a UI dentro dele, e o usuário decide se, ao abrir, quer a UI do arquivo ou a que já está na tela.

Workspaces são datablocks completos: têm operadores de add, duplicate, delete, reorder to front/back (`bl_ui/space_topbar.py:632-652`), e aparecem como abas via `layout.template_ID_tabs(window, "workspace", ...)` no topbar (`bl_ui/space_topbar.py:36`). Em fullscreen a aba some e vira um "Back to Previous" (`bl_ui/space_topbar.py:37-38`).

A estrutura em C não está nesta árvore de scripts: `bl_ui/__init__.py:68-96` só registra os módulos de UI; não há nada em Python sobre serialização de `bScreen`/`ScrArea`. O que dá para afirmar lendo estes arquivos é o de cima.

## O que o Spellcaster deve copiar

- **Ordem fixa de menus por painel: View, Select, Add, <substantivo do painel>** (`bl_ui/space_sequencer.py:225-238`, `bl_ui/space_node.py:263-266`). O orquestrador ganha `View / Select / Add / Módulo`; a timeline de cues ganha `View / Select / Add / Cue`; o ILDA player ganha `View / Select / Add / Frame`. Quem aprende um painel navega os outros sem ler nada. Cobre `design/PRINCIPIOS.md §4`.
- **O menu View abre com os toggles de região e fecha com o menu de área** (`bl_ui/space_sequencer.py:467-473`, `bl_ui/space_info.py:67-89`). "Onde sumiu o Inspector" e "quero essa janela em tela cheia" ficam sempre no mesmo lugar, em toda Face. Ajuda o operador de `PRINCIPIOS.md` às 23h, e casa com `Shift+1..7` e o Ctrl+crase já definidos em `design/SHORTCUTS.md`.
- **Um keymap de UI global que vale sobre qualquer campo de qualquer painel** (`keymap/:1002-1037`): `I` insere keyframe no campo sob o mouse, `Backspace` volta ao default, `Shift+Ctrl+C` copia o data path. Para o Spellcaster: `Backspace` volta o parâmetro ao valor da cena, e o equivalente do copy-data-path copia **o comando CLI daquele campo** — que é exatamente o que `PRINCIPIOS.md §4` exige de toda tela. Serve cenas/cues DMX, ILDA player e NDI→ILDA sem código por painel.
- **Estado como tinta no fundo do campo, com fator de mistura, não como borda ou ícone** (`bl_ui/space_userpref.py:1095-1133`; `Blender_Light.xml:322-340`). O Spellcaster tem um accent só, e o mesmo mecanismo resolve: um `blend` sobre o cinza do campo distingue *armado*, *ao vivo*, *em fade*, *sobrescrito por cue* sem inventar cor nova. E cada estado precisa de um par selecionado/não-selecionado, senão some quando a linha está selecionada. Vale para todos os quatro temas de `design/TEMAS.md`.
- **Três níveis distintos de "não age agora": `active` (apagado, editável), `enabled` (cinza, travado), `alert` (vermelho)** (`bl_ui/space_sequencer.py:1605`; `bl_ui/space_node.py:82`; `bl_ui/space_text.py:28-31`). Universo que não responde = `alert`. Fixture com cue mutado = `active` False, ainda editável. Saída não armada em modo ensaio = `enabled` False. Isso resolve o modo ensaio das cenas/cues DMX sem esconder controle nenhum.
- **Solo apaga o toggle de visibilidade em vez de escondê-lo** (`bl_ui/properties_data_bone.py:288-295`). No orquestrador e no patch de universos, solo em um módulo apaga os mute dos outros mas mantém todos na tela e clicáveis — a mesa não muda de forma no meio do show. Casa com `PRINCIPIOS.md §3`.
- **Paleta de comandos que indexa rótulos de menu, com busca por nome interno escondida atrás de "modo desenvolvedor"** (`bl_ui/space_topbar.py:505-507`). No Spellcaster o rótulo do menu já É o nome do registry, então a paleta indexa os dois de uma vez, e `SEARCH_ON_KEY_PRESS` (`bl_ui/space_node.py:273`) faz qualquer menu longo — lista de fixtures, lista de módulos do orquestrador, lista de frames ILDA — virar campo de busca ao primeiro caractere.
- **Painel de "ajustar última operação" acionável por tecla, em vez de desfazer e refazer** (`keymap/:839`, região `HUD`). Depois de um GO, o operador quer corrigir o fade sem repetir o cue. O Aprendiz é o lugar natural dessa superfície: ele já sabe o que acabou de acontecer.
- **Popovers no header em vez de diálogos**: Playback e Keying no Timeline são popovers, não menus nem janelas (`bl_ui/space_time.py:88-105`); o padrão toggle+seta aparece em todo editor (`bl_ui/space_sequencer.py:206-215`). Isso entrega o "zero modal para operação" de `design/SHORTCUTS.md` com um mecanismo só, reaproveitável no kpps/tamanho/cor do ILDA player e nos parâmetros de Canny do NDI→ILDA.
- **Dois gestos de container e rota, roubados inteiros para o orquestrador**: `Tab` entra e sai de qualquer container — meta-strip no sequencer, node group no editor de nós (`keymap/:3055`, `:2253`) — e serve igual para grupo de cues, sub-patch e clipe ILDA composto; e `delete_reconnect` (`bl_ui/space_node.py:394`, `keymap/:2237`) apaga um elo **religando o que estava dos dois lados**, que é exatamente tirar um módulo de conversão do meio da rota sem desmontar o patch.

## O que NÃO copiar

- **A tecla `Space` significar coisas diferentes conforme uma preferência** (`keymap/:774-788`). Blender tem três configurações mutuamente exclusivas para Space (ferramenta, busca, play) e o resto do keymap se remaneja em volta. Num show, a tecla mais usada não pode depender de configuração: `Space` é play e ponto, como já está em `design/SHORTCUTS.md`.
- **Duas famílias de atalho para a mesma ação por causa de legado** (`keymap/:844-858`, o ramo `params.legacy`). Blender carrega o keymap 2.7x inteiro como alternativa. Custo: qualquer mudança precisa ser feita duas vezes e testada nos dois ramos. O Spellcaster tem um mapa e remapeamento em `config.json`; não deve ter um "modo antigo".
- **Teclas do numpad como parte do vocabulário básico** (`NumPad .` para enquadrar seleção, `NumPad 0` para centrar no playhead, `NumPad +/-` para expandir canais — `keymap/:3060-3061`, `:3763-3764`). Um teclado de notebook na mesa de montagem não tem numpad. Enquadrar e navegar precisam viver em teclas principais (`Home`, `=`/`-`, `Shift+Z` já escolhidos em `SHORTCUTS.md`).
- **`X` como tecla de apagar** (`keymap/:2235`, `:3048`, `:3755`). Fica ao lado de `C` e `V` e não tem nada a ver com a palavra "delete". Num software onde apagar um cue no meio do show é irreversível na prática, apagar é `Delete`, e só.
- **Sete colunas de restrição configuráveis por preferência** (`bl_ui/space_outliner.py:412-421`). O usuário escolhe quais toggles existem, então dois operadores na mesma versão veem interfaces diferentes e a documentação não bate com a tela. Contraria `PRINCIPIOS.md §3` diretamente: o operador precisa achar o botão de olhos fechados, e ele só acha se o botão sempre esteve lá.
- **Espalhar o mesmo conceito em `mute`, `hide`, `disable`, `enable`, `protect`, `lock` e `restrict`** (`bl_ui/space_sequencer.py:990-1004` usa lock/mute; `bl_ui/space_dopesheet.py:827-831` usa mute/protect; `bl_ui/space_outliner.py:229-238` usa show/hide/enable/disable; `bl_ui/space_node.py:586-599` usa mute/hide/collapse). Quatro editores, quatro vocabulários para "não age agora". O Spellcaster tem um registry: **um verbo por conceito, em todo lugar** — `mute`, `lock`, `solo`, e nada mais.
- **Painéis do editor definidos fora da camada declarativa** — as abas "Active F-Curve" e "Active Keyframe" do Graph Editor não existem em Python (busca por esses rótulos em `scripts/**/*.py` não retorna nada; `bl_ui/space_graph.py` só declara popovers de header). Resultado: um add-on não consegue reordenar nem estender essas abas. No Spellcaster, se o Graph é a interface (`PRINCIPIOS.md §1`), então **todo** painel tem que ser declarado no mesmo lugar, sem exceção privilegiada.
- **Menu Strip com 25 itens sem busca** (`bl_ui/space_sequencer.py:1100-1197`). É o que `PRINCIPIOS.md §4` proíbe explicitamente. O próprio Blender reconhece o problema e resolveu só no menu Add, com `SEARCH_ON_KEY_PRESS` (`bl_ui/space_node.py:273`) — a solução existe, mas não foi aplicada onde mais dói.
- **Ícone sem rótulo em ação com consequência**: o header do sequencer tem oito controles seguidos que são `icon_only=True` ou `text=""` (`bl_ui/space_sequencer.py:173-215`), incluindo `overlap_mode`, que muda o que acontece quando um strip é solto em cima de outro. Rótulo em texto em tudo que muda comportamento, conforme `PRINCIPIOS.md §4`.
