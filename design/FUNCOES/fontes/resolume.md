# Resolume Arena 7.21.1 — auditoria de interface para o Spellcaster

Auditoria feita lendo apenas arquivos da instalação e da pasta de documentos do usuário nesta máquina.
O Arena não foi aberto. Nada aqui vem de memória: cada afirmação cita o arquivo lido.
Versão de referência em todos os arquivos do usuário: `versionInfo majorVersion=7 minorVersion=21 microVersion=1 revision=37851`.

## Fontes lidas

| Arquivo | O que é |
|---|---|
| `C:\Program Files\Resolume Arena\default\Example.avc` (354.122 B) | composição de fábrica, XML; 2.758 elementos, 56 tags distintas |
| `C:\Users\email\Documents\Resolume Arena\Compositions\aniver 30.avc` (197.486 B) | show real do usuário; 1.364 elementos, 42 tags |
| `C:\Users\email\Documents\Resolume Arena\Compositions\climao.avc` (1.207.870 B) | show real do usuário; 4.373 elementos, 42 tags |
| `...\Shortcuts\Keyboard\Default.xml` | mapa de teclado inteiro, 42 atalhos; idêntico ao de fábrica em `default\Shortcuts\Keyboard\Default.xml` (só `presetId` e `versionInfo` diferem) |
| `...\Shortcuts\MIDI\{Default,APC MINI,APC 40 MK II,Nano Kontrol 2}.xml` | mapeamentos MIDI; `Default.xml` é do usuário (7 faders do APC40 mkII) |
| `...\Shortcuts\OSC\{Default,OSC_TD_TEST}.xml` | `Default` vazio; `OSC_TD_TEST` com 3 mapeamentos de saída |
| `...\Shortcuts\DMX\Default.xml` | `<ShortcutManager name="DMXShortcutManagerShortcuts"/>` — vazio |
| `...\Shortcuts\activePresets.xml` | qual preset está ativo por protocolo |
| `...\Fixture Library\*.xml` (24 fixtures do usuário) e `default\Fixture Library\LED 16.xml` (única de fábrica) | esquema de personality |
| `...\Preferences\{config,recentLayout,dmx,midi,osc,AdvancedOutput,SimpleOutput,server}.xml` | preferências e layout de painéis |
| `...\Presets\Advanced Output\amiver 30.xml` (78.923 B) | o patch DMX real do show: 1 Lumiverse, 11 fixtures |
| `...\Presets\Interface\*.xml` (9) e `default\Presets\Envelopes\*.xml` | presets de layout e de envelope |
| `C:\Program Files\Resolume Arena\docs\help\English.xml` | 412 verbetes de ajuda contextual (`<help components> → <title> + <text>`) |
| `C:\Program Files\Resolume Arena\docs\gui\English.txt` | 1 arquivo de strings da GUI, formato `"chave"="valor"` |
| `C:\Program Files\Resolume Arena\rest\docs\swagger.yaml` (208.471 B) | OpenAPI da REST API: o modelo de dados mais limpo que o produto expõe |

Skins/temas: **não existem**. Um `find` por `*.css`, `*skin*`, `*theme*`, `*.qss` em `C:\Program Files\Resolume Arena` retorna só
`rest\docs\index.css`, `rest\docs\swagger-ui.css` e `rest\landing\index.css` — folhas do Swagger UI e da landing do webserver, não da aplicação.
O que o Arena chama de "preset de interface" é só arranjo de painéis: `Presets\Interface\Panels Left & Right.xml` usa o mesmo schema `<Layout>` do `recentLayout.xml`.

## Anatomia da tela

Prova: `...\Preferences\recentLayout.xml`. O layout é uma árvore `Window → FlexContainer(orientation) → TabbedContainer → <PainelX>`,
com `width_desired` / `height_desired` por painel — layout de console, não responsivo.

Painéis instanciados na sessão real do usuário (na ordem em que aparecem no XML):

- `<LayersAndClips>` — **o grid de clips e a tira de layers são UM painel só** (`recentLayout.xml`, `Window/FlexContainer/FlexContainer/FlexContainer/LayersAndClips`, `height_desired=318`). Ocupa a metade de cima da janela.
- `<Toolbar>` (`height_desired=36`) — faixa fina entre o grid e os painéis de baixo.
- Dois `<Monitor>` empilhados à esquerda: `instance=0 subject_type="Composition" checker_board=0 showUI=0` e `instance=1 subject_type="Preview" checker_board=1 showUI=1`. Ou seja: **output e preview são o mesmo widget parametrizado**, não dois componentes.
- `<TabbedContainer>` com `<Composition>` + `<Layer>` (abas de propriedade do que está selecionado).
- `<TabbedContainer>` com `<Clip>`.
- `<TabbedContainer>` com `<Files>`, `<Compositions>`, `<Effects>`, `<Sources>`, `<Recording>` — o browser é uma pilha de abas, não uma árvore.

Em `<HiddenPanels>` (existem, estão fechados, e guardam a aba de onde saíram via `unhide_panel_id` + `unhide_target_index`):
`RenderQueue`, `ClipTime` (com dois `<Clock mode= show_time_remaining=>`), `Notes`, `Group`, `Slices`, `SMPTE` (dois campos com `colorIndex`), `Shortcuts`.

Abaixo, `<Params name="LayoutParams">` é uma lista plana de 33 booleanos `Show X` — a interface inteira é ligada e desligada por checkbox,
não por "modo" ou "workspace": `Show Compositions`, `Show Layer`, `Show Clip`, `Show Composition`, `Show Group`, `Show Ableton Link`,
`Show Audio Controls`, `Show Layer Transport Controls`, `Show Layer Transition Controls`, `Show Crossfader`, `Show Help`, `Show Dashboard`,
`Show Autopilot`, `Show Cue Points`, `Show Beat Looper`, `Show FFT Gain`, `Show Undo Toolbar`, `Show Monitor`, `Show SMPTE`,
`Show Denon DJ StageLinQ`, `Show Pioneer DJ TCNet`, `Show FPS & Stats`, `Show Shortcuts`, `Show Effects`, `Show Sources`,
`Show Clip Time`, `Show Notes`, `Show Slices`, `Show Files`, `Show Render Queue`, `Show Recordings`, `ShowFileBrowserThumbnails`, `showClipTimeRemaining`.

Consequência para o Spellcaster: cada um desses `Show X` é uma linha de show que o operador liga por necessidade da noite, e o layout salvo é um preset nomeado (`Presets\Interface\Big Monitor.xml`, `Dual Display.xml`, `Many Layers.xml`, `Triple Display.xml`, ...). Isso é exatamente a separação Face/Theme do `design/PRINCIPIOS.md §5`, só que sem o Theme.

## Objetos e verbos

Modelo canônico em `rest\docs\swagger.yaml`, seção `components/schemas` (linha 5174 em diante).

| Objeto | Onde | Verbos / campos observados |
|---|---|---|
| `Composition` | swagger:6037 | `bypassed`, `master`, `speed`, `cliptarget`, `cliptriggerstyle`, `clipbeatsnap`, `dashboard`, `audio`, `video`, `crossfader`, `decks[]`, `layers[]`, `columns[]`, `layergroups[]`, `tempo_controller`; endpoints `POST /composition/action` (`undo`/`redo`, swagger:109) e `POST /composition/disconnect-all` (swagger:138) |
| `Deck` | swagger:5724 | `closed` (bool, read-only), `name`, `colorid`, `selected`, `scrollx`; `/decks/{i}/select`, `/decks/add`, `/duplicate`, `/by-id/{id}/open`, `/by-id/{id}/close` |
| `Column` | swagger:5682 | `name`, `colorid`, `connected` (**ChoiceParameter**, read-only); `/columns/{i}/connect`, `/columns/add`, `/duplicate` |
| `Layer` | swagger:5947 | `bypassed`, `solo`, `crossfadergroup`, `master`, `maskmode`, `ignorecolumntrigger`, `faderstart`, `dashboard`, `audio`, `video` (`VideoTrackLayer` com `autosize`), `transition`, `clips[]`, `autopilot`; `/select`, `/clear`, `/clearclips`, `/add`, `/duplicate`, `/effects/video/{add,move,offset}` |
| `LayerGroup` | swagger:5995 | tudo de Layer menos `maskmode`/`faderstart`, mais `speed`, `layers[]`, `/move-layer`, `/add-layer`, `/layergroups/{i}/columns/{c}/connect` (**coluna dentro do grupo**) |
| `Clip` | swagger:5612 | `name`, `colorid`, `selected`, `connected` (Choice), `target`, `triggerstyle`, `ignorecolumntrigger`, `faderstart`, `beatsnap`, `transporttype`, `transport`, `dashboard`, `audio`, `video`, `thumbnail`; `/connect`, `/select`, `/open`, `/openfile`, `/clear`, `/thumbnail/{last-updated}` |
| `AutoPilot` | swagger:5555 | um único campo: `target` (ChoiceParameter). O resto do autopilot (duração, ação) mora no clip: ajuda `Duration` = "how long this clip will play when the autopilot is active", `Autopilot Action` = "Set the behaviour of the clip when in Autopilot mode", `Loops` = "Determine how many times the clip repeats" |
| `TransportTimeline` / `TransportBPMSync` | swagger:5590 e 5562 | `position` + `controls`: `playdirection`, `playmode`, `playmodeaway`, `duration`, `speed`; a versão BPM acrescenta `bpm`, `syncmode`, `beatloop` |
| `LayerTransition` | swagger:5874 | `duration`, `blend_mode` |
| `CrossFader` | swagger:5701 | `phase`, `behaviour`, `curve`, `sidea` (`ParamEvent`), `sideb` (`ParamEvent`), `mixer` |
| `TempoController` | swagger:5933 | `tempo` (Range) + quatro `ParamEvent`: `tempo_pull`, `tempo_push`, `tempo_tap`, `resync` |
| `Effect` / `VideoEffect` / `AudioEffect` | swagger:5751, 5463 e 5445 | `idstring`, `name`, `display_name`, `bypassed` (nullable — "primary Transform for example is not allowed to be bypassed"), `mixer`, `params`, `effect`, `presets[]` |
| `Source` | swagger:5802 | `idstring`, `name`, `presets[]` |
| `Screen` / `Slice` / `Lumiverse` / `Fixture` | não estão na REST API | só existem no Advanced Output (ver §Fixtures e DMX) |

Verbos que existem só como caminho de atalho, não como recurso REST (lidos em `Shortcuts\Keyboard\Default.xml`):
`/composition/connectnextcolumn`, `/connectprevcolumn`, `/selectnextdeck`, `/selectprevdeck`,
`/composition/selectedlayer/connectnextclip`, `/connectprevclip`, `/composition/selectedlayer/clear`, `/composition/disconnectall`.

Vocabulário de ejeção, direto da ajuda (`docs\help\English.xml`): `Clear All Layers` = "Eject ALL the clips. From ALL the layers.";
`Clear Layer` = "Clear the playing clip from this layer."; `Layer Solo` = "Show this layer, the whole layer and nothing but this layer.";
`Column Trigger` = "Start all clips in this column at once."; `Decks` = "Decks are like your records, and your composition is your record bag."

## Estados

- **Clip conectado não é booleano.** `Clip.connected` e `Column.connected` são `ChoiceParameter` read-only (swagger:5633 e 5697), não `ParamBoolean`. A ajuda confirma quantos estados: em `MIDI Out Status` — "for instance clip triggers can have 5 different states! Colors galore!" (`docs\help\English.xml`). Cada estado tem cor própria de LED no controlador.
- `selected` é `ParamBoolean` **read-only** em Clip, Layer, LayerGroup e Deck. Seleção é consequência de `/select`, não campo editável.
- `Deck.closed` é bool read-only; a lista de decks aparece no `.avc` como `<DeckInfo name id closed>` (ex.: `aniver 30.avc`, três `DeckInfo` com `closed="0"`).
- `bypassed` e `solo` são `ParamBoolean` em Layer e LayerGroup; `bypassed` também existe na Composition (o "Master Bypass" = "Makes all output go black.").
- Estado de tempo: os quatro eventos do `TempoController` são `ParamEvent` — não guardam valor, só disparam. O `.avc` guarda só `Tempo` (`Example.avc` = 128; `climao.avc` = 129.69098774160480048; `aniver 30.avc` nem serializa, está no default 120).
- Estado de fixture: **dois níveis de habilitação**, `Enable Lumiverse` ("Turn off the fixtures in this Lumiverse in one fell swoop") e `Enable Fixture` ("This will stop Resolume from sending DMX data to this fixture") — ambos em `docs\help\English.xml`.
- **Não existe estado "armado" nem "ensaio" em lugar nenhum.** Nem no swagger, nem no `recentLayout.xml`, nem na ajuda. Assim que um Lumiverse existe em `Presets\Advanced Output\amiver 30.xml`, ele envia. O único freio é o `Enabled` do screen/fixture.

## Atalhos e mapeamento

### Mapa de teclado de fábrica (42 atalhos)

`Shortcuts\Keyboard\Default.xml`. Cada `<RawInputMessage key="N">` é `keycode << 32 | modificadores`;
os keycodes são ASCII para teclas normais e `0x10025`/`0x10027` (65573/65575) para seta esquerda/direita.
Modificadores observados: `0` (nenhum), `1` (Shift), `3` (Shift + segundo modificador).

| Tecla | Alvo (`path` do XML) |
|---|---|
| `1` `2` `3` | `/composition/layers/{1,2,3}/select` |
| `Shift`+`1`…`9` | `/composition/columns/{1..9}/connect` |
| `Q W E R T Y U I` | `/composition/selectedlayer/clips/{1..8}/connect` |
| `←` / `→` | `/composition/selectedlayer/connectprevclip` / `connectnextclip` |
| `Shift`+`←` / `→` | `/composition/connectprevcolumn` / `connectnextcolumn` |
| `Shift`+mod2+`←` / `→` | `/composition/selectprevdeck` / `selectnextdeck` |
| `Espaço` | `/composition/tempocontroller/tempotap` |
| `,` / `.` | `/composition/tempocontroller/tempo` com `Subtarget type="1" optionIndex="8"` / `optionIndex="7"` (nudge para baixo/cima) |
| `/` | `/composition/tempocontroller/resync` |
| `B` / `Shift`+`B` | `/composition/selectedlayer/bypassed` / `/composition/bypassed` |
| `S` | `/composition/selectedlayer/solo` |
| `X` / `Shift`+`X` | `/composition/selectedlayer/clear` / `/composition/disconnectall` |
| `M` / `Shift`+`M` | `/composition/selectedlayer/master` / `/composition/master` |
| `V` | `/composition/selectedlayer/video/opacity` |
| `A` | `/composition/selectedlayer/audio/volume`, com `ValueRange min=0.0630957…` (fader logarítmico, −24 dB) |
| `P` | `/composition/selectedclip/video/effects/transform/positionx` **e** `positiony`, no mesmo `P`, com `behaviour` diferente (4098 vs 12290) e `ValueRange 0.4375..0.5625` |
| `G` | `/composition/objects/1505384963468/video/effects/addsubtract/effect/g` — atalho amarrado a um objeto por id numérico |

Gramática que sai daí: **sem modificador age no selecionado, `Shift` sobe um nível de escopo** (layer→coluna, layer→composição),
letras da fileira de cima são a coluna de clips do layer selecionado, e o mesmo `P` vira eixo X ou Y conforme o `behaviour`
(mouse horizontal/vertical, ver strings `"Mouse"`, `"Horizontal"`, `"Vertical"`, `"Sticky"` em `docs\gui\English.txt:427-430`).
Não há `Space` para play/pause: `Space` é tap de BPM. Não há atalho de GO, de cue, de blackout, nem de saída armada.

### Gramática do XML de um mapeamento (igual nos quatro protocolos)

```
<Shortcut uniqueId behaviour paramNodeName inputDeviceName outputDeviceName hasCustomOutputPath>
  <ShortcutPath name="InputPath"          path="/composition/..." translationType allowedTranslationTypes/>
  <ShortcutPath name="OutputPath"         path="..."/>   <!-- feedback: endereco OSC ou device MIDI -->
  <ShortcutPath name="InputSiblingPath"   path=".../selected"/>  <!-- de onde ler o estado -->
  <ShortcutPath name="OutputSiblingPath"  path=".../selected"/>  <!-- para onde devolver o estado -->
  <Subtarget type optionIndex/>
  <ValueRange min max/>
  <RawInputMessage key value numSteps/>
  <NamedValues><Value first="Off" second="0"/><Value first="On" second="1"/></NamedValues>
</Shortcut>
```

- `translationType` / `allowedTranslationTypes` codificam o escopo do alvo. A ajuda (`Shortcut Target`) nomeia os três: *"Selected..."* (o clip/layer/grupo selecionado no momento), *"This..."* (aquele objeto específico, onde quer que ele vá) e *"By Position"* (por índice: sempre o primeiro layer, mesmo reordenando). No XML: `1` para alvos absolutos (`/composition/columns/3/connect`), `8` para alvos de layer selecionado, `2`/`4` para alvos por objeto/por clip selecionado; `allowed` é a máscara do que aquele alvo aceita (`1`, `3`, `7`, `11`).
- `RawInputMessage key` é um pacote de 8 bytes: `[tipo:1][hash do device:4][data1:2][status:1]`.
  Verificado: APC MINI `0x0100000000000790` = tipo 1, device 0 (qualquer), nota `0x07`, status `0x90` (NoteOn ch1);
  `Shortcuts\MIDI\Default.xml` `0x0204cc397d0007b2` = tipo 2, device `04cc397d` (APC40 mkII), CC `0x0007`, status `0xB2` (CC ch3);
  OSC `0x0600000000000000` = tipo 6 e mais nada — casamento é por endereço, o `path` já é a chave.
  Teclado é tipo 0 com keycode e modificador nos 32 bits baixos.
- `numSteps="128"` aparece em todo mapeamento MIDI: a resolução do controlador é do mapeamento, não do parâmetro.
- `NamedValues` é a tabela de cor de LED (`<Value first="On" second="0.039370078740157479769"/>` = velocity 5/127 no APC MINI).
  Ajuda `MIDI Out Velocity`: "This will set the color of the pad."

### Modos de atalho

Lidos em `docs\gui\English.txt:409-430 e :467`: `Absolute`, `Button`, `Relative`, `Fake relative`, `Piano`, `Value`, `Toggle`;
mais `Mouse` / `Horizontal` / `Vertical` / `Sticky`, `Invert Value`, `Range`, `14-Bit`, `CC` / `CC Fine`, `Loop`, `Step Size`,
`Select Next Item`, `Select Previous Item`, `Random Item`, `Jump To Playhead`.
Ajuda `Shortcut Mode`: "The two most important ones to know about are Toggle and Value. Toggle will let you switch between two values with each press. Value will always set the parameter to a fixed value."
Ajuda `Piano Mode`: "On key down the parameter will jump to the max value. When the key is released the parameter will jump back to the min value." (= momentâneo)
Ajuda `MIDI Controller Mode`: "Endless dials should be set to 'Relative'. Fixed MIDI controllers to 'Absolute'."
No XML isso vira o campo `behaviour` (valores vistos: `0`, `8`, `26`, `1024`, `1028`, `4098`, `12290`, `14338`) — bitfield não documentado.

### Mapeamento DMX de entrada

`Shortcuts\DMX\Default.xml` está **vazio**. A gramática está só na ajuda: `DMX Lumiverse` ("Choose which internal Resolume universe this DMX shortcut is part of"),
`DMX Channel` e `DMX Channel Offset` ("shift all the assigned DMX channels up or down"); e nas strings `"Universe:"`, `"Channel:"`, `"With offset"`, `"Send DMX channel to assign it to this interface element."` (`docs\gui\English.txt`).
`Preferences\dmx.xml` na máquina: `<DmxController automapEnabled="1" artNetName="Arena nautilus" bindAdapter="ethernet_32768"><Inputs/></DmxController>` — nenhuma entrada configurada.

### Estado dos presets nesta máquina

`Shortcuts\activePresets.xml` guarda quatro ids (`DMXShortcutPreset`, `KeyboardShortcutPreset`, `MidiShortcutPreset`, `OSCShortcutPreset`).
As composições referenciam presets **por nome**: `climao.avc` tem `Param name="OscShortcutPreset" value="OutputAllMessages"` — e não existe
`OutputAllMessages.xml` em `Shortcuts\OSC\` (só `Default.xml` e `OSC_TD_TEST.xml`). Referência por nome que quebra sem aviso.

## Fixtures e DMX

### Esquema de uma personality

25 arquivos lidos (24 do usuário + `LED 16` de fábrica). O schema completo, sem exceção, é:

```
<Fixture uuid="<32 hex>" fixtureName="...">
  <versionInfo .../>
  <Params>
    <ParamRange storage="0" name="Dimmer" T="DOUBLE" default="0" value="255">
      <PhaseSourceStatic phase="1"/><BehaviourDouble/>
      <ValueRange name="defaultRange" min="0" max="255"/>
      <ValueRange name="minMax"       min="0" max="255"/>
      <ValueRange name="startStop"    min="0" max="255"/>
    </ParamRange>
    ...                                   <!-- 1 ParamRange = 1 canal DMX -->
    <ParamFixturePixels name="Pixels">    <!-- no maximo um por fixture -->
      <ParamRange name="Width"/> <ParamRange name="Height"/>   <!-- 1..512 -->
      <ParamChoice name="Color Format" default="rgb" value="rgbw"/>
      <ParamChoice name="Distribution" T="INT32" value="170"/>
      <ParamRange  name="Gamma" default="2.5" min="1" max="3"/>
    </ParamFixturePixels>
  </Params>
</Fixture>
```

Contagem sobre os 25 arquivos: `ParamRange` 123×, `ParamFixturePixels` 23×, `ParamChoice` 46×, `ValueRange` 357×. **Nenhum outro tipo de nó.**

O que isso significa na prática:

- **Não existe taxonomia de canal.** Não há tipo `dimmer`, `pan`, `tilt`, `strobe`, `color wheel`. Só `ParamRange 0..255` com um `name` livre. As fixtures reais do usuário provam: `core - PARLED` tem `Dimmer` + `Pixels(rgbw)`; `core - ribalta43ch tilt` tem `Dimmer `, `Shutter`, `Pixels(l)`; `srg - strobo fita 11chrgb` tem `Dimmer`, `Shutter`, `Pixels(rgb)` e mais dois canais chamados `New Parameter N`.
- **Não existe 16 bits**, nem canal fine/coarse, nem ranges com rótulo (nada de "0-7 = fechado, 8-134 = strobe"). `teste_parametros.xml` são 34 `ParamRange` idênticos chamados `New Parameter 1..34` — foi assim que o usuário patcheou uma moving de 34 canais.
- **Ordem = endereço.** A ordem dos elementos no `<Params>` é a ordem dos canais. A ajuda `DMX Parameter` diz "You can drag parameters around to change their order."
- O bloco de pixels é o coração: `Color Format` observado com valores `rgb`, `rgbw`, `gbr`, `brg`, `l` (luminância); `Distribution=170` em 100% dos arquivos (ajuda `Distribution`: "The open arrow signifies the first channel, the direction of the subsequent arrows define how the channels 'snake' through the fixture"). `LED 16` de fábrica usa a chave antiga `Color Space="1"` em vez de `Color Format` — deriva de formato entre versões.

### O patch: Lumiverse, Screen, Slice

`Preferences\AdvancedOutput.xml` é só um ponteiro (`<ScreenSetup presetFile="amiver 30"/>`); o patch inteiro está em
`Presets\Advanced Output\amiver 30.xml`. Estrutura real do show:

```
ScreenSetup
 └ CurrentCompositionTextureSize width=1920 height=1080
 └ screens
    └ DmxScreen name="artnet" uniqueId="1784918734030" LumiverseId="1"
       ├ Params: Name, Enabled, Hidden, Auto Span, Align Output
       ├ Params Output: Opacity, Brightness, Contrast, Red, Green, Blue   (-1..1)
       ├ guides: 2× ScreenGuide (grid e imagem de referencia)
       ├ OutputDevice → OutputDeviceDmx name="Lumiverse" deviceId="Lumiverse"
       │    Framerate (1..40, default 30, valor 30) · Delay (0..150 ms, valor 40)
       │    Subnet=0 · Universe=1
       └ layers: 11× DmxSlice
            Params Common : Name, Enabled
            Params Input  : Input Source · Fixture (uuid) · Start Channel (1..131072) · Filter Mode
            Params Output : Flip, Brightness, Contrast, Red, Green, Blue, Soft Edge
            InputRect  : 4 vertices em pixels da composicao   (ex.: 400.5,810.75 → 693.5,1009.75)
            OutputRect : 4 vertices normalizados (-0.5..0.5)
            FixtureInstance → Fixture (copia inteira da personality com os valores vivos)
```

Os 11 slices do show, com nome, fonte e canal inicial:

| Slice (nome dado pelo usuário) | Input Source | Fixture (uuid) | Start Channel |
|---|---|---|---|
| `40 - 68 core - atomic39ch color` | `3:2` | `13ed8267…` | 40 |
| `80 - 108 core - atomic39ch color` | `3:2` | `13ed8267…` | 80 |
| `120 - 148 core - atomic39ch color` | `3:3` | `13ed8267…` | 120 |
| `170 - 198 core - atomic39ch color` | `3:3` | `13ed8267…` | 170 |
| `240 - 268 core - atomic39ch color` | `3:5` | `13ed8267…` | 240 |
| `300 - 304 core - PARLED` … `330 - 334` | `3:1` / `3:4` | `b4e49617…` | 300, 310, 320, 330 |
| `500 - 502 FOG` / `505 - 507 FOG` | `3:7` / `3:6` | `649146e3…` | 500, 505 |

Leituras diretas:

- **DMX no Arena é vídeo amostrado.** Um fixture não recebe valores: recebe os pixels que caem dentro do `InputRect` dele na textura da composição, convertidos pelo `Color Format`/`Distribution`/`Gamma` da personality. Ajuda `Fixture Output Preview`: "Preview of the RGB data that this fixture is sending out."
- **Lumiverse ≠ universo.** `Start Channel` vai de 1 a 131.072 = 256 × 512. O Lumiverse é um espaço de endereço contínuo que o `OutputDeviceDmx` fatia em universos Art-Net (`Subnet` 0-15 × `Universe` 0-15, conforme ajuda `Subnet Number` e `Universe`). Ajuda `DMX Output`: "Here you can visually re-arrange the start channels of your fixtures or check for any overlap."
- **O usuário endereça pelo nome do slice.** Nenhum campo do XML guarda "40 a 68"; ele escreveu isso no `Name` porque a UI não mostra a faixa ocupada na lista. Sintoma de falta na interface, não estilo.
- A personality é **copiada** para dentro do patch (`FixtureInstance/Fixture`), com **`fixtureName=""`** e `uuid` novo — o vínculo com a biblioteca é só o `ParamChoice name="Fixture"` do slice. Editar a personality na biblioteca não atualiza o show, e o show não sabe mais o nome do que carrega.
- O único ajuste temporal do DMX é o `Delay` (0-150 ms) no dispositivo de saída, aplicado ao Lumiverse inteiro. Não há delay por fixture, nem fade, nem curva de dimmer por canal.

## Tipos de parâmetro

Duas gramáticas para o mesmo modelo. A da REST (limpa) e a do XML de arquivo (real).

### REST (`swagger.yaml`, `components/schemas`, linha 5174+)

| Tipo (`valuetype`) | Campos próprios | Widget que a API sugere |
|---|---|---|
| `ParamBoolean` | `value: bool` | toggle |
| `ParamChoice` (e `ParamState`) | `value: string`, `index: int`, `options: [string]` | `choice_buttons` ou `choice_combobox` |
| `ParamColor` | `value: "#rrggbb[aa]"`, `palette: [string]` | `color_picker`, `color_pallette`, `slider_color_*` |
| `ParamEvent` | **nenhum** — só dispara | botão de trigger |
| `ParamNumber` | `value: int64` | `spinner` |
| `ParamRange` | `min`, `max`, **`in`**, **`out`**, `value` | `slider` / `rotary` |
| `ParamString` | `value: string` | `text` |
| `ParamText` | `value: string multilinha` | `text_multiline` |

Todos carregam `id: int64` e `view: ParameterView`. O `ParameterView` (swagger:5202) é a camada de apresentação, separada do valor:
`suffix` (ex.: `%`), `step`, `multiplier`, `display_units` ∈ `{real, integer, percent, degrees, decibels, frames_per_second, milliseconds, seconds, beats, fractions}`,
`control_type` ∈ `{based_on_param, choice_buttons, choice_combobox, spinner, duration_spinner, slider, slider_color_red…alpha…opacity, color_pallette, color_picker, rotary, text, text_multiline}`.

Três detalhes que valem mais que o resto:

1. **`in` / `out` no `ParamRange`** — "The lowest/highest value we clamped the range to". O range útil é separado do range físico. É isso que faz o atalho `A` do teclado mapear volume em `0.0630957..1` sem alterar o parâmetro.
2. **`ParamEvent` é um tipo, não um `bool` que volta a zero.** Trigger e toggle são coisas diferentes desde o schema: `CrossFader.sidea`/`sideb` e os quatro do `TempoController` são `ParamEvent`.
3. **`ParameterCollection`** é um mapa `nome → parâmetro de qualquer tipo` (swagger:5431). `dashboard`, `mixer`, `params`, `effect` e `sourceparams` usam isso. Não há schema fechado de efeito: o efeito descreve os próprios parâmetros.

### XML de arquivo (`.avc`, fixtures, presets)

Nomes diferentes para o mesmo modelo: `<Param T="STRING|BOOL|UINT32|INT32|UINT8|COLOR|DOUBLE">`, `<ParamRange>`, `<ParamChoice storeChoices>`,
`<ParamColor channelmode paletteEnabled color interpolated>`, `<ParamText>`, `<ParamPixels>`, `<ParamFixturePixels>`.
Um `ParamRange` sempre carrega três `<ValueRange>`: `defaultRange`, `minMax`, `startStop`.

E carrega **a fonte de animação como filho**, que é o mecanismo mais interessante do arquivo:

| Nó | O que é | Ocorrências |
|---|---|---|
| `PhaseSourceStatic phase=` | valor parado | 361 / 191 / 755 (Example / aniver / climao) |
| `PhaseSourceTimeline` | segue a timeline do clip | 3 / 15 / 31 |
| `PhaseSourceTransportTimeline` | segue o transport, com `defaultBeatsDuration` e `defaultMillisecondsDuration` | 47 / 17 / 63 |
| `PhaseSourceDashboardLink linkId="/link1" linkName="RGB"` | segue um dial do Dashboard | 8 / 0 / 0 |
| `Modifier → ModifierEnvelope → points → point x y curve` | envelope desenhado | 2 em `Example.avc` |

Ou seja: **todo parâmetro numérico pode virar animado trocando o filho**, sem mudar o tipo do parâmetro.
Os envelopes são presetáveis: `default\Presets\Envelopes\ADSR.xml` é `<Preset uniqueId="MOD_ENVELOPE" className="Envelope">` com 5 `point x/y/curve`
(`curve` inteiro: 1, 12, 33 vistos). Ajuda `Envelope`: "Double click to add and remove keyframes, right click each keyframe to change its interpolation."

## Arquivo (.avc)

Um `.avc` é XML puro, raiz `<Composition>`, sem compressão.

```
Composition (name uniqueId numDecks currentDeckIndex numLayers numColumns compositionIsRelative)
 ├ versionInfo
 ├ CompositionInfo (name description width height)
 ├ Params  : Name · Speed · Beat Snap · KeyboardShortcutPreset · MidiShortcutPreset · OscShortcutPreset · DmxShortcutPreset
 ├ Params name="Dashboard" : Link 1..Link 8
 ├ TempoController → Params → ParamRange Tempo
 ├ CrossFader · ClipTransition · CompositionView(FoldParams/FoldState) · Notes(Note)
 ├ VideoTrack / AudioTrack da composicao (RenderPass encadeados)
 ├ DeckInfo × N  (name id closed)
 ├ Deck (uniqueId closed numLayersWithContent numColumnsWithContent numLayers numColumns deckIndex)
 │   └ Clip (uniqueId layerIndex columnIndex)
 │        ├ Params : Name · LoadProgress · TransportType
 │        ├ PreloadData → VideoFile / AudioFile
 │        ├ Transport → Params → ParamRange Position → DurationSource
 │        │                                          → PhaseSourceTransportTimeline → Beats_d / Beats_double
 │        ├ ClipView → FoldParams → FoldState
 │        └ VideoTrack (manualDuration) : Width Height RScale GScale BScale AScale
 │             ├ RenderPass(type=RenderPassChain) → RenderPass(type=TransformEffect|Alpha|…)
 │             ├ ChoosableMixer name="Blend" → ParamChoice "Blend Mode"
 │             └ PrimarySource → VideoSource(type width height) → VideoFormatReaderSource | GeneratorVideoSource
 ├ Column × N (uniqueId columnIndex) [+ ColumnAttributes index name — so no Example.avc]
 └ Layer × N (uniqueId layerIndex) : Params Name · ClipTransition · LayerView · AudioTrack · VideoTrack
```

### Os 40 elementos mais frequentes (`Example.avc`, 2.758 elementos)

`ParamRange` 418 · `PhaseSourceStatic` 361 · `Params` 275 · `Param` 274 · `RenderPass` 202 · `FoldState` 164 · `ParamChoice` 67 ·
`ChoosableMixer` 66 · `ValueRange` 65 · `PrimarySource` 65 · `View` 60 · `VideoTrack` 52 · `FoldParams` 51 · `DurationSource` 50 ·
`Beats_d` 50 · `Clip` 47 · `Transport` 47 · `PhaseSourceTransportTimeline` 47 · `ClipView` 47 · `VideoSource` 47 · `PreloadData` 45 ·
`VideoFile` 33 · `VideoFormatReaderSource` 33 · `AudioTrack` 23 · `AudioEffectChain` 23 · `ParamColor` 19 · `AudioFile` 18 ·
`AudioFileSource` 18 · `EmbeddedThumbnail` 12 · `Column` 9 · `PhaseSourceDashboardLink` 8 · `ColumnAttributes` 7 · `point` 6 ·
`DeckInfo` 4 · `Deck` 4 · `Choice` 4 · `Layer` 3 · `ClipTransition` 3 · `LayerView` 3 · `PhaseSourceTimeline` 3.
Exemplo de atributos de `RenderPass`: `{name: RenderPassChain, type: RenderPassChain, uniqueTypeId: RenderPassChain, uniqueId: 1621406654259, baseType: RenderPassChain}`.
Exemplo de `Clip`: `{name: Clip, uniqueId: 1621342719991, layerIndex: 0, columnIndex: 0}`.

### O que muda entre `Example.avc` e as composições do usuário

| | Example.avc | aniver 30.avc | climao.avc |
|---|---|---|---|
| decks / layers / colunas | 4 / 3 / 9 | 3 / 7 / 10 | 3 / 7 / 14 |
| elementos `<Clip>` | 47 | 70 | 98 |
| clips com conteúdo | 47 | 17 | 63 |
| `compositionIsRelative` | `1` | `0` | `0` |
| fontes de vídeo | `VideoFormatReaderSource` 33, `GeneratorVideoSource` 13, `CompositionRouterVideoSource` 1 | `GeneratorVideoSource` 17 | `GeneratorVideoSource` 63 |
| efeitos (`RenderPass type`) | 22 tipos distintos, com `TextGenerator`, `WireGenerator`, `Metaballs` | 6 tipos: Transform, SolidColor, Alpha, Add, Lines | 10 tipos, com `StroboscopeGenerator` 8× e `Spiral` 8× |
| nós exclusivos | `AudioFile`, `AudioFileSource`, `Beats_d`, `Choice`, `ColorPalette`, `ColumnAttributes`, `Modifier`, `ModifierEnvelope`, `ParamText`, `PhaseSourceDashboardLink`, `SliceInputs`, `VideoFile`, `VideoFormatReaderSource`, `WireRenderPass`, `point`, `points` | — | — |
| nós que só os dois do usuário têm | — | `Beats_double`, `Notes`, `Note` | idem |

Quatro fatos que importam para o formato `.spell`:

1. **Só o que difere do default é serializado.** Nenhum dos três arquivos guarda `Target`, `Trigger Style`, `Beat Snap` ou `Fader Start` no clip, embora o swagger declare os quatro. Arquivo pequeno, mas impossível de auditar sem o binário que conhece os defaults.
2. **Grid inteiro no disco.** `aniver 30.avc` tem 70 `<Clip>` = 7 layers × 10 colunas do primeiro deck: as células vazias são serializadas com `Transport`, `VideoTrack`, `RenderPass` e tudo. `climao.avc`: 98 = 7 × 14. O `Example.avc` grava só clips com conteúdo. Comportamento diferente entre versões, no mesmo formato.
3. **Deriva de nome de nó entre versões.** `Beats_d` (Example, 2021) virou `Beats_double` (arquivos de 2026); `Color Space` virou `Color Format` na fixture. Nomes de elemento fazem parte do contrato e mudaram sem alias.
4. **Thumbnail embutida.** 12 `<EmbeddedThumbnail encoding="LZF" width=160 height=120 bitdepth=32 data="base64…">` em `Example.avc`, 11 de 320×240 em `aniver 30.avc`. É de onde vem 1,2 MB de `climao.avc` para 98 clips que são quase todos geradores de cor sólida.

## O que o Spellcaster deve copiar

- **`connect` como pressão de botão, não como evento.** O corpo de `POST /composition/layers/{l}/clips/{c}/connect` é um booleano: "analogous to whether the mouse is pressed down on the clip. If omitted, true and false are both send" (swagger:3946). Isso dá momentâneo e latch com uma primitiva só. Serve direto ao **cenas e cues DMX** (um cue que segura enquanto a tecla estiver pressionada) e ao **ILDA player** (shutter momentâneo).
- **Estado de clip como enumeração de 5 valores, não booleano** (`Clip.connected` é `ChoiceParameter`, swagger:5633; ajuda `MIDI Out Status`: "clip triggers can have 5 different states"). O `design/PRINCIPIOS.md §2` diz que cor significa estado — então o estado precisa ter mais de dois valores para a cor ter o que dizer. Serve ao **cenário interativo** (aparelho vazio / carregado / armado / ao vivo / erro) e ao **Aprendiz**, que só consegue avisar "o galvo não acompanha" se o estado tiver esse degrau.
- **`in`/`out` separados de `min`/`max` no `ParamRange`** (swagger:5369). Um parâmetro tem faixa física e faixa útil, e o mapeamento remapeia sem tocar no parâmetro (`<ValueRange min="0.0630957" max="1"/>` no atalho `A` do teclado). Serve ao **orquestrador**: um mapeamento sACN→parâmetro precisa disso para não exigir um nó de escala em cada rota.
- **Caminho textual único como identidade de tudo** (`/composition/selectedlayer/clips/3/connect`). O mesmo caminho é chave de atalho de teclado, endereço OSC, alvo MIDI e rota REST — quatro protocolos, um espaço de nomes (`Shortcuts\Keyboard\Default.xml`, `Shortcuts\OSC\OSC_TD_TEST.xml`, `swagger.yaml`). É literalmente o `spellcaster.core.registry` do `CLAUDE.md`, e o que faz o `design/PRINCIPIOS.md §4` ("quem aprende a GUI já sabe operar por SSH") funcionar. Serve ao **orquestrador** e ao **Aprendiz** como paleta de comandos.
- **Escopo do alvo declarado no mapeamento, com três modos** — *Selected / This / By Position* (ajuda `Shortcut Target`; campo `translationType`). É o que permite um controlador de 8 faders operar 40 layers. Serve às **cenas e cues DMX** (um encoder que sempre pega a fixture selecionada) e ao **orquestrador**.
- **Caminho de retorno no mesmo objeto de mapeamento** (`OutputPath`, `InputSiblingPath`, `OutputSiblingPath`, `NamedValues`). Feedback de LED não é feature à parte: é campo do mapeamento. Serve ao **orquestrador** (mesa MIDI com LED correto) e ao **cenário interativo** (o recorte na maquete acende com o estado real, não com o comando enviado).
- **Fonte de animação como filho do parâmetro** (`PhaseSourceStatic` / `Timeline` / `TransportTimeline` / `DashboardLink`, e `ModifierEnvelope` com `point x y curve`). Trocar o filho transforma valor fixo em valor animado sem mudar tipo nem endereço. Serve à **timeline/cues DMX** e ao **ILDA player** (kpps ou tamanho seguindo a timeline com a mesma sintaxe).
- **Delay e framerate por saída, não globais** (`OutputDeviceDmx`: `Framerate` 1-40, `Delay` 0-150 ms, em `Presets\Advanced Output\amiver 30.xml`). Hardware real chega fora de fase. Serve ao **NDI→ILDA** (compensar latência do NDI contra o DMX) e às **cenas DMX**.
- **Painel de saída que mostra sobreposição de canal** (ajuda `DMX Output`: "visually re-arrange the start channels of your fixtures or check for any overlap"). Serve às **cenas e cues DMX**: é o único jeito de o operador ver que patcheou duas fixtures em cima da outra antes do show.
- **Ajuda contextual como arquivo de dados, uma frase por elemento** (`docs\help\English.xml`, 412 verbetes `components → title + text`). Isso é o corpo do **Aprendiz** pronto: o companion não precisa de modelo, precisa de uma tabela `elemento → uma frase`, carregada por Face, e de um lugar para pendurar os avisos de estado.

## O que NÃO copiar

- **Personality sem tipo de canal.** As 25 fixtures lidas são listas de `ParamRange 0..255` com nome livre; não existe `pan`, `tilt`, `strobe`, `16-bit`, nem range com rótulo. O resultado está nos arquivos do usuário: `teste_parametros.xml` com 34 canais chamados `New Parameter 1..34`, e `srg - strobo fita 11chrgb.xml` com dois canais anônimos no meio. Sem tipo de canal não há fade correto (dimmer interpola, color wheel não), não há 16 bits para pan/tilt, e o Aprendiz não tem o que checar. O editor do Spellcaster precisa de tipo por canal desde o primeiro commit.
- **Personality copiada para dentro do patch, e com o nome perdido.** `FixtureInstance/Fixture` em `amiver 30.xml` tem `fixtureName=""` e `uuid` novo, diferente do da biblioteca. Corrigir a personality não conserta o show, e o show não sabe o nome do que carrega. Referência por id estável + versão, não cópia anônima.
- **Sem armar saída e sem modo ensaio.** Nenhum campo em swagger, `recentLayout.xml` ou ajuda corresponde a isso; existir Lumiverse já é enviar. O `design/SHORTCUTS.md` já reserva `Ctrl+Shift+Enter` / `Ctrl+Shift+R` — manter, é o que separa uma ferramenta de VJ de uma ferramenta de show.
- **Sem GO, sem cue, sem blackout no teclado.** Os 42 atalhos de `Shortcuts\Keyboard\Default.xml` são todos de trigger de clip, coluna, deck e BPM. Não há um único atalho de sequência. Os "Cue Points" da ajuda são seis marcadores **dentro de um clip de vídeo**, não cues de show. Cue de show é a função 4 do `design/TEMAS.md` e não tem equivalente aqui — não há o que copiar, só o que evitar assumir.
- **Cor de estado misturada com cor de organização.** Todo objeto tem `colorid` (`ChoiceParameter` em Clip, Column, Layer, LayerGroup, Deck — swagger) *e* estado com cor (`connected`, `bypassed`, `solo`). Duas semânticas de cor na mesma célula. O `design/PRINCIPIOS.md §2` proíbe: um accent, e ele significa estado. Rótulo colorido de organização precisa de outro canal (borda, texto, ícone), nunca do preenchimento.
- **33 booleanos `Show X` no lugar de Faces.** `recentLayout.xml/Params[@name='LayoutParams']`. Recuperar uma tela é lembrar de 33 caixas e nove presets nomeados (`Big Monitor`, `Many Monitors Left`, `Panels On Top`...). Face nomeada com conteúdo declarado (`design/PRINCIPIOS.md §5`), não checkbox por widget.
- **Formato que só grava o que difere do default.** Os três `.avc` omitem `Target`, `Trigger Style`, `Beat Snap` e `Fader Start` dos clips embora o schema REST os declare. Um `.spell` que só grava o diff é indiferenciável de um `.spell` corrompido, e diferenças de default entre versões viram mudança silenciosa de show.
- **Thumbnail embutida no arquivo de show.** `EmbeddedThumbnail` LZF base64 (12× em `Example.avc`, 11× de 320×240 em `aniver 30.avc`) é o que faz `climao.avc` ter 1,2 MB para 98 clips de cor sólida. Cache ao lado, com hash — nunca dentro do `.spell`.
- **Preset referenciado por nome.** `climao.avc` pede `OscShortcutPreset="OutputAllMessages"`, que não existe em `Shortcuts\OSC\`. Referência por id + fallback declarado, e o Aprendiz avisa na abertura.
- **Nome de nó como contrato, sem alias.** `Beats_d` → `Beats_double`, `Color Space` → `Color Format`: renomeações silenciosas entre versões dentro da mesma extensão de arquivo. Se o `.spell` é JSON, versionar o schema e manter leitor dos nomes antigos.
- **DMX como amostragem de vídeo, e só isso.** No Arena um fixture recebe o retângulo de pixels que cai em cima dele (`DmxSlice/InputRect` em coordenadas da composição). Funciona lindamente para pixel mapping e é inútil para um canal de gobo, de macro ou de velocidade de pan. O Spellcaster precisa dos dois caminhos: valor por canal (cena) *e* amostragem de superfície (pixel map) — e o modelo de dados tem que aceitar os dois na mesma fixture.
