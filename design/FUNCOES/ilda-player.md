# ILDA player — `spell ilda play` (tema LASER)

Tocar um `.ild` num DAC (EtherDream hoje, `spellcaster/protocols/ilda/etherdream.py`) com transporte, loop, calibração e segurança. O módulo Python já tem `Point(x,y,r,g,b,blank)`, `optimize(dwell, blank_gap, max_step, angle)` e `safety(min_size, max_intensity, zone)` em `frame.py`; este arquivo diz como isso vira interface.

| Item | Quem resolveu melhor | Por quê |
|---|---|---|
| Objetos e verbos | MadMapper, com o player-como-aparelho do Capture | O output laser e a superfície laser do MadMapper têm o conjunto completo e nomeado de parâmetros físicos; o Capture mostra que o player é um aparelho patcheado, com taxa de quadros própria e dois canais DMX |
| Estados | TouchDesigner | `debugchan` dá o estado de cada ponto emitido; `PlayBarPending` dá o estado "mudou mas não aplicou"; e a omissão dele (nenhum estado de segurança) mostra o que não repetir |
| Zonas da tela | Blender, com o "Arm your laser" do MadMapper | Transporte de seis botões num só `row`, com Pause maior quando toca; o armar fica no canto superior direito do preview, não em menu |
| Atalhos | Blender, com a tabela de painel do TouchDesigner | Keymap "Frames" válido em toda janela; `I`/`Backspace` sobre o campo; `Shift+Space` como único play em performance |
| Arquivo | MadMapper | Declara que o ILDA não carrega PPS, separa "gravar a FPS fixo" de "gravar como stream", e guarda tudo do laser em pares nome-valor legíveis |

## 1. Objetos e verbos

**Clipe** — um `.ild` (formatos 0, 1, 2, 4, 5 já em `ild.py`). Verbos: adicionar, remover, renomear, duplicar. Um clipe não sabe seu PPS: o formato não carrega taxa de playback, "o hardware trata o arquivo como fluxo de pontos: um frame de 500 pontos dura menos que um de 700" [fontes/madmapper.md § Laser]. Logo taxa de quadros e PPS são do **player e da saída**, nunca do arquivo. O Capture confirma pelo outro lado: `ILDAFrameRate` é propriedade do `MediaPlayer`, não da mídia [fontes/capture.md § Patch e universos].

**Frame** e **Ponto** — o que o clipe contém. Verbos sobre o frame: ir para, marcar In/Out, loop. Nenhum verbo sobre ponto isolado na GUI; ponto é dado de diagnóstico (ver Estados).

**Player** — o transporte. Verbos: play, pause, stop, locate, loop no intervalo In/Out, velocidade, taxa de quadros pedida. É um **aparelho patcheado**: responde a dois canais DMX (controle play/pause/stop/replay e seleção de mídia) e a uma playlist de 256 entradas [fontes/capture.md § Patch e universos]. Isso é o que faz `spell cue` disparar um clipe laser como dispara um dimmer.

**Saída laser** — o DAC. Parâmetros, com os nomes que o MadMapper grava em `customSettings` e o TouchDesigner expõe no Laser CHOP, agrupados por `/` no rótulo (regra transversal 1):

| Grupo/Parâmetro | Unidade | Origem | Nota |
|---|---|---|---|
| Dispositivo/Tipo | enum etherdream, helios, shownet | TD `type` | menu auto-populado com o que está na rede [fontes/touchdesigner.md § Laser e NDI] |
| Dispositivo/Endereço | IP | TD `netaddress` | descoberta é problema conhecido; lista + campo |
| Dispositivo/Fila | amostras, frames ou segundos | TD `queuetime` + `queueunits` | "costuma ser útil reduzir ao enviar poucos pontos" |
| Dispositivo/PPS | pontos/s | MM `Device/PPS`, TD `outputrate` | clamp físico do galvo, faixa útil do show separada (`min/max` vs `norm`, regra 1) |
| ILDA/FPS pedido | Hz | MM `ILDA/Desired FPS` | o que o operador quer |
| ILDA/FPS real | Hz, somente leitura | MM `ILDA/ILDA FPS` | `= PPS / pontos por frame`; abaixo de 35 pisca, acima de 45 não se vê a varredura |
| ILDA/Pontos por frame | contagem, somente leitura | MM `ILDA/Point Count` | a terceira variável da fórmula, sempre ao lado das outras duas |
| ILDA/Atualização | enum tudo-desenhado, todo-frame | TD `updatemethod` | o parâmetro que explica o flicker por excesso de pontos |
| Blanking/Passo com cor, Passo apagado | distância por amostra | TD `stepsize`, `bstepsize` | nosso `max_step` |
| Blanking/Hold de canto mín, máx | amostras | TD `mincornerhold`, `maxcornerhold` | interpolado pelo ângulo; nosso `dwell` + `angle` |
| Blanking/Pré-liga, Pós-liga, Pré-desliga, Pós-desliga | ms | TD `preblankoff`, `postblankoff`, `preblankon`, `postblankon` | nosso `blank_gap`; MM condensa em `Blank Delay/Smth/Curve` |
| Blanking/Repetir início, fim | pontos | MM `Start Repeat`, `End Repeat` | "o software nunca sabe onde o feixe realmente está" |
| Blanking/Fade entrada, saída | 0..1 | MM `In Fade`, `Out Fade` | evita o hot point do começo e do fim do path |
| Cor/Escala R G B | 0..1 | TD `redscale`…, MM `Color Levels` | |
| Cor/Tensão mínima R G B | 0..1 | MM `Min Voltage` | "para que um cinza escuro não vire vermelho" |
| Cor/Deslocamento R G B | **pontos ILDA** | MM `Time Shift` | unidade certa: o atraso é de amostras, não de parede; TD usa ms (`colordelay`) e perde |
| Cor/Curva R G B | curva | MM `Response` | |
| Segurança/Área de varredura | 0..1 | MM `ILDA/Scan Area` | nosso `zone`; "ouvir o scanner: o som deve ser liso" |
| Segurança/Máscaras | polígonos com opacidade e inversão | MM `Masks` | vale também para o cursor de teste |
| Segurança/Tamanho mínimo, Intensidade máxima | | nosso `safety()` | TD não tem nenhum dos dois; herdar a omissão é proibido |
| Geometria/Escala X Y, Rotação, Flip, Trocar XY | | TD `xscale yscale rotate swap`, MM `scale rotation flip` | |
| Teste/Padrão de teste, Nível | | MM `Test Pattern` | |

Cada linha dessas é um `@command` do registry com tipo declarado (regra 3): PPS é valor, Padrão de teste é toggle, Armar é trigger com confirmação.

**Calibração** — os `calib_*` já existentes; o MadMapper faz isso fotografando o que o laser desenha com câmera e deformando para o ponto de vista do laser [fontes/madmapper.md § Objetos e verbos]. Fica como verbo do painel Outputs, não do player.

**Shutter** — verbo próprio, com estado próprio. O TouchDesigner faz blanking implícito quando os três scales de cor são zero, "um estado que não aparece em lugar nenhum da UI" [fontes/touchdesigner.md § O que NÃO copiar]. Aqui shutter é nomeado. Segurar a tecla é shutter momentâneo, pelo mesmo mecanismo do `connect` booleano do Resolume: "análogo a estar com o mouse pressionado" [fontes/resolume.md § O que copiar].

## 2. Estados

Um accent, misturado no fundo do campo (regra 4). Os estados que o player precisa mostrar, em ordem de gravidade:

1. **Não armado / armado.** Sem equivalente em Resolume ("assim que um Lumiverse existe, ele envia") nem TouchDesigner. MadMapper tem "Arm your laser!" no canto superior direito dos previews [fontes/madmapper.md § Estados]. Aqui: `Ctrl+Shift+Enter` arma, e o estado é o único que troca a cor do frame inteiro do preview.
2. **Shutter fechado / aberto.** Nomeado, indicador próprio, nunca inferido de cor zero.
3. **DAC: sem dispositivo, procurando, conectado, erro.** Os três estados de universo do Capture, `(no)`, `Searching..`, `(auto)`, mais erro [fontes/capture.md § Estados e mensagens]; "atividade não garante funcionamento" vale aqui também.
4. **Transporte: parado, tocando, pausado, em loop, e pendente.** Os quatro do TouchDesigner (`PlayBarOff`, `On`, `Reset`, `Pending` vermelho) [fontes/touchdesigner.md § Estados]. Pendente = mudou PPS ou fila e o DAC ainda não aplicou.
5. **Galvo não acompanha.** FPS real abaixo de 35 Hz, ou pontos por frame acima de `PPS / FPS pedido`. É `warning`, não `error`: o laser continua, pisca. O Aprendiz fala por aqui (`aprendiz-menu.md`).
6. **Fora da área segura.** Pontos cortados por `zone` ou máscara: contagem por frame, `warning` se maior que zero.

Diagnóstico por ponto: o engine emite, por amostra de saída, o mesmo enum do `debugchan` do Laser CHOP: `-1` início de frame, `0` cor, `1` hold de canto, `2` hold do primeiro ponto, `3`/`4` pré e pós blank-on, `5` blanking, `6`/`7` pré e pós blank-off [fontes/touchdesigner.md § Laser e NDI]. O preview pinta o ponto pelo estado quando o toggle "diagnóstico" está ligado; `spell ilda monitor` imprime o mesmo. É o que transforma "o traço está com rabo" em número.

Congelar: o MadMapper separa congelar o engine de congelar cada saída (`laserOutputsFrozen`) e avisa que congelar a saída não para o playback [fontes/madmapper.md § Estados]. Aqui: pausar o transporte é uma coisa; fechar o shutter é outra; as duas aparecem.

## 3. Zonas da tela

Conforme `SHORTCUTS.md § Interface` (Media Pool à esquerda, viewer central, Inspector à direita, timeline embaixo), preenchido assim:

- **Esquerda, Media Pool**: lista de clipes `.ild`, com frames e duração por clipe. Lista maior que 8 itens vira busca ao digitar (regra 10).
- **Centro, viewer**: o frame atual. Preview e saída são **o mesmo widget parametrizado** (Resolume: `Monitor subject_type=Composition|Preview`) [fontes/resolume.md § Anatomia da tela]. Modos do viewer: frame do clipe, frame otimizado, diagnóstico por ponto. Máscaras e área de varredura desenhadas por cima. No canto superior direito: **Armar** e o indicador de shutter [fontes/madmapper.md § Estados].
- **Direita, Inspector**: os grupos da tabela acima, dobráveis; o grupo `ILDA/` sempre aberto porque mostra a fórmula. Toggle com seta para blanking e cor: o botão liga, a seta abre o popover de ajuste, "nunca dois botões separados" [fontes/blender.md § Anatomia de um editor].
- **Embaixo, Timeline**: régua de frames do clipe, In/Out, marcadores. Header de transporte = um único `row` com rewind, frame anterior, play reverso, play, frame seguinte, fast-forward; tocando, os dois plays viram um Pause com o dobro da largura [fontes/blender.md § Anatomia de um editor].
- **Rodapé, status bar**: o que os botões do mouse fazem agora, último aviso, tarefa rodando, estatísticas (PPS, FPS real, pontos) [fontes/blender.md § Estados].

Menu do painel: `View, Select, Add, Frame` (regra 6). Nada flutuante: a Palette do TouchDesigner que se fecha "para ganhar espaço" está no não-copiar [fontes/touchdesigner.md § O que NÃO copiar].

Face performance: viewer grande, transporte, Armar, shutter, PPS e FPS real. Sem Inspector.

## 4. Atalhos

Tudo de `SHORTCUTS.md` vale (Space, J/K/L, setas, Home/End, I/O, `Ctrl+L` loop, `M` marcador, `Ctrl+Shift+Enter` armar, `Ctrl+Shift+R` ensaio, `Esc` segurado blackout). O que entra:

| Ação | Tecla | Origem | Conflito |
|---|---|---|---|
| Play/pause na Face performance | `Shift+Space`; `Space` sozinho desabilitado | TD `PanelShortcuts.txt` [fontes/touchdesigner.md § Atalhos] | nenhum; é a coluna nova de `SHORTCUTS.md` (regra 8) |
| Shutter momentâneo | segurar `Esc` fecha; `Esc` toque fecha painel | `SHORTCUTS.md` blackout, Resolume `connect` pressionado | nenhum |
| Keyframe no campo sob o mouse | `I` | Blender keymap "User Interface" [fontes/blender.md § Atalhos] | `I` = In quando o mouse não está sobre campo; o keymap de UI só vale sobre campo, como no Blender |
| Voltar o campo ao valor de cena | `Backspace` sobre o campo | Blender | `Backspace` = cue anterior fora de campo; desligado na Face performance |
| Copiar o comando CLI do campo | `Shift+Ctrl+C` | Blender `copy_data_path` | nenhum |
| Padrão de teste | `Shift+T` | (nosso; MadMapper tem o botão sem tecla) | nenhum |
| Escolher ordem de grandeza e arrastar | segurar botão do meio sobre o número, mover vertical, arrastar horizontal | TD Value Ladder [fontes/touchdesigner.md § Parâmetros] | gesto, não tecla; `Alt+botão direito` sem botão do meio |
| Ladder no rótulo move X e Y juntos; no campo, um só | idem | TD | |

Gramática: sem modificador age no selecionado, `Shift` sobe um nível (Resolume: layer → composição) [fontes/resolume.md § Atalhos e mapeamento]; é o mesmo "Shift estende" de `SHORTCUTS.md`. Não entra: `Space` configurável, numpad, `X` apagar (regra 11).

## 5. Arquivo

No `.spell`:

- `clips[]`: `{uid, name, path}` com `path` relativo ao show e `relpath` declarado; cópia de segurança do `.ild` dentro do pacote do show quando exportado (TD `savebackup`) [fontes/touchdesigner.md § Arquivo]. Nenhum PPS no clipe.
- `player`: `{clip, fps, loop, in, out, speed, dmx: {universe, channel}}`. Os dois canais DMX do Capture.
- `outputs[]` (compartilhado com `patch`): todos os parâmetros da tabela do §1, **completos**, não só o delta (regra 12), com enum por identificador estável, não pelo rótulo (`"ILDA/Mode": "Preserve Image Quality"` do MadMapper é o erro) [fontes/madmapper.md § O que NÃO copiar]. Máscaras como polígonos normalizados com opacidade e inversão.
- `calibration` por saída, do `calib_*`.

Fora do `.spell`: IP descoberto (vai para `config.json` do perfil), zoom do viewer, lista de dispositivos vista na rede, thumbnails de frame.

Exportar: dois modos, os do "Movie Mode" do MadMapper, porque o ILDA não carrega PPS: **a FPS fixo** (para tocar de volta no Spellcaster) e **como stream** (para cartão SD do laser ou outro DAC) [fontes/madmapper.md § Laser]. O modo vira flag de `spell ilda export`.

Versão: abrir e tocar diferente é pior que recusar. O TouchDesigner 2025.30000 passou a gerar a 192 000 e reamostrar, removeu dois parâmetros, e avisou só no wiki [fontes/touchdesigner.md § O que NÃO copiar]. `"version"` no topo, migração local, falha alta.
