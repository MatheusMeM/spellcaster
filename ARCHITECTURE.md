# Spellcaster — arquitetura (protótipo Python F0–F6; spellcore Rust R0, R1, R3, R4, R7)

## Árvore

```
spellcaster/
  __init__.py            __version__
  cli.py                 spell: play, commands (gerados do registry)
  core/
    clock.py             Clock
    engine.py            Engine
    registry.py          @command, call, schema, REGISTRY
    universe.py          Universe, Universes
  protocols/
    sacn.py              E1.31 out/in
    artnet.py            Art-Net 4 out/in
    osc.py               OSC 1.0 out/in
    netscan.py           análise de rede (__main__)
    ilda/
      frame.py           Point, Frame, optimize, safety
      ild.py             read, write
      generators.py      figuras, medgrupo, render (__main__)
      etherdream.py      EtherDream, Emulator
shows/
  medgrupo.py            look(t), DUR
tests/                   test_artnet, test_clock, test_engine, test_ilda, test_netscan,
                         test_osc, test_registry, test_sacn
```

## Fluxo de dados

```
show .py  ──look(t)──▶  Engine.apply  ──▶  Universes  ──bytes──▶  out.send(universe, data)
                ▲                                                  (SacnOut | ArtNetOut)
             Clock.run
```

1. `Clock.run(fn, duration)` chama `fn(t)` a cada `1/fps` s. `t` vem de `perf_counter`.
2. `Engine.tick(look, t)` chama `look(t)`. O retorno é um dict `{addr: [valores]}` (universo 1) ou `{(universo, addr): [valores]}`.
3. `Engine.apply(frame)` grava cada lista em `Universes.get_or_create(u).set(addr, valores)`.
4. Para cada universo, `Engine.tick` converte o buffer em `bytes` e chama `out.send(u.number, data)` em cada saída.

### Assinaturas públicas (`spellcaster/core`)

- `Clock(fps=30)`: `.time`, `.state` ("stop" | "play" | "pause"), `play()`, `pause()`, `stop()`, `locate(t)`, `run(fn, duration=None)`.
- `Universe(number)`: `.number`, `.data` (bytearray de 512), `set(addr, values)`. Endereço 1-based, clamp 0-255, ignora o que passar de 512.
- `Universes(dict)`: `get_or_create(number)`.
- `Engine(outputs, fps=30)`: `.outputs`, `.universes`, `.clock`, `apply(frame)`, `tick(look, t)`, `run(look, duration=None)`.
- `registry.command(name=None, mcp=True)`: decorador. `registry.call(name, **kwargs)`: coerce por anotação e chama. `registry.schema()`: lista de `{name, doc, params: [{name, type, default}], mcp}`.

### Registry → CLI → MCP

`cli.main()` percorre `schema()` e cria um subparser por comando. Parâmetro sem default vira argumento posicional. Parâmetro `bool` vira flag `--nome` (`store_true`). Os demais viram `--nome` com o default. `call()` converte strings do argparse para o tipo anotado (`int`, `float`, `str`, `bool`). O mesmo `schema()` alimentará o gerador MCP (F5): cada entrada com `mcp=True` vira uma tool com schema JSON; entradas `mcp=False` ficam atrás de um tool genérico `run_command`.

## Protocolos

Todas as saídas expõem `send(universe: int, data: bytes)` e `close()`.

### sacn (`protocols/sacn.py`)

- Porta 5568. Multicast por universo `239.255.{u>>8}.{u&255}`. Discovery em `239.255.250.214`.
- `packet(universe, data, cid, seq, source_name="Spellcaster", priority=100) -> bytes`: função pura, mesmos bytes do gerador do MED GRUPO.
- `parse(pk) -> dict | None`: pacote data (vector 4) ou discovery (vector 8).
- `interfaces() -> list[str]`: IPs IPv4 locais por `getaddrinfo(gethostname())` mais `127.0.0.1`.
- `SacnOut(universes=(1,), priority=100, source_name="Spellcaster", interfaces=None)`: um socket por IP com `IP_MULTICAST_IF`. `send` envia multicast em cada socket e unicast em `127.0.0.1`. `close()`.
- `SacnIn(universes=(1,))`: thread daemon. `get(universe) -> bytes | None`, `.sources`, `close()`.
- Sem `ponytail:`. Não implementado: sync E1.31, merge por prioridade na entrada. O discovery da rede é varrido por `netscan.scan_sacn`, que decodifica com este `parse`.

### artnet (`protocols/artnet.py`)

- Porta 6454. Broadcast em `2.255.255.255`, `10.255.255.255`, `255.255.255.255`.
- `port_address(universe) -> int`: `universe - 1`, 15 bits.
- `artdmx(universe, data, sequence, physical=0) -> bytes`: dados 2..512 bytes, comprimento par.
- `artpoll(flags=0x06, priority=0x10) -> bytes`, `artsync() -> bytes`.
- `parse(packet) -> dict | None`: ArtDmx, ArtPoll, ArtPollReply, ArtSync.
- `ArtNetOut(targets=None, broadcast=True, port=6454)`: `send(universe, data)`, `close()`. Sequência 1..255.
- `ArtNetIn(universes, host="0.0.0.0", port=6454)`: thread. `.frames[universe] -> bytes`, `close()`.
- O ArtPoll da rede é disparado por `netscan.scan_artnet`, que decodifica a resposta com este `parse`.
- Não implementado: emitir ArtPollReply (o Spellcaster não aparece como nó para outros controladores).

### osc (`protocols/osc.py`)

- UDP, porta escolhida pelo chamador. Sem porta padrão.
- `message(address, *args) -> bytes`. Tipos: `int` (i/h), `float` (f), `str` (s), `bytes`/`Blob` (b), `True`/`False`, `None`, `Ellipsis` (impulso), tupla `("d"|"h"|"t", valor)` para tipo explícito.
- `bundle(elements, tt=IMMEDIATE) -> bytes`, `timetag(t=None) -> int` (NTP 64 bits).
- `parse(data) -> (address, args) | ("#bundle", timetag, elementos)`.
- `pattern_re(pattern) -> re.Pattern`: suporta `*`, `?`, `[a-z]`, `[!a-z]`, `{a,b}`.
- `OscOut(host, port)`: `send(address, *args)`, `bundle(msgs, tt=IMMEDIATE)`, `close()`. Este `send` não segue o contrato `(universe, data)`; OSC não é saída DMX.
- `OscIn(port, host="0.0.0.0")`: thread. `on(pattern, fn)` chama `fn(address, *args)`. `close()`.
- `osc.py:153` ponytail: timetag futuro em bundle é ignorado e o conteúdo executa já. Falta: agendar no `Clock` quando o engine expuser fila de eventos.

### netscan (`protocols/netscan.py`)

- Executável: `python -m spellcaster.protocols.netscan [--json] [--timeout N]`.
- `interfaces() -> list[{name, ip, mask, gateway}]`: `ipconfig` no Windows, `ip -j addr` + `ip -j route` no Linux, fallback `parse_ip_addr`. Sem loopback.
- `parse_ipconfig(text)`, `parse_ip_addr(text, route_text="")`: parsers puros, testados com saída pt-BR e en.
- `scan_artnet(timeout=2, ifaces=None)`: ArtPoll em broadcast global, 2.x, 10.x e broadcast de cada subrede; decodifica com `artnet.parse`.
- `scan_sacn(timeout=3, ifaces=None)`: entra no multicast de discovery em cada interface; decodifica com `sacn.parse`.
- `scan_etherdream(timeout=2)`: beacons UDP 7654.
- `suggest(ifaces, windows=WINDOWS) -> list[str]`: regras de subrede para Art-Net, comando `netsh` pronto (texto, não executa), aviso de interfaces na mesma subrede.
- `scan_all(timeout=2) -> dict`: os três scans em threads. `report(d) -> str`.
- Sem `ponytail:`. Um decodificador por protocolo: os pacotes vêm de `artnet.parse` e `sacn.parse`.
- Não implementado (ROADMAP F1): mDNS `_osc._udp`, discovery IDN, medida de latência. O verbo `spell net` (único: devolve o dict com `report`, usado também pela GUI e pelo MCP) está em `cli.py`.

### Dois `interfaces()`

| Função | Retorno | Fonte | Uso |
|---|---|---|---|
| `sacn.interfaces()` | `list[str]` de IPs, inclui `127.0.0.1` | `socket.getaddrinfo` | escolher `IP_MULTICAST_IF` em `SacnOut` |
| `netscan.interfaces()` | `list[dict]` com `name`, `ip`, `mask`, `gateway`, sem loopback | `ipconfig` / `ip` | relatório de rede, broadcast por subrede, sugestões |

`sacn` não depende de subprocess e funciona sem `ipconfig`. `netscan` precisa da máscara para calcular broadcast e subrede. Manter os dois até F2 decidir uma origem única.

### ilda (`protocols/ilda/`)

- Coordenadas ±32767. Cores 0-255.
- `frame.Point(x, y, r=0, g=0, b=0, blank=False)`: dataclass com clamp. `.lit`.
- `frame.Frame(points=None, name="")`: `len()`, iteração, `bbox(lit_only=True)`.
- `frame.optimize(frame, dwell=2, blank_gap=4, max_step=1200, angle=25) -> Frame`: dwell em vértices, pontos apagados nas transições, interpolação de saltos.
- `frame.safety(frame, min_size=2000, max_intensity=255, zone=None) -> Frame`: escurece figura menor que `min_size`, limita intensidade, apaga fora da zona.
- `ild.read(path) -> list[Frame]`, `ild.write(path, frames, fmt=5, name="", company="spell", palette=None)`. Formatos 0, 1, 2 (paleta), 4, 5.
- `generators`: `ellipse`, `circle`, `polyline`, `rect`, `blank_to`, `ease`, `medgrupo(t, sx=20000, sy=10000) -> Frame`, `render(fn, fps=25, dur=46.8, **kw) -> list[Frame]`.
- `etherdream`: beacon UDP 7654, stream TCP 7765, little-endian.
  - `EtherDream(ip, port=7765, capacity=1800)`: `connect(timeout=2)`, `prepare()`, `begin(pps, low_water=0)`, `send(points)`, `ping()`, `play(frames, pps=20000, chunk=None) -> Thread`, `stop()`, `close()`. `send` aqui é o comando `d` do DAC, não o contrato DMX.
  - `Emulator(host="127.0.0.1", port=0, capacity=1800)`: servidor TCP para testes. `start()`, `stop()`, `beacon()`, `.received`, `.commands`.
  - `parse_beacon(b)`, `parse_status(b)`, `parse_response(b)`, `encode_data(points)`.
- `etherdream.py:107` ponytail: `_loop` espera o buffer esvaziar com `sleep` + `ping` antes de enfileirar. Falta: fluxo por `low_water` e reconexão em queda.
- Não implementado (ROADMAP): `idn.py` (IDN-Stream), `helios.py` (USB).

### core

- `clock.py:59` ponytail: quando um tick atrasa, `run` reancora `nxt` no instante atual e não acumula o atraso. Falta: medir jitter a 60 Hz com muitos universos (risco listado no ROADMAP).

## Onde entram as próximas fases

| Fase | Entra em | Depende de |
|---|---|---|
| F2 perfis e patch | `fixtures/profile.py`, `fixture.py`, `group.py`, `library/*.json`; verbo `spell calib` | `Universe.set`, registry |
| F3 timeline, `.spell`, player | `timeline/model.py`, `tracks.py`, `cues.py`, `render.py`; `show.py`; `player/player.py`; track `laser` usa `ilda.frame`, `ild`, `etherdream` | `Clock`, `Engine.apply` recebe o dict de `timeline.render` no lugar de `look(t)` |
| F4 GUI | `gui/server.py` (HTTP + WebSocket), `gui/web/`, `gui/window.py` (pywebview) | registry via WebSocket; `netscan.scan_all` para o painel Network |
| F5 MCP | `mcp/server.py` (stdio) gera tools de `registry.schema()`; resources: show, patch, rede, log | registry, comando `net` |
| F6 portátil e Lite | PyInstaller onedir; `spell serve --headless`, `spell tui` (curses); tarball aarch64 | tudo acima sem `pywebview` |

## spellcore (Rust) — estado em R1 + R3 + R4 + R7

Core do produto reescrito em Rust (PRD v1.1). O pacote Python `spellcaster/` continua no repo
como implementação de referência e gerador dos fixtures de conformidade; não recebe funcionalidade
nova. O `spellcore` reproduz byte a byte a saída sACN/Art-Net do `shows/medgrupo.spell` e os
bytes/números do `spellcaster/protocols/ilda/`.

```
spellcore/
  Cargo.toml     workspace, edition 2021; release: lto "thin", codegen-units 1, panic abort
  engine/        clock, universe, timeline, show, registry, cues, hook, player
  protocols/     lib.rs (trait Output + fila), sacn, artnet, osc, netscan
  script/        lib.rs (Fx: track `fx` em Rhai), graph.rs (Graph runtime da seção 10 do PRD)
  laser/         frame (otimização + safety), ild, feed (multi-feed), dac/{etherdream,helios,idn}
                 bin/feeds (bench de 4 feeds), benches/laser.rs, tests/conformance.rs + fixtures
  pixelmap/      lib.rs (Order, Fixture, PixelMap, Frame, Sampling, Mapper, grid), rayon
                 bin/map_bench (gate de 2 ms), benches/pixelmap.rs, tests/conformance.rs
  mcp/           lib.rs (servidor MCP sobre `rmcp`, stdio), install.rs (`mcp install`)
  cli/           binário `spellcore`: play, net, commands, mcp (clap derive, structs fixas)
  bench/         Criterion (benches/core.rs, benches/graph.rs) + binários jitter e throughput
tests/conformance/
  gen.py         gera os fixtures a partir do pacote Python
  medgrupo_u1.bin, sacn_packet.bin, artnet_packet.bin
  capture_sacn.py  valida o binário Rust contra o fixture, ao vivo, em 127.0.0.1
shows/
  medgrupo.spell   tracks pyfx (Python) + fx (Rust, `medgrupo.rhai`) + laser; cada lado ignora o outro
  medgrupo_r0.spell  219 tracks dmx assados (67 161 keys hold): conformidade offline e bench Criterion
```

Grafo de dependências: `protocols` autocontido; `engine -> protocols`; `script -> engine, rhai`;
`laser -> protocols` (só o beacon Ether Dream); `pixelmap -> rayon, serde` (não depende de
nenhum crate do workspace); `mcp -> engine, rmcp, tokio`; `cli -> engine, protocols, script, mcp`;
`bench -> engine, protocols, script`. O engine não conhece GUI, MCP, Rhai, laser nem pixelmap.

O `pixelmap` (R3) amostra um frame RGB/RGBA em coordenada normalizada `(u, v)` por pixel físico
e escreve um buffer de 512 canais por universo — o par `(universo, &[u8; 512])` que
`protocols::Output::send` recebe. `Mapper::new` agrupa as fixtures por universo e aloca os
buffers uma vez; `render` não aloca e paraleliza por universo com `rayon`. Amostragem nearest
por default, bilinear opcional. Universo 0, canal 0 e canal acima de 512 são descartados na
compilação; canal que estoura os 512 no fim (510 + RGBW) é truncado; `u`/`v` fora de `0..1`
clampam na borda. O `.spell` guarda o mapa no bloco `pixelmaps` (fora de `tracks`, preservado
pelo `extra` do `Show`), com `{name, source: "media/<id>", fixtures: [{universe, channel, order,
u, v}]}`; a ligação com o player espera a R2 (mídia), então o crate ainda roda sozinho, sobre um
`&[u8]` de teste. A versão wgpu do PRD entra quando `map_bench` não fechar os 2 ms.

### Contratos

- `Clock::new(fps)`, `Clock::run(FnMut(f64), Option<f64>)`, `stats() -> Stats {p50,p99,max,frames,drift}`.
  Thread em prioridade alta, `timeBeginPeriod(1)` no Windows, fase fixa (`next += period`).
  Margem de spin calibrada em runtime pelo overshoot medido do `sleep` (EWMA, 0,3–2 ms).
  **O `t` entregue ao frame é travado no quadro** (`round(t·fps)/fps`): é o mesmo `i/fps` do
  gerador de fixtures, e sem isso um `fx` contínuo (seno) amostrado no tempo medido diverge em ±1.
- `Universes` 1-based, buffers `[u8; 512]` pré-alocados, `get_or_create` por busca binária.
- `Timeline::apply(&mut Universes, t)` sem alocação por frame; keys por `partition_point`.
  Curvas: linear, hold, in, out, inout, bezier — mesmas fórmulas do `timeline/model.py`.
- Ordem fixa do frame: `Timeline::apply` → cada `FrameHook` na ordem de registro (tracks `fx`,
  depois o Graph) → tracks `osc`/`media`/`cue` → `CueList::update` → programmer → I/O. Igual ao
  `_tick` do Python.
- `engine::hook`: `trait FrameHook { frame(t, &mut Universes); input(key, value); reset(t) }`,
  `enum Ev { Cmd, Osc, Widget, Param, Notify }`, `trait EventSink { emit(&Ev) }`. É por aqui que
  `script` se pluga sem o engine conhecer Rhai.
- `engine::cues`: `CueList::new(&[Value])`, `go(t, Option<usize>)`, `update(t, &mut Universes)`,
  `reset()`. Chave `"1/100"` → `(1, 100)`; fade linear; `follow` encadeia.
- `engine::player`: `Player::new(show, base, looping)`, `hook(Box<dyn FrameHook>)`,
  `start(osc_port)` (thread de transporte + OscIn `/spellcaster/play|pause|stop|locate`),
  `Handle {play, pause, stop, locate, cue_go, state}`, `player::current()` é o player vivo do processo.
- `Registry::add::<A: JsonSchema + DeserializeOwned>(nome, doc, fn)`; erro é `String`.
  `registry::base()` sem `Clock`: `load`, `show_get`, `pause`, `stop`, `locate`, `cue_go`,
  `transport_state`; os de transporte agem em `player::current()`. `play_show` e `net` são
  registrados pela CLI, que é quem conhece `script` e `protocols`.
- `engine::edit` (ligado em `base()`): edição do show aberto (`OPEN`, um por processo) —
  `show_new`/`show_set`/`show_save`, `track_add`/`track_del`, `key_set`/`key_del`,
  `cue_set`/`cue_del`, `patch_add`/`patch_del`/`patch_check`/`profiles`. Porte de
  `gui/api.py` + footprint de `fixtures/patch.py`; é por aqui que a GUI Tauri e o MCP editam.
- `mcp::Spell` (crate `mcp`) implementa `rmcp::ServerHandler` sobre um `Registry`: uma tool por
  comando, com o `inputSchema` que o `schemars` gerou; resources `spell://show` (o `show_get`) e
  `spell://commands` (o `Registry::schema()`). Erro de comando volta como `isError`, não como erro
  JSON-RPC. `play_show` bloqueia, então roda em thread e a tool volta na hora; enquanto o MCP roda,
  a linha de status do `play` sai no stderr (no stdio o stdout é o canal JSON-RPC).
  `mcp::install` grava a entrada "spellcaster" no config do Claude — não é comando do registry,
  para que a IA não reescreva a própria configuração. Só stdio: o HTTP streamable do `rmcp` é um
  `tower::Service` e ainda exigiria axum/hyper.
- `trait Output { fn send(&mut self, universe: u16, data: &[u8; 512]); fn close(&mut self); }`.
  Cada saída tem thread própria e fila de 2 frames por universo; `send()` nunca bloqueia o engine.
- `script::Fx::new(path, universe)`: compila o `.rhai` uma vez, estado persiste entre frames; API
  exposta ao script: `set(universe, addr, values)`, `set(addr, values)`, `st(i)`/`st(i, v)`
  (estado). Rhai com `default-features = false` + `std, sync, no_custom_syntax, no_time, only_i64,
  no_module, no_closure`; `max_expr_depths(256, 256)` porque o default (64/32) recusa o show.
- `script::Graph::new(&Value, Box<dyn EventSink>)`: catálogo fechado (`in.*`, `logic.*`, `math.*`,
  `time.*`, `cmd`, `out.*`), ordem topológica na compilação, pinos por índice, zero alocação por frame.
- `laser`: `Point`, `Frame`, `optimize(&[Point], Safety) -> Vec<Point>` (dwell, blanking,
  interpolação, limite de kpps, tamanho mínimo de figura), `ild::read/write` (formatos 0, 1, 2, 4, 5),
  `trait Dac` para Ether Dream (TCP, com `Emulator`), Helios (USB) e IDN (UDP),
  `Feed::start(Box<dyn Dac>, pps, buffer_frames, Safety)` com thread própria por DAC.
  Safety é obrigatória no `Feed`: não há caminho para o DAC sem ela.

### Conformidade (o que "byte a byte" mede)

| Teste | Referência | Resultado |
|---|---|---|
| `cargo test -p engine` (timeline R0) | `medgrupo_u1.bin` (2577 frames × 512) | igual |
| `cargo test -p script --test medgrupo` (`medgrupo.rhai`) | `medgrupo_u1.bin` | igual nos 2577 frames |
| `cargo test -p laser --test conformance` | fixtures de `optimize`/`safety`/`.ild` gerados do Python | igual |
| `capture_sacn.py` (ao vivo, `medgrupo_r0.spell`) | pacotes sACN recebidos em 127.0.0.1 | 89/89 iguais |
| `capture_sacn.py --show shows/medgrupo.spell` (ao vivo, fx Rhai) | idem | 89/89 iguais |

### Como buildar

A pasta do repo está no Google Drive: `target/` nunca pode nascer dentro dela.

```powershell
$env:CARGO_TARGET_DIR = "$env:TEMP\spellcore_target"
cd spellcore
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --workspace
cargo run --release -p bench --bin jitter
cargo run --release -p bench --bin throughput
cargo run --release -p laser --bin feeds -- --secs 30
cargo run --release -p pixelmap --bin map_bench
cargo bench -p bench
```

`spellcore/.cargo/config.toml` (não versionado) fixa o mesmo caminho para quem esquecer a variável;
`target/` está no `.gitignore` como segunda barreira. Dependências e justificativa: `spellcore/README.md`.

### Números medidos (desktop x64, Windows 11, release)

| Métrica | Alvo do PRD | Medido |
|---|---|---|
| Jitter entre frames, 60 Hz, p99 | < 1 ms | 0,22 ms (max 0,28 ms) |
| Drift em 10 s a 60 Hz | 0 frames | 0 |
| 16 sACN + 16 Art-Net a 60 Hz | < 3 % de um núcleo | 0,2 % (2,7 % na primeira execução, cache frio) |
| Boot até o primeiro frame DMX | < 2 s | 0,002 s |
| RSS em repouso | < 60 MB | 5,6 MB |
| `Timeline::apply` do show inteiro | — | 3,73 µs |
| Graph de 500 nós por frame | < 0,1 ms | 8,4 µs (2,0 µs sem `math.expr`) |
| 4 feeds laser × 30 kpps, cpu das threads de feed | < 1 % de um núcleo | 0,83 % em 30 s (GetThreadTimes quantiza em 15,6 ms: rodar ≥ 30 s) |
| Pixel mapping, 100 000 px a 60 Hz (CPU, rayon) | < 2 ms por frame | 0,105 ms p50, 0,316 ms p99 (bilinear: 0,196 / 0,493) |
| Binário `spellcore.exe` release | < 20 MB | 3,8 MB (2,5 MB antes do rmcp; 0,95 MB antes do Rhai) |

## spellgui/web (GUI Tauri) — base da R5

```
spellgui/web/
  canvaskit.js   kit de canvas: view {x, zoom, y}, world<->screen, bisect/near (hit-test),
                 selecao esparsa, marquee, pan, zoom no cursor, dirty-flag, DPR, laco rAF
  timeline.js    timeline sobre o kit: lanes do .spell, keyframes em arrays paralelos, curvas
                 (linear, hold, in, out, inout, bezier) iguais as do engine, regua com timecode,
                 snapping em markers/keyframes, scrub, In/Out, loop, atalhos de design/SHORTCUTS.md
  index.html     pagina minima: abre shows/medgrupo.spell (?show=<caminho> troca), barra e menu
tests/test_spellgui_timeline.py   Chrome headless --dump-dom sobre uma pagina que roda as asseracoes em JS
```

O app Tauri ainda não existe: `spellgui/web/` é a base de canvas da R5, servida por qualquer HTTP
estático e testada em Chrome headless. O `canvaskit.js` é o único lugar que sabe de pan, zoom,
seleção, marquee, DPR e dirty-flag; a timeline (e o graph, na R9) só desenham e respondem a
`k.on.{down,move,up,marquee,menu,frame}`. Todo hit-test é `CK.bisect`/`CK.near` sobre listas
ordenadas por tempo, e o desenho percorre só as lanes visíveis, como no protótipo Python. Não há
WebSocket nem engine: o transporte é um relógio local e a edição volta para o JSON do show em
memória (`TL.commit`). Cores só por token de `design/tokens/spellcaster.css`.
