# Cenário interativo — a maquete (tema TEATRO DE PAPEL)

O viewer central como maquete do palco: os aparelhos, o laser e as saídas desenhados sobre uma planta ou foto, clicáveis. É a Face sobre o patch; não tem comando próprio porque cada clique dispara um comando que já existe.

| Item | Quem resolveu melhor | Por quê |
|---|---|---|
| Objetos e verbos | Capture | O modo Foco resolve a ambiguidade central: o mesmo clique não pode significar "quero este aparelho" e "aponte para cá"; camada com `travada`, `não selecionável` e `sem simulação` separados; posições de câmera em catálogo |
| Estados | Resolume, com Blender e Capture | Cinco estados por célula, um accent; retorno pelo estado real, não pelo comando enviado; solo apaga sem remover; simulação desligável sem apagar |
| Zonas da tela | Capture, com o MadMapper | Vista nomeada pelo uso, não Alpha/Beta/Gamma; painel de controle só em Ao vivo; input view e output view são duas vistas do mesmo objeto |
| Atalhos | Capture | A tabela de modificadores da vista 3D é completa e consistente; Blender dá `H`/`Shift+H`/`Alt+H` |
| Arquivo | Capture | Posição de cenário separada de valor de canal; importação com mapeamento de colunas; o dashboard do Chataigne mostra o que não gravar (aparência no item) |

## 1. Objetos e verbos

**Maquete.** Uma vista sobre o patch. Fundo: imagem de referência que não sai na saída (o `background` de output do MadMapper, e a imagem de fundo do Morpher do Chataigne) [fontes/madmapper.md § Objetos e verbos], [fontes/chataigne.md § Objetos e verbos]. Guias de grade e de imagem, os dois `ScreenGuide` do Resolume [fontes/resolume.md § Fixtures e DMX].

**Recorte.** Um aparelho, grupo, saída laser ou cue posto na maquete. Sabe virar item a partir de qualquer controlável (`createDashboardItem()`) [fontes/chataigne.md § Objetos e verbos]: arrastar da tabela do patch para a maquete cria o recorte. Campos: alvo (endereço do registry), posição, tamanho, camada, rótulo. Aparência (cor, contorno, fonte) **não** é campo do recorte [fontes/chataigne.md § O que NÃO copiar]; é do Theme.

**Camada.** Não é só visibilidade: `travada`, `não selecionável`, `incluir em relatórios`, `simular` [fontes/capture.md § Objetos e verbos]. "Dá para desligar a simulação de uma camada inteira sem apagá-la": é o botão de performance na hora do show.

**Vista.** Nomeada pelo uso: `palco`, `plateia`, `varas`; nunca Alpha, Beta, Gamma [fontes/capture.md § O que NÃO copiar]. Posições de câmera guardadas em catálogo e disparáveis por DMX [fontes/capture.md § Objetos e verbos].

**Posição de cenário.** A "Scene" do Capture: posição e visibilidade de objetos, sem valor de canal [fontes/capture.md § Objetos e verbos]. Guarda "onde as coisas estão no segundo ato"; a cue guarda "quanto de luz". Nomes diferentes porque são coisas diferentes.

**Ponto de foco.** Onde os aparelhos selecionados apontam.

Verbos, do menu de contexto da cena do Capture [fontes/capture.md § Objetos e verbos] e do modo de edição do MadMapper:

| Verbo | Gesto | Origem |
|---|---|---|
| Selecionar | clique; `Shift` soma; `Ctrl` alterna; caixa da esquerda para a direita pega só o que está inteiro dentro, da direita para a esquerda também o que encosta; duplo clique desce um nível no grupo | Capture [fontes/capture.md § Cena 3D e interação] |
| Apontar (modo Foco) | com o modo ligado, clicar num ponto **não seleciona**: aponta os aparelhos já selecionados para lá | Capture, Focus mode |
| Mover / girar | arrastar dentro do contorno; `Shift` trava ortogonal e 5°; `Ctrl` desliga o encaixe; região externa do triângulo gira cada um no próprio eixo | Capture |
| Fan de pan/tilt | `Ctrl`+arrastar escala, `Alt`+arrastar desloca | Capture |
| Atribuir | arrastar gel ou gobo sobre a seleção; aqui, arrastar cue ou cena sobre o recorte | Capture `Common/Assign` |
| Cuear | em modo editar cue, clicar num recorte já selecionado inclui/exclui sua geometria na cue (o MadMapper cuea "Input Geometry" no input view, "Output Geometry" no output view) | MadMapper [fontes/madmapper.md § Cenas e cues] |
| Isolar | solo | Blender Outliner, "Isolate primeiro, sozinho" [fontes/blender.md § Objetos e verbos] |
| Esconder / esconder o resto / revelar | `H` / `Shift+H` / `Alt+H` | Blender |
| Guardar câmera | posição atual vira entrada do catálogo | Capture |
| Medir | clique inicia, clique termina, `Shift`+clique adiciona ponto, `Esc` limpa | Capture |
| Talkback | clicar na maquete devolve pan/tilt à fonte que controla o aparelho (mesa por OSC, CITP) | Capture, DMX talkback [fontes/capture.md § O que copiar] |
| Morfar | cursor X/Y entre cenas postas como pontos, peso por Voronoi | Chataigne Morpher [fontes/chataigne.md § O que copiar] |

Talkback muda a natureza do nó: a maquete é **entrada** do graph, "igual a um módulo OSC" [fontes/capture.md § O que copiar]. Seleção sincronizada nos dois sentidos com a mesa (CITP/FSEL, EOS/OSC) é o mesmo mecanismo.

## 2. Estados

Um accent por recorte, com o enum de cinco estados do Resolume ("clip triggers can have 5 different states") [fontes/resolume.md § O que copiar]: `vazio` (sem patch), `patcheado`, `armado`, `ao vivo`, `erro`. Mais `selecionado` como par de cada um (regra 4).

**Retorno pelo estado real.** O recorte acende com o que o aparelho está recebendo, lido do engine, não com o comando que a maquete enviou; é o `OutputSiblingPath` do Resolume, "o recorte na maquete acende com o estado real, não com o comando enviado" [fontes/resolume.md § O que copiar].

**Dirigido por**: tinta `inner_driven` quando o valor vem de cabo, cue ou Parrot (`isControlledByParrot`) [fontes/chataigne.md § Estados visuais]; o operador vê que mexer ali não vai adiantar.

**Solo** apaga os toggles dos outros recortes sem removê-los [fontes/blender.md § Objetos e verbos]. **Camada sem simulação**: recortes apagados, presentes, clicáveis (`active` falso) [fontes/blender.md § Estados]. **Camada travada**: `enabled` falso. **Universo do recorte sem resposta**: `alert`.

**Modo ativo** (Foco, Medir, Editar cue): indicador na status bar e no cursor; nunca só no cursor. O Capture sai do Foco pelo botão do Selection Navigator [fontes/capture.md § Cena 3D e interação]; aqui `Esc`.

**Valores mistos** na seleção múltipla, string própria [fontes/capture.md § Estados e mensagens].

**Custo visível**: `Taxa | Detalhe`, qualidade adaptativa, limite de resolução, informação de desempenho [fontes/capture.md § Cena 3D e interação]. Aviso quando o custo sobe, não quando o teto estoura (`TooManySmokeObjects` chega tarde) [fontes/capture.md § O que NÃO copiar].

## 3. Zonas da tela

- **Centro, viewer**: a maquete. É o "viewer central grande" de `SHORTCUTS.md § Interface`. Abas de vista nomeadas no topo do viewer (`palco | plateia | varas`), como as abas de workspace do Blender [fontes/blender.md § Arquivo]. Duas vistas do mesmo objeto quando fizer sentido: input (o que entra) e output (o que sai), lado a lado ou uma só, com o toggle do MadMapper [fontes/madmapper.md § Anatomia da tela].
- **Esquerda**: camadas, com os quatro toggles por linha, e o catálogo de posições de câmera.
- **Direita, Inspector**: o do aparelho selecionado (`cenas-cues-dmx.md § 3`). O Selection Navigator do Capture, que "aparece junto da seleção" [fontes/capture.md § Anatomia da tela], não entra como painel flutuante (`PRINCIPIOS.md §3`); seus verbos vão para o menu de contexto e para a paleta, e o que ele mostra vai para o Inspector, que não se move.
- **Painel de controle** por tipo de aparelho selecionado, "só existe no modo Live" [fontes/capture.md § Cena 3D e interação]: aqui, só na Face performance, com Luz (liga/desliga a seleção), Home, e os parâmetros do tipo.
- **Status bar**: o que os botões do mouse fazem agora, no modo atual [fontes/blender.md § Estados]. É onde a tabela de modificadores do §4 é ensinada, gesto a gesto.

Menu: `View, Select, Add, Recorte`. Select traz os dez critérios do Capture (por camada, por localização, por modelo, por tipo, por grupo, …) [fontes/capture.md § Objetos e verbos]. Nenhuma janela flutuante (relatório, planta, patch da mesa): "às 23h em sala escura é janela perdida atrás de outra" [fontes/capture.md § O que NÃO copiar].

## 4. Atalhos

Modificadores na maquete, tabela do Capture, inteira [fontes/capture.md § Atalhos]:

| Ação | Entrada |
|---|---|
| Orbitar / panoramizar | botão do meio, ou `Alt`+arrastar; `Shift` segurando troca um pelo outro |
| Girar sem mover a câmera | `Ctrl` |
| Zoom movendo o ponto focal / mudando o campo de visão | `Shift`+roda / `Ctrl`+roda |
| Somar à seleção / alternar item | `Shift`+clique / `Ctrl`+clique |
| Caixa: só inteiro dentro / também o que encosta | arrastar → / arrastar ← |
| Descer um nível no grupo | duplo clique |
| Mover ortogonal, girar de 5°, ajuste fino de slider | `Shift` |
| Desligar encaixe | `Ctrl` |
| Fan escala / fan offset | `Ctrl`+arrastar / `Alt`+arrastar |

Teclas que entram:

| Ação | Tecla | Origem | Conflito |
|---|---|---|---|
| Focar o viewer (a maquete) | `Shift+8` | (nosso) | `SHORTCUTS.md` para em `Shift+7`; fecha o ponto aberto do README |
| Modo Foco | `F` | (nosso; Capture não tem tecla) | nenhum; em `ndi-ilda.md` `F` congela quadro, mas ali o viewer é outro e o atalho age no painel focado |
| Sair do modo | `Esc` toque | Capture, `SHORTCUTS.md` | segurado é blackout |
| Esconder / esconder o resto / revelar | `H` / `Shift+H` / `Alt+H` | Blender [fontes/blender.md § Atalhos] | nenhum |
| Solo | `Shift+S` | `SHORTCUTS.md` | |
| Posição de câmera 1..9 | `Alt+1` … `Alt+9` | (nosso; Blender usa numpad, proibido pela regra 11) | nenhum; `Alt+Shift+1..9` continua sendo workspace |
| Guardar posição de câmera | `Ctrl+Alt+1..9` | (nosso) | nenhum |
| Selecionar tudo / nada / inverter | `Ctrl+A` / `Ctrl+Shift+A` / `Ctrl+I` | `SHORTCUTS.md` + Blender | `Ctrl+I` é importar marcadores em `SHORTCUTS.md`; no viewer focado, inverter; ponto a registrar |
| Enquadrar tudo / seleção | `Shift+Z` / `\` | `SHORTCUTS.md` | |
| Medir | `Shift+M` no viewer | (nosso) | `Shift+M` edita marcador na timeline; painel focado decide |

Regra do Capture que não entra: atalho só com `Ctrl`+letra, "nenhuma tecla nua" [fontes/capture.md § O que NÃO copiar].

## 5. Arquivo

No `.spell`, chave `stage`:

- `views[]`: `{uid, name, background: {path, relpath, backup}, camera: {…}, guides}`.
- `items[]`: `{uid, target (endereço), view, layer, x, y, w, h, rotation, label}`. Geometria do recorte é dado do show porque é a planta física, como `PositionX/Y/Z` de `Object` no Capture [fontes/capture.md § Patch e universos]. Cor, contorno, fonte, opacidade de fundo: nunca (o `DashboardControllableItem` do Chataigne grava `textColor`, `contourColor`, `opaqueBackground`, e mistura dado com tema) [fontes/chataigne.md § Estados visuais].
- `layers[]`: `{uid, name, visible, locked, selectable, simulate}`.
- `positions[]` (posições de cenário): `{uid, name, items: {uid: {x, y, visible}}}`, separado de `cues[]`.
- `cameras[]`: `{uid, name, view, …}`, disparável por cue como qualquer endereço.

Fora do `.spell`: seleção, zoom atual, modo ativo, qualidade adaptativa.

Importar: CSV de aparelhos com mapeamento de colunas para posição, rotação, unidade, foco de pan/tilt, canais de modo, identificando o aparelho por uma propriedade escolhida, com relatório do que foi atualizado, adicionado e pulado [fontes/capture.md § Arquivo]. Exportar: planta e relatório (equipamento, ponto de ancoragem, cabo, aparelho, grupo, localização) [fontes/capture.md § Objetos e verbos]; símbolos de planta de SVG.
