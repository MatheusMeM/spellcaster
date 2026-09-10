# Atalhos e gramática de interface

Referência para R5 (GUI) e R9 (editores). Fonte: Adobe Premiere Pro e DaVinci Resolve; desde `design/FUNCOES/timeline-daw.md`, **Ableton Live** também é origem válida, para o que uma DAW resolve melhor que um NLE (automação, follow, loop na seleção). Regra: quem edita vídeo opera o Spellcaster sem aprender nada novo; o que não existe lá (cues, saídas, laser) segue a mesma lógica de modificadores. Atalhos remapeáveis em `config.json` (`"keys": {...}`), mas o mapa abaixo é o padrão e o que a documentação ensina.

## Gramática (herdada dos dois)

- Espaço toca e para. J/K/L é o shuttle: J recua, L avança, repetir acelera (×2, ×4, ×8), K para; K+J / K+L avança quadro a quadro (Premiere).
- Um par I/O define o intervalo de trabalho; tudo que é "loop", "render", "export" usa esse intervalo.
- Setas movem 1 quadro; Shift multiplica por 5 (Premiere) — aqui 1 quadro = 1/fps do show.
- Ctrl = comando, Shift = estende/amplia, Alt = variante/limpa. Nunca inventar quarta combinação.
- Painéis têm foco; o atalho age no painel focado (Premiere). Painel com foco tem borda de 1 px no accent (design/PRINCIPIOS.md §2).
- Zoom com `=` / `-`, ajustar tudo com `Shift+Z` (Premiere) ou `\` (Resolve: ambos valem).
- Marcadores: `M` cria no playhead, `Shift+M` edita, `Ctrl+Shift+←/→` navega (Resolve). Marcadores importados de corte de vídeo são marcadores comuns, cor cinza.
- Roda anda nos tracks, `Ctrl`+roda dá zoom, `Shift`+roda anda de lado, `Alt`+roda muda a altura das faixas (Ableton; o Resolve usa `Option` para zoom e `Command` para andar, e isso brigaria com `Ctrl` = comando). Nunca há scroll de página.
- Durante um arrasto, `Alt` solta o snap e `Shift` afina o valor (Ableton). O snap volta sozinho ao soltar o botão (Resolve).

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
| Loop na seleção de tempo (In/Out viram a seleção) | `Ctrl+L` com seleção ativa | Ableton (Loop Selection) | falta |
| Follow: a vista acompanha o playhead | `Alt+Shift+F` | Ableton | falta |
| Marcador no playhead | `M` | ambos | feito |
| Editar marcador | `Shift+M` | ambos | falta |
| Editar o marcador sob o playhead | `M` de novo | Resolve (manual p.548, p.781) | falta |
| Marcador anterior / próximo | `Ctrl+Shift+←` / `Ctrl+Shift+→` | Resolve | feito |
| Previz do playhead: DMX do track focado (entrada quando armado), quadro ILDA e planta do patch | `Alt+M` | (nosso; `M` já é marcador, `Ctrl+M` já é exportar) | feito |
| Zoom in / out / ajustar | `=` / `-` / `Shift+Z` ou `\` | Premiere / Resolve | feito |
| Voltar ao zoom anterior | `Shift+Z` de novo | Resolve (manual p.647) | falta |
| Enquadrar tudo na faixa de visão geral | duplo-clique na faixa | Ableton §6.1 | falta |
| Rolar as tracks (vertical) | roda | (nosso; a página não rola — quem rola são as tracks) | feito |
| Pan na timeline (horizontal) | `Shift`+roda, ou arrastar com botão do meio | ambos | feito |
| Zoom no cursor | `Ctrl`+roda | ambos | feito |
| Altura das faixas | `Alt`+roda, ou `Alt++` / `Alt+-` | Resolve p.648 (lá é `Shift`) / Ableton | falta |
| Dobrar / desdobrar as lanes do track focado | `U` | Ableton (Fold/Unfold) | falta |
| Travar / destravar o track focado | `Shift+L` | (nosso; par de `Shift+D` e `Shift+S`) | falta |
| Keyframe no playhead (track focado) | `Ctrl+K` | Premiere (add edit) | feito |
| Keyframe na curva, no tempo do clique | duplo-clique na lane | Ableton (Automação) | falta |
| Adicionar / remover keyframe por parâmetro | `Ctrl+Click` no losango | Resolve | falta |
| Empurrar a seleção 1 quadro / 5 quadros | `,` / `.` e `Shift+,` / `Shift+.` | Resolve (manual p.533, p.625) | falta |
| Duplicar a seleção | `Alt` + arrastar | Resolve / MadMapper | falta |
| Soltar o snap no meio do arrasto | segurar `Alt` | Ableton (Automação) | falta |
| Ajuste fino do valor ao arrastar | segurar `Shift` | Ableton (Automação) | falta |
| Desenhar automação (draw mode) | segurar `B` | Ableton | falta |
| Selecionar tudo / nada | `Ctrl+A` / `Ctrl+Shift+A` | ambos | feito |
| Buscar e criar nó no cursor (PATCHBAY) | `Shift+A` | Blender (Add) | feito |
| Copiar / colar / cortar / apagar | `Ctrl+C` / `Ctrl+V` / `Ctrl+X` / `Delete` | ambos | feito |
| Desfazer / refazer | `Ctrl+Z` / `Ctrl+Shift+Z` | ambos | feito |
| Easing do keyframe selecionado: menu | `Ctrl+E` | (nosso) | falta |
| Easing do keyframe selecionado: ciclar linear→in→out→inout→hold | `Ctrl+Shift+E` | (nosso) | feito |
| Mute / solo do track focado | `Shift+D` / `Shift+S` | Premiere (disable) / (nosso) | feito |
| Snapping liga/desliga | `S` | Premiere | feito |
| Record arm do track focado | `R` | Resolve (Fairlight) | feito |
| Novo / abrir / salvar como | `Ctrl+N` / `Ctrl+O` / `Ctrl+Shift+S` | ambos | falta |
| Salvar | `Ctrl+S` | ambos | feito |
| Importar marcadores de vídeo/áudio | `Ctrl+I` | Premiere (import) | falta |
| Exportar (render do show) | `Ctrl+M` | Premiere | n.a. |
| Ajuda (esta tabela + os comandos do registry) | `?` | (nosso) | feito |
| Página: TIMELINE, PATCHBAY, TEATRO, FACE, LASER | `Shift+1` … `Shift+5` | Resolve (Shift+2..7 páginas) | feito |
| Página AJUDA | `Shift+6` | Resolve (páginas) | feito |
| Foco de painel dentro da página | `Shift+7` … | Premiere (Shift+1..9) | n.a. |
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

- **Página `Shift+1..6`** é do `spellgui/web/nav.js`, a barra que está no topo das seis páginas; a
  tecla `?` funciona em todas porque essa mesma barra carrega o `help.js`.
- **Exportar** está `n.a.` porque não há render de show no registry; `Ctrl+M` continua reservado.
- **Blackout** está `falta`, e não `n.a.`, porque a ação existe: é o widget `blackout` da Face,
  que manda `input {key:"widget:blackout"}` ao Graph. O que falta é a tecla.

## Interface (o que copiar de cada um)

- **Premiere**: timeline com régua no topo, tracks empilhados com cabeçalho à esquerda (nome, mute, solo, lock, record arm), playhead vermelho que atravessa tudo, marcadores na régua, In/Out como barra cinza, painel Inspector ("Effect Controls") com keyframes ao lado do valor. Workspaces nomeados com abas no topo.
- **Resolve**: páginas como abas fixas no rodapé (aqui: Patch, Timeline, Graph, Outputs, Network, Log), viewer central grande (aqui: previz ou VU dos universos), Inspector à direita, "Media Pool" à esquerda (aqui: shows, perfis, clipes laser, faces). Botões de página são texto em caixa alta, sem ícone — combina com design/PRINCIPIOS.md §4.
- **Dos dois**: zero modal para operação; diálogo só para abrir/salvar. Tudo que muda estado tem atalho e aparece no menu com o atalho ao lado.
