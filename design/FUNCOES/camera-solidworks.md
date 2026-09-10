# Câmera no padrão SolidWorks — o manual, e o que o Spellcaster usa

O pedido do Matheus (09/09/2026): *"estude os comandos de movimentação e navegação de câmera do solidworks, ache o manual descrevendo o que os alt shift ctrl fazem ao interagir com click esq e click do meio e o scroll do mouse."*

Fonte: **SOLIDWORKS Design Help 2026** (help.solidworks.com), lido em 10/09/2026, uma URL por linha da tabela; e o **SolidWorks Quick Reference — Keyboard Shortcuts**, documento oficial da Dassault (`SWQRCENG06060`), para a tabela de teclado: https://files.solidworks.com/supportfiles/Release_Notes/2007/English/quick_reference.pdf

O motivo de copiar o SolidWorks e não o Blender: o objeto na tela é **um aparelho**, não uma cena. Quem opera CAD passa o dia girando uma peça em torno do ponto que clicou, e é exatamente esse gesto que a vista SHOW precisa. As outras duas vistas do programa não são CAD — e por isso não recebem o mapa inteiro (§3).

## 1. Mouse — o que o manual diz

| Gesto | Efeito no SolidWorks | Página do manual |
|---|---|---|
| Arrastar com o **botão do meio** | Rotate View (só peça e montagem) | https://help.solidworks.com/2026/english/SolidWorks/Sldworks/r_Middle_Mouse_Button.htm |
| **Clicar com o meio** num vértice, aresta ou face, depois arrastar com o meio | gira **em torno daquele ponto**, não do centro da tela | https://help.solidworks.com/2026/english/SolidWorks/Sldworks/r_Middle_Mouse_Button.htm |
| **Ctrl** + arrastar com o meio | Pan (em desenho 2D ativo o Ctrl não é preciso) | https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_pan_fundamentals.htm |
| **Shift** + arrastar com o meio | Zoom In/Out | https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_zoom_in_out.htm |
| **Alt** + arrastar com o meio | **Roll View** — gira a vista no plano da tela, em torno do centroide | https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_roll_view.htm |
| **Roda** para frente e para trás | Zoom **na posição do cursor**. Com o cursor fora da área gráfica, o zoom é no centro do modelo | https://help.solidworks.com/2026/english/SolidWorks/Sldworks/r_Middle_Mouse_Button.htm |
| **Roda**, com `View > Modify > Zoom About Screen Center` ligado | Zoom no centro da tela em vez de no cursor | https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_zoom_in_out.htm |
| **Roda**, com `Reverse mouse wheel zoom direction` ligado | inverte o sentido da roda | https://help.solidworks.com/2026/english/SolidWorks/sldworks/HIDD_OPTIONS_VIEW_ROTATION_display.htm |
| Arrastar com o **botão esquerdo**, com a ferramenta Rotate / Pan / Zoom ativa | o mesmo que o meio faz direto — o esquerdo só gira depois de escolher a ferramenta na barra View | https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_rotate_view.htm |
| **Zoom to Area**: arrastar uma caixa | enquadra a caixa | https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_zoom_to_area.htm |
| Botão **direito** na área gráfica → `Rotate about scene floor` | trava o eixo vertical: o modelo não tomba no horizonte | https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_rotate_view.htm |

Dois ajustes que o manual trata como opção de sistema e que aqui viram **calibração**, não constante escondida: `Mouse speed` ("para controle mais fino e rotação mais lenta, mova o slider para a esquerda") e `Arrow keys` (o incremento angular das setas), ambos em https://help.solidworks.com/2026/english/SolidWorks/sldworks/HIDD_OPTIONS_VIEW_ROTATION_display.htm

## 2. Teclado — o que o manual diz

Da referência rápida oficial (`quick_reference.pdf`, p. 1), verbatim na coluna do meio:

| Tecla | Efeito | Origem |
|---|---|---|
| Setas | `Rotate horizontally or vertically` (incremento configurável, 15° de fábrica) | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_rotate_view.htm |
| `Shift`+setas | `Rotate 90º` | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_rotate_view.htm |
| `Alt`+setas | `Rotate about screen center` = **roll** (o manual atual: "Hold down Alt and press the left-right arrow keys") | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_roll_view.htm |
| `Ctrl`+setas | `Pan` | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_pan_fundamentals.htm |
| `Z` / `Shift+Z` | `Zoom in/out` — **`Z` afasta, `Shift+Z` aproxima** | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_zoom_in_out.htm |
| `Ctrl+Shift+Z` | `Previous view` (desfaz até 10 mudanças de vista) | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_previous_view.htm |
| `F` | `Zoom to fit` | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_zoom_to_fit.htm |
| `Ctrl+1` … `Ctrl+7` | Front, Back, Left, Right, Top, Bottom, Isometric | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/r_standard_views_toolbar_2.htm |
| `Ctrl+8` | `Normal To` — olha perpendicular à face selecionada; de novo, vira 180° | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_viewing_models_normal_to.htm |
| `Espaço` | abre a caixa Orientation (vistas padrão e vistas nomeadas) | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/c_orientation_dialog_box.htm |
| `Ctrl+Espaço` | View Selector: o cubo de vistas; `Alt` seleciona as faces de trás | https://help.solidworks.com/2026/english/SolidWorks/sldworks/c_view_selector.htm |

## 3. O mapa que **este** programa usa

A regra 2 do `README.md` desta pasta vale aqui: cada gesto é um endereço textual (`cam.rotL`, `cam.fit`, `cam.front`), registrado em `Bind.def`, remapeável por tecla e por MIDI, e listado em `Bind.manifest()`. O que muda por vista **não é o teclado, é a lei da câmera** — `CAM.mode("free" | "rear" | "inside")`, em `cam.js`.

Três vistas, três leis. O aparelho é o modelo; a vista diz o que se pode fazer com ele.

| Gesto | `show` = **free** (o visualizador) | `rear` = **fixo** (a traseira é o menu) | `inside` = **restrito** (tampa aberta) |
|---|---|---|---|
| Arrastar meio | gira em torno do ponto clicado | — | gira dentro dos limites |
| Ctrl+meio | pan | — | — |
| Shift+meio | zoom | — | — |
| Alt+meio | **roll** | — | — |
| Arrastar esquerdo no vazio | gira (notebook sem botão do meio) | — | gira dentro dos limites |
| Arrastar esquerdo **numa peça** | é da peça, nunca da câmera (o knob: §4 de `FUNCOES/README` regra 3 — valor é um tipo, e o gesto de valor é o do TouchDesigner) | idem | idem |
| Roda no vazio | zoom no cursor, passo pequeno | — | — |
| Roda sobre o encoder | gira o encoder | gira o encoder | — |
| Setas / `Shift`+setas / `Ctrl`+setas / `Alt`+setas | 15° · 90° · pan · roll | — | — |
| `F`, `Ctrl+1..7`, `Z`, `Shift+Z` | como o SolidWorks | — | — |
| `1` `2` `3` | troca de vista (é do programa, não do SolidWorks) | idem | idem |

**`show` — free.** O SolidWorks inteiro, com três coisas que o SolidWorks não precisa ter e um visualizador de laser precisa:

- **Amortecimento crítico** em vez do `lerp` de fator fixo. `lerp(k = dt·5)` depende do frame rate e nunca chega: a câmera "nada" atrás do mouse e continua andando depois que o botão soltou. Mola com `ζ = 1` (`x += v·dt ; v += (−2ω·v − ω²·(x−alvo))·dt`, `ω = 18`) chega ao alvo sem passar dele e para.
- **Sensibilidade proporcional à distância**: girar e pan com a câmera a 20 cm do aparelho não pode andar o mesmo tanto que a 3 m. É o `Mouse speed` do manual, só que automático.
- **Limites**: `d ≥ R + folga` (R = raio da caixa do aparelho), `d ≤` metade da sala, `|pitch| ≤ 85°`, alvo dentro da sala. A câmera nunca entra no aparelho nem atravessa a parede.

**`rear` — fixo.** A traseira **é o menu**; câmera solta em cima de um menu é a mesma coisa que um menu que anda quando você passa o mouse. A pose é calculada da normal do painel traseiro, enquadrando os 400 × 180 mm com folga, e **recalculada quando a janela muda de tamanho** (o enquadramento depende do aspect). Sem arrasto, sem meio, sem roda-zoom, sem setas. O único movimento é um respiro de ±2° que segue o mouse, e ele **não muda a distância**: é paralaxe, não navegação.

**`inside` — restrito.** Órbita em torno do centro da mesa óptica, `yaw ∈ [−60°, +60°]` em relação à normal da tampa aberta, `pitch ∈ [20°, 80°]`, **distância fixa**. É a mesma ideia do `Rotate about scene floor` do manual (travar um eixo para o modelo não tombar), levada ao limite: aqui o que se trava é a caixa inteira, para a tampa aberta nunca entrar no quadro por trás da câmera.

## 4. O que fica de fora, e por quê

| Do SolidWorks | Por quê não |
|---|---|
| `Zoom to Area` (caixa de seleção) | o arrasto com o esquerdo já é gira-no-vazio; dar duas funções ao mesmo botão pede uma ferramenta modal, e ferramenta modal em cima de um aparelho é exatamente o que este programa não é |
| `Previous view` (`Ctrl+Shift+Z`) com pilha de 10 | as vistas do programa são três, nomeadas, com tecla: `1`, `2`, `3`. Uma pilha de vistas anônimas por cima disso é estado escondido |
| `Espaço` = caixa Orientation, `Ctrl+Espaço` = View Selector | `Espaço` já é play (regra 11 do `FUNCOES/README`: "Space é play, e ponto"). As vistas padrão continuam em `Ctrl+1..7` |
| `Ctrl+8` Normal To | precisa de face selecionada, e aqui clicar numa peça é abrir a tela dela, não selecioná-la |
| `Zoom About Screen Center` como opção | uma opção a menos: a roda sempre vai no cursor (regra do ponytail — sem config para valor que não muda) |
| `Rotate about scene floor` como toggle | o `pitch` limitado já impede o tombo, nas três vistas, sem toggle |
| Trimetric / Dimetric | `Ctrl+7` isométrico basta |
| `Reverse mouse wheel zoom direction` | **fica**, e continua ligado por padrão (`cam.reverse`, persistido em `localStorage`), porque é a única opção do manual que existe justamente porque metade das pessoas quer o contrário da outra metade |
