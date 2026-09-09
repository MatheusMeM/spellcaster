# MadMapper 5.6.8 — auditoria de interface (fonte primária)

Levantamento feito lendo os arquivos instalados em `C:\Program Files\MadMapper 5.6.8\` e `C:\Users\email\AppData\Roaming\GarageCube\MadMapper\`. Nada aqui vem de memória: cada afirmação cita o arquivo, a página do PDF ou a chave do projeto. O aplicativo não foi aberto.

## Fontes lidas

PDFs em `C:\Program Files\MadMapper 5.6.8\Resources\Guides\` (texto extraído com PyMuPDF):

- `Introduction to the User Interface.pdf` (29 p.)
- `Scenes and Cues.pdf` (10 p.)
- `Cue Scheduler.pdf` (5 p.)
- `MadLaser Guide.pdf` (20 p.)
- `Laser Scanner Guide.pdf` (7 p.)
- `Laser Materials Documentation.pdf` (5 p.)
- `Materials Documentation.pdf` (15 p.)
- `OSC Channels.pdf` (6 p.)
- `Modules.pdf` (10 p.)
- `Preferences.pdf` (12 p.)
- `Control Surface Module Guide.pdf` (3 p.)
- `Stream Deck Plugin Guide.pdf` (4 p.)
- `Masking.pdf` (9 p.)
- `My First Video Mapping.pdf` (15 p.)
- `MadLaser AVB Support.pdf` (6 p.)
- `Realtime Reactive Visuals.pdf` (7 p.)
- `Knowledge Base.pdf` e `The MadMapper FAQ.pdf` (35 e 26 p., quase só licenciamento; só a seção "Controls" do Knowledge Base p. 4 rendeu)

Arquivos binários e de dados:

- `Resources\SampleProjects\*.mad` (6 projetos). Formato descoberto e decodificado por script próprio (`madparse.py`, no scratchpad da sessão); **os 6 arquivos foram lidos 100%** (byte final = byte do arquivo), então o esquema abaixo é completo, não amostrado.
- `Resources\Materials\All Widgets Template\All Widgets Template.fs`, `Resources\LaserMaterials\Beam Patterns 1\Beam Patterns 1.fs`, `Resources\FactorySurfaceLaserLineFX\Beams\Beams.fs`, e os `.info` das mesmas pastas.
- `Resources\app.css` (folha QSS de Qt Widgets).
- `Resources\Translations\mm_fr.qm` (strings UTF-16 extraídas: menus e ações).
- `C:\Users\email\AppData\Roaming\GarageCube\MadMapper\LEDFixtureLib.mfl` (XML).
- Listagem de `Resources\FactoryModules\`, `SystemModules\`, `FactoryGenerators\`, `FactoryLaserGenerators\`, `Macro\` e de `C:\Users\email\Documents\MadMapper\`.

Não lido por opacidade: os `main.ldat` de módulos e geradores de fábrica (`Resources\FactoryModules\Cue Scheduler\main.ldat` etc.) são binários cifrados/comprimidos — `od` mostra entropia alta desde o primeiro byte, sem cabeçalho legível e sem strings extraíveis. Logo, **o formato interno de um módulo do MadMapper não pôde ser lido**; o que se sabe de módulo vem do PDF e da serialização do projeto.

## Anatomia da tela

Quatro zonas fixas (`Introduction to the User Interface.pdf` p. 2):

1. **Media Panel** — todo o lado **direito**. Dividido em Media Bin (grade de thumbnails ou lista) e Media Inspector embaixo, com informações e parâmetros da mídia selecionada (p. 3). 2. **Views** — o centro, partido em duas: **Input View** (esquerda, escolhe que parte da mídia é mapeada — as UVs) e **Output Preview** (direita, posiciona a superfície e corrige a perspectiva) (p. 9–10). 3. **Project Tabs** — canto superior esquerdo, cinco abas: Surfaces, Light Fixtures, Projectors (Outputs), Modules, Master Settings (p. 13). Cada aba é lista em cima + Inspector embaixo. 4. **Tool Panels** — painel inferior, escolhido por dropdown: Scenes and Cues, Control List, Library, MadLight Recorder, DMX Monitor, Fixtures Editor, Code Editor (p. 25–28).

Sempre visível: a barra de status de tooltip no rodapé, que mostra informação contextual do elemento sob o mouse (p. 2). O fluxo declarado é bidirecional, "da esquerda (surfaces, fixtures, modules) para a direita (media) e vice-versa" (p. 2).

O painel de ferramentas é **a única parte redimensionável** por splitter, e é o único que pode ser desacoplado em janela própria, com botão "manter no topo das outras janelas" (`Introduction to the User Interface.pdf` p. 25).

Barra de ferramentas acima das views (p. 11–12): lado a lado / só input / só output, alternar orientação vertical-horizontal, posição X/Y do handle selecionado, grupos de seleção nomeados na hora, undo/redo, travar transformações afins, snapping, "ver só o selecionado", zoom in/out, ajustar à janela, zoom no selecionado, e (só no output) os preview groups "Master" e "Selection".

A UI é **Qt Widgets clássico**, não QML: `Resources\app.css` é um QSS que estiliza `QMenuBar`, `QMenu`, `QSlider`, `QSpinBox`, `QTableView`, `QSplitter`, `QStatusBar` e classes próprias (`CueCell`, `CueGridViewWidget`, `CueEditorToolBar`, `CueEditorInspector`, `CueBankComboBox`, `QMappableButtonGrid`, `MediaGridWidget`, `QAttributeWidget`, `HandlePosWidget`, `DMXUniverseDialog`, `LaserScannerDialog`, `LEDScannerDialog`). O tema é uma lista curta de variáveis `$`: `$toolbar_background`, `$groupbox_background`, `$list_background`, `$list_item_background`, `$text_color`, `$disabled_text_color`, `$selection_blue`, `$active_selection_blue`, `$disabled_selection_blue`, `$cue_color`, `$preview_group_green`, `$slider_spinbox_etc_background`. Ou seja: **dois accents (azul de seleção, verde de preview group) e um token separado só para a cor de cue**.

## Objetos e verbos

**Superfícies** (`Introduction…` p. 14–17): Quad, Line, Triangle, Circle, Mask, 3D Surface (.OBJ), Group. Verbos por item da lista: renomear (clique no nome), mostrar/esconder, travar (lock), mover na lista — e a ordem da lista é ordem de camada, o primeiro fica na frente. O Inspector traz blend mode, opacidade, posição, perspectiva, cores, FX, soft-edge/feathering, mesh warping e bezier mesh warping. Selecionando várias com Shift, o Inspector mostra só os parâmetros comuns.

**Fixtures / light mapping** (p. 18–20): DMX Fixture, DMX Line, DMX Circle, Group. Em MadLight só se trabalha no input view. Fixture **não tem opacidade** — a de cima sobrescreve a de baixo, não mistura. Inspector: definição de fixture (da biblioteca, com botão Edit para o Fixture Editor), canal e DIP switch DMX, filtering, **response curve** (como a luminosidade do aparelho responde ao valor DMX), luminosidade, cor, posição, tamanho, rotação, flip.

**Mídias** (p. 5–7), por categoria: Generators, Materials, ISFs, Quartz Composer, Images, Movies, Images Folders (com watchdog de pasta), Live Input, Syphon/Spout/NDI. A distinção que importa: *generator* é renderizado uma vez numa textura de tamanho fixo; *material* é um shader renderizado separadamente **em cada superfície** que o usa, com mais precisão e mais custo. Arrastar um material para a categoria Generators o converte em textura. Cada mídia mostra em quanto surfaces está aplicada, e clicar nesse número seleciona todas elas.

**Outputs** (p. 20–22): projetores, sem ordem de camada. Inspector: output size com trava de aspect ratio e botão de capturar a resolução do destino, flip, Destination, imagem de background (não sai no output) e máscara .png (sai). Ferramentas: mostrar/esconder janela de preview, ligar test pattern, publicar o conteúdo ao vivo em Syphon ou NDI.

**Modules** (`Modules.pdf` p. 2–10): instanciáveis várias vezes, renomeáveis, com botão de ativar/desativar por instância. Os de fábrica em disco (`Resources\FactoryModules\`): Audio Player, Calendar Scheduler, Control Surface, Controls Combiner, Cue Scheduler, DMX Router, Device Activity, Firmata, Idle Cue, MIDI Out, MadLight Player, MiniMad Controller, OSC Out, Oscillator, Oscillator2D, OscillatorBank, PJLink, Pollution, RandomNoise, Startup Cue, Weather. Os de sistema (`Resources\SystemModules\`): BPM Global, Media Playback.

Ponto arquitetural: no arquivo de projeto, **generator, laser generator e módulo são a mesma entidade**. `modules.modules[]` tem `moduleName`, `instanceName`, `id`, `activated`, `isSystemModule`, `isGenerator`, `isLaserGenerator` e `attributes` (`Laser Example.mad`). O Laser Mixer, por exemplo, tem `attributes` = `{"Input1": "/medias/76", "Input2": "/medias/75", "Mixer/Mode": "Morphing", "Mixer/Mix": 0.0, "Output": ""}` — um módulo referencia mídias por URL e publica a saída como mídia.

**Laser** (`MadLaser Guide.pdf` p. 3–4): saídas laser ilimitadas; protocolos Ilda via EtherDream ou Helios USB, ShowNet, e FB4 via Pangolin Beyond. `MadLaser AVB Support.pdf` p. 2–3 acrescenta AVB / Dante por placa de áudio (48 e 96 kHz expostos), e nomeia como equivalentes de DAC "Etherdream, Helios, ShowNET, Moncha". Superfícies laser: Laser Quad, Laser Line, Laser Text, Group (p. 9, 15, 17, 19).

**Ferramentas de calibração**: Laser Scanner (menu Tools) fotografa com câmera Sony/Canon/NDI o que o laser desenha e deforma a imagem para o ponto de vista do laser (`Laser Scanner Guide.pdf` p. 2, 5–6); Spatial Scanner e LED Scanner no mesmo menu (`Introduction…` p. 29).

Menus, pelas strings de `Resources\Translations\mm_fr.qm`: Fichier, Edition, Projet, Sorties, Outils, Vue, Compte, Aide — com ações "Raccourcis clavier" (uma janela de atalhos existe no app, mas nenhum PDF a transcreve), "Scanner Spatial", "Scanner LED", "Enregistreur MadLight", "Exporter/Importer les définitions de Fixtures", "Nouveau Projet avec Mur d'Ecrans", "Créer des Lignes depuis le Contour", "CUER TOUT" (o CUE ALL).

## Cenas e cues

O modelo, de `Scenes and Cues.pdf` p. 2–10, confirmado pela serialização em `SampleProjects\DMX LED Bar Example.mad` (chave `cueBanksV4`).

**Grade.** Um projeto tem N Cue Banks (`cueBanksV4.cueBanks[]`), cada um com `bank_name`, `bank_id`, `column_count` (16 por padrão), `row_count` (8), e `cues[]` achatado de 128 células — índice = `linha * column_count + coluna`. A grade cresce sozinha quando se grava algo na última linha ou coluna (p. 3). `cueBanksV4.activeCueBank` guarda o bank ativo e `cueBanksV4.liveMode` o modo Live.

**A primeira linha é reservada a Scenes** (p. 3). Diferença exata (p. 2): uma Scene guarda *tudo* (surfaces, fixtures e medias), não permite remover entradas nem guardar parte de um objeto, e ao disparar **esconde toda superfície/fixture criada depois dela**. Uma Cue guarda o que você escolher, com a granularidade que quiser.

**O que uma cue é, em dados.** Cada célula não vazia é: `{name, comment, color, pixmap/pixmapId, is_scene_cue, transition_settings, entries[]}`. E cada entrada de `entries[]` é `{url, value, ownerUrl, display_path, transition_mode, transition_settings}`. Exemplos reais do `DMX LED Bar Example.mad`:

    {"url": "/fixtures/9/color/red", "value": 1.0,
     "ownerUrl": "/fixtures/9/color",
     "display_path": "/Fixtures/LED BARS/Color/Red",
     "transition_settings": {"type": 0, "duration": 0.0, "params": {}}}

    {"url": "/fixtures/1/visual", "value": "/medias/9",
     "display_path": "/Fixtures/LED BARS/LED BAR-1/Visual"}

Isto é o achado central: **uma cue é uma lista de pares (endereço OSC, valor) com fade por entrada**. O mesmo endereço que o `OSC Channels.pdf` documenta como API externa é a chave de armazenamento interno da cue. O valor pode ser número ou uma referência a outro objeto (`/medias/9`), e `display_path` é só o rótulo legível.

**Transição.** `transition_settings` existe no nível da cue e, opcionalmente, por parâmetro: `{type, duration, params}`. O PDF (p. 5) diz o mesmo em prosa: por padrão todo parâmetro usa a transição da cue, mas se pode dar transição local a cada um ("faz fade da opacidade mas não do RGB"), inclusive "No Transition". Duplo clique no rótulo `Fade: xxx` da célula muda o tempo (p. 5).

**Disparo** (p. 8–9): botão play na célula; botão da coluna, que dispara a scene daquela coluna ou, se não houver scene, todas as cues da coluna de baixo para cima; botão "Next Column"; Auto Play; modo Live (aperta a célula, sem edição — feito para touch screen); Cue Scheduler; Calendar Scheduler; e controls (MIDI/OSC/DMX/teclado).

**Auto Play**, em dados: `auto_play_settings = {active, mode, timing:{source, timeDuration, beatsDuration, switchOnMovieLoopEnd}}`, e cada coluna tem `column_settings[i]` com `{name, disabled, hasLocalAutoPlayTimingSettings, autoPlayTiming{...}}` — ou seja, **timing global com override por coluna**. Cada linha tem `row_settings[i] = {name, disabled}`.

**Escalonamento** (`Cue Scheduler.pdf` p. 3–5): aba Schedule (ano/mês/dia/hora/minuto/segundo, cada campo aceitando "Each" para recorrência), aba Automate (Use Auto-Play Timings / a cada N segundos / a cada N beats do BPM global / trocar no fim do loop do filme), aba Cue (bank alvo, célula ou coluna específica, ou faixa de 1 a 16 com Loop, Random e Skip Empty), aba Manual (Go Next Now / Go Previous Now). O módulo confere o relógio **uma vez por segundo**. Com dois módulos em conflito, o último da lista vence (p. 5).

**Edição** (p. 6–7): em Edit Mode (`Cmd+Shift+C`) o app pinta um overlay vermelho em todo parâmetro "cueável"; contorno vermelho = está na cue; contorno laranja = está, mas com valor diferente do atual ou ausente em parte da multisseleção. Clicar num widget adiciona/remove/atualiza o parâmetro nas cues selecionadas; para *mudar* o valor sem cuear, segura Shift. Um botão "CUE ALL" aparece nos itens da lista de surfaces e nos visuais. Backspace remove o parâmetro das cues selecionadas — e isso não vale para Scenes. Clicar numa surface já selecionada no input preview cuea/descuea a "Input Geometry"; no output preview, a "Output Geometry".

**Multisseleção de cues** (p. 5) é o mecanismo de edição em massa: selecionar 10 cues, escolher a entrada `Surfaces / Quad 1 / Output Geometry` e apertar "Update from current values".

## Estados

- **Preview vs output**: os previews podem ser escondidos por completo ("Show/hide the UVs view and preview") para ganhar performance, removendo dois renders de tela (`Introduction…` p. 13).
- **Freeze**: três estados distintos e independentes no Master Settings (p. 24) e no arquivo: `engineFrozen` (congela o engine, Master Speed vai a 0, exceto se houver Ableton Link/BPM), `videoOutputsFrozen`, `dmxOutputsFrozen`, `laserOutputsFrozen`. Congelar a saída **não** para o playback. Os quatro masters são separados: `masterLevel`, `masterVideoLevel`, `masterDmxLevel`, `masterLaserLevel`, `masterAudioLevel` — e "se o Master está em zero, tudo está em zero".
- **Por superfície**: `visible`, `locked` (`Introduction…` p. 14 e chaves homônimas no `.mad`). Trava global de transformações afins com `Alt+B` (`Preferences.pdf` p. 12).
- **Laser armado**: existe um comando explícito "Arm your laser! (top right of the previews)" (`Laser Scanner Guide.pdf` p. 5). É um estado de topo da janela, ao lado das previews, não um menu.
- **Cue em transição**: a barra de progresso aparece na própria célula que gerou a transição; se a cue tem transições diferentes por parâmetro, o progresso reflete a mais longa; com transição do tipo "damper" o progresso não significa nada além de "há transição rodando" (`Scenes and Cues.pdf` p. 10). Se outra cue já estava fazendo transição naquele parâmetro, a anterior é descartada.
- **Preview groups**: `previewSurfaceGroups` no `.mad` guarda `[{name:"Master", includeMaster:true, surfaces:[]}, {name:"Selection", ..., surfaces:["/laser_surfaces/1"]}]` — o preview filtra quais superfícies aparecem, e isso é estado salvo, não transitório.
- **Modo autônomo**: preferência "nunca pedir confirmação" mais "iniciar em fullscreen ao abrir com um arquivo de projeto" mais "minimizar no startup" (`Preferences.pdf` p. 11) — o pacote de instalação permanente.

## Atalhos e controle (teclado, MIDI, OSC)

Os PDFs **não** trazem tabela de atalhos (o app tem uma janela "Raccourcis clavier" no menu Ajuda, segundo `mm_fr.qm`, mas ela não está em disco como texto). Os atalhos efetivamente documentados:

| Ação | Tecla | Fonte |
|---|---|---|
| Nova máscara / novo laser path | `Alt+A` | `Masking.pdf` p. 9; `MadLaser Guide.pdf` p. 16 |
| Fechar o path desenhado | `Enter` | `Masking.pdf` p. 9; `Introduction…` p. 15 |
| Adicionar ponto ao path | `Alt+Click` | `Masking.pdf` p. 9 |
| Remover ponto | `Delete` | `Masking.pdf` p. 9 |
| Habilitar bezier num ponto / ligar-desligar tangentes | botão direito | `Masking.pdf` p. 6, 9 |
| Renomear item da lista | duplo clique | `Masking.pdf` p. 9 |
| Copiar / colar / duplicar | `Ctrl+C` / `Ctrl+V` / `Ctrl+D` | `Masking.pdf` p. 9; `MadLaser Guide.pdf` p. 17 |
| Duplicar cue para outra célula | `Alt` + arrastar | `Scenes and Cues.pdf` p. 6 |
| Entrar em Edit Cues | `Cmd+Shift+C` | `Scenes and Cues.pdf` p. 7 |
| Travar transformações afins | `Alt+B` | `Preferences.pdf` p. 12 |
| Fullscreen / sair do fullscreen | `Cmd+U` / `Cmd+T` | `My First Video Mapping.pdf` p. 7 |
| Salvar | `Ctrl+S` | `My First Video Mapping.pdf` p. 15 |
| Próximo handle do quad | `Tab` | `Introduction…` p. 10 |
| Mover handle com precisão | setas | `My First Video Mapping.pdf` p. 11 |
| Remover mídia do pool | `Delete` | `Introduction…` p. 5 |
| Criar grupo com a seleção | `Shift` + clique no ícone de grupo | `Introduction…` p. 16 |

**Gramática de mapeamento** (`Introduction…` p. 26): um botão **Learn** põe o app em modo de aprendizado e **todas as funções mapeáveis ficam realçadas em roxo**. Fontes de controle: teclado, MIDI, OSC, Audio, controle de Playstation, módulos do próprio MadMapper (Oscillator etc.) e "outros" (Leap Motion, que precisa ser ligado nas preferências). A Control List filtra por categoria. Um control aceita **uma nota MIDI só, mas aceita simultaneamente uma nota, uma tecla e um canal OSC** (`Knowledge Base.pdf` p. 4).

**Endereçamento OSC** (`OSC Channels.pdf` p. 2–6). Todo parâmetro tem endereço predefinido; os descobríveis por três caminhos: botão direito no widget → "Copy OSC address"; diálogo de Controls → `+` mostra a hierarquia inteira; ou por OSC Query. A hierarquia é o modelo de dados exposto:

    /surfaces/[selected | Grupo/Nome]/opacity | visible | blend_mode | invert_mask
    /surfaces/.../visual/{type,number,name}
    /surfaces/.../output/{x,y,scale,rot,3d_rot_x,3d_rot_y,3d_rot_z}
    /surfaces/.../output/handles/[0-3]/{x,y}
    /surfaces/.../input/{x,y,scale,rot,flip}
    /surfaces/.../color/{red,green,blue,rgba,hue,saturation,value}
    /surfaces/.../lights/0/{distance,latitude,longitude,pos_x,pos_y,pos_z,red,green,blue,color}
    /fixtures/[selected | Grupo/Nome]/{visible,luminosity,response,sliders/[n],color/...,input/...}
    /medias/{select,previous,next}  /medias/per_type_selection/next_[mediaType]
    /medias/[selected | nome]/{assign,assign_to_all_surfaces,restart,play_forward,pause,
        play_backward,begin,loop,position,position_sec,position_frame,previous_frame,next_frame,
        play_speed,absolute_speed,loop_start,loop_end,audio_level}
    /cues/active_bank
    /cues/[bank]/{auto_play}
    /cues/[bank]/columns/{start_next,start_previous,select_next,select_previous,
        start_selected,start_by_number,[column]}
    /cues/[bank]/scenes/by_name/[nome] | by_cell/col_[n]
    /cues/[bank]/cues/by_name/[nome] | by_cell/col_[n]/row_[m]
    /cues/[bank]/cues/{start_next,start_previous,select_next,select_previous,start_selected,
        start_by_number}
    /master/{master_level,master_video_level,master_dmx_level,master_audio_level,
        audio_input_level,video_color/...,freeze_engine,engine_speed,reset_engine_speed,
        freeze_video_output,freeze_dmx_output,output_cursor,output_cursor_size,test_pattern}
    /modules/[nome]/{select,active,...}
    /outputs/[nome]/{enabled,show_desktop_window,show_test_pattern,publish_to_syphon_spout,
        publish_to_ndi}
    /application/{mad_light_recorder/start_recording, preview/active_preview_group,
        medias/add, medias/remove, view/fullscreen}

Detalhe que resolve um problema real: o token `selected` no lugar do nome ("controle age no que está selecionado") e o par `by_cell` / `by_name` para cues. Em Edit Controls o app oferece "Map to Cell Position" (padrão) ou "Map To Cue"; com a segunda, mover a cue na grade **não quebra o mapeamento** (`Scenes and Cues.pdf` p. 9).

Diferença declarada entre endereço predefinido e control mapeado: o predefinido não tem "input range", "output range" nem filtro (`OSC Channels.pdf` p. 2). O control mapeado tem, mais faixas de origem/destino e filtros (`Modules.pdf` p. 8).

**Preferências de rede e protocolo** (`Preferences.pdf` p. 4–8): MIDI com dispositivos de Input e um de Feedback separados, e opção global "notas MIDI carregam velocity" (que também pode ser ligada control a control); OSC com porta de entrada 8010 por padrão, porta de feedback, IP de feedback "Auto" (responde a todo IP que mandou algo) ou fixo, e descoberta por Bonjour; DMX Input separado de DMX Output, com aviso explícito de não usar o mesmo device nos dois; Art-Net com escolha de interface de rede, Max FPS 44 por padrão (60 sugerido para controladores LED de ponta), unicast opcional e ArtSync; sACN com interface, Max FPS, **prioridade** para merge com outras fontes, e sincronização E1.31.

**Superfícies de controle**: o módulo Control Surface suporta APC mini mk2, APC40 mk2, Launchpad MK2, Launchpad Mini Mk3 e Launchpad X; os pads espelham automaticamente as cores das scenes e cues, e navega-se o bank pelos botões Up/Down/Left/Right e sliders Offset X/Y do módulo — que podem ser mapeados a qualquer outro controle quando não há device físico (`Control Surface Module Guide.pdf` p. 2–3). O plugin de Stream Deck endereça uma cue por `(coluna, linha, bank)` e reflete o thumbnail ou a cor da célula na tecla; nos dials do Stream Deck+, cola-se um endereço OSC copiado do widget e define-se o incremento por passo em porcentagem; apertar o dial devolve o parâmetro ao default (`Stream Deck Plugin Guide.pdf` p. 3–4).

## Laser (parâmetros reais)

Os nomes abaixo são as chaves literais de `customSettings` do output laser em `SampleProjects\Laser Example.mad` (chave `outputs.outputs[0].customSettings`), com os valores de fábrica do projeto de exemplo, cruzadas com a explicação do `MadLaser Guide.pdf`.

Output (`outputs.outputs[0]`): `outputType = "Laser"`, `name`, `id`, `active`, `laserDeviceUrl`, `laserDisplayName = "None"`, `stageSize`, `scale`, `rotation`, `position`, `flip`, `masks`, `maskOpacity`, `publishInternalLoopback`, `loopbackDispatchProjCount`, `loopbackDispatchTolerance`, `publishPonkEnabled`, `publishPonkIp = "127.0.0.1"`.

    Device/PPS            = "Custom"     Device/Custom PPS = 48000
    Device/Buffer Size    = 3000         Device/Delay      = 0
    ILDA/Desired FPS      = 60.0         ILDA/ILDA FPS     = 60.0
    ILDA/Point Count      = 800          ILDA/Scan Area    = 0.0
    ILDA/Mode             = "Preserve Image Quality"
    ILDA/Blank Delay      = 2.0          ILDA/Blank Smth   = 0.70
    ILDA/Blank Curve      = 0.34         ILDA/Blank Color  = MadColor
    ILDA/Blank using Worst Case = True   ILDA/Enable Frame Blending = False
    Color Levels/{Red,Green,Blue} = 1.0
    Min Voltage/{Red,Green,Blue}  = 0.25
    Time Shift/{Red,Green,Blue}   = 0    (deslocamento em pontos ILDA)
    Response/{Red,Green,Blue}     = curva
    Masks/Render Masks = False  Masks/Level = 0.1  Masks/Color = MadColor
    Test Pattern/Level = 0.25   Test Pattern/Color = MadColor
    Save ILDA Frame = False     Record ILDA Movie = False

Significado, do `MadLaser Guide.pdf` p. 4–8:

- **PPS**: o que o fabricante promete; é enviado junto com o frame e usado para calcular o número de pontos. A faixa é limitada de propósito "para não machucar o galvo"; acima de 45 kpps só faz sentido projetando muito longe com ângulo pequeno (p. 5).
- **Desired FPS** vs **ILDA FPS**: o primeiro é pedido, o segundo é o real, e vale a fórmula explícita `ILDA FPS = PPS / Point Count` (30 kpps / 500 pontos = 60 FPS). Abaixo de 35 FPS o piscar é visível, acima de 45 não se percebe a varredura (p. 5).
- **Blank Delay**: ao pular de um path para outro o feixe é desligado e movido; 100% é um valor calibrado pela equipe em vários aparelhos, e o modo de export permite valores menores (p. 6).
- **Min Voltage**: cada diodo acende a partir de uma tensão diferente; corta-se abaixo de um nível para que um cinza escuro não vire vermelho (p. 6).
- **Time Shift**: alguns projetores atrasam a resposta de cor em relação a XY, e às vezes cada diodo em relação ao outro — daí o offset por canal **medido em pontos ILDA**, não em milissegundos (p. 6).
- **Safety Area** (`ILDA/Scan Area`): impede o feixe de chegar às bordas do campo. O guia avisa para não apertar demais: alguns scanners entram em proteção ou se danificam, e "ouvir o scanner é um bom jeito de notar problema — o som deve ser liso, não estalado" (p. 6).
- **Masks**: regiões protegidas do espaço (pessoas, câmera). Com opacidade 100% o feixe não entra; com 90% ele entra com luminosidade reduzida, "útil em países onde varrer o público é permitido abaixo de um certo nível". Podem ser invertidas (aí definem a área *permitida*), e valem também para o cursor do mouse quando o cursor está sendo desenhado (p. 7).
- **Publish Internal Loopback**: um projetor laser sem destino publica seus paths como uma nova mídia em Live Inputs; `Dispatch Count` reparte os paths dessa composição em várias mídias, uma por projetor real. É como se faz mesh warping de uma composição inteira sem remapear cada superfície (p. 7–8).
- **Store Ilda Frame** e **Record Ilda Movie**: gravam na pasta `ILDA` do workspace. O **Movie Mode** escolhe entre "Record at fixed frame rate" (para tocar de volta no MadMapper) e "Record as ILDA stream" (para tocar num DAC ou no cartão SD do laser), porque o formato ILDA não carrega o PPS de playback e o hardware trata o arquivo como fluxo de pontos: um frame de 500 pontos dura menos que um de 700 (p. 8).

Parâmetros por **superfície laser**, chaves reais de `surfaces[].customSettings` (`Laser Example.mad`), grupo `Laser Render`:

    Max Speed = 1.0        Scan Speed = 1.0     Scan Mode = "Auto"
    Samples = 8192         Skip Black = True    Preserve Order = True
    Optimize Angles = True Angle Min = 34.38    Angle Delay = 0.05
    Start Repeat = 0       End Repeat = 6       Soft Close = 25
    In Fade = 1.0          Out Fade = 0.0
    Beam Mode = "Auto"     Beam Level = 10
    Override Loopback Render Settings = False

E o grupo `Active Segment` (`Start`, `End`, `Strt Smooth`, `End Smooth`), que recorta que trecho do path é desenhado.

Semântica (p. 9–11 e `Laser Materials Documentation.pdf` p. 3–5): **Max Speed** distribui o tempo de varredura entre os paths proporcionalmente ao comprimento, para que uma linha longa e um círculo pequeno saiam com o mesmo brilho; valor máximo 4, limitado porque varrer rápido esquenta o galvo. **Optimize Angles** injeta pontos nos vértices, senão a física do scanner arredonda os cantos; `Angle Min` em graus na UI e em radianos no ISF (`ANGLE_THRESHOLD`). **End Repeat** repete a última posição porque o software nunca sabe onde o feixe realmente está. **In Fade** evita o "hot point" do começo do path (o scanner parte da inércia zero) e **Out Fade** evita o do fim. **Point Intensity** (só em Laser Quad) controla quanto tempo se gasta num ponto isolado — path de comprimento zero —, que é como se faz um feixe forte parado; em Laser Line o equivalente é **Min Points**, mínimo de pontos ILDA por path. **Skip Black** pula os trechos apagados do path; desligá-lo estabiliza a imagem quando a forma é fixa e só a luz varia. **Preserve Order** força a ordem de geração, contra o flicker que a reordenação ótima causa quando os paths se movem.

Vetorização de vídeo → laser (`MadLaser Guide.pdf` p. 11–14; chaves reais em `surfaces[].customSettings` grupo `Process` do `Laser Example.mad`). Dois algoritmos:

- **Find Paths** — pixels acima de `Process/Threshold` viram path; `Use Color` usa a cor do pixel; `Thickness` diz a espessura do traço no material de origem para a esqueletização; `Denoizing`; `Max Res` reduz a mídia de entrada preservando o aspecto. No arquivo aparecem os sub-grupos que o PDF não detalha: `Process/Skeleton/{Mode, Thread Count, Smooth, Max Err Size, Fix Errors}`, `Process/CPU Thinning/{Algo = "zhang_suen_then_guo_hall", Max Iter., Threshold, Blur Size, Thread Cnt, Thrd Margin}`, `Process/GPU Thinning/{Iterations, Threshold, Blur, Blur Size}` e `Process/Recomposing/{Enabled, Angle Tolerance, Angle Weight, Angle Offset, Color Tolerance, Color Weight, Dist Tolerance, Dist Weight, Handle Points, Handle Intersections, Intersection Size}`.
- **Find Contours** — Canny na GPU: `Process/GPU Canny/{Threshold, Canny Size, Blur Size}`, com gêmeo em CPU (`Process/CPU Canny/{Threshold, Threshold Ratio, Size, Gradient}`).

Filtros de saída: `Path Filtering/{Min Length, Max Length}` em porcentagem da maior dimensão da mídia, e `Path Limits/{Mode = "Keep Longest", Count = 100}` — "se aparecerem mais de 50 paths, fique só com os 50 mais longos". `Monitor/Info` é o campo de erro da vetorização; o engine desiste se detectar mais de 2000 paths (p. 14). `Display/Mode = "Output Polylines"` escolhe o que o preview mostra: Source Image, Processed Image, Processed Image + Polylines, ou Output Polylines (p. 13).

Formatos de arquivo: importação e exportação de ILDA, importação de SVG, fontes stick TTF na pasta `Stick Fonts` do workspace, com as OneLineFonts empacotadas — fontes de esqueleto puro, que varrem muito mais rápido que o contorno das letras (`MadLaser Guide.pdf` p. 3, 15, 18). Há também `IldaMoviesIndexCache` em `AppData\Roaming\GarageCube\MadMapper\` — o app indexa os .ild.

**Laser Material** (`Laser Materials Documentation.pdf` p. 1–5): shader GLSL 150 core com o mesmo cabeçalho ISF dos materiais de vídeo, mas em vez de retornar cor por pixel implementa

    void laserMaterialFunc(int pointNumber, int pointCount,
                           out vec2 pos, out vec4 color, out int shapeNumber, out vec4 userData)

`pos` em −1..1, `color` com alpha ignorado (não há composição em path 2D), e **`shapeNumber`: toda vez que muda em relação à amostra anterior, começa um path novo**. Padrão de 8192 amostras, ajustável em `RENDER_SETTINGS.POINT_COUNT` (2 para uma reta, 1000 para um círculo). O material pode fixar os parâmetros de render do output — `MAX_SPEED`, `SKIP_BLACK`, `PRESERVE_ORDER`, `ANGLE_OPTIMIZATION`, `ANGLE_THRESHOLD`, `ANGLE_MAX_DELAY`, `FIRST_POINT_REPEAT`, `LAST_POINT_REPEAT`, `POLY_FADE_IN`, `MIN_ILDA_POINTS_PER_POLYLINE` — e a própria documentação avisa para não abusar disso, porque tira do usuário o ajuste no nível da superfície (p. 3). O frame anterior chega como `sampler2D mm_LastFrameData`, com layout fixo: linha 0 = `rg` posição e `b` número da forma, linha 1 = cor, linha 2 = userData (p. 5).

## Parâmetros de material/módulo (esquema)

Este é o esquema de "parâmetro que vira widget", e é o mesmo para material de vídeo, material laser, Surface FX e Laser Line FX. Arquivo canônico: `Resources\Materials\All Widgets Template\All Widgets Template.fs`. A pasta do material contém `<Nome>.fs`, opcional `<Nome>.vs`, `thumbnail.jpg|png`, texturas importadas e um `.info` XML com `<author>` e `<date>`.

Cabeçalho JSON num comentário `/*{ … }*/` no topo do `.fs`, com `CREDIT`, `DESCRIPTION`, `TAGS`, `VSN`, `INPUTS`, `GENERATORS`, `IMPORTED`, `RASTERISATION_SETTINGS`, `RENDER_SETTINGS`.

Um INPUT é `{"LABEL", "NAME", "TYPE", "MIN", "MAX", "DEFAULT", "VALUES", "FLAGS"}`. Tipos vistos em disco e na documentação: `float`, `floatRange` (par min-max num vec2), `int`, `long` (enum, com `VALUES` e `DEFAULT` pelo texto da opção), `bool`, `event` (verdadeiro por um frame), `color`, `point2D`, `curve` (com `INTERPOLATION: "catmull_rom"` e `DISPLAY: "linear"`), `audio` (waveform) e `audioFFT` (espectro, com `SIZE`, `ATTACK`, `DECAY`, `RELEASE`).

Duas convenções que valem a pena roubar:

- **O LABEL define a árvore da UI.** `"LABEL": "Noise/Amount"` cria a caixa de grupo "Noise" com o widget "Amount" dentro; a ordem dos grupos é a ordem de aparição no cabeçalho, e aceita mais de um nível (`"Scale/Animation/Active"`). Não existe declaração de layout separada (`Materials Documentation.pdf` p. 10).
- **FLAGS muda o widget sem mudar o tipo**: `button` transforma um `bool` de checkbox em push button; `button,trigger` faz botão momentâneo; `button_grid` num `long` vira grade de botões; `spinbox` transforma um slider de `int`/`float` em spin box; `no_alpha` tira o alpha do color picker; `generate_as_define` compila o valor como `#define` para o shader usar `#ifdef` (`Materials Documentation.pdf` p. 11–12).

**GENERATORS** é a peça que resolve, sem timeline, o problema de animar com parâmetro variável. São filtros nomeados cujo output é um uniform float, e cujos `PARAMS` aceitam um valor literal, o nome de um INPUT ou o nome de outro GENERATOR (`Materials Documentation.pdf` p. 5–9). Tipos: `time_base` (integra velocidade ao longo do tempo, com `speed`, `reverse`, `speed_curve`, `strob`, `bpm_sync`, `link_speed_to_global_bpm` — mudar a velocidade não faz a animação saltar, que é exatamente o defeito de escrever `sin(speed*TIME)`), `animator` (formas Smooth/In/Out/Linear/Cut/ Noise), `damper` (`hardness`, `damping`), `adsr` (`attack`, `decay`, `release`), `linear_filter` (`duration`), `ease` (`type` EaseIn/EaseOut/EaseInOut, `curve` 1..10), `multiplier` (até 4 entradas), `incrementer` (dois INPUTs de evento, +1 e −1), e `pass_thru` — que pega **qualquer canal do app por URL** como uniform, ex.: `{"TYPE": "pass_thru", "PARAMS": {"input_value": "/custom/BPM/bpmPos"}}`. Ou seja: o mesmo espaço de endereços das cues e do OSC alimenta o shader.

`IMPORTED` declara texturas com `TYPE` `2D`/`CUBE`/`3D`, `PATH` relativo à pasta do material, `GL_TEXTURE_MIN_FILTER`, `GL_TEXTURE_MAG_FILTER`, `GL_TEXTURE_WRAP` e `DEPTH` (`Materials Documentation.pdf` p. 4). `RASTERISATION_SETTINGS` liga render-to-texture com `DEFAULT_WIDTH/HEIGHT/PIXEL_FORMAT` e `REQUIRES_LAST_FRAME` para feedback via `mm_LastFrame` (p. 9–10). Restrições declaradas: sem multi-pass ISF, nomes de INPUT devem começar com `mat_` (ou `fx_` nos FX) para não colidir quando material e Surface FX rodam na mesma superfície, e há nomes reservados que o app recusa com mensagem de erro (p. 2–3).

No projeto salvo, esses parâmetros viram um dicionário plano com a chave sendo o LABEL completo: `"FX/Beams/Group Offset": 1.0`, `"Laser Render/Angle Min": 34.37`, `"Global BPM/BPM Source": "Manual"`. Não há esquema no arquivo de projeto — o esquema mora no `.fs`, o projeto guarda só pares nome-valor, mais `collapsedParameterGroups` (quais caixas estão dobradas na UI).

## Arquivo

`.mad` é **QDataStream de Qt**, não JSON nem zip. Layout: magic `0B AD BA BE`, `quint32` versão de stream (12), e a seguir um `QVariantMap` serializado — `quint32` contagem, e por entrada uma `QString` UTF-16BE com prefixo de tamanho em bytes mais um `QVariant` (`quint32` type id, `quint8` isNull, payload). Tipos usados: bool, int, double, QString, QStringList, QByteArray, QVariantMap, QVariantList, QSize, QPointF, QImage/QPixmap (marcador `qint32` 0/1 seguido do PNG cru, sem tamanho), uma matriz de 9 doubles para `perspectiveUv`, e quatro tipos de usuário identificados por nome: `MadColor` (modo `"rgb"` + 4 doubles), `FilePath` (uma QString), `QVector<float>` (contagem + doubles) e `IndexFloatMap`.

Raiz com 44 chaves, iguais nos 6 exemplos. Agrupadas:

- **Conteúdo**: `visuals.visuals[]` (a media bin), `surfaces[]`, `surfacesRelations[]` (a árvore de grupos, como lista de índices de pai), `meshes3D[]`, `outputs{outputs[], masks[], backgrounds[]}`, `stageMasks[]`, `stageBackgrounds[]`, `modules.modules[]`, `mappings[]` (os controls), `linkedOutputs[]`, `linkedS3Ds[]`.
- **Show**: `cueBanksV4{cueBanks[], activeCueBank, liveMode}`, `selectionGroups`, `previewSurfaceGroups[]`, `colorPalette{vsn, colors[]}`, `customColorTable[]`, `notes`.
- **Estado ao vivo**: `masterLevel`, `masterVideoLevel`, `masterDmxLevel`, `masterLaserLevel`, `masterAudioLevel`, `audioInputLevel`, `engineSpeed`, `engineFrozen`, `videoOutputsFrozen`, `dmxOutputsFrozen`, `laserOutputsFrozen`, `videoMasterColor`, `dmxMasterColor`, `laserMasterColor`, `masterCustomParameters` (que é só o grupo `Global BPM/...`: `BPM`, `BPM Source`, `Range`, `Beat`, `TAP`, `Resync`, `Ableton Link`, `Peers`, `Midi Input`, `Enable Setting BPM`).
- **UI persistida**: `previewsData` (23 chaves: geometria da janela, estado dos QSplitter em hex, zoom e scroll de cada view, `outputPreviewTransform` como matriz 3×3), `resourceEditorData`, `openedFilesInEditor`, `openedFoldersInEditor`, `materialsInEdition`, `editedVisualId`, `mediaRatioForced` e numerador/denominador, `bankIterCount`, `bankGenCount`, `projectPath`.

Uma superfície (`surfaces[]`) carrega: `type` (`quad`, `fixture`, `laser_quad`, `laser_lines`, `group`), `surfaceId`, `name`, `visible`, `locked`, `visualId`, `opacity`, `blendMode`, `modulation` (MadColor), `positions` e `uvs` como listas de pontos, `position`/`positionUv`, `scale`/`scaleUv`, `rotation`/`rotationUv`, `uvFlip`, `isPerspective`, `warpingEnabled`, `aspectRatioMode`, `userAspectRatio`, `geometryPrecision`, `feathering`, a família `softEdge{Left,Right,Top,Bottom}{Width,Curve}` mais `softEdgeGamma` e `softEdgeActive`, `masks[]`, `customSettings` e `collapsedParameterGroups`.

Uma fixture é uma superfície de `type = "fixture"` com `startChannel`, `artnetUniverse`, `dmxtype = "ArtNet"`, `responsePower`, `sliders` (IndexFloatMap), `filtering`, `filteringMode`, `filterKernelSize`, `filterAnamorphicKernelSize`, e um sub-objeto `fixture` embutido: `{group, product, type: "RGB", width, height, pixelMapping, avoidCrossUniversePixels, ignoreAlpha, favorite, isValid}`. O `pixelMapping` é uma string de offsets de canal separados por espaço (`"1 4 7 10 13 …"`). A biblioteca do usuário, `AppData\Roaming\GarageCube\MadMapper\LEDFixtureLib.mfl`, é XML com exatamente esses campos: `<LEDFixture group product favorite><PixelMapping avoidCrossUniversePixels width height type>offsets</PixelMapping></LEDFixture>`. Há `.mflb` e `.bbkp` ao lado — backups da mesma biblioteca.

Workspace do usuário (`Preferences.pdf` p. 3 e `C:\Users\email\Documents\MadMapper\`): pastas `Generators`, `LaserGenerators`, `LaserMaterials`, `Materials`, `Modules`, `Stick Fonts`, `Surface2DFX`, `Surface3DFX`, `SurfaceLaserLineFX`, `SurfaceLineFX` — e, quando usadas, `ILDA` e as sequências MadLight. O caminho do workspace é uma preferência, apontável para Dropbox ou rede, e existe justamente para trocar de conjunto de recursos por projeto.

## O que o Spellcaster deve copiar

- **Cue = lista de pares (endereço, valor) com fade por entrada.** É literalmente `{url, value, transition_settings:{type,duration}}` em `cueBanksV4.cueBanks[].cues[].entries[]`. Isso encaixa direto no `PRINCIPIOS.md §1` (o graph é a interface): se todo widget é um nó com endereço, a cue é um diff sobre o graph e não precisa de estrutura própria. **Beneficia: cenas e cues DMX** — e de graça dá cue de qualquer coisa, inclusive kpps do laser e estado de módulo.
- **Um endereço só, servindo de OSC, de chave de cue e de entrada de shader.** O `pass_thru` dos GENERATORS lê `/custom/BPM/bpmPos` como uniform (`Materials Documentation.pdf` p. 7), a cue grava `/fixtures/9/color/red`, e o OSC externo usa o mesmo caminho. **Beneficia: orquestrador** — é o argumento contra ter um espaço de nomes para o registry e outro para a rede.
- **`by_cell` vs `by_name` no mapeamento de cue** (`Scenes and Cues.pdf` p. 9). Mapear por posição na grade ou pela identidade da cue é uma escolha que o operador faz, e a segunda sobrevive a reorganizar o show às 23h. **Beneficia: cenas e cues DMX.**
- **Edit Mode com overlay: vermelho = está na cue, laranja = está com valor diferente.** Mexer no widget cuea; segurar Shift mexe sem cuear (`Scenes and Cues.pdf` p. 7–8). Um modo, dois estados de contorno, zero modal — e resolve "o que exatamente está gravado nesta cue" sem abrir inspector. **Beneficia: cenas e cues DMX, cenário interativo.**
- **Separar `Desired FPS` de `ILDA FPS` e mostrar os dois, com `Point Count` ao lado.** A relação `ILDA FPS = PPS / Point Count` (`MadLaser Guide.pdf` p. 5) é a única coisa que o operador precisa ver para entender por que o frame pisca. **Beneficia: ILDA player, NDI→ILDA** — e é exatamente o aviso que o Aprendiz deve dar ("o galvo não acompanha", `TEMAS.md` linha 15).
- **Time Shift por canal de cor medido em pontos ILDA, não em ms** (`MadLaser Guide.pdf` p. 6). É a unidade certa: o atraso do hardware é de amostras, não de tempo de parede. **Beneficia: ILDA player.**
- **In Fade / End Repeat / Point Intensity como parâmetros de primeira classe da superfície.** São a calibração física do galvo (inércia no começo, incerteza de posição no fim, ponto parado como feixe). **Beneficia: ILDA player, NDI→ILDA.**
- **Máscara de segurança com opacidade, invertível, e válida também para o cursor** (`MadLaser Guide.pdf` p. 7). Máscara binária não serve para a regra real (varrer o público abaixo de um nível). **Beneficia: ILDA player.**
- **Loopback interno como mídia.** Um output laser sem destino publica seus paths como Live Input, e `Dispatch Count` reparte em N mídias (`MadLaser Guide.pdf` p. 7–8). É composição em cadeia sem inventar um grafo de nós à parte. **Beneficia: NDI→ILDA, orquestrador.**
- **LABEL com `/` gera a árvore de widgets, e FLAGS troca o widget sem trocar o tipo** (`Materials Documentation.pdf` p. 10–12). Um só lugar declara nome, tipo, faixa, default, agrupamento e forma de widget. **Beneficia: todas as Faces, e sobretudo o Aprendiz**, que precisa de um esquema legível para explicar cada parâmetro.
- **GENERATORS como filtros nomeados entre controle e valor** (`time_base`, `damper`, `adsr`, `ease`, `incrementer`). O `time_base` existe porque `sin(speed*TIME)` salta quando se mexe na velocidade — bug real que o Spellcaster vai ter no primeiro fader de velocidade que fizer. **Beneficia: orquestrador, cenário interativo.**
- **Cue Scheduler confere o relógio a 1 Hz e, em conflito, o último módulo da lista vence** (`Cue Scheduler.pdf` p. 5). Regra de desempate simples, declarada, sem prioridade configurável. **Beneficia: orquestrador.**

## O que NÃO copiar

- **Scene como "cue que esconde o que veio depois"** (`Scenes and Cues.pdf` p. 2). É uma exceção cravada na primeira linha da grade, com regras próprias (não dá para remover entrada, não dá para gravar parte de um objeto, Backspace não funciona). Duas semânticas na mesma grade é armadilha de ensaio. Se o Spellcaster precisa de "estado completo", que seja uma cue com flag `exclusiva`, não uma classe separada.
- **Grade de 16×8 achatada em `cues[128]` com índice implícito.** Mover uma cue muda o índice, o que é justamente o motivo de existir o `by_name`. Guardar a cue com `(col, row)` explícito, ou melhor, com id próprio e posição como atributo.
- **Thumbnail PNG embutido byte a byte dentro do arquivo de show.** No `DMX LED Bar Example.mad` uma única cue carrega 17 658 bytes de PNG. O `.spell` é JSON e vai para o git; o thumbnail vai para fora, referenciado.
- **Estado de UI dentro do arquivo de projeto.** `previewsData` guarda posição de janela, largura salva, orientação do splitter e estados de `QSplitter` em hex; `projectPath` do exemplo de fábrica ainda aponta para `/Users/matt/Projects/MadMapper/Dev/forge/...`, o Mac do desenvolvedor. Isso fecha a porta a diff, a merge e a rodar o mesmo show no Pi sem tela. Layout em `config.json` do perfil, não no show.
- **Formato binário proprietário sem versão semântica.** A única coisa versionada é `streamver = 12` (versão do QDataStream), e chaves como `cueBanksV4` carregam a versão no *nome*. Migrar isso exige o app. `.spell` fica JSON com `"version"` no topo.
- **Módulo como blob cifrado.** `Resources\FactoryModules\*\main.ldat` não é legível nem por `strings` — nem os módulos de fábrica, nem os geradores (`Laser Text`, `Laser Grid`). Quem compra o software não consegue ler nem versionar o que roda no show dele. O módulo do Spellcaster tem que ser arquivo de texto.
- **Fixture sem opacidade, sobrescrevendo a de baixo** (`Introduction…` p. 19). Regra herdada do pixel mapping que não vale para luz de palco, onde HTP e LTP são o esperado. Definir a política de merge explicitamente e não deixar "a última ganha" por acidente de implementação.
- **Duas cores de accent mais um token separado só para cue** (`app.css`: `$selection_blue`, `$active_selection_blue`, `$preview_group_green`, `$cue_color`). `PRINCIPIOS.md §2` proíbe mais de um accent por Theme; aqui azul significa "selecionado", verde significa "preview group" e a cor da cue é escolhida pelo usuário por célula — três significados de cor competindo na mesma tela.
- **Enum como string livre no arquivo salvo.** `"Process/CPU Thinning/Algo": "zhang_suen_then_guo_hall"`, `"ILDA/Mode": "Preserve Image Quality"`, `"Device/PPS": "Custom"`. Renomear a opção na UI quebra o arquivo salvo em silêncio. Guardar o identificador estável e rotular na apresentação.
- **A janela de atalhos como única documentação dos atalhos.** O menu Ajuda tem "Raccourcis clavier" (`mm_fr.qm`), mas nenhum dos 31 PDFs traz a tabela; os atalhos aparecem espalhados, em parênteses, no meio de tutoriais de máscara e de laser. `SHORTCUTS.md` já faz o contrário, e é o caminho certo.
