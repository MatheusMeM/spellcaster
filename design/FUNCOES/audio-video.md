# Áudio e vídeo — o áudio toca no engine, o vídeo toca na GUI

Pedido do Matheus (09/09/2026): *"quero drag n drop de elementos e saídas e mídias e laser e áudio e vídeo"*, e antes *"nem como visualizar como essas coisas se mexem no tempo"*. Hoje o Spellcaster não toca som nem imagem: `spellcore/Cargo.toml` tem nove membros e nenhuma dependência de áudio, e nenhuma página tem `<audio>` ou `<video>`.

Este arquivo diz **onde cada mídia toca** — e a resposta é diferente para as duas, por um motivo que não é preferência.

Fontes: **Ableton Live 12** §27 (Working with Video) e §7 (Session View), citado por seção. **DaVinci Resolve 20**, forma de onda na timeline, citado por página. **Resolume Arena**, *Syphon & Spout* e *Screens*, para o caso da segunda saída.

| Item | Quem resolveu melhor | Por quê |
|---|---|---|
| Onde o áudio toca | nenhum dos três (todos são o próprio host de áudio) | Nós somos um show-control com relógio próprio (`spellcore/engine/src/clock.rs`); a decisão é nossa e está em §3 |
| Quem manda no relógio | Ableton §27.2.3 | *"video clips are usually set as tempo leaders, while audio clips are left as tempo followers"* — declarar quem lidera, em vez de esperar que fiquem juntos |
| Janela de vídeo | Ableton §27.2.2 | Janela flutuante, sempre acima, duplo-clique vai a tela cheia num segundo monitor, `Ctrl+Alt+V` mostra/esconde |
| Forma de onda | Resolve p.643 | Três opções declaradas (retificada ou não, cheia ou com divisória, contorno) — e o mais útil: a altura do track é independente do modo de desenho |
| Segunda saída como "tela" | Resolume, *Screens* | Syphon/Spout/NDI *"can be treated like a separate physical screen"*: a saída de vídeo é um objeto do show, igual a um universo |
| Formato de vídeo | nenhum | Ableton só aceita `.mov` (§27.1). Nós aceitamos o que o navegador aceita, que é mais |

## 1. Os dois tracks novos

```json
{"type": "audio", "clips": [{"t0": 0, "len": 85.9, "src": "media/plenaria.mp3", "offset": 0}],
 "gain": [[0, 1.0]], "clock": true}
{"type": "video", "clips": [{"t0": 12.0, "len": 30.0, "src": "media/vinheta.mp4", "offset": 0}],
 "screen": 1}
```

- `clips[]` é o mesmo de `daw-arranjo.md §4.1`, sem exceção.
- `gain` é uma lane de automação como qualquer outra: um `keys` de nome próprio, interpolado igual. Fade in/out é dois keyframes, e é por isso que `daw-arranjo.md` C13 não precisa de objeto de fade.
- `clock: true` marca o track de áudio que **é** o relógio (§3). Um por show; o segundo dá erro na carga, não silêncio.
- `screen` diz em que saída de vídeo o track aparece (§4). Ausente = viewer da GUI.
- Sem `universe`, sem `address`: não são tracks de rede. `timeline.rs:376` (`resolvido`) não os reconhece, então `ignored()` os reporta — e é isso que `pontos-falhos.md` item 13 manda deixar de ser invisível.

Formatos: áudio `.wav`, `.mp3`, `.flac`; vídeo `.mp4`, `.webm`. **Divergência declarada com o Ableton**, §27.1: *"Live can import movies in Apple QuickTime format (.mov)"*. Não copiamos: o vídeo toca num `<video>` de WebView, e `.mov` é justamente o que o WebView costuma não abrir. O critério é o do reprodutor, não o do concorrente.

**Também divergimos do Ableton §27.1** em *"Live will only display video for video clips residing in the Arrangement View. Movie files that are loaded into the Session View are treated as audio clips"*. Aqui não há essa assimetria: a Session View é uma grade de cues (`daw-sessao.md`), não um segundo lugar onde a mídia mora.

## 2. Áudio toca no engine

**Por quê no engine e não na GUI:** o áudio é o relógio (§3), e o relógio não pode depender de uma janela estar aberta. `spell play` por SSH num Raspberry Pi, sem GUI nenhuma, tem de tocar o som do show. Se o áudio morasse no `<audio>` da página, o show sem GUI ficaria mudo e o relógio sem referência.

**Dependência nova, e a escada de `CLAUDE.md`.** Não há áudio na stdlib do Rust, e nenhuma das dependências já instaladas (`serde`, `axum`, `tokio`, `socket2`, `clap`, `rmcp`, `schemars`, `criterion`) toca som. Só então: **`rodio`**, que decodifica `wav`, `mp3`, `flac` e `ogg` e abre o dispositivo padrão do sistema, e é a menor coisa que faz o trabalho inteiro. Entra como membro novo do workspace ou como dependência do `engine`, atrás de uma feature `audio` — para o build do Pi poder sair sem ela se não houver placa.

**Comandos novos:**

| Comando | Argumentos | Faz |
|---|---|---|
| `audio_devices` | — | Lista os dispositivos de saída, com o padrão marcado |
| `audio_open` | `device?` | Abre; sem argumento, o padrão |
| `audio_close` | — | Fecha |
| `audio_peaks` | `file, n` | Devolve `n` pares `[min, max]` normalizados do arquivo — é a forma de onda |
| `audio_pos` | — | Posição real do dispositivo, em segundos (§3) |

`audio_peaks {file, n}` é lido **uma vez** pela GUI ao carregar o clipe e desenhado no canvas; não passa por WebSocket a cada quadro. `n` é o número de pixels do clipe na tela, então o custo não cresce com a duração do arquivo. Resolve p.643 dá as opções de desenho (retificada ou espelhada, com ou sem divisória) e a regra que importa: *"Track heights in the Edit page are independent of the Thumbnail and Waveform view settings"* — a forma de onda se ajusta à altura, não o contrário.

`play_show` (que mora na CLI, não no registry) passa a abrir o áudio junto com as saídas de rede, e a fechar junto.

## 3. O áudio é o relógio, e como isso se faz sem reescrever o relógio

Ableton §27.2.3 declara quem lidera: *"video clips are usually set as tempo leaders, while audio clips are left as tempo followers"*. Nós invertemos, e por motivo físico: **placa de som tem cristal próprio e não se pode empurrar; `<video>` de navegador tem `currentTime` que se pode empurrar.** Logo o áudio lidera e o vídeo segue.

O relógio do engine já existe e não precisa mudar de forma. `Clock` guarda `t0: Instant` e devolve `t0.elapsed()` (`clock.rs:26,63`); `locate` já sabe recolocar a origem: `tr.t0 = Instant::now() - Duration::from_secs_f64(t)` (`clock.rs:98`).

Então a correção de deriva é **o `locate` que já existe, chamado de vez em quando com o valor da placa**:

1. Sem track `clock: true`, nada muda: o relógio é o `Instant`, como hoje.
2. Com track `clock: true`, a cada segundo o engine compara `clock.time()` com `audio_pos()`.
3. Diferença abaixo da tolerância: nada. Acima: recoloca a origem, sem pular nem repetir quadro (o próximo `frame()` já sai no lugar certo).
4. **Tolerância: 20 ms**, com o número no comentário `ponytail:` e o critério escrito — 20 ms é abaixo do limiar em que o ouvido separa dois transientes, e acima do jitter que o próprio `Clock` já mede e reporta (`Stats { p50, p99, max, drift }`, `clock.rs:14-21`). Se a placa derivar mais que isso por segundo, o problema é a placa e o `drift` já aparece no painel.

É uma função (`Clock::sync(pos)`), três linhas, reusando `locate`. Nenhum segundo relógio, que é o que a regra 9 de `FUNCOES/README.md` proíbe.

## 4. Vídeo toca na GUI

**Por quê na GUI:** decodificar H.264 em Rust é uma dependência pesada (ou um `ffmpeg` do sistema) para entregar pixels que teriam de voltar para a tela. O WebView já tem decodificador de vídeo, aceleração de hardware e um elemento com `currentTime` escrevível. Escrever um decodificador ao lado dele é a definição de código que não precisa existir.

**Viewer, dentro da janela do editor.** Um `<video>` no lugar que `SHORTCUTS.md § Interface` já reservou (*"viewer central grande (aqui: previz ou VU dos universos)"*), dividindo espaço com o previz da frente `previz`.

**Sincronismo, o contrato:**

- O `<video>` **não** toca sozinho seguindo o relógio dele. A cada evento `transport` do bus, a página compara `video.currentTime + clip.t0 - clip.offset` com `t` do engine.
- Diferença abaixo da tolerância: deixa correr (deixar o vídeo correr sozinho é o que dá imagem fluida).
- Diferença entre a tolerância e um limite maior: corrige com `playbackRate` (0,97 a 1,03) até fechar — é o que evita o solavanco visível de um `seek`.
- Acima do limite maior: `currentTime = ...`, e o solavanco é aceito porque a alternativa é imagem errada.
- **Tolerância 40 ms** (pouco mais de um quadro a 25 fps) e **limite 250 ms**, os dois com `ponytail:` e critério: abaixo de 40 ms ninguém vê; acima de 250 ms o `playbackRate` demoraria mais que o próprio salto para fechar.
- Transporte parado: o vídeo fica pausado no quadro certo, e `locate` faz `seek`. Scrub na régua faz `seek` sem tocar.

**Segunda janela, no monitor do projetor.** Ableton §27.2.2: *"a separate, floating window that always remains above Live's main window"*, e *"The video can be shown in full screen (and optionally on a second monitor) by double-clicking in the Video Window"*, com `Ctrl+Alt+V` mostrando e escondendo (§41.1). Resolume, *Screens*: uma saída é um objeto do show e *"Every output can only have a single screen associated with it"*.

Aqui isso é: **uma segunda janela wry, sem barra, no monitor escolhido, mostrando só os tracks de vídeo cujo `screen` aponta para ela.** As duas janelas falam com o mesmo engine pelo mesmo bus, então o sincronismo é o mesmo contrato acima, rodando duas vezes. `show.outputs[]` ganha `{"type":"screen","monitor":1}`, e o `screen` do track (§1) é o índice dessa saída — quem arrasta a saída para o track é `browser-dnd.md §4`, sem gesto novo.

**Isto depende inteiramente da frente `gui-janela`.** Hoje não há crate de janela nenhum (`spellcore/Cargo.toml:3`), então não há primeira janela, quanto mais segunda. O que entra **agora**, sem esperar: o track `video`, o clipe, o desenho na timeline e o viewer dentro da página. O que espera: a segunda janela e o `screen`.

## 5. NDI e Spout: o que trava, exatamente

`ndi-ilda.md` já especifica a função NDI inteira (fonte, vetorização, saúde do link) e ela continua valendo. O que este arquivo acrescenta é por que ela não entra agora, e o que entra no lugar.

- **NDI** precisa do SDK da NewTek/Vizrt, que é download registrado e licença própria; não há crate que possa ser vendorizada no repo, e `FUNCOES/README.md §14` exige que tudo que carregamos seja texto versionável.
- **Spout** (Windows) e **Syphon** (macOS) são compartilhamento de textura por GPU. Resolume, *Syphon & Spout*: entrada *"always enabled"*, saída ligada pelo Output Menu, e o nome anunciado é `App Name` + `Server Name`. Para nós isso exigiria contexto GPU compartilhado entre o wry e o processo — que é exatamente a coisa que o WebView não expõe.
- **O que entra no lugar, e resolve o caso real:** a segunda janela wry em tela cheia no projetor (§4). A razão de existir NDI/Spout num show é levar imagem daqui para o projetor ou para outro software; para o projetor, a janela resolve, e sem SDK nenhum.
- **O que continua bloqueado:** mandar imagem para *outro programa* (Resolume, OBS) e receber imagem *de* outro programa. Isso é NDI/Spout de verdade e fica escrito como bloqueio de licença, não de esforço.

## 6. O que fica de fora

- **`.mov`** (Ableton §27.1). Ver §1: o critério é o do reprodutor.
- **Consolidate / Reverse / Crop trocando vídeo por áudio** (Ableton §27.2.1: *"This replacement only occurs internally — your original movie files are never altered"*). Não temos nenhum dos três; `daw-arranjo.md` C11 já recusou Consolidate pelo mesmo motivo — não renderizamos mídia.
- **Warp markers no vídeo definindo hit points** (Ableton §27.2.3.1). É tempo musical, e `timeline-daw.md` item 38 já recusou compasso.
- **Mixer de áudio, sends, retornos.** Um track de áudio com `gain` e um dispositivo de saída basta para um show. Mixer é outro produto.
- **Entrada de áudio (gravar do microfone, detecção de batida).** Sem pedido. Quando houver, a porta é `input {key:"audio:level"}`, que o graph já sabe ler.
- **Áudio por track com roteamento para saídas diferentes.** Um dispositivo, um par estéreo. O limite fica em `ponytail:` no `audio_open`.

## 7. Atalhos

| Ação | Tecla | Origem | Conflito |
|---|---|---|---|
| Mostrar / esconder a janela de vídeo | `Ctrl+Alt+V` | Ableton §41.1 | nenhum |
| Tela cheia da janela de vídeo | duplo-clique nela | Ableton §27.2.2 | gesto |
| Voltar ao tamanho original | `Alt`+duplo-clique | Ableton §27.2.2 | gesto; e é o `Alt = variante` da nossa gramática |

## 8. Testes

Rust, ao lado dos testes que já existem em `engine/tests/`:

| O que prova | Como |
|---|---|
| `Clock::sync` corrige | relógio em `t=10.0`, `sync(10.030)` → `time()` passa a `≈10.030`; `sync(10.005)` → não mexe |
| `Clock::sync` sem track `clock` | nunca é chamado; o `Instant` continua mandando |
| Dois tracks com `clock: true` | erro na carga, com o índice dos dois |
| `audio_peaks` | arquivo de 1 s com uma senoide, `n=10` → 10 pares, todos com `max > 0.9` e `min < -0.9` |
| `audio_peaks` com `n` maior que o número de amostras | não estoura; devolve `n` pares |
| Track `audio` sem dispositivo aberto | o show toca em silêncio e **avisa**, não falha (é o estado "sem feed" de `ilda-player.md §2`) |

`node`, na GUI:

| O que prova | Como |
|---|---|
| `AV.corrige(dt)` | `dt = 0.02` → `{acao:"nada"}`; `dt = 0.1` → `{acao:"rate", rate:1.03}`; `dt = 0.5` → `{acao:"seek"}` |
| `AV.tempoDoVideo(clip, t)` | `t` fora do clipe → `null` (o `<video>` fica pausado, não em quadro errado) |

Prova visual: screenshot headless de `index.html` com um track `audio` mostrando a forma de onda e um track `video` mostrando o clipe, e o viewer com o quadro correspondente ao playhead.
