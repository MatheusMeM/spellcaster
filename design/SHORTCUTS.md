# Atalhos e gramática de interface

Referência para R5 (GUI) e R9 (editores). Fonte: Adobe Premiere Pro e DaVinci Resolve. Regra: quem edita vídeo opera o Spellcaster sem aprender nada novo; o que não existe lá (cues, saídas, laser) segue a mesma lógica de modificadores. Atalhos remapeáveis em `config.json` (`"keys": {...}`), mas o mapa abaixo é o padrão e o que a documentação ensina.

## Gramática (herdada dos dois)

- Espaço toca e para. J/K/L é o shuttle: J recua, L avança, repetir acelera (×2, ×4, ×8), K para; K+J / K+L avança quadro a quadro (Premiere).
- Um par I/O define o intervalo de trabalho; tudo que é "loop", "render", "export" usa esse intervalo.
- Setas movem 1 quadro; Shift multiplica por 5 (Premiere) — aqui 1 quadro = 1/fps do show.
- Ctrl = comando, Shift = estende/amplia, Alt = variante/limpa. Nunca inventar quarta combinação.
- Painéis têm foco; o atalho age no painel focado (Premiere). Painel com foco tem borda de 1 px no accent (design/PRINCIPIOS.md §2).
- Zoom com `=` / `-`, ajustar tudo com `Shift+Z` (Premiere) ou `\` (Resolve: ambos valem).
- Marcadores: `M` cria no playhead, `Shift+M` edita, `Ctrl+Shift+←/→` navega (Resolve). Marcadores importados de corte de vídeo são marcadores comuns, cor cinza.

## Mapa padrão

A coluna **estado** é o que o código faz hoje (base `dac3e0a`, páginas de `spellgui/web`), não o
que se pretende: `feito` = a tecla está ligada na página dona da ação; `falta` = a ação existe no
produto e a tecla não; `n.a.` = a tela ou o modo que a tecla controla ainda não existe. Quem
mantém a coluna é quem mexe na tecla — `spellgui/web/help.html` lê esta tabela e a mostra ao
operador.

| Ação | Tecla | Origem | Estado |
|---|---|---|---|
| Play / pause | `Space` | ambos | feito |
| Shuttle recuar / parar / avançar | `J` / `K` / `L` | ambos | feito |
| Quadro anterior / próximo | `←` / `→` | ambos | feito |
| 5 quadros | `Shift+←` / `Shift+→` | Premiere | feito |
| Início / fim | `Home` / `End` | ambos | feito |
| Keyframe anterior / próximo (track focado) | `↑` / `↓` | Premiere (edit points) | feito |
| Marcar In / Out | `I` / `O` | ambos | feito |
| Limpar In / Out / ambos | `Alt+I` / `Alt+O` / `Alt+X` | Premiere | feito |
| Ir para In / Out | `Shift+I` / `Shift+O` | Premiere | feito |
| Loop no intervalo In–Out | `Ctrl+L` | Premiere | feito |
| Marcador no playhead | `M` | ambos | feito |
| Editar marcador | `Shift+M` | ambos | falta |
| Marcador anterior / próximo | `Ctrl+Shift+←` / `Ctrl+Shift+→` | Resolve | feito |
| Monitor DMX do track focado (512 barras) | `Alt+M` | (nosso; `M` já é marcador, `Ctrl+M` já é exportar) | feito |
| Zoom in / out / ajustar | `=` / `-` / `Shift+Z` ou `\` | Premiere / Resolve | feito |
| Pan na timeline, arrastando | botão do meio | ambos | feito |
| Pan na timeline, com a roda | roda com `Shift` | ambos | falta |
| Keyframe no playhead (track focado) | `Ctrl+K` | Premiere (add edit) | feito |
| Adicionar / remover keyframe por parâmetro | `Ctrl+Click` no losango | Resolve | falta |
| Selecionar tudo / nada | `Ctrl+A` / `Ctrl+Shift+A` | ambos | feito |
| Copiar / colar / cortar / apagar | `Ctrl+C` / `Ctrl+V` / `Ctrl+X` / `Delete` | ambos | feito |
| Desfazer / refazer | `Ctrl+Z` / `Ctrl+Shift+Z` | ambos | falta |
| Easing do keyframe selecionado: menu | `Ctrl+E` | (nosso) | falta |
| Easing do keyframe selecionado: ciclar linear→in→out→inout→hold | `Ctrl+Shift+E` | (nosso) | feito |
| Mute / solo do track focado | `Shift+D` / `Shift+S` | Premiere (disable) / (nosso) | feito |
| Snapping liga/desliga | `S` | Premiere | feito |
| Record arm do track focado | `R` | Resolve (Fairlight) | feito |
| Novo / abrir / salvar como | `Ctrl+N` / `Ctrl+O` / `Ctrl+Shift+S` | ambos | falta |
| Salvar | `Ctrl+S` | ambos | feito |
| Importar marcadores de vídeo/áudio | `Ctrl+I` | Premiere (import) | falta |
| Exportar (render do show) | `Ctrl+M` | Premiere | n.a. |
| Ajuda (esta tabela + os comandos do registry) | `?` | (nosso) | falta |
| Foco de painel: Patch, Timeline, Graph, Outputs, Network, Log, Inspector | `Shift+1` … `Shift+7` | Premiere (Shift+1..9) / Resolve (Shift+2..7 páginas) | n.a. |
| Maximizar painel focado | `` Ctrl+` `` | Premiere | n.a. |
| Workspace (Face `editor`) 1..9 | `Alt+Shift+1` … `Alt+Shift+9` | Premiere | n.a. |
| Alternar Face `editor` ↔ `performance` | `Tab` (segurar para espiar, toque para trocar) | (nosso, PRD §10) | n.a. |
| Tela cheia / kiosk | `Shift+F` | Resolve | falta |
| Cue GO | `Enter` | (nosso; consoles de luz) | feito |
| Cue voltar / pular para cue | `Backspace` / `Ctrl+G` | (nosso; consoles de luz) | falta |
| Armar saídas reais / modo ensaio | `Ctrl+Shift+Enter` / `Ctrl+Shift+R` | (nosso, PRD §10) | n.a. |
| Blackout (segurar) | `Esc` segurado 0,5 s | (nosso) | falta |
| Fechar sem sair (volta ao editor) | `Esc` | ambos | n.a. |

Onde a leitura da coluna não é óbvia:

- **Desfazer / refazer** está `falta` porque quem é dono da ação é a timeline (`timeline.js`), que
  não liga `Ctrl+Z`; no PATCHBAY (`graph.js`) as duas teclas já funcionam sobre `show_patch`.
- **Pan com a roda** está `falta` porque hoje `Shift`+roda rola na vertical (`canvaskit.js`), não
  na horizontal; arrastar com o botão do meio já faz o pan.
- **Ajuda `?`** está `falta` porque a página existe (`spellgui/web/help.html`, que liga a tecla em
  `help.js`) mas as outras páginas ainda não carregam `help.js` — passa a `feito` quando o
  `nav.js` entrar nelas.
- **Exportar** está `n.a.` porque não há render de show no registry; `Ctrl+M` continua reservado.
- **Blackout** está `falta`, e não `n.a.`, porque a ação existe: é o widget `blackout` da Face,
  que manda `input {key:"widget:blackout"}` ao Graph. O que falta é a tecla.

## Interface (o que copiar de cada um)

- **Premiere**: timeline com régua no topo, tracks empilhados com cabeçalho à esquerda (nome, mute, solo, lock, record arm), playhead vermelho que atravessa tudo, marcadores na régua, In/Out como barra cinza, painel Inspector ("Effect Controls") com keyframes ao lado do valor. Workspaces nomeados com abas no topo.
- **Resolve**: páginas como abas fixas no rodapé (aqui: Patch, Timeline, Graph, Outputs, Network, Log), viewer central grande (aqui: previz ou VU dos universos), Inspector à direita, "Media Pool" à esquerda (aqui: shows, perfis, clipes laser, faces). Botões de página são texto em caixa alta, sem ícone — combina com design/PRINCIPIOS.md §4.
- **Dos dois**: zero modal para operação; diálogo só para abrir/salvar. Tudo que muda estado tem atalho e aparece no menu com o atalho ao lado.
