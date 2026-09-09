# Capture 2024 — auditoria de interface (fonte para o Spellcaster)

Auditoria funcional, não estética. Tudo aqui vem de duas fontes citadas em cada afirmação:
o arquivo de idioma instalado e o manual online da versão 2024. Nada de memória sobre o produto.

## Fontes lidas

- `C:\Program Files\Capture 2024\Languages\English.c2l` — 1885 linhas, 137 seções, lido
  inteiro. Formato `Section <Nome>` + `Phrase <Chave> <Texto>`; citado como `Seção/Chave`.
  Ao lado, `Portuguese.c2l` (mesmas chaves, `ISO pt / Version 328`), fonte do vocabulário.
- `C:\Program Files\Capture 2024\Installation\Presentation.zip` — só listado (18 entradas), não extraído.
- Manual online 2024, base `https://www.capture.se/Manual/en-UK/2024/`, páginas lidas por
  WebFetch: `introduction.html` (índice completo), `DesignViews.html` (navegação, seleção,
  manipulação, control pane, focus mode, measure mode), `UniversesTab.html`,
  `FixturesTab.html`, `DesignTab.html`, `MediaTab.html`, `SnapshotsTab.html`,
  `ToolsMenu.html`, `NavigateMenu.html`, `WindowMenu.html`, `FileMenu.html`,
  `Appendix.html` (protocolos e tabelas DMX).
- Não instalado, não aberto: o aplicativo. Nenhuma afirmação aqui vem de uso.
- Existe manual em pt-BR, mas só da versão 2022 (`/Manual/pt-BR/2022/Introduction.html`) — não foi lido.

## Anatomia da tela

**Barra de menus** (`Menus`): File, Edit, View, Navigate, Tools, Window, Arrangements, Help.
Oito menus, ordem fixa. `Navigate` não abre janela: leva o foco para um pane ou aba
(manual NavigateMenu.html — "tool panes and organizational windows").

**Abas de nível superior**, cada uma com sua seção de strings (`*Tab`): **Design** (o
projeto: objetos, camadas, vistas, cenas, relatórios), **Fixtures** (planilha de todos os
aparelhos), **Universes** (universos DMX, conectividade, link com console), **Media**
(capturas de vídeo, players, streaming), **Snapshots** (gravações de DMX/vídeo/laser/câmera)
e **Library** (biblioteca instalável de aparelhos, treliça, materiais, gobos).

**Tool panes** da aba Design, cada um com seção própria e sempre com o mesmo trio
(lista + Add/Delete/Rename + propriedades): Project, Selected Items, Views, Layers,
Filters, Fixture Groups, Camera Positions, Scenes, Materials, Gobos, Frame Lists, Symbols,
Reports, Plot Styles, Plots, Universes (`*ToolPane`). Dezesseis categorias em uma coluna —
nenhuma string de busca em nenhuma dessas seções.

**Vistas de simulação**: exatamente três, de nome fixo — Alpha, Beta, Gamma
(`SimulatorView/AlphaName`, `BetaName`, `GammaName`). Cada vista tem um `Type`
(`TypeWireframe`, `TypePlot`, `TypeLive`, `TypeCustom`) e, no Custom, projeção ortográfica
ou perspectiva, face wireframe ou sólida, e "model space" Screen/Plot/Live. Layout da
janela: só `Actions/Quad` e `Actions/Wide`. O resto vai para janelas flutuantes
(`ReportWindow`, `PlotWindow`, `FrameListWindow`, `ConsolePatchWindow`, `OptionsWindow`).

**Widgets dentro da vista 3D** (`SimulatorView`): `Widgets`, `HiddenObjects`,
`DimBackground`, `ProjectInformation`, `FixtureInformation`, `SelectionNavigator`,
`ViewNavigator`. O Selection Navigator é um painel de comandos que aparece junto da seleção
— e `OptionsWindow/NavigatorOnExternalSelection` mostra que ele aparece também quando a
seleção vem de fora, do console.

## Objetos e verbos

Organizado por objeto. Verbos são Phrases de `SimulatorView` (menu de contexto da cena),
`Actions` (menu File/Tools) e dos tool panes.

**Fixture (aparelho)** — `Fixture`, `LightingFixture`, `MotionFixture`, `EffectFixture`.
Verbos: `PatchAction`, `UnpatchAction`, `ChannelAction`, `CircuitAction`, `UnitAction`,
`FocusAction`, `RemoveFiltersAction`, `RemoveGobosAction`, `ReplaceAction`,
`DuplicateAction`, `Common/Assign` (arrastar gobo/gel na seleção) e `Sequential` —
numeração em lote de Unit, Circuit, Patch e Channel (EditMenu.html#sequential).

**Universe** — `DMXUniverse`, `UniversesToolPane`, `AddUniversesDialog`. Verbos:
`Common/Add`, `AddMultiple`, `ResetExternalUniverse`, `ResetLevels`,
`ConnectivityOptions..`, `ConfigureMANet2..`, `ConnectivityStatus..`.

**Layer (camada)** — `Layer`, `LayersToolPane`. Verbos: `SelectUsedBySelectedObjects`,
`SelectUnused`, `SelectUsingObjects`, `DeselectUsingObjects`, `AddToAllFilters`,
`RemoveFromAllFilters`. A camada não é só visibilidade: tem `Locked`, `Unselectable`,
`IncludeInReports`, `FixtureInformation` e `FixtureSimulation` — dá para desligar a
**simulação** de uma camada inteira sem apagá-la.

**Filter** — `FiltersToolPane`: container de **camadas e universos** (`LayersAndUniverses`),
com `Include`, `IncludeByDefault`, `ApplyToAllViews`, `ClearAllViewFilters`; disparável por
DMX até 255 (DesignTab.html).

**View / Camera** — `SwingToTop`, `SwingToFront`, `SwingToSection`, `SwingToSelection`,
`FocusSelection`, `FocusAll`, `StoreCamera`, `Position {0}`, `Fullscreen..`, `SaveImage..`,
`RenderImage..`. Posições vivem em catálogos (`CameraPositionCatalog`), disparáveis por DMX.

**Scene** — `Goto`, `RecallSelectedObjects`, `StoreSelectedObjects`. Cena no Capture guarda
**posição e visibilidade de objetos** (`Object/IncludeInScenes`), não valores de canal
(DesignTab.html): é mudança de cenário entre partes do espetáculo.

**Snapshot** — conteúdos `CameraContents`, `DMXContents`, `VideoContents`, `LaserContents`;
verbos `RecordStill`, `RecordMovie..`, `Play`, `Stop`, `RecallDMX`, `RenderMovie..`.

**Fixture group** — numerados, criados a partir da seleção, com Update e Duplicate; não
guardam ordem de seleção (DesignTab.html#FixtureGroups).

**Documentação** — `Plot`, `PlotStyle`, `PlotSymbol`, `Report`, `ReportItem`,
`ExportFocusSheets`, `ExportDocumentation`. Relatórios prontos (`ReportsToolPane`):
Equipment, Rigging point, Cable, Fixture, Fixture groups, Fixture locations, Frame lists.

**Seleção** — `SelectAllActionGroup`, dez critérios: By Layer, By Location, By Model,
By Drawing Block Name, Motion Controlled, Connected Truss, Fixtures on Truss,
By Fixture Type, By Fixture Group, By Cable Type.

## Patch e universos

**Os quatro números de um aparelho são coisas diferentes**, e a tradução PT confirma:

| Campo | `.c2l` | PT oficial | O que é |
|---|---|---|---|
| Patch | `Fixture/Patch`, `PatchUniverse`, `PatchChannel` | Patch / Universo do patch / Canal do patch | endereço DMX: universo + canal inicial |
| Channel | `Fixture/Channel` | **Canal (ID)** | número do aparelho no console (identidade), não endereço |
| Unit | `Object/Unit` | **Etiqueta** | número físico da unidade na vara/palco |
| Circuit | `Fixture/Circuit` | Circuito | circuito elétrico |

Ainda em `Fixture`: `Purpose`, `Groups`, `Mode`, `ConsoleIdentifier`, `ExternalIdentifier`,
e `ControllerMode {0} mode` / `ControllerPatch {0} patch` — o mesmo aparelho carrega patch
e modo **por controlador**. `Unpatched` é estado nomeado, não vazio.

Geometria vem de `Object` (`Layer`, `Location`, `PositionX/Y/Z`, `RotationX/Y/Z`,
`Identifier`, `Note`, `Hidden`, `CastsShadows`, `IncludeInScenes`, `MotionFixture`) e o
comportamento mecânico de `LightingFixture`: `InvertPan`/`InvertTilt`/`InvertZoom`/
`InvertIris`/`InvertColorMix`, `LimitPanStart`/`LimitPanEnd`/`LimitTiltStart`/
`LimitTiltEnd`, `IntensityScale`, `SimulateFocus`, `Optics`, `Photometry`, `ThrowsLight`,
`InternalAccessory`, `ExternalAccessory`, `Filters`, `Gobos`. Inverter pan e limitar curso
são propriedades da **instância**, não do perfil — é assim que se corrige um aparelho
pendurado de cabeça para baixo.

**Diálogo de patch** (`Patch`): `StartAddress`, `FixtureOverlap` "Fixtures per channel:",
`ChannelOffset` "Channel offset of fixtures:", `ChannelsRequired` (calculado e mostrado
antes de confirmar) e `ContinueUntilComplete`. O transbordo é pergunta, não erro:
`OverflowWithContinue` — "All channels in universe '{0}' have been used. Do you wish to
continue patching in universe '{1}'? Stopping will leave one or more fixtures unpatched."

**Universo** (`DMXUniverse`): `Universe` (nome), `PatchBase` (posição numérica do universo,
independente do nome), `PatchStyle` `Indexed`/`Contiguous` (contíguo = faixa única 1–2048 no
teatro, UniversesTab.html), `ExternalUniverse` com `(auto)`, `(no)` e `Searching..`, e
`BlindLevelsMode` (o que fazer com DMX blind de sACN e CITP — alguns consoles mandam o
programador ou o preview de cue por ali). Criação em lote:
`AddUniversesDialog/NumberOfUniverses` + `FirstUniverseName`. A vista do universo
(`DMXUniverseView`) tem dois seletores: `Mode:` Channels ou Fixtures e `Levels:` `%` ou `DMX`.

**Entrada de dados** (Appendix.html#Protocols): Art-Net, CITP, Compulite VC, EntTec DMX USB
Pro Mk1/Mk2, ETC Eos por OSC ("bidirectional channel selection and pan/tilt feedback is
supported"), MA-Net2 (exige MA v2.9+), MA-Net3 (plugin no grandMA3), Streaming ACN. Outros:
Blacktrax RTTrP, CITP/CAEX (laser), CITP/MSEX (vídeo), Kinesys K2, LaserAnimation, NDI,
OSC, Pangolin Beyond, PosiStageNet. OSC: "1.0 e 1.1 sobre TCP e UDP, porta padrão 4004",
com `/ping`, `/getCatalogs`, posições de câmera e controle de ambient lighting, exposure,
bloom e white balance. Rede em `Communication`: `Interface`, `IPAddress`,
`ServerPortNumber`, `MulticastIPAddress`/`MulticastPortNumber`, `EnableDiscovery`,
`FirstUniverse`/`LastUniverse`, `CITPMulticastAddress` Legacy/Standard, `CITPVideoFormat`,
`PangolinNumberOfProjectors`.

**Link com console** (`UniverseTab`): `ProjectConsoleLink` com `(Automatic)` e `(Disabled)`;
só um console por vez. `ProjectConsoleLinkViewPatch` "View Fixture Patch.." mostra o que
está patcheado no console e **falta** no projeto; `ConsolePatchWindow` lista
Fixture / Library fixture / Position com `Identify..` e `ImportAtPosition`.
Três capacidades além de DMX (UniversesTab.html):
- **DMX talkback** (antes "autofocus"): clicar no 3D manda pan/tilt de volta ao console — CITP/SDMX, EOS por OSC, Hog 4.
- **Fixture selection**: seleção sincronizada nos dois sentidos — CITP/FSEL, CITP/CAEX, EOS/OSC, Hog 4 (só recebe).
- **Fixture patch**: troca de patch bidirecional — CITP/FPTC, CITP/CAEX.

**Media player como aparelho patcheado** (`MediaPlayer`, MediaTab.html): `OutputResolution`,
`ILDAFrameRate`, e modo `Mode_Full` "Full (256 Playlist Entries)" / `Mode_Legacy`
"Legacy (8 Playlist Entries)". Pelo Appendix, o player ocupa 2 canais DMX (controle
play/pause/stop/replay + seleção de mídia); câmeras ocupam 12 canais no modo Standard
(catálogo, posição, tempo, amortecimento, curvatura, luz ambiente, exposição); caixas de
fumaça, 4; DMX movers e rotators, 8 ou 16 bits.

## Cena 3D e interação

**Navegação** (DesignViews.html#navigation): botão do meio **ou** Alt+clique esquerdo em
qualquer lugar da vista; Shift alterna entre rotação e panorâmica; Ctrl gira sem mover a
câmera. Zoom em botões abaixo do cubo: Shift+zoom move o ponto focal junto, Ctrl+zoom muda
o campo de visão em vez da posição. Modos em `Navigator`: `OrbitNavigation`,
`FreeFlightNavigation`, `LookAroundNavigation`, mais `NoSnapping`/`Snapping`, `Orthogonal`,
`Quality`, e um cubo de orientação de seis faces. Preferências em `OptionsWindow`:
`ZoomToCursor`, `InvertZoom`, `SlidingEdges`, `3DMouseNavigation` Camera/Object,
`RotationSnapAngle`, `LiveUpdateTransformations`.

**Seleção** (DesignViews.html#selection): clique seleciona; Shift+clique soma;
Ctrl+clique alterna item a item. Retângulo da esquerda para a direita pega só o que está
inteiro dentro; da direita para a esquerda pega também o que encosta. Clicar em objeto
agrupado seleciona o grupo inteiro; duplo-clique desce um nível — inclusive em grupos
implícitos, como um aparelho com acessórios.

**Manipulação**: arrastar dentro do contorno vermelho move; Shift trava em ortogonal; os
objetos encaixam no contorno de outros e Ctrl desliga o encaixe; os cantos escalam
(redistribuem). Um triângulo vermelho é a alça de rotação: região interna gira o conjunto
como bloco, região externa gira cada objeto no próprio eixo; Shift trava em 5°.

**Control pane** (DesignViews.html#ControlPane) — só existe no modo Live. Cada tipo de
aparelho selecionado vira uma coluna. Botões rápidos: Light (liga/desliga a seleção), Home
(volta ao padrão, com pan/tilt opcional), slider de luz ambiente e de transparência do
fundo. Slider com Shift dá ajuste fino; zoom, DMX mover e rotator aceitam número digitado;
o fanning de pan/tilt escala com Ctrl e vira offset com Alt. Parâmetros (`Control`):
Dimmer, Shutter, Intensity, Pan/Tilt, Framing Shutters, Iris, Gobo Rotation, Filament
Angle, Zoom, Focus, Frost, Colour, CTO, CTB, Keystone, Shift, Scale, Media Segment, Motion,
Rotation, Pump, Valve, Home, Blinder.

**Focus mode** — este é o "cenário interativo" de verdade: em modo de foco, clicar num
objeto **não** o seleciona, aponta os aparelhos selecionados para aquele ponto
(DesignViews.html#FocusMode). Sai pelo botão do selection navigator. O plano de foco
(`FocusPlane`, `AllFocusPlanes`) alterna entre invisível, grade (alinhar), sólido
(tirar distração) e heatmap (`HeatmapMin`/`HeatmapMax`, avaliar nível de iluminação).

**Measure mode**: clique inicia, clique termina; Shift+clique adiciona pontos; o cursor
mostra X, Y e Z. Escape limpa a medida; Escape sem medida sai do modo.

**Atmosfera e realismo**: `Smoke` (`Density`, `Variation`, `EdgeSoftness`, `AutoSize`),
`AllSmoke/Speed` (a velocidade de toda a fumaça de uma vez), `HDRI`, `ReflectionPlane`;
na câmera, `WhiteBalance`, `HueClamp`, `AmbientLighting`, `FillLighting`, `BloomEffect`,
`AutomaticExposure` e `LaserFlickerEffect` — a cintilação do laser é parâmetro de câmera,
não decoração.

**Custo da cena** é visível: `VisualisationSettingsDialog` escolhe entre `Framerate` e
`Detail`, com `AdaptiveQuality` "(recommended)", `ResolutionLimit` e
`ShowPerformanceInformation`.

## Estados e mensagens

**Estados nomeados**, cada um com string própria (não são cores sem legenda):
`Fixture/Unpatched`; `DMXUniverse/ExternalUniverse` com `(auto)` / `(no)` / `Searching..`;
`Layer/Current` (a camada onde objetos novos nascem); `Property/MixedValues` para seleção
múltipla divergente; `Property/WaitingFor` "(Waiting for '{0}'..)"; `MediaTab/Requesting` e
`Receiving` — a mídia diz o que pediu e o que está recebendo; e `MainWindow/Locked`,
`Expired`, `Demo`, `VideoCardIssues` no título da janela.

**Conectividade** (`ConnectivityStatusDialog`): `InitFailed` "Initialisation failed.",
`PotentiallyBlocked` "Potentially blocked by firewall.", `NetworkInitFailed`, e
`OpenLogFolder` "Open Log Folder..". O manual diz que verde significa **atividade**, e que
atividade não garante funcionamento — a distinção está escrita, não implícita.

**Divergência projeto × console** (`UniverseTab`), as duas mensagens mais úteis do arquivo:
- `ProjectConsoleLinkUniverseMismatch`: "One or more project fixtures are patched to universes not in control by the console..."
- `ProjectConsoleLinkFixtureMismatch`: "A difference in patch or type of fixture between console and project fixtures has been detected. The link between these fixtures will be broken."

**Avisos de patch e exportação**: `Hog4DuplicateChannels` ("Some fixtures use the same
channel numbers and may not import correctly in the Hog 4." — endereço duplicado detectado
na exportação), `Hog4MissingChannels`, `ExportFocusSheets/MissingUnits` ("One or more
fixtures did not have a unit set and were not exported!"),
`MainWindow/FixtureIdInconsistenciesFoundAndFixed` e `UnresolvedFixturesGroupsMayHaveChanged`.

**Limites da cena** (`Navigator`), avisos de teto: `TooManySmokeObjects`, `TooManyHDRIs`,
`TooManyReflectionPlanes`, `TooManyLiveFilters`.

**Arquivo** (`MainWindow`): `FileSizeError`, `ChecksumError` ("The file data is incorrect,
it is not as originally written."), `FormatTooNewError`,
`FormatUnsupportedWithoutLibraryError` e `FileErrorUseBackup` — que oferece o backup
automático em vez de só falhar. `SaveProjectLimitations`: "The project contains features
that cannot be saved and will be left out!" No startup (`Application`) são sete falhas
separadas e nomeadas: configuração, licenciamento, rede, framework de vídeo, conectividade
externa, recursos e processamento em tempo real — errar cedo com nome próprio.

## Atalhos

O `.c2l` **não tem seção de atalhos** e nenhuma Phrase contém "Ctrl", "Shift" ou "F1":
atalho não é string traduzível. O que existe é o editor deles, `OptionsWindow/KeyBindings`,
com colunas `Command` e `Binding` e botões `Clear` e `Reset`. A gramática permitida
(ToolsMenu.html#OptionsKeyBindingsTab) no Windows é `Ctrl` + A-Z, 0-9, vírgula, ponto,
hífen ou mais, com Shift e/ou Alt opcionais (no macOS o mesmo com Cmd, e Ctrl vira
modificador extra); conflito entre bindings customizados avisa, conflito com o SO não.
Ou seja: **nenhuma tecla nua** (letra, Espaço, J/K/L, setas) pode ser atalho de comando —
e o manual não publica a lista de bindings padrão em nenhuma das páginas lidas.

**Modificadores na vista 3D** (DesignViews.html) — estes sim, documentados:

| Ação | Entrada |
|---|---|
| Orbitar / panoramizar | botão do meio, ou Alt + arrastar |
| Trocar orbitar ↔ panoramizar | Shift (segurando) |
| Girar sem mover a câmera | Ctrl / Cmd |
| Zoom movendo o ponto focal | Shift + zoom |
| Zoom mudando o campo de visão | Ctrl / Cmd + zoom |
| Somar à seleção / alternar item | Shift + clique / Ctrl + clique |
| Caixa: só o que está inteiro dentro | arrastar da esquerda para a direita |
| Caixa: também o que encosta | arrastar da direita para a esquerda |
| Descer um nível no grupo | duplo-clique |
| Mover só em ortogonal, rotação de 5°, ajuste fino de slider | Shift |
| Desligar encaixe ao mover | Ctrl / Cmd |
| Fan de pan/tilt (escala) | Ctrl / Cmd + arrastar |
| Fan de pan/tilt (offset) | Alt + arrastar |
| Ponto extra na medição / limpar e sair | Shift + clique / Escape |

Na aba Fixtures a tabela "can be navigated and edited as a spreadsheet using the arrow and
Enter/Return keys" (FixturesTab.html), com busca no canto superior direito e ordenação por
clique no cabeçalho da coluna (seta indica a direção).

## Vocabulário EN → PT

Da tradução oficial (`Portuguese.c2l`). A terceira coluna é a decisão para o Spellcaster.

| EN | PT (Capture) | Spellcaster |
|---|---|---|
| Fixture | Aparelho | **Aparelho** (não "fixture", não "luminária") |
| Patch (subst. e verbo) | Patch / "Fazer patch" | **Patch** — o termo é do ofício |
| Unpatch / Unpatched | Retirar do patch / Sem patch | idem |
| Universe / Patch universe / Patch channel | Universo / Universo do patch / Canal do patch | Universo / Canal |
| Patch base | Base do patch | Base do patch |
| Patch style: Indexed / Contiguous | Indexado / Contínuo | Indexado / Contínuo |
| Start address | Endereço Inicial | Endereço inicial |
| Channels required | Canais requeridos | Canais necessários |
| Channel (ID do aparelho) | **Canal (ID)** | Canal (ID) — a desambiguação é boa, copiar |
| Ch / Chs (canal DMX) | Canal / Canais | Canal DMX |
| Unit | **Etiqueta** | **Unidade** — "Etiqueta" perde o sentido de numeração |
| Dimmer / Intensity | Dimmer / Intensidade | idem |
| Shutter | Cortina (Shutter) | Shutter |
| Blinder | Máscara (Blinder) | Blinder |
| Framing shutters | **Facas** | Facas |
| Zoom | **Zum** | **Zoom** — "Zum" não é usado por ninguém |
| Focus | Foco / Afinação (`PlotFocus`) | Foco (parâmetro), **Afinação** (o ato) |
| Frost | Difuso (Frost) | Frost |
| Gobo / Gobo rotation | Gobo / Rotação do Gobo | Gobo / Rotação do gobo |
| CTO / CTB | CTO (correção para Âmbar) / CTB (para Azul) | CTO / CTB |
| Filter (de camadas) | **Filtragem por camadas** | **Vista filtrada** — "Filtro" fica só para gelatina |
| Filter (gelatina) | Filtro | Gel |
| Fixture group | Agrupamento / Grupo de Aparelhos | Grupo |
| Wireframe / Plot / Live | Aramada / Planta / Ao vivo | Aramada / Planta / **Ao vivo** |
| Camera position / Store camera | Posição de Câmeras / Gravar visão desta Câmera | Posição de câmera / Guardar câmera |
| Scene | Cena | Cena (mas ver "não copiar", item 5) |
| Snapshot | **Instantâneo** | Instantâneo |
| Truss / Rigging point | Estrutura / Ponto de ancoragem | Treliça / Ponto de ancoragem |
| Smoke | **Caixa de fumaça** | Fumaça |
| Media player / ILDA frame rate | Reprodutor de Mídia / Taxa de Quadros ILDA | Player / Taxa de quadros ILDA |
| Frame list / Frame | **Lista de Caixilho / Caixilho** | **Roda de cor / Posição** |
| Material | **Textura** | **Material** |
| Connectivity status / Console link | Status da Conexão / Link com console | Estado da conexão / Vínculo com console |
| Levels / Blind levels | Intensidades / Níveis para o blind | Níveis / Níveis em blind |
| Plot / Report | Planta (Plotagem) / Relatório | Planta / Relatório |

## Arquivo

- Tipos declarados em `FileTypes`: `CaptureProjectFiles` "Capture Project Files",
  `CaptureVersionProjectFiles` "Capture {0} Project Files", `CapturePresentationFiles`
  "Capture Presentation Files". **As extensões não aparecem no `.c2l` nem nas páginas lidas
  do manual** — não afirmo `.c2p` sem fonte.
- Importação de modelo: `.3ds`, `.dxf`, `.dwg`, `.gltf`/`.glb`, `.c4d`, `.mvr`, `.pdf`,
  `.skp`, `.obj` (FileMenu.html). Exportação de modelo: DWG/DXF, glTF, MVR.
- Dados de aparelho: CSV, TSV, Lightwright, XML do grandMA2, XML do Hog 4. A importação
  (`ImportDataDialog`) mapeia colunas do arquivo para `PositionX/Y/Z`, `RotationX/Y/Z`,
  `DefaultUnit`, `FocusPan`, `FocusTilt`, `ModeChannels`, identificando aparelhos por uma
  propriedade escolhida (`IdentifyFixtureBy`) ou por nome de bloco de desenho
  (`DrawingNameMatching`: Exact / Contains), e devolve um relatório com `FixturesUpdated`,
  `FixturesAdded`, `LinesSkipped`, `MatchingGroupedImportedObjects`.
- Símbolos de planta importam de SVG (`PlotSymbolsToolPane/ImportSymbol`).
- **`Installation\Presentation.zip`** (170.267.492 bytes, 18 entradas, só listado): contém
  `Presenter.exe` (115 MB) e `Presenter.app/` para macOS (com `libndi.dylib` e
  `Resources2.blob`). Nenhum projeto ou demo dentro. É o **payload do "Export
  Presentation"**: segundo FileMenu.html, a apresentação exportada é "a ZIP archive file
  containing" um executável Windows, um app macOS, um projeto **não modificável** e as
  configurações de conectividade; quem recebe extrai, roda, e o visualizador abre o projeto
  na vista Alpha, com um painel de snapshots no menu Window. O player standalone é o mesmo
  binário para todo mundo e o show é o projeto travado ao lado — exatamente a arquitetura
  do player standalone do Spellcaster.

## O que o Spellcaster deve copiar

- **Focus mode como modo, não como clique especial** (DesignViews.html#FocusMode): existe um
  modo em que clicar na maquete aponta os aparelhos já selecionados para o ponto clicado, em
  vez de selecionar o que foi clicado. Isso resolve a ambiguidade central do **cenário
  interativo** (função 4): o mesmo clique não pode significar "quero este aparelho" e
  "aponte para cá". Dois modos, um atalho, sem modal.
- **Talkback: a maquete é entrada, não só saída** (UniversesTab.html#dmx-talkback): clicar no
  3D devolve pan/tilt ao console por CITP/SDMX, OSC ou Hog. Para o **orquestrador** (função 3)
  isso define o nó de cena como bidirecional — a maquete é uma fonte de eventos no Graph,
  igual a um módulo OSC.
- **Três estados nomeados de universo** — `(auto)`, `(no)`, `Searching..`
  (`DMXUniverse/ExternalUniverse`) — mais `ResetLevels` e `ResetExternalUniverse`
  (`UniversesToolPane`). Para **cenas e cues DMX** (função 4) e para o painel Outputs:
  "sem sinal" e "procurando" são coisas diferentes, e o operador precisa ver qual das duas.
- **O diálogo de patch mostra `Channels required` antes de confirmar** e trata transbordo de
  universo como pergunta com "Continue until complete" (`Patch/ChannelsRequired`,
  `OverflowWithContinue`). `spell patch` deve dizer quantos canais vai consumir e para onde
  transborda antes de escrever — **cenas e cues DMX**.
- **Quatro números separados por aparelho, e numeração sequencial em lote para os quatro**:
  patch, canal (ID), unidade e circuito (`Fixture/Patch`, `Channel`, `Object/Unit`,
  `Fixture/Circuit`), com Sequential Unit / Circuit / Patch / Channel
  (`SimulatorView/Sequential`). Quem monta usa os quatro, e patchar 40 pares em uma operação
  é o que separa ferramenta de brinquedo — **cenas e cues DMX**.
- **Simulação desligável por camada** (`Layer/FixtureSimulation`, `Locked`, `Unselectable`):
  esconder é diferente de não simular e de não poder clicar. No **cenário interativo** isso é
  o botão de performance na hora do show, sem apagar nada da maquete.
- **Snapshot que grava DMX, vídeo, laser e câmera juntos, e na reprodução sobrepõe a entrada
  externa** (SnapshotsTab.html: "DMX or media from external sources has no effect on the
  visualisation"). É o modo ensaio do Spellcaster pronto: um gravador só para os quatro
  fluxos, e "Recall DMX" para congelar um estado quando não há sinal — serve ao **ILDA
  player** (função 1), ao **NDI→ILDA** (função 2) e ao ensaio de cues.
- **O player de mídia é um aparelho patcheado**: `ILDAFrameRate`, playlist de 256 entradas
  (`MediaPlayer/Mode_Full`) e 2 canais DMX (controle + seleção). Para o **ILDA player**: um
  `.ild` no palco deve responder a canal DMX como qualquer outro aparelho, e a taxa de quadros
  é propriedade do player, não do arquivo.
- **Connectivity Status que separa "há tráfego" de "funciona"** e nomeia a causa provável
  (`PotentiallyBlocked` "Potentially blocked by firewall.", `OpenLogFolder`): é o padrão de
  fala do **Aprendiz** (função 5) — a mensagem diz o que fazer e o log está a um clique.
  Junto com a **tabela de aparelhos navegável como planilha** (setas + Enter, ordenável por
  cabeçalho, com busca — FixturesTab.html), dá ao Aprendiz/menu e ao cenário interativo a
  superfície de texto que `PRINCIPIOS.md §4` exige.

## O que NÃO copiar

- **Três vistas fixas chamadas Alpha, Beta e Gamma** (`SimulatorView/AlphaName`..`GammaName`).
  Nome opaco que não diz nada sobre a função, e número fechado em três. Fere
  `PRINCIPIOS.md §4` (o nome na GUI ensina o vocabulário). No Spellcaster, vista tem nome de
  uso: `palco`, `plateia`, `varas`.
- **Layout só em duas disposições, Quad e Wide** (`Actions/Quad`, `Wide`), com o resto em
  janelas flutuantes (`ReportWindow`, `PlotWindow`, `FrameListWindow`, `ConsolePatchWindow`).
  Janela flutuante às 23h em sala escura é janela perdida atrás de outra. `PRINCIPIOS.md §3`
  pede grade fixa; `SHORTCUTS.md` já resolve com painéis focáveis por `Shift+1..7`.
- **Atalhos limitados a `Ctrl`+letra/dígito** (ToolsMenu.html#OptionsKeyBindingsTab). Essa
  gramática torna impossível o mapa do `SHORTCUTS.md`: Espaço, J/K/L, I/O, `M`, `S`, `R`,
  setas. Não herdar a restrição — herdar só a ideia do editor de bindings com Command/Binding
  e botão Reset.
- **Dezesseis tool panes em coluna, sem busca** (de `ProjectToolPane` a `UniversesToolPane`).
  `PRINCIPIOS.md §4` proíbe menu com mais de 8 itens sem busca. A paleta de comandos resolve
  isso; a lista rolável não.
- **"Scene" com o sentido do Capture**. Lá, cena guarda posição e visibilidade de objetos de
  cenário (`Object/IncludeInScenes`) e o manual diz que não há mecanismo de armazenamento e
  recall de valores. No Spellcaster cena é **valor por aparelho**. Usar a mesma palavra para
  as duas coisas quebraria o contrato do registry; o equivalente do Capture aqui é
  "posição de cenário", separado.
- **"Blind levels" como opção por universo** (`DMXUniverse/BlindLevelsMode`). É um remendo
  para consoles que mandam o programador junto com o palco. No Spellcaster, preview é modo do
  motor (ensaio × ao vivo, `Ctrl+Shift+R` / `Ctrl+Shift+Enter`), não uma caixinha escondida na
  propriedade de cada universo.
- **Avisos de teto só quando já estourou**: `TooManySmokeObjects`, `TooManyHDRIs`,
  `TooManyReflectionPlanes`, `TooManyLiveFilters` (`Navigator`). Chegam depois que a cena já
  engasgou. O Aprendiz deve avisar quando o custo sobe, não quando o limite estoura — e o
  limite deve estar visível antes, como o `ChannelsRequired` do patch está.
- **`DisableAdaptiveQualityWarning`** — "Are you really sure? ... please contact support so
  we can improve it.": diálogo que pede desculpa por um bug em vez de corrigi-lo. E **todo o
  aparato de licença** (`Licensing`, `Unlock`, `Lock`, `GetKeyFileDialog`, `UnlockName`,
  `Upgrade`: 65 strings; um sexto do arquivo de idioma é DRM). Fora do escopo.
- **Termos ruins da tradução PT**: "Zum" (Zoom), "Caixilho" para frame de roda de cor,
  "Textura" para Material, "Etiqueta" para Unit, "Filtragem por camadas" e "Filtro" para duas
  coisas diferentes na mesma tela. Ver a coluna Spellcaster do vocabulário: a tradução do
  Capture é boa fonte de vocabulário do ofício e má fonte de nomenclatura consistente.
