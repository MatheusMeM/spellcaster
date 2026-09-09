# spellcore — core do Spellcaster em Rust (R0)

Workspace Cargo com o engine, os protocolos, a CLI e o bench. Sem GUI, sem Godot, sem mídia.
O pacote Python `spellcaster/` continua sendo a implementação de referência: o spellcore tem
que reproduzir byte a byte a saída dele (fixtures em `tests/conformance/`).

```
spellcore/
  Cargo.toml        workspace (edition 2021, release: lto thin, codegen-units 1, panic abort)
  engine/           clock, universe, timeline (keys/curvas), show (.spell v1), registry
  protocols/        trait Output, sacn, artnet, osc, netscan
  cli/              binário `spellcore`: play, net, commands
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
cargo bench -p bench                             # Criterion (curvas, packet, timeline)
```

Alvos da tabela do PRD (desktop x64) e o que foi medido nesta máquina (Windows 11, R0):

| Métrica | Alvo | Medido |
|---|---|---|
| Jitter entre frames, 60 Hz, p99 | < 1 ms | 0,42 ms (max 0,57 ms) |
| Drift em 10 s a 60 Hz | 0 frames | 0 |
| 16 sACN + 16 Art-Net a 60 Hz | < 3 % de um núcleo | 0,78 % |
| Boot até o primeiro frame DMX | < 2 s | 0,002 s |
| RSS em repouso | < 60 MB | 5,6 MB |
| `spellcore.exe` release | < 20 MB | 0,95 MB |

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
| `schemars` | engine (registry), cli | schema JSON de cada comando, consumido pela CLI e depois pelo MCP |
| `clap` | cli | parser de argumentos com subcomandos gerados do registry em runtime |
| `socket2` | protocols | `std::net::UdpSocket` não expõe `IP_MULTICAST_IF` nem `SO_REUSEADDR`, exigidos por sACN |
| `criterion` | bench (dev) | medida estatística de jitter/latência exigida pelo PRD |

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
    pub name: String, pub doc: String, pub schema: serde_json::Value, pub mcp: bool,
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
pub fn discover(timeout: Duration) -> Vec<Source>;

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
pub struct ArtNetIn;  // .frame(universe) -> Option<Vec<u8>>
pub fn poll(timeout: Duration) -> Vec<Reply>;

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
spellcore play <show.spell> [--loop]     toca o show (sACN / Art-Net conforme "outputs")
spellcore net [--json] [--timeout N]     varredura de rede (Art-Net, sACN, Ether Dream) + sugestões
spellcore commands                       lista o registry (nome, doc, schema)
```

Os subcomandos são construídos em runtime a partir de `Registry::schema()`.
