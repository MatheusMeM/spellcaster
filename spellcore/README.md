# spellcore — core do Spellcaster em Rust (R0, R1, R3, R4, R7)

Workspace Cargo com o engine, os protocolos, a CLI e o bench. Sem GUI, sem Godot, sem mídia.
O pacote Python `spellcaster/` continua sendo a implementação de referência: o spellcore tem
que reproduzir byte a byte a saída dele (fixtures em `tests/conformance/`).

```
spellcore/
  Cargo.toml        workspace (edition 2021, release: lto thin, codegen-units 1, panic abort)
  engine/           clock, universe, timeline (keys/curvas), show (.spell v1), registry, edit (edicao do show aberto)
  protocols/        trait Output, sacn, artnet, osc, netscan
  pixelmap/         amostragem de frame -> bytes DMX por universo (rayon); bin `map_bench`
  mcp/              servidor MCP (rmcp, stdio) + `mcp install`
  serve/            barramento: HTTP + WebSocket JSON-RPC + monitor DMX + MCP em /mcp
  cli/              binário `spellcore` (play, net, commands, mcp, serve); a CLI e o `registry()`
                    completo moram em `lib.rs`, para o `gui` usar o MESMO registry
  gui/              binário `spellcaster`: a janela (tao + wry/WebView2) com o barramento em
                    processo — `spellcaster [show.spell] [--dir RAIZ]`
  bench/            Criterion + binários `jitter` e `throughput`
```

## Build — o `target/` NUNCA fica no repo

A pasta do repo está dentro do Google Drive: `target/` derruba o sync e enche a conta.
Exporte o alvo antes de qualquer comando cargo.

```powershell
# PowerShell
$env:CARGO_TARGET_DIR = "$env:TEMP\spellcore_target"
cargo test --workspace
cargo build --release
```

```bash
# bash (Git Bash)
export CARGO_TARGET_DIR="$TEMP/spellcore_target"
cargo test --workspace
```

O arquivo `spellcore/.cargo/config.toml` (não versionado, `.gitignore`) fixa o mesmo caminho
para quem esquecer da variável. `target/` está no `.gitignore` do repo como segunda barreira.

## Bench

```powershell
$env:CARGO_TARGET_DIR = "$env:TEMP\spellcore_target"
cargo run --release -p bench --bin jitter        # Clock a 60 Hz por 10 s: p50/p99/max e drift
cargo run --release -p bench --bin throughput    # 16 sACN + 16 Art-Net a 60 Hz por 10 s: %CPU
cargo run --release -p pixelmap --bin map_bench   # 100 000 px de um 1080p: ms/frame, gate de 2 ms
cargo bench -p bench                             # Criterion (curvas, packet, timeline)
cargo bench -p pixelmap                          # Criterion (100k px nearest e bilinear)
```

`map_bench` aceita `--pixels N --frames N --width N --height N --bilinear` e sai com código 1
se o p99 por frame passar de 2 ms.

Alvos da tabela do PRD (desktop x64) e o que foi medido nesta máquina (Windows 11, R0):

| Métrica | Alvo | Medido |
|---|---|---|
| Jitter entre frames, 60 Hz, p99 | < 1 ms | 0,42 ms (max 0,57 ms) |
| Drift em 10 s a 60 Hz | 0 frames | 0 |
| 16 sACN + 16 Art-Net a 60 Hz | < 3 % de um núcleo | 0,78 % |
| Boot até o primeiro frame DMX | < 2 s | 0,002 s |
| RSS em repouso | < 60 MB | 5,6 MB |
| `spellcore.exe` release | < 20 MB | 0,95 MB (R0) / 3,8 MB com Rhai e rmcp |
| Pixel mapping, 100 000 px a 60 Hz (CPU) | < 2 ms por frame | 0,105 ms p50, 0,316 ms p99 |

Criterion: `Timeline::apply` do show inteiro (219 tracks, 67 161 keyframes) em 3,73 µs;
`Keys::eval` 862 ns; `sacn::packet` 44 ns; `artnet::artdmx` 37 ns.

Conformidade ao vivo (o binário Rust contra o fixture do Python):

```
C:\Python313\python.exe tests/conformance/capture_sacn.py --secs 3
```

## Dependências (uma linha de justificativa cada)

| Crate | Onde | Por quê |
|---|---|---|
| `serde` + `serde_json` | engine, protocols, cli | o `.spell` e o `net --json` são JSON; nada na stdlib lê JSON |
| `schemars` | engine (registry) | schema JSON de cada comando, consumido pela CLI e pelas tools do MCP; reexportado em `engine::schemars` para que `cli` não pine a própria versão |
| `clap` (feature `derive`) | cli | parser de argumentos; cinco subcomandos em structs fixas |
| `rmcp` + `tokio` | mcp | SDK oficial do Model Context Protocol; é async, e o runtime `current_thread` mora só dentro de `mcp::serve_stdio` |
| `socket2` | protocols | `std::net::UdpSocket` não expõe `IP_MULTICAST_IF` nem `SO_REUSEADDR`, exigidos por sACN |
| `criterion` | bench, laser, pixelmap (dev) | medida estatística de jitter/latência exigida pelo PRD |
| `rayon` | pixelmap | 100 000 px por frame em ~590 universos independentes; pool de trabalho sem escrever um |
| `tao` + `wry` | gui (só `cfg(windows)`) | a janela e o WebView2 que o Windows 11 já traz; é o Tauri sem o Tauri (uma janela, um webview, nenhum menu nativo, updater ou tray para justificar o framework inteiro). Custo medido: ~200 crates novos no `Cargo.lock`, todos atrás do `cfg(windows)`. Licenças: `wry` Apache-2.0 OR MIT, `tao` **Apache-2.0 só** — o repositório é MIT, e Apache-2.0 é compatível, mas pede o aviso de atribuição no pacote (`packaging/`) |
| `axum` + `tokio` | serve | HTTP, WebSocket e o `tower::Service` do MCP streamable em um servidor só; o `rmcp` já exigia hyper/tower, e escrever handshake de WS na mão no servidor não paga |

Nada mais entra sem justificativa e sem medir o tamanho do binário.
Windows API (`timeBeginPeriod`, `SetThreadPriority`, `GetProcessTimes`) é declarada com
`extern "system"` direto — evita `windows-sys` inteiro por três símbolos.

## Contratos entre os crates (fixados antes de escrever código)

### `engine::clock`

```rust
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State { Stop, Play, Pause }

#[derive(Clone)]                       // clonável: transporte de outra thread age no mesmo relógio
pub struct Clock { /* Arc<inner> */ }

impl Clock {
    pub fn new(fps: u32) -> Clock;
    pub fn fps(&self) -> u32;
    pub fn time(&self) -> f64;                       // segundos
    pub fn state(&self) -> State;
    pub fn play(&self);
    pub fn pause(&self);
    pub fn stop(&self);
    pub fn locate(&self, t: f64);
    /// Chama f(t) a cada 1/fps s até stop() ou t >= duration. Bloqueia a thread chamadora.
    /// Prioridade alta + timeBeginPeriod(1) no Windows; sleep até a margem antes do alvo, spin no resto.
    /// A margem é CALIBRADA em runtime pelo overshoot medido do sleep (EWMA, presa em 0,3–2 ms):
    /// margem fixa de 1 ms custa ~6 % de um núcleo a 60 Hz só de spin. É o único botão do relógio.
    pub fn run<F: FnMut(f64)>(&self, f: F, duration: Option<f64>);
    /// Jitter do último run: (p50, p99, max) em segundos, e drift = frames perdidos.
    pub fn stats(&self) -> Stats;
}

pub struct Stats { pub p50: f64, pub p99: f64, pub max: f64, pub frames: u64, pub drift: i64 }
```

### `engine::universe`

```rust
pub struct Universe { pub number: u16, pub data: [u8; 512] }
impl Universe {
    /// addr é 1-based; clamp 0..255; ignora o que passar de 512 (igual ao Python).
    pub fn set(&mut self, addr: u16, values: &[f64]);
    pub fn set_bytes(&mut self, addr: u16, values: &[u8]);
}

pub struct Universes { /* Vec<Universe> ordenado por número, sem alocação no caminho quente */ }
impl Universes {
    pub fn new() -> Universes;
    pub fn get_or_create(&mut self, number: u16) -> &mut Universe;
    pub fn get(&self, number: u16) -> Option<&Universe>;
    pub fn iter(&self) -> impl Iterator<Item = &Universe>;
    pub fn numbers(&self) -> Vec<u16>;
}
```

### `engine::timeline`

```rust
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Curve { Linear, Hold, In, Out, InOut, Bezier }   // curva do segmento que CHEGA no keyframe
impl Curve {
    pub fn from_str(s: &str) -> Curve;                    // desconhecido -> Linear
    pub fn ease(self, u: f64, c: (f64, f64)) -> f64;      // u em 0..1
}
pub const BEZ: (f64, f64) = (0.42, 0.58);

pub enum Value { Num(f64), List(Vec<f64>), Text(String) }

pub struct Keyframe { pub t: f64, pub value: Value, pub curve: Curve, pub c: (f64, f64) }

pub struct Keys { /* keys ordenados + times: Vec<f64> */ }
impl Keys {
    pub fn new(keys: Vec<Keyframe>) -> Keys;
    pub fn len(&self) -> usize;
    /// Avaliação por partition_point (busca binária). Sem alocação para Value::Num.
    pub fn value(&self, t: f64) -> Option<&Value>;        // degrau/texto
    pub fn eval(&self, t: f64, out: &mut Vec<f64>) -> bool;  // interpolado; false se vazio
    pub fn crossed(&self, t0: f64, t1: f64) -> &[Keyframe];
}

pub struct Track { pub kind: String, pub universe: u16, pub address: u16, pub keys: Keys, /* .. */ }

pub struct Timeline { pub fps: u32, pub duration: Option<f64>, pub tracks: Vec<Track> }
impl Timeline {
    pub fn new(show: &Show) -> Result<Timeline, String>;
    /// Escreve os tracks DMX (tipos "dmx" e "artnet") nos Universes no instante t. Zero alocação.
    pub fn apply(&mut self, universes: &mut Universes, t: f64);
}
```

Tipos de track na R0: `dmx` e `artnet` (mesmo resolvedor, destinos diferentes).
`pyfx` é Python e não existe em Rust — o show MED GRUPO entra assado em tracks `dmx`
(ver `tests/conformance/gen.py`). Track de tipo desconhecido é ignorado com aviso.

### `engine::show` (`.spell` versão 1, mesmo arquivo do Python)

```rust
#[derive(Serialize, Deserialize)]
pub struct Show {
    pub name: String, pub fps: u32, pub duration: Option<f64>,
    pub outputs: Vec<OutputCfg>, pub tracks: Vec<serde_json::Value>,
    pub version: u32, /* resto preservado em #[serde(flatten)] extra */
}
pub enum OutputCfg { Sacn { universes: Vec<u16>, priority: u8, source_name: String, interfaces: Option<Vec<String>> },
                     ArtNet { targets: Option<Vec<String>>, broadcast: bool },
                     Unknown(String) }
pub fn load(path: &Path) -> Result<Show, String>;
pub fn save(path: &Path, show: &Show) -> Result<(), String>;
```

`load` aceita `version` <= 1; `version` maior é erro. `save` grava JSON compatível com o Python.

### `engine::registry`

```rust
pub struct Command {
    pub name: String, pub doc: String, pub schema: serde_json::Value,
    pub f: Box<dyn Fn(serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>,
}
pub struct Registry { /* Vec<Command> na ordem de inserção */ }
impl Registry {
    pub fn new() -> Registry;
    pub fn add<A: schemars::JsonSchema + serde::de::DeserializeOwned>(
        &mut self, name: &str, doc: &str,
        f: impl Fn(A) -> Result<serde_json::Value, String> + Send + Sync + 'static);
    pub fn get(&self, name: &str) -> Option<&Command>;
    pub fn schema(&self) -> serde_json::Value;                 // [{name, doc, params, mcp}]
    pub fn call(&self, name: &str, args: serde_json::Value) -> Result<serde_json::Value, String>;
    pub fn iter(&self) -> impl Iterator<Item = &Command>;
}
```

`add` deriva o schema com `schemars::schema_for!(A)`. Todo comando do produto passa por aqui;
CLI, OSC-API, GUI e MCP são clientes. Erro é `String` — sem `anyhow` na R0.

### `protocols`

```rust
pub trait Output: Send {
    fn send(&mut self, universe: u16, data: &[u8; 512]);
    fn close(&mut self);
}
```

`close(&mut self)` e não `close(self)` do PRD: `Box<dyn Output>` exige object safety.

```rust
// protocols::sacn
pub const PORT: u16 = 5568;
pub fn mcast(universe: u16) -> std::net::Ipv4Addr;                 // 239.255.{u>>8}.{u&255}
pub fn packet(universe: u16, data: &[u8], cid: &[u8; 16], seq: u8,
              source_name: &str, priority: u8) -> Vec<u8>;         // igual a sacn_packet.bin
pub fn parse(pk: &[u8]) -> Option<Packet>;                          // data (vector 4) e discovery (8)
pub fn interfaces() -> Vec<Ipv4Addr>;                               // IPv4 locais + 127.0.0.1
pub struct SacnOut;
impl SacnOut {
    pub fn new(universes: &[u16], ifaces: Option<Vec<Ipv4Addr>>) -> std::io::Result<SacnOut>;
    pub fn with(universes: &[u16], priority: u8, source_name: &str,
                ifaces: Option<Vec<Ipv4Addr>>) -> std::io::Result<SacnOut>;
    pub fn cid(&self) -> [u8; 16];
}
impl Output for SacnOut { .. }
pub struct SacnIn;
impl SacnIn { pub fn new(universes: &[u16]) -> io::Result<SacnIn>;
              pub fn get(&self, universe: u16) -> Option<[u8; 512]>;
              pub fn close(&mut self); }

// protocols::artnet
pub const PORT: u16 = 6454;
pub fn port_address(universe: u16) -> u16;                          // universe - 1
pub fn artdmx(universe: u16, data: &[u8], sequence: u8) -> Vec<u8>; // igual a artnet_packet.bin
pub fn artpoll() -> Vec<u8>;
pub fn artsync() -> Vec<u8>;
pub fn parse(pkt: &[u8]) -> Option<Packet>;
pub struct ArtNetOut;
impl ArtNetOut { pub fn new(targets: Option<Vec<String>>, broadcast: bool) -> io::Result<ArtNetOut>;
                 pub fn sync(&mut self); }
impl Output for ArtNetOut { .. }

// protocols::osc
pub enum Arg { Int(i32), Long(i64), Float(f32), Double(f64), Str(String), Blob(Vec<u8>),
               Bool(bool), Nil, Impulse }
pub fn message(address: &str, args: &[Arg]) -> Vec<u8>;
pub fn bundle(elements: &[Vec<u8>], tt: u64) -> Vec<u8>;
pub const IMMEDIATE: u64 = 1;
pub fn timetag(secs: f64) -> u64;
pub fn parse(data: &[u8]) -> Option<Parsed>;                        // Msg | Bundle
pub fn matches(pattern: &str, address: &str) -> bool;               // * ? [a-z] [!a-z] {a,b}
pub struct OscOut; impl OscOut { pub fn new(host: &str, port: u16) -> io::Result<OscOut>;
                                 pub fn send(&self, address: &str, args: &[Arg]);
                                 pub fn close(&mut self); }
pub struct OscIn;  impl OscIn  { pub fn new(port: u16) -> io::Result<OscIn>;
                                 pub fn on(&mut self, pattern: &str,
                                           f: impl Fn(&str, &[Arg]) + Send + 'static);
                                 pub fn close(&mut self); }

// protocols::netscan
pub struct Iface { pub name: String, pub ip: String, pub mask: String, pub gateway: Option<String> }
pub fn interfaces() -> Vec<Iface>;                                  // ipconfig / ip -j addr
pub fn parse_ipconfig(text: &str) -> Vec<Iface>;
pub fn parse_ip_addr(text: &str, route_text: &str) -> Vec<Iface>;
pub fn scan_artnet(timeout: Duration, ifaces: &[Iface]) -> Vec<Node>;
pub fn scan_sacn(timeout: Duration, ifaces: &[Iface]) -> Vec<Source>;
pub fn scan_etherdream(timeout: Duration) -> Vec<Dac>;
pub fn suggest(ifaces: &[Iface]) -> Vec<String>;                    // regra 2.x / 10.x + netsh
pub fn scan_all(timeout: Duration) -> Scan;                         // Serialize p/ `net --json`
pub fn report(s: &Scan) -> String;
```

Cada saída de rede roda em thread própria com fila de 2 frames por universo; quando a fila
enche, o frame velho é descartado (`send` nunca bloqueia o engine).

### `serve` — barramento

```
spellcore serve [--port 8000] [--dir .] [--show x.spell]
```

Um processo toca o hardware; toda página e toda IA falam com ele. Porta `0` = uma livre; ao
subir, o servidor imprime **no stderr** a linha `serve http://127.0.0.1:<porta>`.

Contrato (as outras frentes leem daqui, palavra por palavra):

```
HTTP  GET /commands -> Registry::schema()   GET /show -> show_get full   GET /<arquivo> -> <--dir>/<arquivo>
WS    /ws  request  {"id":7,"cmd":"locate","args":{"t":12.5}}
           resposta {"id":7,"result":...,"rev":n} | {"id":7,"error":"texto","rev":n}
           evento   {"event":"transport","data":<TransportState>} | {"event":"show","data":{"rev":n}} | {"event":"log","data":{"text":...}} | {"event":"widget","data":{"id","prop","value"}}
           binário  topic:u8 | universe:u16 LE | 512 bytes   (topic 1 = dmx de saída)
Comando `input {key, value}` no registry alimenta FrameHook::input do player vivo (chaves "widget:go", "key:Space", "module:laser/stat/fps").
```

| Detalhe | Regra |
|---|---|
| cabeçalho | toda resposta HTTP leva `Cache-Control: no-store` |
| `--dir` | padrão `.` = **raiz do repositório**, não `spellgui/web`: as páginas referenciam `../../design/tokens/spellcaster.css` e `../../shows/*.spell`, então a página abre em `/spellgui/web/index.html` e o que ela referencia sai do mesmo servidor. Em troca, o repositório inteiro (`.git` incluído) fica legível em 127.0.0.1 — aceitável enquanto o socket for só loopback |
| estático | só segmentos simples relativos: `..`, segmento vazio, `\` e `:` são recusados com 403 antes de tocar o disco; o caminho não é percent-decodificado. `/` = `index.html` |
| comando | cada request do WS chama `Registry::call`; erro do comando volta como `{"id","error"}` e **não** derruba a conexão |
| `play_show` | está no `BACKGROUND` do crate `mcp`: roda em thread e a resposta volta na hora, com `"<cmd> iniciado em background"`; erro vira evento `log` |
| `rev` | contador único, o do engine (`engine::edit::rev()`). Não há lista de comandos de leitura: o `serve` lê `rev()` antes e depois de cada chamada, **toda** resposta carrega o `rev` de depois, e o broadcast `{"event":"show","data":{"rev":n}}` sai **só** quando o número mudou. `load` e `show_get {file}` trocam o show inteiro e por isso incrementam |
| `transport` | a cada mudança de estado e a 10 Hz enquanto o player anda (sondagem de `player::current()`) |
| `widget` | `out.widget` do graph (o `Ev::Widget` que o sink da CLI recebe) vira `{"event":"widget","data":{"id","prop","value"}}` |
| monitor | um `FrameHook` global copia os universos do frame, no máximo a 40 Hz e só quando há cliente WS; sai como frame binário de 515 bytes |
| `/mcp` | `StreamableHttpService` do `rmcp` (feature `transport-streamable-http-server`) sobre o mesmo `Spell` do stdio, sem sessão e com resposta JSON: `POST /mcp` de um `initialize` devolve o JSON-RPC direto |
| `--show` | `load` (é o que `GET /show` lê) e o player parado em `t=0`; solta-se com o comando `resume` |

`serve` é subcomando da CLI, não comando do registry: é ele que monta o `Registry` (com
`play_show` e `net`) e o passa para `serve::serve`. O crate `serve` não tem lógica de produto —
só transporte.

**Fora de escopo, e por quê:** autenticação e TLS (por isso o socket abre **só em 127.0.0.1**:
sem token, abrir a LAN entregaria o hardware a quem estiver no wifi — `--host` entra junto com o
token); mDNS; mais de um show por processo (um `OPEN`, um `CURRENT`).

### Fixtures de conformidade (`tests/conformance/`, gerados por `gen.py`)

| Arquivo | Conteúdo |
|---|---|
| `medgrupo_u1.bin` | `u32 LE nframes` + `nframes × 512` bytes do universo 1, t = n/30 |
| `sacn_packet.bin` | pacote E1.31: universo 1, seq 0, CID `00..0f`, "Spellcaster", prio 100, dados `[0..255,0..255]` |
| `artnet_packet.bin` | ArtDmx: universo 1, seq 0, mesmos dados |
| `../../shows/medgrupo_r0.spell` | o mesmo show com o pyfx assado em 219 tracks `dmx` (67 161 keyframes) |

Regenerar: `C:\Python313\python.exe tests/conformance/gen.py`.

## `laser::trace` — FÓSFORO (bitmap → contorno → ILDA)

A metade do NDI→ILDA que não depende de SDK (`design/FUNCOES/ndi-ilda.md`, algoritmo
"Contornos"). Puro Rust, sem dependência nova; a entrada é um buffer RGBA cru, venha ele do NDI,
do Spout ou de um arquivo.

```rust
/// `color: None` amostra o pixel de origem sob cada ponto.
pub struct Opts { threshold: u8, epsilon: f32, max_points: usize, invert: bool, color: Option<(u8, u8, u8)> }
/// Caminhos em coordenadas ILDA, um por objeto, fechados; antes de blanking/optimize/safety.
pub fn paths(rgba: &[u8], w: usize, h: usize, o: &Opts) -> Vec<Vec<Point>>;
/// Quadro pronto para o `Feed`: caminhos + salto apagado + `optimize` + `safety` padrão.
pub fn trace(rgba: &[u8], w: usize, h: usize, o: &Opts) -> Vec<Point>;
```

Máscara por luma BT.601 · contorno externo por seguimento de borda de Moore, um caminho por
objeto (buraco não vira caminho) · simplificação Ramer–Douglas–Peucker com `epsilon` em pixels
da entrada · corte proporcional por caminho até `max_points` (contado **antes** do `optimize`) ·
ordenação por vizinho mais próximo · teto de `MAX_PATHS = 2000` caminhos por quadro. A imagem
entra centrada com a proporção preservada, no alcance ILDA ±32767 com Y para cima.
Determinístico: mesma entrada, mesmos bytes.

```
trace in.rgba WxH out.ild [--threshold N] [--epsilon F] [--max-points N] [--invert] [--sampled]
```

Medido em release: **4,4 ms** por quadro 1920×1080 com 20 objetos (meta 8 ms). O custo é das duas
passadas de quadro inteiro (máscara e varredura de candidatos, 2 Mpx cada); a vetorização em si
toca só os pixels de contorno — um objeto ou vinte dá o mesmo tempo. Teste: `laser/tests/trace.rs`,
que gera as entradas RGBA no próprio arquivo.

## CLI

```
spellcore play <show.spell> [--loop] [--osc-port N]   toca o show (sACN / Art-Net conforme "outputs")
spellcore net [--json] [--timeout N]                  varredura de rede + sugestões
spellcore commands                                    lista o registry (nome, doc, schema)
spellcore mcp                                         servidor MCP em stdio
spellcore mcp install --target desktop|code [--yes]   registra o servidor no Claude
spellcore serve [--port N] [--dir D] [--show S]       barramento HTTP + WebSocket + MCP
```

Os cinco subcomandos são structs `clap::Args` fixas. `PlayArgs` e `NetArgs` servem as duas
pontas: `clap` para o argv e `JsonSchema` + `Deserialize` para o registry. O transporte
(`load`, `show_get`, `pause`, `stop`, `locate`, `cue_go`, `transport_state`) continua no
registry mas **não** é subcomando: ele age no player vivo NESTE processo, e um segundo
processo não tem player nenhum. Quem os usa é o MCP (e, depois, a GUI).

---

# R1 — contratos fixados (orquestrador, antes de qualquer código)

Escopo da R1: cues, `.spell` completo, track `fx` em Rhai, Graph runtime, Player com transporte
remoto por OSC, CLI `play` headless. Sem GUI, sem mídia, sem laser (outro agente escreve
`spellcore/laser/`; nada aqui toca nesse diretório).

Novo membro do workspace: `script/` (crate `script`). Grafo de dependências da R1:

```
protocols  (autocontido)
engine     -> protocols, serde, serde_json, schemars
script     -> engine, rhai, serde_json
cli        -> engine, protocols, script, clap
bench      -> engine, protocols, script
```

`engine -> protocols` é novo e deliberado: o Player abre as saídas do `.spell` e escuta o
transporte remoto por OSC, exatamente como `spellcaster/player/player.py` importa `..protocols`.
`protocols` continua sem conhecer `engine`; não há ciclo. O engine continua sem conhecer GUI,
MCP, Rhai e laser.

## Ordem de avaliação de um frame (fixa; é o que a conformidade mede)

1. `Timeline::apply(&mut universes, t)` — tracks `dmx` e `artnet`.
2. Cada `FrameHook` na ordem em que foi registrado (tracks `fx` na ordem do `.spell`, depois o Graph).
3. Tracks de efeito colateral: `osc` e `media` não-Capture (envia quando o valor muda), `cue`
   (`crossed(prev, t)` dispara `CueList::go`).
4. `CueList::update(t)` escreve o snapshot corrente nos Universes.
5. O programmer (`player::Prog`): o override manual do operador, HTP por canal, por cima da
   timeline **e** da cue viva — o operador sobrepõe o que a cue está segurando.
6. Os ganchos **globais** (`player::hook_global`, o monitor do `serve`): o frame já completo,
   com o programmer dentro, antes de sair na rede.
7. I/O: cada universo escrito vai para todas as saídas.

Igual ao `_tick` + `_side` do Python.

## Tipos de track no `.spell` (R1)

| `type` | Campos | Semântica |
|---|---|---|
| `dmx` | `universe`, `address`, `keys` | R0, inalterado |
| `artnet` | idem | R0, inalterado |
| `osc` | `address` (texto), `keys`, `args?` | envia por OSC quando o valor muda (`last != v`) |
| `media` | `universe`, `address`, `clip`, `keys` (texto `play`/`replay`/`stop`) | Capture: ch1 = 10/20/28, ch2 = `clip`. Com `"player"` diferente de `"capture"` vira track OSC (`address/valor`) |
| `cue` | `keys` (texto `GO` ou número = índice) | `crossed(prev, t)` dispara a cue |
| `fx` | `script` (caminho relativo ao `.spell`), `universe` | script Rhai com estado persistente entre frames |
| `fixture` | reservado | parseado e ignorado com aviso (a F2 do Python resolve; o Rust não na R1) |

Track de tipo desconhecido continua sendo ignorado com aviso.

## `engine::hook` (novo módulo)

```rust
use crate::universe::Universes;

/// Escreve nos Universes uma vez por frame, depois de Timeline::apply.
/// É por aqui que `script` (Rhai fx e Graph) se pluga sem o engine conhecer Rhai.
pub trait FrameHook: Send {
    fn frame(&mut self, t: f64, uni: &mut Universes);
    /// Evento de entrada para o Graph. Chave: "widget:go", "key:Space", "osc:/spell/go",
    /// "midi:144/60", "marker:pico". Enfileirado; consumido no próximo frame(). Default: ignora.
    fn input(&mut self, _key: &str, _value: f64) {}
    /// locate/stop: zera o estado persistente. Default: no-op.
    fn reset(&mut self, _t: f64) {}
}

/// Saída de evento do Graph para o mundo. Uma implementação em `cli` (comando do registry,
/// OscOut, stderr); o engine só declara o trait porque não conhece o registry vivo nem a GUI.
#[derive(Clone, Debug, PartialEq)]
pub enum Ev {
    Cmd { name: String, args: serde_json::Value },   // nó `cmd`
    Osc { address: String, args: Vec<f64> },         // `out.osc`
    Widget { id: String, prop: String, value: f64 }, // `out.widget`
    Param { target: String, value: f64 },            // `out.param` ("fixture.canal")
    Notify { text: String },                         // `out.notify`
}
pub trait EventSink: Send { fn emit(&mut self, e: &Ev); }
```

`Ev` aloca `String`: eventos são raros (um GO, um OSC de entrada), não acontecem por frame.
`// ponytail: Ev com String ; virar índice no catálogo se algum graph passar a emitir por frame.`

## `engine::cues` (porte de `spellcaster/timeline/cues.py`)

```rust
pub struct Cue { pub name: String, pub fade: f64, pub wait: f64, pub follow: bool,
                 pub values: Vec<((u16, u16), Vec<f64>)> }   // ordem do JSON preservada
pub struct CueList { /* state, index, cue corrente, from, pending */ }
impl CueList {
    pub fn new(specs: &[serde_json::Value]) -> CueList;
    pub fn len(&self) -> usize;
    pub fn index(&self) -> i32;                       // -1 = nenhuma disparada
    /// Dispara a próxima cue (ou a de índice dado). O fade começa depois do `wait` dela.
    pub fn go(&mut self, t: f64, index: Option<usize>) -> bool;
    /// Avança o fade e escreve o snapshot corrente nos Universes. Zero alocação por frame.
    pub fn update(&mut self, t: f64, uni: &mut Universes);
    pub fn reset(&mut self);
}
```

Chave `"1/100"` -> `(1, 100)`; `"100"` -> `(1, 100)` (o `key()` do Python). Fade linear,
`u = 1` quando `fade <= 0`; ao chegar em `u >= 1` com `follow`, dispara a próxima.

## `engine::player`

```rust
pub struct Player { /* timeline, cues, hooks, saídas, Arc<Shared> */ }

#[derive(Clone)]
pub struct Handle { /* Arc<Shared>: clock + fila de cue + contadores */ }

#[derive(Serialize)]
pub struct TransportState { pub t: f64, pub state: &'static str, // "stop"|"play"|"pause"
                            pub cue: i32, pub frames: u64, pub fps: u32,
                            pub duration: Option<f64>, pub universes: Vec<u16> }

impl Player {
    /// Carrega timeline + cues e abre as saídas de `show.outputs` (sacn, artnet, osc).
    /// `base` = diretório do .spell (caminhos de `fx`/clipes são relativos a ele).
    pub fn new(show: Show, base: PathBuf, looping: bool) -> Result<Player, String>;
    /// Hooks de script/graph, antes de `start()`, na ordem em que devem rodar.
    pub fn hook(&mut self, h: Box<dyn FrameHook>);
    pub fn handle(&self) -> Handle;
    pub fn clock(&self) -> Clock;
    /// Sobe a thread de transporte e, se `osc_port` (argumento ou `transport.osc_port`),
    /// o OscIn com /spellcaster/play|pause|stop|locate f. Registra este player como o CURRENT.
    pub fn start(&mut self, osc_port: Option<u16>) -> Result<(), String>;
    pub fn wait(&self, timeout: Option<Duration>) -> bool;   // bloqueia até stop/fim
    pub fn close(&mut self);                                  // idempotente; limpa o CURRENT
}

impl Handle {
    pub fn play(&self); pub fn pause(&self); pub fn stop(&self);
    pub fn locate(&self, t: f64);
    pub fn cue_go(&self, index: Option<usize>);
    pub fn state(&self) -> TransportState;
}

/// Player vivo neste processo (o `CURRENT` do Python). Usado pelos comandos do registry.
pub fn current() -> Option<Handle>;
```

`locate`/`stop` chamam `reset()` em todo hook e `CueList::reset()`. `looping` sem `duration`
não repete (não há fim). O transporte remoto por OSC nunca toca nos Universes direto.

## Registry — os 46 comandos numa tabela

Auditoria de setembro/2026. Uma linha por comando: os argumentos com tipo e default (o schema que
o `schemars` gera do struct de `Args`, o mesmo que sai em `GET /commands`, nas tools MCP e em
`spellcore commands <nome>`), o que faz, o que devolve e o que dispara o comando na GUI. As
seções abaixo continuam sendo a explicação; esta tabela é o índice.

Regras de leitura: `arg:tipo` é obrigatório, `arg:tipo?` é opcional sem default, `arg:tipo=v` tem
default `v`. "Dispara na GUI" cita a página (`index.html` = TIMELINE, `teatro.html` = TEATRO,
`patchbay.html` = PATCHBAY, `laser.html` = LASER, `face.html` = FACE) e a tecla de
`design/SHORTCUTS.md` quando existe; `—` é comando que hoje só a IA (MCP), o OSC e a
`help.html` chamam.

| Comando | Argumentos | Faz | Devolve | Dispara na GUI |
|---|---|---|---|---|
| `load` | `file:string` | abre o `.spell` **e valida a timeline** | `{name, fps, duration, tracks, ignored}` | PATCHBAY: campo do caminho + `Abrir`; `serve --show` |
| `show_get` | `file:string=""`, `full:boolean=false` | abre (ou reusa) e resume; `full` devolve o `.spell` inteiro | resumo ou o show | TIMELINE, PATCHBAY e TEATRO no boot; `GET /show` |
| `resume` | — | solta o player pausado | estado do transporte | TIMELINE: `Play`, `Space`, `L` |
| `pause` | — | pausa o player | estado do transporte | TIMELINE: `Pause`, `Space`, `K` |
| `stop` | — | para o player | estado do transporte | TIMELINE: `Stop` |
| `locate` | `t:number` | salta para `t` segundos | estado do transporte | TIMELINE: régua, `←`/`→`, `Home`/`End` |
| `cue_go` | `index:integer?` | dispara a próxima cue, ou a de índice dado | estado do transporte | TEATRO: `GO` (`Enter`) |
| `transport_state` | — | lê o transporte sem tocar em nada | `{t, state, cue, frames, fps, duration, universes}` | — (a GUI recebe o evento `transport`) |
| `input` | `key:string`, `value:number=0` | entrega um evento aos ganchos do player (o Graph) | `{key, value}` | FACE: todo widget |
| `show_new` | — | zera o show aberto (sACN no universo 1, 60 s) | o show inteiro | — |
| `show_set` | `data:any` | **importa** um show inteiro (objeto ou texto JSON) | o show inteiro | — |
| `show_save` | `file:string=""` | grava; sem `file`, no caminho do último aberto | o caminho | TIMELINE: `Salvar`, `Ctrl+S` |
| `track_add` | `type:string="dmx"`, `universe:integer=1`, `address:integer=1`, `name:string=""` | acrescenta um track vazio | o índice | TIMELINE: `+Track` |
| `track_del` | `index:integer` | remove o track | o track removido | TIMELINE: `-Track` |
| `key_set` | `track:integer`, `t:number`, `value:any=null`, `curve:string="linear"` | cria ou substitui o keyframe em `t` | os keys do track | TIMELINE: `Ctrl+K`, arrastar, `Ctrl+V` |
| `key_del` | `track:integer`, `t:number` | apaga o keyframe em `t` (tolerância 1 ms) | quantos saíram | TIMELINE: `Delete`, `Ctrl+X` |
| `cue_set` | `index:integer?`, `name:string=""`, `fade:number=0`, `wait:number=0`, `follow:boolean=false`, `values:object={}` | cria (sem `index`) ou substitui uma cue | o índice | TEATRO: lista de cenas |
| `cue_del` | `index:integer` | remove a cue | a cue removida | TEATRO: apagar cena |
| `patch_add` | `name:string`, `profile:string`, `universe:integer=1`, `address:integer=1` | patcheia e revalida o patch inteiro | a grade do patch | TEATRO: `Adicionar` |
| `patch_del` | `name:string` | tira a fixture do patch | a entrada removida | TEATRO: apagar fixture |
| `patch_check` | — | grade do patch + o primeiro erro | `{rows, error}` | TEATRO: a grade |
| `profiles` | — | nomes dos `.json` em `profiles/` | lista de nomes | TEATRO: select de perfil |
| `show_patch` | `ops:array`, `rev:integer?` | JSON Patch (RFC 6902) no show aberto; tudo ou nada | `{rev, undo}` | TIMELINE, PATCHBAY e TEATRO: **toda** edição |
| `graph_get` | — | o `graph` do show | `{nodes, edges}` | — (o PATCHBAY lê o graph pelo `show_get full`) |
| `face_get` | — | a face inline, ou `faces/<nome>.face.json` | a face ou `null` | — (a FACE hoje busca o `.face.json` direto) |
| `profile_get` | `name:string` | o perfil inteiro (canais, `ranges`, `wheel`) | o JSON do perfil | TEATRO: widgets da fixture |
| `level_set` | `universe:integer=1`, `address:integer`, `values:array=[]` | escreve no override do programmer (HTP) | quantos canais | — (o TEATRO escreve por `fixture_set`) |
| `level_clear` | `universe:integer?` | solta o override de um universo, ou de todos | quantos canais saíram | TEATRO: `Solta` |
| `level_get` | `universe:integer?` | o override atual | `{"u/end": v}` | — |
| `cue_capture` | `name:string=""`, `fade:number=0`, `wait:number=0`, `follow:boolean=false` | o override vira cue nova e o override é solto | o índice | TEATRO: `Capturar` |
| `fixture_set` | `name:string`, `channel:string`, `value:number` | resolve fixture + canal do perfil e chama `level_set` | `{universe, address, value}` | TEATRO: sliders da fixture |
| `module_add` | `file:string=""`, `data:any=null` | valida um `module.json` e põe na tabela de módulos vivos | `{name, version}` | — |
| `module_del` | `name:string` | tira o módulo da tabela | o manifesto removido | — |
| `module_list` | — | módulos vivos | `[{name, type, version}]` | PATCHBAY: catálogo do nó `module` |
| `module_get` | `name:string` | o manifesto inteiro | `{name, type, version, parameters, values, commands}` | PATCHBAY: o nó `module` |
| `play_show` | `file:string`, `loop:boolean=false`, `osc_port:integer?` | **sobe** um player e toca até o fim ou Ctrl+C | `{name, frames, jitter_p99_ms, jitter_max_ms, drift}` | TIMELINE: `Play` sem player vivo; CLI `spellcore play` |
| `net` | `timeout:number=2`, `json:boolean=false` | varre a rede (interfaces, Art-Net, sACN, Ether Dream) | relatório de texto, ou o scan cru | CLI `spellcore net` |
| `graph_check` | — | compila o graph do show aberto sem rodar | `{nodes, error}` | PATCHBAY: a cada edição do graph |
| `laser_dacs` | `timeout:number=2` | procura DACs (Ether Dream por beacon, IDN por scan) | `[{type, id, host}]` | LASER: `Procurar` |
| `laser_open` | `dac:string`, `host:string=""`, `kpps:number=30`, `safety:any=null` | abre o DAC e sobe o feed (a safety nunca desliga) | `{feed, dac, pps}` | LASER: `Abrir` |
| `laser_play` | `feed:integer`, `file:string`, `fps:number=30`, `loop:boolean=false` | empurra os frames do `.ild` ao DAC | `{feed, file, frames, fps, loop}` | LASER: `Play` |
| `laser_stop` | `feed:integer` | para o playback; o DAC continua aberto | `{feed, playing:false}` | LASER: `Stop` |
| `laser_close` | `feed:integer` | para e fecha (apaga o DAC) | `{feed, dac, closed}` | LASER: `Fechar` |
| `laser_param` | `feed:integer`, `path:string`, `value:number` | um parâmetro do feed (geo, limit, safe, shutter) | `{feed, path, value, shutter}` | LASER: sliders e `Shutter` |
| `laser_stats` | `feed:integer` | estado do feed | `{playing, file, stat/*, jitter, cpu, safety}` | LASER: painel de stats (4 Hz) |
| `laser_files` | `dir:string=""` | lista os `.ild` do diretório (vazio = `shows/`) | `{dir, files:[{name, path, bytes}]}` | LASER: `Listar` |

### O que a auditoria corrigiu, e o que ficou de pé

- **`load(path)` virou `load(file)`.** O mesmo caminho de `.spell` se chamava `path` em `load` e
  `file` em `show_get`, `show_save`, `play_show` e `module_add`. `path` continua aceito por uma
  rodada (`#[serde(alias = "path")]`) e o `doc` do comando avisa que é deprecated; clientes
  atualizados: `spellgui/web/graph.js`, `serve::abre`, `cli/tests/serve.rs`,
  `engine/tests/patch.rs`. Dentro do repositório não sobrou chamador do nome velho: o alias fica
  só por script e sessão MCP já escritos, e sai na rodada 3. O `spellcaster/` Python não muda
  porque tem registry próprio (`spellcaster/core/registry.py`) e nunca chama o do Rust.
- **`track_add(label)` virou `track_add(name)`.** O campo que ele escreve no `.spell` chama-se
  `name`, e `patch_add` já usava `name` para a mesma ideia. `label` continua aceito por uma
  rodada, pelo mesmo mecanismo; cliente atualizado: `spellgui/web/timeline.js`.
- **Nada foi removido.** Os quatro pares suspeitos têm cliente e razão:
  `load` × `show_get{file}` (um valida a timeline, o outro só resume — o `doc` agora diz isso),
  `resume` × `play_show` (um solta o player vivo, o outro sobe um), `show_new` × `show_set`
  (um zera, o outro importa), `laser_stop` × `laser_close` (um para o arquivo, o outro apaga o
  DAC).
- **Ficou de pé de propósito:** `track_del(index)`/`cue_del(index)` nomeiam o alvo do comando e
  `key_set(track, t)` nomeia o container do keyframe — nomes diferentes para coisas diferentes.
  `resume`/`pause`/`stop` continuam três verbos sem argumento, e não um `transport(state=…)`,
  porque é assim que aparecem no mapa de atalhos, nas tools do MCP e nos endereços OSC.
- **Descrição por argumento:** os doze argumentos que ainda não tinham `description`
  (`track_add.universe`, `key_del.track`, `cue_set.name`, `cue_del.index`, `patch_add.universe`,
  `patch_del.name`, `level_set.universe`, os quatro de `cue_capture` e `laser_param.value`)
  ganharam a sua. `spellgui/web/test/help.test.js` falha se um comando ou argumento novo entrar
  sem texto.
- **Uma fonte para o texto:** `DOC_PLAY`, `DOC_NET` e `DOC_GRAPH_CHECK` em `cli/src/main.rs` são
  ao mesmo tempo o `doc` do registry e o `about` do clap, então `spellcore play --help` e
  `GET /commands` dizem a mesma frase (teste:
  `about_do_clap_e_doc_do_registry_sao_o_mesmo_texto`). Para os comandos que **não** são
  subcomando da CLI, `spellcore commands <nome>` imprime o `doc` e o schema de um só — é o
  `spellcore <cmd> --help` deles.

## `engine::registry::base()`

Assinatura muda para `pub fn base() -> Registry` (sem `Clock`: o transporte age no player vivo).
Comandos: `load` (R0), `resume`, `pause`, `stop`, `locate`, `cue_go`, `transport_state` e, desde
a R7, `show_get`. Os de transporte usam `player::current()`; sem player vivo devolvem
`Err("sem player em execucao")`. `show_get(file="", full=false)` abre o `.spell` (ou reusa o
último aberto neste processo, o `OPEN` do `spellcaster/mcp/tools.py`) e resume nome, fps,
duração, saídas, patch, tracks, cues e o transporte vivo; é ele que alimenta o resource
`spell://show`. `full=true` devolve o `.spell` inteiro (o que a GUI desenha).
`resume()` é o par do `pause` (o `Handle::play`): sem ele, quem pausava pelo registry só voltava
a tocar subindo outro player. Chama-se `resume` e não `play` porque `play` é o subcomando da
CLI que SOBE um player — esse é o `play_show`.
`play_show` **não** entra aqui: ele monta os hooks de `script` e é registrado pela CLI, como
`play` e `net` na R0.

### `engine::edit` — edição do show aberto

Porte de `spellcaster/gui/api.py` (mais a checagem de footprint de `fixtures/patch.py`), ligado
em `base()` por `edit::register`. Todo comando age no `OPEN`; sem show aberto, abre um novo
(o `SHOW = NEW` do Python), então a IA pode chamar `track_add` antes de qualquer arquivo.

| Comando | Faz | Devolve |
|---|---|---|
| `show_new` | zera: "novo show", sACN no universo 1, 60 s, `patch`/`cues`/`markers` vazios | o show inteiro |
| `show_set(data)` | substitui pelo JSON dado (objeto ou texto JSON); `migrate`; chaves `_x` caem | o show inteiro |
| `show_save(file="")` | grava (sem `file`, no caminho do último `load`/`show_get`/`show_save`) | o caminho |
| `track_add(type="dmx", universe=1, address=1, label="")` | track vazio no fim | índice |
| `track_del(index)` | remove | o track |
| `key_set(track, t, value=0, curve="linear")` | cria ou substitui o keyframe em `t` (`\|Δt\| < 1 µs`); `value` texto que é JSON vira JSON; lista ordenada | keys do track |
| `key_del(track, t)` | apaga em `t` (tolerância 1 ms) | quantos saíram |
| `cue_set(index?, name, fade, wait, follow, values)` | cria (sem `index`) ou substitui; `values` = `{"u/end": v \| [v...]}`, chave validada por `cues::key` | índice |
| `cue_del(index)` | remove | a cue |
| `patch_add(name, profile, universe=1, address=1)` | acrescenta e valida o patch inteiro; sobreposição ou estouro de 512 recusa e desfaz | a grade |
| `patch_del(name)` | tira pelo nome | a entrada |
| `patch_check()` | grade (`name`, `profile`, `universe`, `address`, `channels`) + `error` da primeira fixture que não entra | `{rows, error}` |
| `profiles()` | nomes dos `.json` em `profiles/` | lista |
| `show_patch(ops, rev?)` | JSON Patch (RFC 6902: `add`, `remove`, `replace`, `test`) sobre o show aberto; aplica numa cópia e só comita se todas passarem **e** o resultado ainda desserializar em `Show`; `rev` diferente da atual recusa (`"rev 3 != 5"`) | `{rev, undo}` |
| `graph_get()` | o `graph` do show (seção 10 do PRD) | `{nodes, edges}` |
| `face_get()` | `face` inline, ou `faces/<nome>.face.json` quando `face` é texto | a face ou `null` |
| `profile_get(name)` | o perfil inteiro (canais com `offset`, `fine`, `ranges`, `wheel`) para o cliente montar widget | o JSON do perfil |

Transporte, além dos da R0/R7: `resume()` continua o player pausado (a metade que faltava do
`pause`; chama-se `resume` porque `play` é o subcomando da CLI e o registry já tem `play_show`)
e `input(key, value)` entrega o evento a `FrameHook::input` do player vivo — é por ele que
widget, tecla e módulo alimentam o Graph.

`profiles/` é a primeira que existir entre: ao lado do `.spell`, um nível acima dele (`shows/` e
`profiles/` irmãos, como no repo e no pendrive), o cwd e a pasta do executável. O perfil é lido
para nome e footprint (`max(offset, fine) + 1`); `fixture_set` resolve o nome do canal no JSON
cru do próprio perfil; faixas e roda só viajam cruas no `profile_get`. Teste: `engine/tests/edit.rs`, binário próprio
porque `OPEN` é um por processo.

`show_patch` é a via preferida de edição: o cliente manda a lista de ops e recebe `{rev, undo}`,
onde `undo` já vem **na ordem de aplicação** — mandá-la de volta como veio desfaz a edição byte a
byte. A pilha de undo é do cliente; o engine só guarda o contador `rev` (`engine::edit::rev()`),
que sobe a cada edição bem-sucedida e é o que o barramento faz broadcast. Duas GUIs no mesmo show:
quem manda com `rev` velha leva erro em vez de sobrescrever a edição do outro. `show_set` continua
existindo para **importar** um show inteiro. Teste: `engine/tests/patch.rs`.

`graph_get` é leitura e mora no engine; editar o graph é `show_patch` em `/graph`. **Compilar** o
graph é `graph_check`, registrado pela CLI (só ela conhece o crate `script`), que compila o graph
do show aberto e devolve `{nodes, error}` — `error` é texto, não exceção, para o editor mostrar ao
lado do nó.

### `engine::module` — módulo = app declarado

Um app (o laser, o player de mídia, um Pi na rede) não é código do core: é um `module.json` que
declara `parameters` (o que se manda), `values` (o que ele devolve, só leitura) e `commands`
(`context: action | mapping | both`, o `CommandContext` do Chataigne). O endereço textual
`grupo/nome` é a identidade (regra 2 de `design/FUNCOES/README.md`) e o `type` do parâmetro é o que
gera o widget (regra 1): `float`, `int`, `bool`, `trigger`, `color`, `string`, `enum` (com
`options`); `min`/`max` é clamp físico e `norm` é a faixa útil do slider, separada dele. O core só
guarda a tabela dos módulos vivos deste processo; o PATCHBAY monta o nó a partir dela sem conhecer
o app, e os valores ao vivo chegam pelo comando `input` com a chave `module:<nome>/<path>`. Esta
seção é a especificação do formato; o exemplo vivo é `modules/laser.json`, o primeiro módulo
declarado.

| Comando | Faz | Devolve |
|---|---|---|
| `module_add(file="", data={})` | lê o manifesto (`file` = nome em `modules/` ou caminho de um `.json`; ou `data` inteiro), valida (endereço `a/b` sem espaço, `type` e `context` conhecidos, `min < max`, `default` dentro da faixa, `enum` com `options`) e põe na tabela; mesmo `name` substitui. Recusado devolve todos os erros de uma vez, cada um citando o path | `{name, version}` |
| `module_del(name)` | tira da tabela | o manifesto removido |
| `module_list()` | módulos vivos | `[{name, type, version}]` |
| `module_get(name)` | o manifesto inteiro | `{name, type, version, parameters, values, commands}` |

`modules/` resolve pela mesma regra de `profiles/` (`edit::recurso_dir`). Campos alheios do
manifesto do Chataigne (`hasInput`, `dependency`, `label`, `unit`, `args`) são lidos e
descartados. Teste: `engine/tests/module.rs`, binário próprio porque a tabela é uma por processo.

### `laser_*` — ILDA player (registrado pela CLI)

`spellcore/cli/src/laser_cmd.rs`. Mora na CLI, e não no engine, pela mesma razão de `play_show`
e `net`: o engine não conhece o crate `laser`. Um feed = um DAC aberto; a tabela `FEEDS` é para
o laser o que `player::current()` é para o transporte (um processo, N feeds).

| Comando | Faz | Devolve |
|---|---|---|
| `laser_dacs(timeout=2)` | Ether Dream por beacon (`netscan`) e IDN por scan | lista de `{type, id, host}` |
| `laser_open(dac, host="", kpps=30, safety?)` | abre o DAC e sobe o `Feed`; `safety` = `{min_size, max_intensity, zone}`, nunca desligável | `{feed, dac, pps}` |
| `laser_play(feed, file, fps=30, loop=false)` | thread que lê o `.ild` e faz `feed.push` no ritmo (o `.ild` não carrega taxa); sem `loop`, o fim do arquivo desarma o transporte e `laser_stats` volta a `playing:false` | `{feed, file, frames, fps, loop}` |
| `laser_stop(feed)` | para o playback; o DAC continua aberto | `{feed, playing:false}` |
| `laser_close(feed)` | para e fecha (o `Drop` do `Feed` apaga o DAC) | `{feed, dac, closed}` |
| `laser_param(feed, path, value)` | um parâmetro do feed (tabela abaixo) | `{feed, path, value, shutter}` |
| `laser_stats(feed)` | `playing`, arquivo, `stat/sent`, `stat/dropped`, `stat/errors`, jitter, cpu e a safety corrente | objeto |
| `laser_files(dir="shows")` | os `.ild` do diretório | `{dir, files:[{name, path, bytes}]}` |
| `clip_frame(clip, t=0, index?, fps=30)` | um quadro do `.ild` para desenhar (o previz da timeline): `index` escolhe direto, senão é `floor(t*fps)` com o clipe repetindo, a conta do player. `clip` sem caminho resolve na pasta do `.spell` aberto; não toca em DAC nenhum | `{clip, index, frames, name, points:[[x, y, r, g, b, blank]]}` com `x` e `y` normalizados em -1..1 |

`path` de `laser_param` (os mesmos paths de `modules/laser.json`, a declaração do módulo laser);
as chaves `stat/*` de `laser_stats` são os `values` do mesmo arquivo, só as que `FeedStats` conta:

| path | campo | faixa |
|---|---|---|
| `geo/x`, `geo/y` | `Transform.x`, `Transform.y` | unidades ILDA, ±32767 |
| `geo/scale` | `Transform.scale` | 0..4 |
| `geo/rot` | `Transform.rot` | graus, +-180 |
| `limit/r`, `limit/g`, `limit/b` | `Transform.color.0/.1/.2` | 0..1 |
| `safe/min_size` | `Safety.min_size` | unidades ILDA, 0..32767 |
| `safe/max_intensity` | `Safety.max_intensity` | 0..255 |
| `shutter` | zera `max_intensity` e devolve o valor guardado ao abrir | 0 ou 1 |

`curve/r|g|b`, `Blanking/*` e `Cor/Time Shift` da tabela do `ilda-player` ficam de fora: entram
quando o `Feed` tiver LUT de cor e o `optimize` for parametrizável em runtime.

Página: `spellgui/web/laser.html` + `laser.js` (DAC, kpps, arquivo, play/stop, sliders de
`geo/*` e `limit/*`, botão shutter, stats por polling a 4 Hz). Teste: `spellcore/cli/tests/laser.rs`,
binário próprio, sobe o `Emulator` Ether Dream do crate `laser` e conversa com o registry pelo
servidor MCP em outro processo (a tabela `FEEDS` é uma por processo).

### Programmer — a camada manual do operador (tema TEATRO DE PAPEL)

`Prog`, em `player.rs`: um `Option<u8>` por canal (valor e máscara de "tocado" na mesma
estrutura), aplicado **depois de `CueList::update` e antes do I/O**, HTP por canal: o
operador sobrepõe a cue viva no mesmo canal. Soltar um canal zera o valor preso no buffer no
frame seguinte, antes da timeline, para o que a timeline possui voltar a valer. O programmer não vai para o `.spell`: quem grava é a cue.

| Comando | Faz | Devolve |
|---|---|---|
| `level_set(universe=1, address, values)` | escreve no override a partir de `address`; HTP sobre a timeline e a cue viva; lista vazia escreve zero | quantos canais |
| `level_clear(universe?)` | solta um universo, ou todos | quantos canais saíram |
| `level_get(universe?)` | o override atual, no formato `values` de cue | `{"u/end": v}` |
| `cue_capture(name, fade, wait, follow)` | o override vira cue nova no fim da lista (mesma via de `cue_set`) e o override é solto | índice |
| `fixture_set(name, channel, value)` | resolve fixture do patch + nome do canal no perfil e chama `level_set` | `{universe, address, value}` |

Sem player vivo, os cinco devolvem `sem player em execucao`. Teste: `engine/tests/programmer.rs`,
binário próprio (`CURRENT` e `OPEN` são globais do processo).

## `script` (crate novo)

```rust
/// Track {"type":"fx","script":"medgrupo.rhai","universe":N}. Compila o .rhai uma vez,
/// mantém o estado entre frames, expõe `set(universe, addr, values)` ao script.
pub struct Fx;
impl Fx { pub fn new(path: &Path, universe: u16) -> Result<Fx, String>; }
impl engine::FrameHook for Fx { }

/// Graph da seção 10 do PRD, compilado do JSON de `show["graph"]`.
pub struct Graph;
impl Graph {
    pub fn new(spec: &serde_json::Value, sink: Box<dyn engine::EventSink>) -> Result<Graph, String>;
    /// Idem, com o diretório do show: é de lá que o nó `module` lê `modules/<nome>.json`.
    pub fn new_in(spec: &serde_json::Value, sink: Box<dyn engine::EventSink>, base: &Path)
        -> Result<Graph, String>;
    pub fn nodes(&self) -> usize;
}
impl engine::FrameHook for Graph { }

/// Hooks de um show, na ordem de execução: um Fx por track "fx" + o Graph, se houver.
pub fn hooks(show: &engine::Show, base: &Path, sink: Box<dyn engine::EventSink>)
    -> Result<Vec<Box<dyn engine::FrameHook>>, String>;
```

API mínima exposta ao script Rhai (fixa; o resto é escolha do crate, documentada aqui depois):
`set(universe, addr, values)` — `values` array de números ou número solto; mesma semântica de
`Universe::set` (clamp 0..255, truncagem para zero, 1-based, ignora o que passar de 512).

Rhai entra com `default-features = false` e o conjunto mínimo de features que faça o show rodar.
O tamanho do `spellcore.exe` release é medido antes e depois e vai para a tabela de dependências.
Medida de referência antes do Rhai (Windows x64, perfil release do workspace): **970 240 bytes**.

### Graph: catálogo fechado (seção 10 do PRD)

`in.widget | in.key | in.osc | in.midi (stub) | in.timer | in.marker | in.state` ·
`logic.and|or|not|latch|toggle|debounce|counter|select` · `math.map|curve|expr` ·
`time.delay|hold` · `cmd` · `out.widget|out.osc|out.param|out.notify` ·
`state` · `module` (os dois últimos são proposta desta rodada, `design/DECISOES.md`).

Semântica de `state`, `module` e `mute`: `design/DECISOES.md` (e o cabeçalho de
`script/src/graph.rs`, que é a mesma tabela do runtime).

JSON: `{"nodes":[{"id","type",...}], "edges":[["no.pino","no.pino"], ...]}`. Compila para lista
de nós em ordem topológica com pinos indexados por inteiro; avaliação por frame sem alocação;
`math.expr` compila o Rhai uma vez. Ciclo no grafo é erro na compilação. Entradas chegam por
fila `in.*` pré-alocada (`FrameHook::input`); saídas saem por fila `out.*` pré-alocada, drenada
para o `EventSink` no fim do frame. Aceite: 500 nós em menos de 0,1 ms por frame
(`bench/benches/graph.rs`).

#### PATCHBAY — o editor do graph (`spellgui/web/patchbay.html`)

O mesmo catálogo, como dado, em `spellgui/web/catalog.js`: config e pinos de cada tipo, mais o
**tipo de porta** (`trigger, bool, number, color, xy, frame, dmx`), que é regra do editor — o
runtime carrega tudo como `f64`. Cabo só liga tipos compatíveis (`trigger` e `bool` são o mesmo
fio); conversão é nó visível (`math.map`, `logic.toggle`), nunca coerção escondida. Um
`module.json` (frente `module`) vira nó por `CATALOG.moduleDef`. `spellgui/web/graph.js` tem o
modelo puro (`GM`: JSON Patch com inverso, ops de nó/cabo/chave, `Shift+Delete` religando, grupo
fechado) e a página. Toda edição sai como `show_patch {ops}` em `/graph/...` e a resposta `undo`
empilha; sem engine a mesma lista roda local. `x`, `y`, `group`, `mute`, `lock`, `label` moram no
nó (o runtime ignora chaves desconhecidas). Demonstração: `shows/patchbay_demo.spell`.
Testes: `node --test spellgui/web/test/graph.test.js`.

## CLI

```
spellcore play <show.spell> [--loop] [--osc-port N]
```

`play` é o nome do subcomando; o comando do registry chama-se `play_show` (a partir da R7 são
dois nomes literais no código da CLI, não mais uma tabela de alias). A CLI monta `Player`, pede os hooks a `script::hooks`, passa um `EventSink`
próprio (`Ev::Cmd` -> registry, `Ev::Osc` -> OscOut, `Ev::Notify`/`Widget`/`Param` -> stderr) e
imprime, **uma linha por segundo, só ASCII**:

```
t=  12.35s state=play cue=3 frames=372 jit_p99=0.41ms u=1,2
```

`--osc-port` sobrepõe `transport.osc_port` do `.spell`. Ctrl+C fecha o player e as saídas.

A CLI também registra `graph_check(graph?)` -> `{nodes, error}`: compila o graph com
`script::graph::Graph::new` sem rodar. Fica aqui pelo mesmo motivo de `play_show` e `net` — o
engine não vê `script`.

## O show MED GRUPO na R1

`shows/medgrupo.spell` ganha um track `{"type":"fx","script":"medgrupo.rhai","universe":1}` **ao
lado** do track `pyfx`, que continua no arquivo: o Python ignora `fx` e o Rust ignora `pyfx`, e
assim `tests/conformance/gen.py` continua regenerando os fixtures a partir do mesmo arquivo.
`shows/medgrupo_r0.spell` (219 tracks `dmx` assados) continua sendo o show da conformidade
offline e do bench Criterion.

Aceite do `fx`: `shows/medgrupo.rhai`, portado de `shows/medgrupo.py`, reproduz
`tests/conformance/medgrupo_u1.bin` nos 2577 frames do universo 1, byte a byte. Divergência de
ponto flutuante que sobreviver é documentada byte a byte (frame, canal, valor, motivo).

## Ambiente dos agentes

```bash
export PATH="$PATH:/c/Users/email/.cargo/bin"
export CARGO_TARGET_DIR="C:/Users/email/AppData/Local/Temp/spellcore_target"
```

Worktree: `C:\Users\email\AppData\Local\Temp\spellcaster-main`. Nunca `target/` dentro dele.
Sem `git commit`, sem `git push`. Testes e saída de bench só em ASCII (console cp1252).

---

# R7 — MCP (crate `mcp`)

Servidor MCP sobre o SDK oficial `rmcp` 3.2. Porte do `spellcaster/mcp/server.py`: **as tools
saem do registry**, nada de lógica de produto no crate.

```
spellcore mcp                                       # stdio: uma mensagem JSON-RPC por linha
spellcore mcp install --target desktop              # %APPDATA%\Claude\claude_desktop_config.json
spellcore mcp install --target code [--path P]      # .mcp.json do diretório corrente
```

| Superfície | Conteúdo |
|---|---|
| tools | uma por comando de `Registry::iter()`: `load`, `show_get`, `resume`, `pause`, `stop`, `locate`, `cue_go`, `transport_state`, `input`, os de edição de `engine::edit` (`show_new`, `show_set`, `show_save`, `track_add`, `track_del`, `key_set`, `key_del`, `cue_set`, `cue_del`, `patch_add`, `patch_del`, `patch_check`, `profiles`, `show_patch`, `graph_get`, `face_get`, `profile_get`, `level_set`, `level_clear`, `level_get`, `cue_capture`, `fixture_set`), os `module_*` de `engine::module` (`module_add`, `module_del`, `module_list`, `module_get`), `play_show`, `net`, `graph_check` e os `laser_*` de `cli/src/laser_cmd.rs` (`laser_dacs`, `laser_open`, `laser_play`, `laser_stop`, `laser_close`, `laser_param`, `laser_stats`, `laser_files`, `clip_frame`). `inputSchema` = o schema que o `schemars` gerou do struct de argumentos |
| resources | `spell://show` (o `.spell` aberto: fps, duração, saídas, patch, tracks, cues, transporte vivo), `spell://commands` (o registry inteiro em JSON), `spell://graph` (o `graph_get`) e `spell://face` (o `face_get`). Cada resource é uma chamada de comando do registry: o crate `mcp` não tem lógica de produto |
| erro | erro de comando volta como `isError: true` com o texto (o cliente lê); só rota inexistente vira erro JSON-RPC |
| `play_show` | bloqueia até o fim do show, então roda em thread e a tool volta na hora (o `BACKGROUND` do Python). Enquanto o MCP roda, a linha de status do `play` vai para o **stderr**: no stdio o stdout é o canal JSON-RPC |

Fora por enquanto, e por quê:

- **`face_patch`/`graph_patch`/`theme_set`** (PRD §10). `face_get` e `graph_get` existem (leem
  o que está no `.spell`), mas o `engine::show` não tem Theme serializado e o Graph só existe
  compilado dentro do `script`; sem estrutura para aplicar JSON Patch, essas três não teriam
  backend — quem edita o show inteiro por JSON Patch é o `show_patch`.
- **`mcp_install` como comando do registry.** No Python ele é `@command` e portanto uma tool.
  Aqui não: uma sessão de IA não deve reescrever a própria configuração — quem instala é o
  operador, pelo terminal.

Teste: `spellcore/cli/tests/mcp.rs` sobe o binário de verdade em stdio, faz `initialize`,
`tools/list`, `tools/call show_get` no `shows/medgrupo.spell`, lê `spell://commands` e
`spell://show`, e confere que `play_show` volta na hora sem sujar o stdout.

## `spellgui/web` — o catálogo de comandos como widget

As páginas (`face.html`, `index.html`) não conhecem comando nenhum: leem o `Registry::schema()`
e montam o formulário a partir do schema de cada `Args` (`widgets.js`: `number` com `min`/`max`
vira slider, `integer` vira spin, `boolean` vira toggle, `enum` vira select, comando sem
propriedade vira botão). O congelado desse schema é `spellgui/web/dev/commands.json`, que a
página usa quando abre sem engine.

| Comando | Faz | Devolve |
|---|---|---|
| `commands` (subcomando da CLI) | imprime `Registry::schema()`; é a fonte do `spellgui/web/dev/commands.json` | a lista de comandos com doc e schema |
| `commands <nome>` | só esse comando — é o `spellcore <cmd> --help` dos verbos que não viraram subcomando da CLI | `{name, doc, params}` |

```
spellcore commands > spellgui/web/dev/commands.json
```

`cli/tests/commands_json.rs` falha se algum comando sair do registry ou mudar de schema sem o
JSON ser regerado, e `spellgui/web/test/help.test.js` falha se um comando ou argumento entrar sem
texto de ajuda. A página que mostra tudo isso ao operador é `spellgui/web/help.html` (atalhos de
`design/SHORTCUTS.md` + o registry vivo, cada comando com um formulário que executa). O resto
(barramento, Face, como abrir) está em `spellgui/web/README.md`.
