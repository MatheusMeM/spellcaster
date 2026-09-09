# Spellcaster — arquitetura (estado em F1)

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
    sacn.py              E1.31 out/in/discovery
    artnet.py            Art-Net 4 out/in/poll
    osc.py               OSC 1.0 out/in
    netscan.py           análise de rede (__main__)
    ilda/
      frame.py           Point, Frame, optimize, safety
      ild.py             read, write
      generators.py      figuras, medgrupo, render (__main__)
      etherdream.py      EtherDream, Emulator
shows/
  medgrupo.py            look(t), DUR
seed/                    show_medgrupo.py, bsw_hold.py, bsw_multi.py, ilda_gen.py (referência)
tests/                   test_artnet, test_clock, test_engine, test_ilda, test_netscan,
                         test_osc, test_registry, test_sacn, test_show_medgrupo
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
- `discover(timeout=3.0) -> list[{cid, name, ip, universes}]`.
- Sem `ponytail:`. Não implementado: sync E1.31, merge por prioridade na entrada.

### artnet (`protocols/artnet.py`)

- Porta 6454. Broadcast em `2.255.255.255`, `10.255.255.255`, `255.255.255.255`.
- `port_address(universe) -> int`: `universe - 1`, 15 bits.
- `artdmx(universe, data, sequence, physical=0) -> bytes`: dados 2..512 bytes, comprimento par.
- `artpoll(flags=0x06, priority=0x10) -> bytes`, `artsync() -> bytes`.
- `parse(packet) -> dict | None`: ArtDmx, ArtPoll, ArtPollReply, ArtSync.
- `ArtNetOut(targets=None, broadcast=True, port=6454)`: `send(universe, data)`, `sync()`, `close()`. Sequência 1..255.
- `ArtNetIn(universes, host="0.0.0.0", port=6454)`: thread. `.frames[universe] -> bytes`, `close()`.
- `poll(timeout=2.0, targets=BROADCASTS) -> list[dict]`.
- `artnet.py:142` ponytail: se a porta 6454 está ocupada, `poll` faz bind em porta efêmera e só recebe respostas unicast. Falta: compartilhar o socket com `ArtNetIn`.
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
- `scan_artnet(timeout=2, ifaces=None)`: ArtPoll em broadcast global, 2.x, 10.x e broadcast de cada subrede. `parse_artpollreply(data)`.
- `scan_sacn(timeout=3, ifaces=None)`: entra no multicast de discovery em cada interface. `parse_sacn_discovery(data)`.
- `scan_etherdream(timeout=2)`: beacons UDP 7654.
- `suggest(ifaces, windows=WINDOWS) -> list[str]`: regras de subrede para Art-Net, comando `netsh` pronto (texto, não executa), aviso de interfaces na mesma subrede.
- `scan_all(timeout=2) -> dict`: os três scans em threads. `report(d) -> str`.
- Sem `ponytail:`. Duplicação declarada no docstring do módulo: `parse_artpollreply` e `parse_sacn_discovery` repetem `artnet.parse` e `sacn.parse`. Unificar quando os dois estiverem estáveis.
- Não implementado (ROADMAP F1): mDNS `_osc._udp`, discovery IDN, medida de latência. Verbo `spell net` existe em `cli.py`.

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
- `generators`: `ellipse`, `circle`, `polyline`, `rect`, `line`, `blank_to`, `ease`, `medgrupo(t, sx=20000, sy=10000) -> Frame`, `render(fn, fps=25, dur=46.8, **kw) -> list[Frame]`.
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
| F5 MCP | `mcp/server.py` gera tools de `registry.schema()`; resources: show, patch, rede, log | registry, `netscan.scan_all` |
| F6 portátil e Lite | PyInstaller onedir; `spell serve --headless`, `spell tui` (curses); tarball aarch64 | tudo acima sem `pywebview` |

## spellcore (Rust)

Core do produto reescrito em Rust (PRD, fase R0). O pacote Python `spellcaster/` continua no repo
como implementação de referência e gerador dos fixtures de conformidade; não recebe funcionalidade
nova. O `spellcore` tem que reproduzir byte a byte a saída sACN/Art-Net do `shows/medgrupo.spell`.

```
spellcore/
  Cargo.toml     workspace, edition 2021; release: lto "thin", codegen-units 1, panic abort
  engine/        clock.rs, universe.rs, timeline.rs, show.rs, registry.rs
  protocols/     lib.rs (trait Output + fila), sacn.rs, artnet.rs, osc.rs, netscan.rs
  cli/           binário `spellcore`: play, net, commands (gerados do registry em runtime)
  bench/         Criterion (benches/core.rs) + binários jitter e throughput
tests/conformance/
  gen.py         gera os fixtures a partir do pacote Python
  medgrupo_u1.bin, sacn_packet.bin, artnet_packet.bin
  capture_sacn.py  valida o binário Rust contra o fixture, ao vivo, em 127.0.0.1
```

### Contratos

- `Clock::new(fps)`, `Clock::run(FnMut(f64), Option<f64>)`, `stats() -> Stats {p50,p99,max,frames,drift}`.
  Thread em prioridade alta, `timeBeginPeriod(1)` no Windows, fase fixa (`next += period`).
  A margem de spin antes do alvo é **calibrada em runtime** pelo overshoot medido do `sleep`
  (EWMA, limitada a 0,3–2 ms): margem fixa de 1 ms custava ~6 % de um núcleo a 60 Hz.
- `Universes` 1-based, buffers `[u8; 512]` pré-alocados, `get_or_create` por busca binária.
- `Timeline::apply(&mut Universes, t)` sem alocação por frame; keys por `partition_point`.
  Curvas: linear, hold, in, out, inout, bezier — mesmas fórmulas do `timeline/model.py`.
- `Registry::add::<A: JsonSchema + DeserializeOwned>(nome, doc, fn)`; erro é `String`, sem `anyhow`.
  CLI, e depois OSC-API, GUI e MCP, são clientes: `Registry::schema()` gera os subcomandos.
- `trait Output { fn send(&mut self, universe: u16, data: &[u8; 512]); fn close(&mut self); }`.
  `close(&mut self)` e não `close(self)` do PRD: `Box<dyn Output>` exige object safety.
- Cada saída tem thread própria e fila de 2 frames **por universo**; ao encher, o frame velho
  daquele universo é descartado. `send()` nunca bloqueia o engine.

### O show MED GRUPO na R0

O track `pyfx` é Python e usa estado entre frames (histerese de pan dos movings), logo não existe
em Rust. `tests/conformance/gen.py` assa o resultado frame a frame em `shows/medgrupo_r0.spell`:
219 tracks `dmx`, 67 161 keyframes com curva `hold` (degrau exato), tempos truncados em 1 µs para
nunca arredondarem para cima. É o arquivo que o `spellcore play` toca. O equivalente vivo do
`pyfx` volta na R1 como track `fx` em script.

### Como buildar

A pasta do repo está no Google Drive: `target/` nunca pode nascer dentro dela.

```powershell
$env:CARGO_TARGET_DIR = "$env:TEMP\spellcore_target"
cd spellcore
cargo test --workspace
cargo build --release
cargo run --release -p bench --bin jitter
cargo run --release -p bench --bin throughput
cargo bench -p bench
```

`spellcore/.cargo/config.toml` (não versionado) fixa o mesmo caminho para quem esquecer a variável;
`target/` está no `.gitignore` como segunda barreira. Dependências e justificativa: `spellcore/README.md`.

### Números medidos (desktop x64, Windows 11)

| Métrica | Alvo do PRD | Medido |
|---|---|---|
| Jitter entre frames, 60 Hz, p99 | < 1 ms | 0,42 ms (max 0,57 ms) |
| Drift em 10 s a 60 Hz | 0 frames | 0 |
| 16 sACN + 16 Art-Net a 60 Hz | < 3 % de um núcleo | 0,78 % |
| Boot até o primeiro frame DMX | < 2 s | 0,002 s |
| RSS em repouso | < 60 MB | 5,6 MB |
| `Timeline::apply` do show inteiro | — | 3,73 µs (33 ms de orçamento a 30 fps) |
| Binário `spellcore.exe` release | < 20 MB | 0,95 MB |
