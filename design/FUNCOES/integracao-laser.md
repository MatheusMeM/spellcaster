# Integração do laser com o orquestrador (rodada 6)

O projetor da rodada 5 vira o módulo `laser/1` do graph (`orquestrador.md`), e cada binding de tecla/MIDI vira uma **rota** `entrada → filtro → endereço`. Nada de UI nova do PATCHBAY: é contrato e dado. Fonte única das ações: `app.js` (`Bind.def(id, label, fn, {addr, ctx, …})`); `Bind.manifest()` gera `module.json` e `Bind.graph()` gera o trecho `graph` do `.spell`. Os dois arquivos versionados em `design/laser/` (`module.json`, `graph.json`) são o dump do protótipo com os bindings padrão, e `tests/test_laser_graph.py` valida os dois.

## Regras

- **Endereço** = `laser/<instância>/<nome>`. O nome é o da CLI (`--kpps`, `limit --r`), agrupado por `/` como o rótulo da tabela de `ilda-player.md §1` (`Cor/Escala R` → `limit/r`, `Dispositivo/Fila` → `queue`, `Geometria/Escala` → `geo/scale`, `ILDA/FPS real` → `fps`, `ILDA/Pontos por frame` → `points`). Nome novo só para o que a tabela não nomeia (`arm`, `play`, `clip`, `net/*`, `dmx/addr`, `temp`, `emitting`).
- **Tipo**: `trigger` dispara, `toggle` inverte (ou recebe bool), `value` recebe número (ou trigger com argumento, ex.: `kpps {step}`). Regra 3 de `README.md`.
- **`context`** (Chataigne): `action` só aceita disparo, `mapping` só aceita valor contínuo, `both` os dois. Mais dois contextos nossos que **não** entram em `commands`: `value` (somente leitura → `values` do manifesto) e `view` (só do previz → bloco `view` do `.spell`).
- **Tipos de porta** do graph: `trigger`, `toggle` (bool), `number`. Cabo só liga tipo compatível: `midi/note:*` e `key/*` são `trigger`; `midi/cc:*` é `number`. `number → number` passa por `filter.lag` (CC contínuo); `trigger → toggle` inverte; `trigger → number` exige argumento (`step`, `file`); `number → trigger` não existe sem filtro de limiar (o PRD §10 não tem; ponto aberto).
- **Porta** escreve-se `<uid>/<porta>` (`midi/cc:1:7`, `laser/1/kpps`), não `uid.port` como `orquestrador.md §5`: o nome da porta já leva `/` (`limit/r`), e assim a porta **é** o endereço do registry (regra 2). Registrado em `DECISOES.md`.
- **Retorno** (LED/fader do controlador): `Bind.feedback` manda `get()` de volta pelo mesmo binding: bool → nota/CC 127|0 (LED), número → CC 0..127 (fader motorizado).

## Tabela: porta ou componente → endereço

Coluna "de onde": `B` = id de `Bind.def` em `app.js`, `K` = `userData.key` em `body.js`/`optics.js`, `P` = pino do Pino (`pino3d.js`).

| Porta / componente / ação | De onde | Endereço | Tipo | context | Porta do graph | Retorno |
|---|---|---|---|---|---|---|
| chave (KEY) | K `keyswitch` · B `key.toggle` | `laser/1/arm` | toggle | both | trigger/toggle | LED |
| rocker + powerCON (AC IN) | K `power` · B `power.toggle` | `laser/1/power` | toggle | both | trigger/toggle | LED |
| play/pausa | B `play.toggle` | `laser/1/play` | toggle | both (`dependency: arm`) | trigger/toggle | LED |
| obturador | K `shutter` · B `shutter` (novo) | `laser/1/shutter` | toggle | both (`dependency: arm`) | trigger/toggle | LED |
| galvos (kpps) | K `galvo` · B `kpps` (cc), `kpps.up`, `kpps.down` | `laser/1/kpps` | value | both (`{step}` no disparo) | number (Lag) / trigger+args | fader |
| ILDA IN (arquivo) | K `ilda` · B `demo` | `laser/1/clip` | trigger `{file}` | action | trigger+args | — |
| ILDA IN (diálogo) | B `file.open` | — | | | | | 
| módulo R/G/B: limite | K `r` `g` `b` · B `lim.r` `lim.g` `lim.b` | `laser/1/limit/r` `g` `b` | value | mapping | number (Lag) | fader |
| módulo R/G/B: curva | K `r` `g` `b` · B `curve.r` `curve.g` `curve.b` (novo) | `laser/1/curve/r` `g` `b` | value | mapping | number (Lag) | fader |
| tamanho | B `size` | `laser/1/geo/scale` | value | mapping | number (Lag) | fader |
| DMX IN (endereço) | K `dmxin` · B `dmx.addr` (novo) | `laser/1/dmx/addr` | value int 1..512 | mapping | number (Lag) | fader |
| driver dos galvos (buffer) | K `galvodrv` · B `queue` (novo) | `laser/1/queue` | value int 1..8 | mapping | number (Lag) | fader |
| NET (RJ45) | K `rj45` · B `net.ndi` `net.spout` `net.artnet` `net.sacn` | `laser/1/net/ndi` `spout` `artnet` `sacn` | toggle | both | trigger/toggle | LED |
| interlock | K `interlock` · B `lock.toggle` | `laser/1/interlock` | bool, somente leitura | value | (sem entrada) | LED |
| ventoinha / temperatura | K `fan` · B `temp` (novo) | `laser/1/temp` | float °C, somente leitura | value | — | — |
| fps real | OLED OUTPUT · B `fps` (novo) | `laser/1/fps` | float Hz, somente leitura | value | — | — |
| pontos por frame | OLED OUTPUT · B `points` (novo) | `laser/1/points` | int, somente leitura | value | — | — |
| emitindo (FEIXE LIBERADO) | HUD · B `emitting` (novo) | `laser/1/emitting` | bool, somente leitura | value | — | LED |
| vista da câmera | B `cam.show` `cam.rear` `cam.inside` · K `lid` | `laser/1/cam/view` | enum show/rear/inside | view | — (bloco `view`) | — |
| névoa | B `fog` | `laser/1/cam/fog` | float 0..1 | view | — (bloco `view`) | — |

O binding `lock.toggle` continua existindo no protótipo (simula tirar o plugue), mas não vira rota: interlock é sensor, entra no `.spell` só como valor.

## O que não vira endereço, e por quê

| Coisa | De onde | Por quê |
|---|---|---|
| câmera: girar, pan, zoom, enquadrar, vistas padrão, sentido da roda | B `cam.rot*` `cam.rot90*` `cam.pan*` `cam.fit` `cam.front…iso` `cam.zoomIn/Out` `cam.reverse` | gesto do viewer, não estado do show; o que se salva é só a vista (`cam/view`) e a névoa, no bloco `view`, que o PRD §10 e `orquestrador.md §5` separam da lógica de propósito |
| splash | `wallSplash`, `skipSplash` | Entry do arco 5E; o `.spell` não sabe que existe |
| Pino e seus pinos | P `pino`, `pino.ilda` `pino.ndi` `pino.orq` `pino.cues` `pino.nfo` `pino.bye` | menu do programa; nenhum pino muda o show |
| OLED, encoder, BACK | K `oled` `enc` `back` · B `oled.up` `oled.down` `oled.ok` `oled.back` | menu físico do painel; as páginas dele já são endereços (`dmx/addr`, `net/*`, `kpps`) |
| ILDA OUT, DMX OUT, USB | K `ildathru` `dmxout` `usb` | passagem de sinal e firmware: sem estado no show (`--chain` é flag da CLI, sem estado no protótipo) |
| abertura, face, lateral, tampa, parafusos | K `aperture` `front` `side` `lid` | painéis de informação; `lid` só troca a vista |
| mesa, dicroicos, espelho de dobra, driver do diodo, placa DAC, fonte | K `bench` `dichro` `fold` `pcb` `dac` `psu` | mecânica e serviço; a placa DAC é o próprio nó `laser/1`; telemetria da fonte fica fora do v1 |
| velocidade do driver dos galvos | slider `speed` em `galvodrv` | calibra o passa-baixa do previz (resposta do galvo simulada), não é parâmetro do aparelho |
| bindings, info, Esc, MIDI conectar, reset, orquestrador | B `bind` `nfo` `esc` `midi.connect` `bind.reset` `orq` | UI do protótipo |

## Manifesto e graph

- `module.json`: formato de `orquestrador.md §5` (`parameters`, `values` com `readOnly`, `commands` com `context`, `dependency`). Diferença: cada `dependency` leva `target` (o §5 omite porque no Chataigne a dependência mora dentro do parâmetro).
- `graph.json`: `nodes[]` (`key`, `midi`, `laser/1` com `params` completos, `lag/N`), `wires[]` (`{from, filter?, to, args?, muted}`), `states[]` vazio (o PRD §10 não tem nó de estado; `DECISOES.md`), `view` em bloco separado com posições e `cam`/`fog` do previz.
- CLI equivalente: `spell graph add laser/1` · `spell graph wire midi/cc:1:7 laser/1/kpps --filter lag`.
- Regenerar os dois: abrir o protótipo com `#orq` e copiar o `<pre>` do painel; ou `errwrap.py` + Chrome headless `--dump-dom` (pré `id=DUMP`).
