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
- Não implementado (ROADMAP F1): mDNS `_osc._udp`, discovery IDN, medida de latência, verbo `spell net`.

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
