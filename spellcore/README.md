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
  cli/              binário `spellcore`: play, net, commands, mcp
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
| `clap` (feature `derive`) | cli | parser de argumentos; quatro subcomandos em structs fixas |
| `rmcp` + `tokio` | mcp | SDK oficial do Model Context Protocol; é async, e o runtime `current_thread` mora só dentro de `mcp::serve_stdio` |
| `socket2` | protocols | `std::net::UdpSocket` não expõe `IP_MULTICAST_IF` nem `SO_REUSEADDR`, exigidos por sACN |
| `criterion` | bench, laser, pixelmap (dev) | medida estatística de jitter/latência exigida pelo PRD |
| `rayon` | pixelmap | 100 000 px por frame em ~590 universos independentes; pool de trabalho sem escrever um |

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

### Fixtures de conformidade (`tests/conformance/`, gerados por `gen.py`)

| Arquivo | Conteúdo |
|---|---|
| `medgrupo_u1.bin` | `u32 LE nframes` + `nframes × 512` bytes do universo 1, t = n/30 |
| `sacn_packet.bin` | pacote E1.31: universo 1, seq 0, CID `00..0f`, "Spellcaster", prio 100, dados `[0..255,0..255]` |
| `artnet_packet.bin` | ArtDmx: universo 1, seq 0, mesmos dados |
| `../../shows/medgrupo_r0.spell` | o mesmo show com o pyfx assado em 219 tracks `dmx` (67 161 keyframes) |

Regenerar: `C:\Python313\python.exe tests/conformance/gen.py`.

## CLI

```
spellcore play <show.spell> [--loop] [--osc-port N]   toca o show (sACN / Art-Net conforme "outputs")
spellcore net [--json] [--timeout N]                  varredura de rede + sugestões
spellcore commands                                    lista o registry (nome, doc, schema)
spellcore mcp                                         servidor MCP em stdio
spellcore mcp install --target desktop|code [--yes]   registra o servidor no Claude
```

Os quatro subcomandos são structs `clap::Args` fixas. `PlayArgs` e `NetArgs` servem as duas
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
6. I/O: cada universo escrito vai para todas as saídas.

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

## `engine::registry::base()`

Assinatura muda para `pub fn base() -> Registry` (sem `Clock`: o transporte age no player vivo).
Comandos: `load` (R0), `pause`, `stop`, `locate`, `cue_go`, `transport_state` e, desde a R7,
`show_get`. Os de transporte usam `player::current()`; sem player vivo devolvem
`Err("sem player em execucao")`. `show_get(file="", full=false)` abre o `.spell` (ou reusa o
último aberto neste processo, o `OPEN` do `spellcaster/mcp/tools.py`) e resume nome, fps,
duração, saídas, patch, tracks, cues e o transporte vivo; é ele que alimenta o resource
`spell://show`. `full=true` devolve o `.spell` inteiro (o que a GUI desenha).
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
| `profile_get(name)` | o perfil inteiro (canais com `offset`, `fine`, `ranges`, `wheel`) para o cliente montar widget | o JSON do perfil |

`profiles/` é a primeira que existir entre: ao lado do `.spell`, um nível acima dele (`shows/` e
`profiles/` irmãos, como no repo e no pendrive), o cwd e a pasta do executável. O perfil é lido
para nome e footprint (`max(offset, fine) + 1`); `fixture_set` resolve o nome do canal no JSON
cru do próprio perfil; faixas e roda só viajam cruas no `profile_get`. Teste: `engine/tests/edit.rs`, binário próprio
porque `OPEN` é um por processo.

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
`time.delay|hold` · `cmd` · `out.widget|out.osc|out.param|out.notify`.

JSON: `{"nodes":[{"id","type",...}], "edges":[["no.pino","no.pino"], ...]}`. Compila para lista
de nós em ordem topológica com pinos indexados por inteiro; avaliação por frame sem alocação;
`math.expr` compila o Rhai uma vez. Ciclo no grafo é erro na compilação. Entradas chegam por
fila `in.*` pré-alocada (`FrameHook::input`); saídas saem por fila `out.*` pré-alocada, drenada
para o `EventSink` no fim do frame. Aceite: 500 nós em menos de 0,1 ms por frame
(`bench/benches/graph.rs`).

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
| tools | uma por comando de `Registry::iter()`: `load`, `show_get`, `pause`, `stop`, `locate`, `cue_go`, `transport_state`, os de edição de `engine::edit` (`show_new`, `show_set`, `show_save`, `track_add`, `track_del`, `key_set`, `key_del`, `cue_set`, `cue_del`, `patch_add`, `patch_del`, `patch_check`, `profiles`), `play_show`, `net`. `inputSchema` = o schema que o `schemars` gerou do struct de argumentos |
| resources | `spell://show` (o `.spell` aberto: fps, duração, saídas, patch, tracks, cues, transporte vivo) e `spell://commands` (o registry inteiro em JSON) |
| erro | erro de comando volta como `isError: true` com o texto (o cliente lê); só rota inexistente vira erro JSON-RPC |
| `play_show` | bloqueia até o fim do show, então roda em thread e a tool volta na hora (o `BACKGROUND` do Python). Enquanto o MCP roda, a linha de status do `play` vai para o **stderr**: no stdio o stdout é o canal JSON-RPC |

Fora por enquanto, e por quê:

- **HTTP streamable.** O `rmcp` traz `StreamableHttpService`, mas é um `tower::Service`: virar
  servidor ainda exige axum/hyper (feature `server-side-http`, +11 crates). Entra quando houver
  MCP remoto no Pi, junto com o `serve` da GUI.
- **`face_get`/`face_patch`/`graph_get`/`graph_patch`/`theme_set`** (PRD §10). O `engine::show`
  não tem Face nem Theme serializados, e o Graph só existe compilado dentro do `script`; sem
  estrutura para ler e aplicar JSON Patch, essas tools não teriam backend.
- **`mcp_install` como comando do registry.** No Python ele é `@command` e portanto uma tool.
  Aqui não: uma sessão de IA não deve reescrever a própria configuração — quem instala é o
  operador, pelo terminal.

Teste: `spellcore/cli/tests/mcp.rs` sobe o binário de verdade em stdio, faz `initialize`,
`tools/list`, `tools/call show_get` no `shows/medgrupo.spell`, lê `spell://commands` e
`spell://show`, e confere que `play_show` volta na hora sem sujar o stdout.
