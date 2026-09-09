# Cenas e cues DMX — `spell patch`, `spell scene`, `spell cue` (tema TEATRO DE PAPEL)

Patchear aparelhos em universos, gravar valores em cenas e cues, e disparar em ordem com fade. Cobre os painéis Patch (`Shift+1`), Outputs (`Shift+4`) e a lista de cues dentro da Timeline (`Shift+2`).

| Item | Quem resolveu melhor | Por quê |
|---|---|---|
| Objetos e verbos | MadMapper para a cue, Capture para o patch, Chataigne para a lista | Cue = lista de pares (endereço, valor) com fade por entrada; aparelho tem quatro números e numeração sequencial em lote; o diálogo de patch diz quantos canais consome antes de gravar; Conductor é a lista de GO |
| Estados | Resolume, com o pendente do TouchDesigner e o overlay do MadMapper | `connected` é enum de cinco estados, não booleano; `connect` é "botão pressionado", momentâneo e latch com uma primitiva; vermelho = está na cue, laranja = está mas com outro valor |
| Zonas da tela | Capture, com o painel de saída do Resolume | Tabela de aparelhos navegável como planilha; vista do universo em Canais ou Aparelhos, níveis em % ou DMX; painel que mostra sobreposição de canal |
| Atalhos | Chataigne, com a gramática do Resolume | `Ctrl+B` cue no playhead, `Shift+PageUp/Down` cue anterior/próximo; sem modificador age no selecionado, `Shift` sobe o escopo |
| Arquivo | MadMapper, corrigido | O esquema da entrada de cue é o certo; a grade achatada, o thumbnail embutido e a cena como classe separada não entram; a personality do Resolume sem tipo de canal é o exemplo a evitar |

## 1. Objetos e verbos

Vocabulário em português, da tradução oficial do Capture com as correções da coluna Spellcaster [fontes/capture.md § Vocabulário EN → PT]: **Aparelho**, **Patch**, **Universo**, **Canal (ID)**, **Unidade**, **Endereço inicial**, **Canais necessários**, **Facas**, **Zoom** (não "Zum"), **Gel**, **Instantâneo**.

**Aparelho.** Quatro números, coisas diferentes [fontes/capture.md § Patch e universos]:

| Número | O que é |
|---|---|
| Patch | universo + canal inicial (endereço DMX) |
| Canal (ID) | número do aparelho na mesa (identidade, não endereço) |
| Unidade | número físico na vara ou no palco |
| Circuito | circuito elétrico |

Mais `perfil` (personality) + `modo`, e overrides **por instância**: inverter pan/tilt/zoom/íris, limitar curso de pan e tilt, escala de intensidade [fontes/capture.md § Patch e universos]. "É assim que se corrige um aparelho pendurado de cabeça para baixo." Verbos: patch, unpatch, duplicar, substituir, numerar em lote (Sequential para Unidade, Circuito, Patch e Canal), agrupar.

**Perfil.** Lista de canais **com tipo**: dimmer, pan, tilt, cor, strobe, gobo, macro; 16 bits (coarse/fine); faixas com rótulo ("0–7 fechado, 8–134 strobe"). O Resolume não tem nada disso, só `ParamRange 0..255` com nome livre, e o resultado nos arquivos do usuário é `New Parameter 1..34` [fontes/resolume.md § O que NÃO copiar]. O tipo decide o fade (dimmer interpola, gobo salta), a resolução, e o que o Aprendiz confere. O Resolume também mostra que um aparelho precisa aceitar **dois caminhos**: valor por canal (cena) e amostragem de superfície (pixel map, o `DmxSlice/InputRect`) [fontes/resolume.md § O que NÃO copiar]; o bloco de pixels da personality (`largura, altura, formato de cor, distribuição, gama`) entra como tipo de canal `pixels`.

**Universo.** `nome`, `base do patch` (posição numérica independente do nome), `estilo: indexado | contínuo` (contínuo = faixa única 1–2048 no teatro) [fontes/capture.md § Patch e universos]; por saída: protocolo, `taxa ≤ 44 Hz` (TouchDesigner) [fontes/touchdesigner.md § Laser e NDI], `delay 0–150 ms` (Resolume) [fontes/resolume.md § Fixtures e DMX], `prioridade` sACN, ArtSync com timeout. Verbos: adicionar (um ou N com nome-base), zerar níveis, reiniciar universo externo. **Política de merge declarada** por tipo de canal (HTP para dimmer, LTP para o resto), porque "a última ganha por acidente de implementação" é o defeito do MadMapper [fontes/madmapper.md § O que NÃO copiar].

**Diálogo de patch.** Mostra `canais necessários` antes de confirmar; transbordo de universo é pergunta com "continuar até completar", não erro [fontes/capture.md § O que copiar]. `spell patch` imprime o mesmo antes de escrever.

**Cue.** O achado central do MadMapper, confirmado no arquivo [fontes/madmapper.md § Cenas e cues]:

```
cue = { uid, name, comment, fade: {type, duration},
        entries: [ { address, value, fade?: {type, duration} } ] }
```

Uma cue é uma lista de pares (endereço do registry, valor) com fade por entrada, opcionalmente diferente do fade da cue ("faz fade da opacidade mas não do RGB"). O valor pode ser número ou referência a outro objeto (`/medias/9`). Se todo widget é um nó com endereço, a cue é um diff sobre o graph e não precisa de estrutura própria. Por tipo de canal, o modo de interpolação do Chataigne: `interpolar | mudar no fim | mudar no início | nenhum` [fontes/chataigne.md § Objetos e verbos].

Verbos: gravar da situação atual ("Update from current values" em multisseleção; "CUE ALL" no aparelho) [fontes/madmapper.md § Cenas e cues]; entrar em modo de edição e clicar no widget para incluir/excluir; GO, voltar, ir para; `momentâneo` (a cue vale enquanto a tecla estiver pressionada, o `connect` do Resolume) [fontes/resolume.md § O que copiar]; `seguir` (auto-follow: o `auto_play` do MadMapper é global com override por coluna; o `ConductorCue` do Chataigne amarra sequência com `autoStart`, `autoNext`) [fontes/chataigne.md § Objetos e verbos]; `condição` (a `ChataigneCue` só está ativa se as condições baterem) [fontes/chataigne.md § Sequências]; `ação ao chegar: nada | pausar | saltar para` (o `TimeCue`).

**Cena.** Cue com flag `exclusiva`: grava tudo e, ao disparar, o que não está nela vai a zero. Não uma classe separada com regras próprias na primeira linha da grade, como no MadMapper [fontes/madmapper.md § O que NÃO copiar]. E não a "Scene" do Capture, que guarda posição e visibilidade de objeto de cenário; isso é "posição de cenário" e mora em `cenario-interativo.md` [fontes/capture.md § O que NÃO copiar].

**Lista de cues.** Conductor: cue atual, próximo, loop, gatilhos anterior/atual [fontes/chataigne.md § Objetos e verbos]. Uma lista por show ou várias; a grade 16×8 do MadMapper é uma Face sobre a mesma lista, não outro objeto.

**Aplicar a N.** Multiplex: uma cue ou rota instanciada N vezes com índice [fontes/chataigne.md § O que copiar]. `spell cue set --fixtures 1-24 dimmer 80`.

**Ensaio sem timeline.** Parrot: grava o que o operador mexer num conjunto de parâmetros e toca de volta (`IDLE | RECORDING | PLAYING`, loop, trims) [fontes/chataigne.md § Estados visuais]. Morpher: cenas como pontos num plano, peso por Voronoi, cursor X/Y mistura [fontes/chataigne.md § O que copiar]; é a maquete do TEATRO DE PAPEL misturando bastidores.

**Parâmetro com três valores ao mesmo tempo.** Constante, valor da cue, valor do cabo/timeline, guardados juntos, com "tem conteúdo" no modo inativo [fontes/touchdesigner.md § O que copiar]. "Tirar um aparelho do controle do timeline para testar um valor fixo às 23h e devolver depois sem reprogramar." O Resolume faz a mesma coisa trocando o filho do parâmetro (`PhaseSourceStatic | Timeline | TransportTimeline | DashboardLink`) sem mudar tipo nem endereço [fontes/resolume.md § Tipos de parâmetro].

## 2. Estados

**Universo / saída**: `sem sinal`, `procurando`, `conectado` (os `(no)`, `Searching..`, `(auto)` do Capture) [fontes/capture.md § O que copiar]; `bloqueado por firewall` como causa nomeada; **não armado / armado** (nenhum dos seis apps tem; Resolume envia assim que o Lumiverse existe) [fontes/resolume.md § Estados]; **ensaio** (o motor calcula, não envia). Dois níveis de habilitação como no Resolume: universo inteiro e aparelho [fontes/resolume.md § Estados].

**Cue**: enum, não booleano. O `Clip.connected` do Resolume tem cinco estados e "cada estado tem cor própria de LED" [fontes/resolume.md § Estados]. Aqui: `vazia`, `carregada`, `próxima` (standby), `ao vivo`, `em fade` (barra de progresso na própria célula, refletindo a transição mais longa) [fontes/madmapper.md § Estados], `erro`. Transição nova no mesmo parâmetro descarta a anterior.

**Entrada em edição** (modo editar cue): contorno vermelho = está na cue; laranja = está, mas com valor diferente do atual, ou ausente em parte da multisseleção [fontes/madmapper.md § Cenas e cues]. Zero modal.

**Campo do aparelho**, tintas do bloco State do Blender [fontes/blender.md § Estados]: `sobrescrito por cue` (o `inner_overridden`), `com keyframe neste tempo` (`inner_key`), `animado, sem key aqui` (`inner_anim`), `dirigido por cabo` (`inner_driven`), `diferente da cena` (`inner_changed`), cada um com par selecionado. Três níveis de não-agir: aparelho com cue mutada = apagado e editável (`active`); saída não armada em ensaio = travado (`enabled`); universo que não responde = vermelho (`alert`) [fontes/blender.md § O que copiar]. Valores divergentes em multisseleção: `Valores mistos`, string própria [fontes/capture.md § Estados e mensagens].

**Pendente**: cue editada e não gravada, vermelho [fontes/touchdesigner.md § Estados].

**Preview/blind** é modo do motor (ensaio × ao vivo), não caixinha por universo como o `BlindLevelsMode` do Capture [fontes/capture.md § O que NÃO copiar].

## 3. Zonas da tela

- **Patch (`Shift+1`)**: tabela de aparelhos como planilha, "navegada e editada com setas e Enter", ordenável por cabeçalho, com busca [fontes/capture.md § Atalhos]. Colunas fixas: Nome, Perfil, Modo, Universo, Endereço, Canais, Canal (ID), Unidade, Circuito, Grupo. Não sete colunas configuráveis por preferência [fontes/blender.md § O que NÃO copiar]. Abaixo da tabela, a vista do universo: `Modo: Canais | Aparelhos`, `Níveis: % | DMX` [fontes/capture.md § Patch e universos], mostrando ocupação e **sobreposição** de canal, "o único jeito de o operador ver que patcheou duas fixtures em cima da outra antes do show" [fontes/resolume.md § O que copiar]. O usuário do Resolume escreveu "40 - 68" no nome do slice porque a UI não mostrava a faixa [fontes/resolume.md § Fixtures e DMX].
- **Outputs (`Shift+4`)**: universos com estado, VU, taxa, delay, prioridade, armar por universo.
- **Timeline (`Shift+2`)**: cues como marcadores com ação na régua (o `TimeCue`) e, no cabeçalho de tracks, a lista de cues com atual e próximo destacados (Conductor). Um painel só para grid de clips e tira de layers, como o `LayersAndClips` do Resolume [fontes/resolume.md § Anatomia da tela].
- **Inspector (`Shift+7`)**: parâmetros do aparelho selecionado por grupo de tipo (Intensidade/, Posição/, Cor/, Feixe/); em modo editar cue, o overlay vermelho/laranja por cima; botão "gravar da situação atual".
- **Viewer central**: VU dos universos ou a maquete (`cenario-interativo.md`).
- **Status bar**: armado/ensaio, cue atual → próxima, master, blackout.

Menus: `View, Select, Add, Aparelho` no Patch; `View, Select, Add, Cue` na Timeline. `Add` com busca ao digitar.

Face performance: GO grande (o Pause do Blender com o dobro da largura) [fontes/blender.md § Anatomia de um editor], lista de cues, master, blackout, armado. Ou a grade de células em modo Live do MadMapper, "aperta a célula, sem edição, feito para touch screen" [fontes/madmapper.md § Cenas e cues]. Nada editável.

## 4. Atalhos

`SHORTCUTS.md` já tem `Enter` GO, `Backspace` voltar, `Ctrl+G` ir para cue, `Ctrl+Shift+Enter` armar, `Ctrl+Shift+R` ensaio, `Esc` segurado blackout, `Ctrl+K` keyframe, `R` record arm, `Shift+D`/`Shift+S` mute/solo. O que entra:

| Ação | Tecla | Origem | Conflito |
|---|---|---|---|
| Cue na posição do playhead | `Ctrl+B` | Chataigne `TimelineAppCommands.cpp:47` [fontes/chataigne.md § Atalhos] | nenhum |
| Cue anterior / próxima (seleção, sem disparar) | `Shift+PageUp` / `Shift+PageDown` | Chataigne | nenhum; `Ctrl+Shift+←/→` continua sendo marcador |
| Passo de tempo | `PageUp` / `PageDown` | Chataigne | nenhum |
| Modo editar cue | `Shift+E` | (nosso; MadMapper usa `Cmd+Shift+C`, que colide com copiar comando CLI) | nenhum |
| Mudar valor sem gravar na cue, em modo edição | segurar `Shift` ao mexer | MadMapper [fontes/madmapper.md § Cenas e cues] | gesto |
| Remover parâmetro da cue selecionada | `Backspace` sobre o campo, em modo edição | MadMapper | `Backspace` fora de campo = cue anterior; keymap de UI só vale sobre campo |
| Keyframe no campo sob o mouse / apagar / limpar animação | `I` / `Alt+I` / `Shift+Alt+I` | Blender keymap "User Interface" [fontes/blender.md § Atalhos] | `I` = In fora de campo |
| Voltar campo ao valor da cena | `Backspace` sobre o campo, fora do modo edição | Blender | idem |
| Copiar comando CLI do campo | `Shift+Ctrl+C` | Blender | nenhum |
| Numerar em lote | comando de menu `Sequential`, sem tecla | Capture | |
| Play/pause na Face performance | `Shift+Space` | TD | regra 8 |
| Cue momentânea | segurar a tecla da cue | Resolume `connect`, Piano Mode | gesto |
| Duplicar cue para outra posição | `Alt` + arrastar | MadMapper | gesto |

Gramática do Resolume: sem modificador age no selecionado (`B` bypass do layer selecionado), `Shift` sobe um nível (`Shift+B` bypass da composição) [fontes/resolume.md § Atalhos e mapeamento]. Aqui: `Shift+D` muta o track focado; `Ctrl+Shift+D` muta o universo do track (proposta, nosso). Mapeamento de controlador por posição na grade **ou** por identidade da cue (`by_cell` vs `by_name`); a segunda "sobrevive a reorganizar o show às 23h" [fontes/madmapper.md § O que copiar]; o mapeamento declara também o escopo do alvo (`Selecionado | Este | Por posição`) [fontes/resolume.md § O que copiar].

## 5. Arquivo

No `.spell`:

- `profiles[]`: perfil com `{id, version, name, modes: {name: channels[]}}`, canal = `{name, type, bits, default, ranges[{from, to, label}]}`. Biblioteca do usuário em pasta de texto, referenciada por `id + version`. A cópia anônima do Resolume (`fixtureName=""`, uuid novo, "corrigir a personality não conserta o show") é o que não fazer [fontes/resolume.md § O que NÃO copiar].
- `fixtures[]`: `{uid, name, profile, mode, universe, address, id, unit, circuit, group, overrides: {invert_pan, limit_pan, intensity_scale, …}}`.
- `universes[]`: `{uid, name, base, style, outputs: [{protocol, rate, delay_ms, priority, artsync}], merge: {dimmer: htp, default: ltp}}`.
- `cues[]`: o esquema do §1, com `uid` próprio e posição como atributo (`list`, `index`; ou `bank`, `col`, `row` para a Face grade). Não a grade achatada `cues[128]` com índice implícito [fontes/madmapper.md § O que NÃO copiar]. Sem thumbnail (uma cue do exemplo do MadMapper carrega 17 658 bytes de PNG). `exclusiva`, `momentânea`, `follow`, `conditions`, `action`.
- `cuelists[]`: `{uid, name, cues[uid], loop, current}`.
- `mappings[]` com `target: {by: uid | cell, scope: selected | this | position}`.

Fora do `.spell`: níveis vivos, estado de conexão, o programador do operador (o Parrot grava em arquivo próprio quando pedido).

Salvar: cópia de segurança automática e oferta dela na abertura corrompida (`FileErrorUseBackup`, `ChecksumError`) [fontes/capture.md § Estados e mensagens]. Importar aparelhos de CSV com mapeamento de colunas e relatório `atualizados / adicionados / linhas puladas` [fontes/capture.md § Arquivo] vira `spell patch import`.
