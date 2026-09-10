# design/FUNCOES — a parte prática de cada função

Regra do Matheus (09/09/2026): não se desenha UI para software sem função. Antes de qualquer skin, cada função do Spellcaster precisa ter respondidas cinco perguntas, e a resposta vem de ler como Blender, TouchDesigner, Resolume Arena, MadMapper, Capture e Chataigne resolveram a mesma coisa, nos arquivos instalados nesta máquina, não de impressão.

Um arquivo por função, sempre com as mesmas seções:

1. **Objetos e verbos** — o que a função chama de coisa e o que se faz com cada uma.
2. **Estados** — parado, armado, ao vivo, erro, pendente; como aparecem.
3. **Zonas da tela** — o que fica onde por padrão e o que é sempre visível.
4. **Atalhos** — o que entra além de `design/SHORTCUTS.md`, com a origem.
5. **Arquivo** — o que a função salva no `.spell`, e o que fica fora.

E no topo de cada um, a tabela de veredito: qual app resolveu melhor cada item.

| Função | Arquivo | Comando | Tema |
|---|---|---|---|
| ILDA player | `ilda-player.md` | `spell ilda play` | LASER |
| NDI → ILDA | `ndi-ilda.md` | `spell ilda from-ndi` | FÓSFORO |
| Orquestrador | `orquestrador.md` | `spell graph` | PATCHBAY |
| Cenas e cues DMX | `cenas-cues-dmx.md` | `spell cue`, `spell scene`, `spell patch` | TEATRO DE PAPEL |
| Cenário interativo | `cenario-interativo.md` | (Face sobre o patch) | TEATRO DE PAPEL |
| Aprendiz (menu principal) | `aprendiz-menu.md` | paleta de comandos | todos |
| Laser como módulo do graph | `integracao-laser.md` | `spell graph add laser/1` | LASER |
| Timeline como DAW | `timeline-daw.md` | `spell timeline` (painel `Shift+2`) | todos |
| Timeline como Arrangement View | `daw-arranjo.md` | `spell timeline` | (editor) |
| Session View (cues em grade) | `daw-sessao.md` | `spell cue` | TEATRO DE PAPEL |
| Browser e drag and drop | `browser-dnd.md` | (painel `Alt+5`) | (editor) |
| Modo de mapeamento | `mapping.md` | `Ctrl+Shift+A` | todos |
| Áudio e vídeo | `audio-video.md` | `spell play` | (editor) |
| Pontos falhos da interface atual | `pontos-falhos.md` | (auditoria) | — |

Os seis últimos vieram da frente `daw-pesquisa`: o pedido do Matheus de *"uma interface de uso e funcionalidade e usabilidade e interface que nem o ABLETON"*, com drag and drop de mídia e saídas e mapeamento *"que nem o resolume Ctrl shift A"*. As fontes deles são os manuais do **Ableton Live 12** (online, citado por seção), do **DaVinci Resolve 20** (PDF local, citado por página) e do **Resolume Arena** (online, citado por página); a citação fica na própria linha, como em `timeline-daw.md`, e não há arquivo em `fontes/` para eles.

As auditorias por app, com `path:linha` em cada afirmação, estão em `fontes/` (`blender.md`, `touchdesigner.md`, `resolume.md`, `madmapper.md`, `capture.md`, `chataigne.md`). Os arquivos de função citam as fontes como `[fontes/app.md § Seção]`; a evidência mora lá, não aqui. Exceção: `timeline-daw.md` cita Ableton Live e DaVinci Resolve na própria linha da tabela (seção do manual, ou página do PDF instalado), porque cada afirmação vale uma linha e um arquivo de auditoria seria a mesma tabela de novo.

## Regras transversais

Apareceram em mais de um app e valem para todas as seis funções. Cada arquivo de função assume estas e só acrescenta o que é dele.

1. **Parâmetro tipado gera o widget; ninguém desenha widget por comando.** Chataigne tem onze tipos e um `createDefaultUI` por tipo; TouchDesigner serializa um parâmetro com `name, label, page, style, size, default, min/max, normMin/normMax, enable, readOnly, help`; MadMapper declara `LABEL, TYPE, MIN, MAX, DEFAULT, FLAGS` e o `LABEL` com `/` monta a árvore de grupos; Resolume separa o valor (`ParamRange`) da apresentação (`ParameterView`: `suffix, step, display_units, control_type`). O esquema mínimo do Spellcaster, por parâmetro do registry: `name` (o da CLI), `label`, `type` (trigger, bool, int, float, enum, string, color, xy, target, file), `min/max` (clamp físico), `norm` (faixa do slider, separada do clamp), `default`, `unit`, `readonly`, `enabled`, `group` (via `/` no label), `flags` (button, momentary, spinbox), `help` (uma frase). [fontes/chataigne.md § Tipos de parâmetro], [fontes/touchdesigner.md § Arquivo], [fontes/madmapper.md § Parâmetros de material], [fontes/resolume.md § Tipos de parâmetro]
2. **Um endereço textual é a identidade de tudo.** Em Resolume `/composition/layers/1/clips/3/connect` é ao mesmo tempo atalho de teclado, endereço OSC, alvo MIDI e rota REST; em MadMapper a cue guarda `/fixtures/9/color/red` e o shader lê `/custom/BPM/bpmPos`; em Chataigne todo `Controllable` tem endereço e o `TargetParameter` aponta para ele. É o `spellcaster.core.registry` validado três vezes: CLI, OSC, cue, mapeamento, MCP e Aprendiz usam o mesmo nome, e a GUI mostra esse nome. [fontes/resolume.md § O que copiar], [fontes/madmapper.md § Cenas e cues]
3. **Trigger, toggle e valor são tipos diferentes desde o esquema.** Resolume tem `ParamEvent` (só dispara), `ParamBoolean` e `ParamRange`; Chataigne tem `Trigger` separado de `Bool`, e `CommandContext {ACTION, MAPPING, BOTH}` diz se um comando aceita disparo, valor ou os dois; TouchDesigner tem `Pulse`, `Momentary` e `Toggle` como estilos distintos. Cada `@command` do registry declara qual é. [fontes/resolume.md § Tipos de parâmetro], [fontes/chataigne.md § Objetos e verbos]
4. **Estado é tinta misturada no fundo do campo, com par selecionado.** Blender tem um bloco de tema só para estado (`error, warning, info, success, inner_anim, inner_key, inner_driven, inner_overridden, inner_changed`) com fator `blend` e uma versão `_sel` de cada; o widget não muda de forma, só de cor. Três níveis de "não age agora": `active` (apagado, editável), `enabled` (travado), `alert` (vermelho). TouchDesigner acrescenta o estado que faltava em `PRINCIPIOS.md §2`: **pendente** (mudança não aplicada), vermelho. [fontes/blender.md § Estados], [fontes/touchdesigner.md § Estados]
5. **Erro e aviso são dado, não só pintura.** Todo nó do TouchDesigner expõe `warnings` e `errors` como contagem; Chataigne tem um `WarningReporter` central onde qualquer objeto registra `setWarningMessage(msg, id)` e a linha do painel leva ao culpado. No Spellcaster o engine é a fonte única: painel, log, `spell` por SSH e Aprendiz leem o mesmo. [fontes/touchdesigner.md § Estados], [fontes/chataigne.md § Estados visuais]
6. **Ordem fixa de menu: View, Select, Add, <objeto do painel>.** O menu View começa com os toggles de região e termina com o bloco de área (split, maximizar, fechar). Verificado em onze editores do Blender. [fontes/blender.md § Anatomia]
7. **Um keymap de UI global vale sobre qualquer campo de qualquer painel.** No Blender `I` insere keyframe no campo sob o mouse, `Backspace` volta ao default, `Shift+Ctrl+C` copia o data path. No Spellcaster: `I` grava o campo na cena ou timeline focada, `Backspace` volta ao valor de cena, `Shift+Ctrl+C` copia o comando CLI daquele campo. [fontes/blender.md § Atalhos]
8. **Duas tabelas de atalho, editor e performance.** TouchDesigner desabilita `Space` dentro de painel e em Perform Mode; só `Shift+Space` pausa, de propósito. `SHORTCUTS.md` ganha uma coluna "Face performance" onde o transporte exige `Shift`. Desabilitar atalho = manter a linha e apagar o comando, para o mapa continuar auditável. [fontes/touchdesigner.md § Atalhos]
9. **Um verbo por conceito.** Blender usa `mute/hide/disable/protect/lock/restrict` para a mesma coisa em quatro editores. Aqui: `mute`, `lock`, `solo`, e nada mais. Solo apaga o toggle dos outros mas não o remove. [fontes/blender.md § O que NÃO copiar]
10. **Paleta de comandos indexa o rótulo do menu, e o rótulo é o nome do registry.** F3 do Blender indexa menus, não operadores; `SEARCH_ON_KEY_PRESS` transforma qualquer menu longo em busca ao primeiro caractere. Menu com mais de 8 itens vira busca. [fontes/blender.md § Paleta de comandos]
11. **Sem numpad, sem `X` para apagar, sem `Space` configurável.** Apagar é `Delete`. `Space` é play, e ponto. [fontes/blender.md § O que NÃO copiar]
12. **Arquivo `.spell`: JSON completo, versionado, com UID.** Não gravar só o delta em relação ao default (Resolume e Chataigne fazem isso e o arquivo fica inauditável); `"version"` no topo e migração local, não conversor remoto (Chataigne) nem renomeação silenciosa de chave (Resolume `Beats_d → Beats_double`); referência entre objetos por UID, nunca por nome curto (Chataigne `sourceState`, Resolume `OscShortcutPreset`); nada de thumbnail (Resolume, MadMapper), nada de estado de janela (MadMapper `previewsData`, Chataigne `layout`) dentro do show. Referência a arquivo externo carrega cópia de segurança e `relpath` declarado (TouchDesigner `savebackup`). [fontes/resolume.md § Arquivo], [fontes/madmapper.md § Arquivo], [fontes/chataigne.md § Arquivo], [fontes/touchdesigner.md § Arquivo]
13. **Layout salvo é preset nomeado com conteúdo declarado, não 33 checkboxes.** Resolume liga cada painel por um booleano `Show X`; Chataigne serializa uma árvore de docks com direção e tamanho preferido. Copiar o formato da árvore para as Faces; não copiar o arraste livre nem os checkboxes. [fontes/resolume.md § Anatomia], [fontes/chataigne.md § Anatomia]
14. **Módulo, perfil e ajuda são texto legível.** MadMapper cifra os módulos de fábrica (`main.ldat`); Chataigne declara módulo em `module.json` com `parameters, values, commands, dependency`; Resolume guarda a ajuda contextual como `elemento → uma frase` em XML. Tudo que o Spellcaster carrega é texto versionável. [fontes/madmapper.md § O que NÃO copiar], [fontes/chataigne.md § Arquivo], [fontes/resolume.md § O que copiar]

## Pontos abertos (não decididos aqui)

- `SHORTCUTS.md` tem `Shift+1..7` para sete painéis; `cenario-interativo.md § Atalhos` propõe `Shift+8` para o viewer/maquete.
- O catálogo de nós do PRD §10 não tem nó de **estado** (State Machine); `orquestrador.md § Objetos` mostra o que Chataigne faz e o que falta. Decisão vai para `DECISOES.md`.
- `Tab` é a troca de Face em `SHORTCUTS.md`, então entrar/sair de grupo no graph fica em `Ctrl+]`/`Ctrl+[`; `Ctrl+X` é recortar, então apagar religando fica em `Shift+Delete` (`orquestrador.md § Atalhos`).
- `Ctrl+B` (cue na posição do playhead, Chataigne) e `Shift+PageUp/PageDown` (cue anterior/próximo) não estão em `SHORTCUTS.md` e não conflitam com nada; propostos em `cenas-cues-dmx.md`, junto com `Shift+E` (modo editar cue).
- `F3` (paleta), `Q` (favoritos), `F9` (ajustar última operação), `Shift+R` (repetir), `F1` (ajuda do elemento) vêm do Blender/TD, não conflitam, e são o teclado do Aprendiz (`aprendiz-menu.md`).
- Teclas nuas por painel focado: `V` (modo do viewer) e `F` (congelar) em `ndi-ilda.md`; `F` (modo Foco), `H`/`Shift+H`/`Alt+H`, `Alt+1..9` (câmeras) em `cenario-interativo.md`; `Shift+T` (padrão de teste) em `ilda-player.md`. `Ctrl+I` colide (importar marcadores × inverter seleção) e precisa de decisão.
- A rodada 4 em `design/0.1.2` renomeou o Aprendiz para Pino; estes arquivos ainda dizem Aprendiz.
