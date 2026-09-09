# Spellcaster

Spellcaster é um media server de luz e laser portátil da Feitiçaria Industrial. Ele roda em Python 3.13 com a stdlib como primeira opção e fala sACN, Art-Net, OSC e ILDA (Ether Dream). O plano inclui GUI web com skins, servidor MCP gerado do registro de comandos e uma versão Lite para Raspberry Pi operada só por CLI via SSH.

Pacote Python: `spellcaster`. CLI: `spell`. Arquivo de show futuro: `.spell` (JSON).

## Estado atual

Fases F0 (fundação e protocolos) e F1 (análise de rede) estão concluídas. Módulos existentes:

| Módulo | Função |
|---|---|
| `spellcaster/cli.py` | CLI `spell`: verbos `play`, `net` e `commands`, gerados do registry |
| `spellcaster/core/clock.py` | Relógio único com tick fixo e transporte play/pause/stop/locate |
| `spellcaster/core/registry.py` | `@command`: registro de comandos com schema tipado |
| `spellcaster/core/universe.py` | Buffers DMX de 512 canais, endereço 1-based |
| `spellcaster/core/engine.py` | Loop `look(t)` → universos → saídas |
| `spellcaster/protocols/sacn.py` | sACN E1.31: saída multicast, entrada, discovery |
| `spellcaster/protocols/artnet.py` | Art-Net 4: ArtDmx out/in, ArtPoll, ArtSync |
| `spellcaster/protocols/osc.py` | OSC 1.0 sobre UDP: mensagens, bundles, pattern matching |
| `spellcaster/protocols/netscan.py` | Interfaces, nós Art-Net, fontes sACN, Ether Dream, sugestões de IP |
| `spellcaster/protocols/ilda/frame.py` | Ponto e frame ILDA, otimização de scan, safety |
| `spellcaster/protocols/ilda/ild.py` | Leitura e escrita de `.ild` (formatos 0, 1, 2, 4, 5) |
| `spellcaster/protocols/ilda/generators.py` | Geradores de figura e o laser do show MED GRUPO |
| `spellcaster/protocols/ilda/etherdream.py` | Cliente Ether Dream (TCP) e emulador para testes |
| `shows/medgrupo.py` | Show MED GRUPO RJ em modo compatibilidade (`look(t)`, `DUR`) |
| `seed/` | Código de origem (gerador sACN, calibração, ILDA) mantido como referência |

F2 em diante (perfis e patch, timeline, GUI, MCP, portátil, Lite) está pendente. Consulte `ROADMAP.md`.

## Como rodar no Windows

Sem venv. Use o interpretador global.

```
C:\Python313\python.exe -m spellcaster.cli --version
C:\Python313\python.exe -m spellcaster.cli commands
C:\Python313\python.exe -m spellcaster.cli play shows\medgrupo.py --fps 30
C:\Python313\python.exe -m spellcaster.cli play shows\medgrupo.py --loop --universes 1,2
C:Python313python.exe -m spellcaster.cli net --timeout 2
C:Python313python.exe -m spellcaster.cli net --as_json
C:\Python313\python.exe -m spellcaster.protocols.ilda.generators saida.ild
C:\Python313\python.exe -m spellcaster.protocols.ilda.generators saida.ild 20000 10000
```

- `play` toca um show `.py` por sACN. Opções: `--fps` (padrão 30), `--loop`, `--universes` (lista separada por vírgula, padrão `1`). O show precisa definir `look(t)`; `DUR` é opcional.
- `commands` imprime o schema do registry em JSON.
- `net` aceita `--timeout` (segundos, padrão 2) e `--as_json`. Também roda como `-m spellcaster.protocols.netscan`.
- `generators` grava o laser MED GRUPO em `.ild`. Os dois argumentos opcionais são a meia-largura e a meia-altura da tela em unidades ILDA (padrão 20000 e 10000).

Com o pacote instalado (`pip install -e .`), `spell` substitui `C:\Python313\python.exe -m spellcaster.cli`.

## Testes

```
C:\Python313\python.exe -m unittest discover -s tests -v
```

43 testes. Loopback UDP em 127.0.0.1 faz o papel de mock. Os testes não imprimem caracteres fora de ASCII.

## Contratos fixos

- Universos numerados a partir de 1 (sACN). Art-Net converte para port-address internamente.
- Toda saída de protocolo expõe `send(universe: int, data: bytes)` e `close()`.
- Todo comando do produto passa pelo `spellcaster.core.registry` (`@command`). CLI, OSC-API, GUI e MCP são clientes do registry e não implementam lógica própria.
- O engine não conhece GUI. Nada em `core`, `protocols`, `fixtures`, `timeline` importa de `gui` ou `mcp`.
- Stdlib antes de dependência. `pyproject.toml` declara zero dependências.
- Toda lógica não trivial deixa um teste `unittest` em `tests/`.
- Simplificação deliberada leva comentário `# ponytail: <limite> ; <quando trocar>`.

## spellcore (Rust)

O core do produto está sendo reescrito em Rust em `spellcore/` (PRD, fase R0: engine, protocolos,
CLI `net` e bench). O pacote Python acima continua como implementação de referência e gerador dos
fixtures de conformidade. A pasta do repo está no Google Drive, então o `target/` do cargo fica
fora dela.

```powershell
$env:CARGO_TARGET_DIR = "$env:TEMP\spellcore_target"
cd spellcore
cargo test --workspace
cargo build --release

# binario em %TEMP%/spellcore_target/release/spellcore.exe
spellcore play ..\shows\medgrupo_r0.spell
spellcore net --timeout 2
spellcore net --timeout 2 --json
spellcore commands

# bench (gate do PRD)
cargo run --release -p bench --bin jitter
cargo run --release -p bench --bin throughput
cargo bench -p bench
```

Fixtures de conformidade (regerar com `C:\Python313\python.exe tests/conformance/gen.py`):
`tests/conformance/medgrupo_u1.bin`, `sacn_packet.bin`, `artnet_packet.bin` e o show assado
`shows/medgrupo_r0.spell`. Para validar o binário Rust ao vivo contra o Python:

```
C:\Python313\python.exe tests/conformance/capture_sacn.py --secs 3
```

Detalhes de árvore, contratos e números do bench em `ARCHITECTURE.md`; dependências e
justificativa em `spellcore/README.md`.

## Documentos

- `ARCHITECTURE.md`: árvore, fluxo de dados, assinaturas públicas e simplificações por protocolo.
- `ROADMAP.md`: decisões de stack, fases F0 a F7, riscos.
- `CLAUDE.md`: regras do repositório.
