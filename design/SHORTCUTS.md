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

| Ação | Tecla | Origem |
|---|---|---|
| Play / pause | `Space` | ambos |
| Shuttle recuar / parar / avançar | `J` / `K` / `L` | ambos |
| Quadro anterior / próximo | `←` / `→` | ambos |
| 5 quadros | `Shift+←` / `Shift+→` | Premiere |
| Início / fim | `Home` / `End` | ambos |
| Keyframe anterior / próximo (track focado) | `↑` / `↓` | Premiere (edit points) |
| Marcar In / Out | `I` / `O` | ambos |
| Limpar In / Out / ambos | `Alt+I` / `Alt+O` / `Alt+X` | Premiere |
| Ir para In / Out | `Shift+I` / `Shift+O` | Premiere |
| Loop no intervalo In–Out | `Ctrl+L` | Premiere |
| Marcador no playhead / editar | `M` / `Shift+M` | ambos |
| Marcador anterior / próximo | `Ctrl+Shift+←` / `Ctrl+Shift+→` | Resolve |
| Zoom in / out / ajustar | `=` / `-` / `Shift+Z` ou `\` | Premiere / Resolve |
| Pan na timeline | roda com `Shift`, ou arrastar com botão do meio | ambos |
| Keyframe no playhead (track focado) | `Ctrl+K` | Premiere (add edit) |
| Adicionar / remover keyframe por parâmetro | `Ctrl+Click` no losango | Resolve |
| Selecionar tudo / nada | `Ctrl+A` / `Ctrl+Shift+A` | ambos |
| Copiar / colar / cortar / apagar | `Ctrl+C` / `Ctrl+V` / `Ctrl+X` / `Delete` | ambos |
| Desfazer / refazer | `Ctrl+Z` / `Ctrl+Shift+Z` | ambos |
| Easing do keyframe selecionado | `Ctrl+E` abre menu; `Ctrl+Shift+E` cicla linear→in→out→inout→hold | (nosso) |
| Mute / solo do track focado | `Shift+D` / `Shift+S` | Premiere (disable) / (nosso) |
| Snapping liga/desliga | `S` | Premiere |
| Record arm do track focado | `R` | Resolve (Fairlight) |
| Novo / abrir / salvar / salvar como | `Ctrl+N` / `Ctrl+O` / `Ctrl+S` / `Ctrl+Shift+S` | ambos |
| Importar marcadores de vídeo/áudio | `Ctrl+I` | Premiere (import) |
| Exportar (render do show) | `Ctrl+M` | Premiere |
| Foco de painel: Patch, Timeline, Graph, Outputs, Network, Log, Inspector | `Shift+1` … `Shift+7` | Premiere (Shift+1..9) / Resolve (Shift+2..7 páginas) |
| Maximizar painel focado | `` Ctrl+` `` | Premiere |
| Workspace (Face `editor`) 1..9 | `Alt+Shift+1` … `Alt+Shift+9` | Premiere |
| Alternar Face `editor` ↔ `performance` | `Tab` (segurar para espiar, toque para trocar) | (nosso, PRD §10) |
| Tela cheia / kiosk | `Shift+F` | Resolve |
| Cue GO / voltar / pular para cue | `Enter` / `Backspace` / `Ctrl+G` | (nosso; consoles de luz) |
| Armar saídas reais / modo ensaio | `Ctrl+Shift+Enter` / `Ctrl+Shift+R` | (nosso, PRD §10) |
| Blackout (segurar) | `Esc` segurado 0,5 s | (nosso) |
| Fechar sem sair (volta ao editor) | `Esc` | ambos |

## Interface (o que copiar de cada um)

- **Premiere**: timeline com régua no topo, tracks empilhados com cabeçalho à esquerda (nome, mute, solo, lock, record arm), playhead vermelho que atravessa tudo, marcadores na régua, In/Out como barra cinza, painel Inspector ("Effect Controls") com keyframes ao lado do valor. Workspaces nomeados com abas no topo.
- **Resolve**: páginas como abas fixas no rodapé (aqui: Patch, Timeline, Graph, Outputs, Network, Log), viewer central grande (aqui: previz ou VU dos universos), Inspector à direita, "Media Pool" à esquerda (aqui: shows, perfis, clipes laser, faces). Botões de página são texto em caixa alta, sem ícone — combina com design/PRINCIPIOS.md §4.
- **Dos dois**: zero modal para operação; diálogo só para abrir/salvar. Tudo que muda estado tem atalho e aparece no menu com o atalho ao lado.
