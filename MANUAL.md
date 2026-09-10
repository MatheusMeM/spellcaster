# Manual do operador — Spellcaster

O programa é o aparelho. A tela principal é o projetor de laser em 3D
(`spellgui/web/laser3d/app.html`): a traseira dele é o menu, a tampa aberta são as preferências,
e a parede do fundo mostra o que sai pelo laser. As outras páginas (timeline, patchbay, teatro,
face, midi, ajuda) e a CLI `spellcore` completam o resto. Este manual só descreve o que existe no
código desta árvore.

## Abrir o programa

**Windows, janela própria.** `spellcaster.exe` (crate `spellcore/gui`, compilado do código com
`cargo build --release -p gui`) sobe o barramento em processo, numa porta livre em `127.0.0.1`, e
abre uma janela direto em `/spellgui/web/laser3d/app.html`. Fechar a janela mata o processo e o
barramento junto. Sem WebView2 a janela não abre. **Da release ou a partir do código**, suba o
barramento com `spellcore` (na release, `spellcore-windows-x64.exe`) e abra no navegador —
`--dir` é a raiz do repositório, não `spellgui/web`, porque as páginas leem
`../../design/tokens/`, `../../faces/` e `../../shows/`. O `Spellcaster.exe` do zip do pendrive é o
protótipo Python (`INSTALL.md` §1), não esta página.

```
spellcaster.exe shows\medgrupo.spell
spellcore serve --port 8000 --dir . --show shows/medgrupo.spell
http://127.0.0.1:8000/spellgui/web/laser3d/app.html
```

**Atalhos de abertura (hash na URL).**

| URL | O que faz |
|---|---|
| `app.html` | splash na parede, depois vai para a vista TRÁS |
| `app.html#tras` | pula a splash, abre na vista TRÁS (menu), chave desarmada |
| `app.html#dentro` | pula a splash, arma a chave e abre a vista DENTRO |
| `app.html#laser` | pula a splash, arma a chave e abre a vista SHOW |

`#dentro` e `#laser` armam a chave no estado da página, sem mandar `laser_open` ao engine: para
armar de verdade (abrir o DAC) use a chave na traseira ou a tecla `S`.

**Sem engine.** `app.html?offline=1` (ou abrir por `file://`) roda a página inteira sem
barramento: `bus.call` só ecoa o que faria, nada sai pela rede. A HUD escreve `ENGINE OFFLINE` e a
gaveta escreve `engine offline` no lugar da lista de arquivos e dos DACs. A query vem antes do
hash: `app.html?offline=1#tras`. WebGL 2 é obrigatório só para o MSAA 4× do menu VÍDEO; sem WebGL 2
essa linha cai sozinha para FXAA. `?perf=1` liga o contador cru em `window.__perf`.

## A tela

**Splash.** O galvo contorna SPELLCASTER LASER na parede, letra por letra, e no fim tudo brilha e a
câmera voa para a traseira: cerca de 7,6 s. Qualquer tecla ou clique pula (com jingle).
**O aparelho** é um projetor de 10 W em 3D: chassi, painel traseiro com portas, display, encoder,
chave, rocker de energia, e a mesa óptica por baixo da tampa. Passar o mouse acende a peça e mostra
a etiqueta dela; clicar abre a aba daquela peça na gaveta. Chapa, aleta, plugue, fonte e ventoinha
são peça inerte: não acendem e o clique não faz nada.

**As três vistas.**

| Vista | Tecla | O que é | O que libera |
|---|---|---|---|
| TRÁS | `2` | o menu | portas, display, encoder, chave, rocker, interlock |
| DENTRO | `3` | as preferências | tampa abre; mesa óptica, módulos R/G/B, obturador, galvos |
| SHOW | `1` | a vista livre | o visualizador: aparelho, feixe, névoa e a parede |

A vista SHOW só libera com a chave armada; sem chave o botão `1 SHOW` fica desabilitado. Clicar
numa porta (ILDA, DMX, NET) estando na SHOW leva para TRÁS; clicar em galvo, obturador ou num
módulo de cor leva para DENTRO.

**Chave, energia e interlock.** O rocker POWER (`P`) liga o aparelho: desligado, o display apaga e
nada mais responde — nem a chave, nem o interlock, nem o encoder. A chave (`S`) arma a emissão e,
com engine, é ela que manda `laser_open` no DAC escolhido; desarmar manda `laser_close`. O
interlock (`I`) é uma entrada: plugue fora = SCAN FAIL, obturador fechado, feixe estacionado. O LED
grande de ARMADO no chassi e o LED de emissão seguem os quatro estados: apagado sem energia,
vermelho piscando desarmado (o de emissão pisca em âmbar), vermelho fixo em SCAN FAIL, verde fixo
em LIVE. Os LEDs da porta NET só acendem com energia.

**A HUD (canto superior direito).** Uma linha por fato vivo, cada uma com o seu LED; linha sem
fato não aparece.

| Linha | Quando aparece | LED |
|---|---|---|
| `ENGINE · rev N · <show>` ou `ENGINE OFFLINE` | sempre | aceso com engine |
| `DAC <tipo> <host> · feed N` | com engine | aceso com feed aberto; âmbar piscando durante a procura; vermelho em erro |
| `sACN · in U` / `ART-NET · in U` / `MIDI IN · in U` | uma por entrada declarada em `show.inputs` | acende com frame recebido nos últimos 2 s |
| `MIDI · <n> in · <n> out` | com MIDI conectado | aceso com pelo menos uma entrada |
| `<n> kpps · <n> pts · <n> fps` e `<arquivo> · frame n/N · DMX <addr>` | sempre | — |
| `<n> fps · <n> ms · <n> draw calls` | com CONTADOR NA HUD = SIM | — |
| `DESLIGADO` / `STANDBY · DESARMADO` / `SCAN FAIL` / `LIVE · ARMADO` | sempre | segue o estado |

Abaixo da HUD, em âmbar, fica o eco do último comando que a página mandou, na forma
`spell <comando> --arg valor`.

**O display do painel traseiro.** Sete páginas — STATUS, SHOW, DMX, NET, TEMP/ILK, ENGINE, ERRO —
com uma linha grande que se lê do outro lado da sala. Alarme (SCAN FAIL, erro) inverte o bloco;
não há cor nova. O encoder gira páginas; apertar entra em modo edição e passa de campo em campo;
BACK sai do campo, ou volta uma página. Campos por página: STATUS = kpps; DMX = endereço e
universo; NET = sACN, Art-Net, NDI, Spout; ERRO = limpar. SHOW, TEMP/ILK e ENGINE só informam.
Erro do engine cai na página ERRO e na boca do Pino; nada trava.

**O Pino.** Um cabo DMX com cinco pinos-botão, preso à câmera no canto inferior esquerdo, ligado
pelo cabo ao DMX OUT do aparelho. Clicar nele abre o menu das telas; cada pino é uma tela.

| Pino | Vai para |
|---|---|
| 1 · LASER | esta página (ILDA IN, na traseira) |
| 2 · FÓSFORO | a porta NET (o conversor ainda não existe) |
| 3 · PATCHBAY | `patchbay.html` |
| 4 · TEATRO DE PAPEL | `teatro.html` |
| 5 · INFO | a aba INFO da gaveta |
| GRAVADOR | `index.html` (a timeline) |
| MESA | `face.html?face=quatro` |

O botão `–` do balão (ou a trava no corpo do Pino) manda o Pino embora: ele desce, o cabo sai da
cena e a corda para de ser simulada; o ícone que fica traz ele de volta (endereço `pino.hide`).

### A gaveta (Tab)

Todo menu e toda configuração moram numa gaveta à direita. `Tab` abre e fecha, `Esc` fecha, o
puxador na borda também; dentro dela o `Tab` volta a ser do teclado. Sete abas:

**LASER.** O arquivo `.ild` e o caminho óptico. ILDA IN: nome e número de frames do arquivo, mais
`ESCOLHER .ILD`, `DEMO` e `PLAY/PAUSA`; TAMANHO de 30 % a 130 %; a lista dos `.ild` no disco do
engine (`laser_files`) com o tamanho em kB, e clicar num deles toca o arquivo no DAC e o desenha
na parede. GALVOS X/Y: KPPS 5 000 a 40 000 (passo 500), BUFFER 1 a 8, VELOCIDADE 0,40× a 1,60×.
MÓDULOS R/G/B (638 / 520 / 445 nm): limite 0–100 % e curva γ 0,50–2,50 por cor, com o gráfico das
três curvas. OBTURADOR: ABERTO ou FECHADO — fecha sem chave ou sem interlock. ILDA OUT é
informativo (encadeado: nenhum).

**DMX.** Endereço 1–512, universo 1–512, o modo de 16 canais listado, e a última leitura recebida
no barramento (`U<universo> ch<endereço> = <valor>`) ou `nenhum frame recebido`. **NET.** Quatro botões de estado com rótulo ON/OFF: NDI, SPOUT, ART-NET, SACN. Abaixo, o DAC: tipo,
host, feed aberto, a linha `spell laser_open`, o campo HOST editável e o botão `PROCURAR DACS`
(que chama `laser_dacs` com 2 s de prazo). Cada DAC achado vira um botão `TIPO host`; clicar
escolhe quem recebe o feixe.

**INTERLOCK.** O interlock é uma entrada, de endereço `laser/1/interlock`. A aba mostra o estado
agora e onde mapear quem o aciona: TECLA (learn), MIDI (learn), OSC (o endereço), ART-NET
(universo e canal, `≥ 128` = fechado), MQTT (tópico) e a linha de CLI. Só tecla e MIDI estão
ligados de verdade — o mapa de OSC, Art-Net e MQTT é guardado no navegador e a linha correspondente
está marcada "em breve". O botão `SIMULAR ABERTURA` / `REPOR O PLUGUE` troca o estado.

**BINDINGS.** Botão `MIDI:` com o estado da conexão, botão `RODA: SOLIDWORKS / NORMAL` (sentido da
roda do mouse), botão `RESET` e a tabela de todas as ações: rótulo, tecla, MIDI e `×` para limpar.

**VÍDEO.** O menu de vídeo do visualizador — seção própria abaixo. **INFO.** O que o programa é, o
estado do engine (`ligado · rev N` ou `offline`), o feed aberto e duas linhas de CLI; `N` fecha.

## Câmera

O mapa é o do SolidWorks; o que muda por vista não é o teclado, é a lei da câmera.

| Gesto | SHOW (livre) | TRÁS (fixa) | DENTRO (restrita) |
|---|---|---|---|
| arrastar botão do meio | gira em torno do ponto clicado | — | gira dentro dos limites |
| `Ctrl` / `Shift` / `Alt` + meio | pan · zoom · roll | — | — |
| arrastar esquerdo no vazio | gira | — | gira dentro dos limites |
| arrastar esquerdo numa peça | é da peça, nunca da câmera | idem | idem |
| roda no vazio | zoom no cursor | — | — |
| roda sobre o encoder | gira o encoder | gira o encoder | — |
| setas / `Shift`+setas / `Ctrl`+setas / `Alt`+setas | 15° · 90° · pan · roll | — | — |
| `F`, `Ctrl+1..7`, `Z`, `Shift+Z` | como no SolidWorks | — | — |
| `1` `2` `3` | troca de vista | idem | idem |

`Z` afasta e `Shift+Z` aproxima — é a ordem do manual do SolidWorks. `F` enquadra o aparelho e
`Ctrl+1..7` são as vistas padrão (frente, trás, esquerda, direita, topo, base, isométrica). Todas
essas teclas valem **só na vista SHOW**: nas outras duas a lei da câmera recusa o movimento.
**TRÁS** é pose fixa, calculada da normal do painel traseiro e refeita quando a janela muda de
tamanho. Sem arrasto, sem roda-zoom, sem setas. O único movimento é um respiro de ±2° que segue o
mouse e não muda a distância — desligável pelo endereço `cam.breathe`. **DENTRO** é órbita presa ao
centro da mesa óptica: yaw ±60° a partir da pose de entrada, pitch de 20° a 80°, distância travada.
**SHOW** é o SolidWorks inteiro, com limites: a câmera nunca entra no aparelho, nem atravessa o
chão ou a parede, e o pitch para em ±85°; a sensibilidade é proporcional à distância.

**O knob do encoder** usa o gesto do TouchDesigner: aperte e arraste o mouse para cima para
aumentar, para baixo para diminuir — um passo a cada 6 px, ou 24 px com `Shift` (ajuste fino).
Soltar sem andar 3 px conta como clique, que é OK. A roda do mouse em cima do knob também gira o
encoder. `RODA: SOLIDWORKS` (padrão, na aba BINDINGS) inverte o zoom em relação ao navegador; o
estado fica guardado no navegador.

## Teclado e MIDI

Toda ação tem um endereço textual, o mesmo em tecla e em MIDI. Tabela padrão, na ordem da aba
BINDINGS.

| Endereço | Rótulo | Tecla padrão |
|---|---|---|
| `cam.show` | vista SHOW | `1` |
| `cam.rear` | vista TRÁS · menu | `2` |
| `cam.inside` | vista DENTRO · preferências | `3` |
| `key.toggle` | chave: arma | `S` |
| `lock.toggle` | interlock | `I` |
| `power.toggle` | energia | `P` |
| `play.toggle` | play / pausa | `Space` |
| `kpps.down` | kpps −1k | `[` |
| `kpps.up` | kpps +1k | `]` |
| `kpps` / `size` / `lim.r` / `lim.g` / `lim.b` | faders: kpps, tamanho, limite de cada cor | sem tecla padrão (aprenda em BINDINGS) |
| `file.open` | abrir .ild | `O` |
| `demo` | demo.ild | `D` |
| `net.ndi` / `net.spout` / `net.artnet` / `net.sacn` | rede: NDI / SPOUT / ART-NET / SACN | sem tecla padrão (aprenda em BINDINGS) |
| `oled.up` / `oled.down` / `oled.ok` | OLED: encoder + / − / OK | sem tecla padrão (aprenda em BINDINGS) |
| `oled.back` | OLED: BACK | `Backspace` |
| `nfo` | info | `N` |
| `bind` | bindings | `B` |
| `drawer` | gaveta: abre e fecha | `Tab` |
| `esc` | fecha a gaveta | `Escape` |
| `dacs` / `midi.connect` / `bind.reset` | procurar DACs, conectar MIDI, reset dos bindings | sem tecla padrão (aprenda em BINDINGS) |
| `cam.reverse` / `cam.breathe` | sentido da roda, paralaxe da vista TRÁS | sem tecla padrão (aprenda em BINDINGS) |
| `cam.rotL` / `cam.rotR` / `cam.rotU` / `cam.rotD` | câmera: gira 15° | `←` `→` `↑` `↓` |
| `cam.rot90L` / `cam.rot90R` / `cam.rot90U` / `cam.rot90D` | câmera: gira 90° | `Shift+←` `Shift+→` `Shift+↑` `Shift+↓` |
| `cam.panL` / `cam.panR` / `cam.panU` / `cam.panD` | câmera: pan | `Ctrl+←` `Ctrl+→` `Ctrl+↑` `Ctrl+↓` |
| `cam.rollL` / `cam.rollR` | câmera: roll | `Alt+←` / `Alt+→` |
| `cam.fit` | câmera: enquadra | `F` |
| `cam.front` … `cam.iso` | vistas padrão | `Ctrl+1` … `Ctrl+7` |
| `cam.zoomIn` | câmera: zoom + | `Shift+Z` |
| `cam.zoomOut` | câmera: zoom − | `Z` |
| `pino.hide` | Pino: some da tela / volta | sem tecla padrão (aprenda em BINDINGS) |
| `video`, `video.preset`, `video.padrao`, `video.fullscreen` e um `video.<id>` por linha do menu VÍDEO | menu de vídeo, predefinição, restaurar padrão, tela cheia e cada opção | sem tecla padrão (aprenda em BINDINGS) |

**Learn de tecla.** Abra BINDINGS (`B`) e clique no botão da coluna da tecla, na linha da ação: ele
escreve `TECLA…` e a próxima tecla vira o binding. `Esc` cancela. Uma tecla só serve uma ação: ao
aprender, ela é tirada de quem a tinha. **Learn de MIDI.** Mesmo caminho, coluna MIDI: o botão escreve `MIDI…` e a próxima mensagem vira o
binding. Antes disso é preciso conectar: botão `MIDI:` na mesma aba (usa Web MIDI do navegador;
sem Web MIDI o botão escreve `sem Web MIDI`). São aceitos note on, note off e control change; a
chave guarda o canal (`cc:1:7`, `note:1:60`). Sem SysEx, sem NRPN, sem MIDI clock. Ação do tipo
`cc` (os faders) recebe o valor 0–127 como 0–1; ação de botão dispara com nota ligada ou CC acima
de 63. Se o controlador tiver saída, o Spellcaster devolve o estado de cada ação mapeada.

O `×` de cada linha limpa a tecla e o MIDI daquela ação; `RESET` devolve tudo ao padrão de fábrica.
Tudo fica no `localStorage` do navegador: `sc-laser-bind` (bindings), `sc-laser` (kpps e último
arquivo), `sc-laser-video` (menu de vídeo), `sc-laser-ilk` (mapa do interlock), `sc-laser-wheel` e
`sc-laser-breathe` (câmera). Nada disso vai para o `.spell`. Com um campo de texto focado o teclado
do aparelho fica desligado; com um fader focado, só as setas e Home/End vão para o fader.

## Vídeo

A aba VÍDEO é um menu de jogo: PREDEFINIÇÃO no topo, os grupos de opções, o bloco DESEMPENHO e o
RESTAURAR PADRÃO embaixo. Toda linha aplica na hora — não há botão APLICAR, nem recarregar a
página, nem mesmo para o MSAA.

**Predefinições:** BAIXO, MÉDIO, ALTO, ULTRA e PERSONALIZADO. O padrão é ALTO, ou MÉDIO em máquina
de quatro núcleos ou menos. PERSONALIZADO não é escolha: é o que o menu escreve quando algum valor
sai da predefinição — repor o valor na mão volta sozinho ao nome dela. Com `prefers-reduced-motion`
ligado no sistema, as animações começam desligadas e o rastro mais curto.

A tabela sai inteira do `video.js`. Grupos: TELA (quantos pixels o programa desenha, e quantas
vezes por segundo), QUALIDADE (serrilhado, sombra e textura), PÓS (o que o composer faz depois da
cena), LASER (parede, rastro, feixes, névoa) e CENA (o aparelho e o que se mexe sozinho). O botão
`TELA CHEIA` fica no fim do grupo TELA.

| Grupo | id | Rótulo | Valores | O que muda |
|---|---|---|---|---|
| TELA | `escala` | ESCALA DE RESOLUÇÃO | 50 / 75 / 100 / 150 / 200 % | multiplica o devicePixelRatio (teto 2×): 200 % é supersampling de verdade |
| TELA | `fov` | CAMPO DE VISÃO | 30° a 90° | o fov da câmera; as vistas fixas se reenquadram a partir dele |
| TELA | `fpsMax` | LIMITE DE FPS | 30 / 60 / 120 / ILIMITADO | pula o trabalho do quadro por relógio, sem soltar o rAF |
| TELA | `hudFps` | CONTADOR NA HUD | SIM / NÃO | acrescenta a linha fps · ms · draw calls na HUD |
| QUALIDADE | `aa` | ANTI-ALIASING | DESLIGADO / FXAA / MSAA 4× | FXAA é um passe do composer; MSAA troca o alvo do composer por um multiamostrado (exige WebGL 2, senão cai para FXAA). Os dois não somam |
| QUALIDADE | `sombras` | SOMBRAS | DESLIGADAS / 1024 / 2048 / 4096 | liga a sombra do spot e o tamanho do mapa |
| QUALIDADE | `sombraTipo` | FILTRO DA SOMBRA | BÁSICO / PCF / PCF SUAVE / VSM | o filtro do mapa de sombra; trocar recria o mapa |
| QUALIDADE | `aniso` | ANISOTROPIA | 1× a 16× | anisotropia das texturas, limitada pelo máximo da GPU |
| QUALIDADE | `reflexo` | REFLEXOS DO AMBIENTE | SIM / NÃO | liga e desliga o environment map da cena |
| QUALIDADE | `reflexoInt` | INTENSIDADE DO REFLEXO | 0 a 2× | multiplica o `envMapIntensity` que cada material já traz |
| PÓS | `bloom` | BLOOM | SIM / NÃO | liga o passe de bloom |
| PÓS | `bloomForca` | FORÇA DO BLOOM | 0 a 2 | intensidade do brilho |
| PÓS | `bloomRaio` | RAIO DO BLOOM | 0 a 1 | espalhamento |
| PÓS | `bloomLimiar` | LIMIAR DO BLOOM | 0 a 1 | a partir de que luminância o pixel brilha |
| PÓS | `bloomRes` | RESOLUÇÃO DO BLOOM | ¼ / ½ / 1× da tela | o tamanho do alvo do bloom |
| PÓS | `tone` | TONE MAPPING | NENHUM / LINEAR / REINHARD / CINEON / ACES | a curva de tom do renderer (recompila os materiais) |
| PÓS | `exposicao` | EXPOSIÇÃO | 0,20 a 3,00 | exposição do tone mapping |
| LASER | `parede` | RESOLUÇÃO DA PAREDE | 512×320 / 1024×640 / 2048×1280 | o alvo onde o rastro mora; trocar limpa a parede |
| LASER | `rastro` | PERSISTÊNCIA DO RASTRO | 0,30 a 0,95 | quanto sobra do quadro anterior a cada 1/60 s |
| LASER | `halo` | HALO DO TRAÇO | 0 a 0,40 | opacidade do halo por ponto |
| LASER | `haloPx` | TAMANHO DO HALO | 4 a 16 px | tamanho do sprite do halo |
| LASER | `feixes` | FEIXES EXTERNOS | 40 / 80 / 160 / 320 | quantos segmentos de feixe saem da abertura para a parede |
| LASER | `poeira` | POEIRA NO FEIXE | SIM / NÃO | partículas dentro do feixe |
| LASER | `nevoa` | NÉVOA | 0 a 1 | ganho do feixe no ar e opacidade das nuvens |
| LASER | `puffs` | PARTÍCULAS DE NÉVOA | NENHUMA / 14 / 28 | quantos sprites de névoa ficam visíveis |
| CENA | `corda` | CABO DO PINO (VERLET) | SIM / NÃO | simula ou congela o cabo do Pino |
| CENA | `movimento` | ANIMAÇÕES (VENTOINHA, LED) | SIM / NÃO | ventoinha girando e piscar do LED; desligado por padrão com `prefers-reduced-motion` |

**DESEMPENHO.** Bloco de leitura, medido e não estimado, atualizado quatro vezes por segundo: fps
e ms/quadro (média de meio segundo), draw calls e triângulos, geometrias e texturas, resolução da
tela em pixels com o dpr, resolução da parede, WebGL 1 ou 2, MSAA ativo, anisotropia máxima e o
nome do renderer da GPU. **RESTAURAR PADRÃO** volta à predefinição da máquina. Tudo é guardado em
`localStorage` (`sc-laser-video`); valor fora de faixa ou fora da lista é recusado na entrada, e a
linha fica com o padrão.

## Laser

**Abrir um `.ild`.** Três caminhos: `ESCOLHER .ILD` (ou `O`), arrastar o arquivo na página, ou
clicar num arquivo da lista do engine na aba LASER. `D` volta para o `demo.ild`. São lidos os
formatos ILDA 0, 1, 4 e 5; o formato 2 (só paleta) é pulado. Arquivo que veio na mão só desenha na
parede: para sair no DAC ele precisa estar em `shows/`, no disco do engine.

**kpps, tamanho e limites.** KPPS de 5 000 a 40 000 (também `[` e `]`). A taxa de frame é
kpps ÷ pontos: abaixo de 25 fps a figura pisca, e acima de 32 kpps o galvo não acompanha e os
cantos viram curvas — o Pino avisa nos dois casos. TAMANHO vai para `laser_param geo/scale`; os
limites de R, G e B vão para `laser_param limit/r|g|b`. A curva γ e o BUFFER são locais: não há
`laser_param` para eles hoje. O kpps também é local: ele é argumento de abertura, então só chega
ao DAC no próximo `laser_open` — desarmar e armar a chave aplica.

**Achar o DAC.** `PROCURAR DACS` na aba NET chama `laser_dacs` com 2 s. O Ether Dream anuncia um
beacon UDP de 36 bytes em `255.255.255.255:7654`, a 1 Hz, e o Spellcaster escuta no IP de cada
placa de rede, não só no coringa. Se nenhum beacon chegar na metade do prazo, ele pergunta o
status por TCP na porta 7765 aos vizinhos da tabela ARP — um Ether Dream responde 22 bytes ao
aceitar a conexão. Cada achado traz `via: beacon` ou `via: tcp`; o achado por TCP não traz
`buffer` nem `max_pps`, porque só o beacon carrega esses campos. IDN é procurado por scan.

**Ether Dream Sitter aberto.** No Windows, com o Sitter aberto, o `bind` em `0.0.0.0:7654` é
recusado com `WSAEACCES` (10013), mesmo com `SO_REUSEADDR`. Nesse caso o beacon não chega: feche
o Sitter, ou digite o IP do DAC no campo HOST da aba NET e arme a chave — abrir por host não
depende de descoberta.

**Safety.** `laser_open` recebe `safety` (`min_size`, `max_intensity`, `zone`) e ela **nunca é
desligável**: não há comando para tirá-la. O interlock age por cima, pelo `laser_param shutter`
(1 fecha, 0 abre), e o obturador também fecha sozinho sem chave. Sem DAC escolhido não há o que
armar: sem engine a chave arma só o modelo na tela. Com feed aberto, a página pede `laser_stats` uma vez por segundo — pontos enviados, descartados,
erros, jitter, cpu e a safety corrente — e mostra o resultado na página ENGINE do display.

## DMX e rede

**Saída.** O show declara as saídas em `outputs`, e o player as abre: sACN, Art-Net e OSC.
Universos são numerados a partir de 1; o Art-Net converte para port-address internamente.

**Entrada.** O show declara `inputs` (`{"type": "sacn", "universe": 1}`, `artnet`, `midi`). O
barramento publica os frames de entrada como topic 2, e é isso que acende os LEDs das linhas de
entrada na HUD — só com frame recebido de verdade. A aba DMX mostra o último valor lido no endereço
escolhido.

**NDI e Spout.** Os quatro botões da aba NET marcam a fonte na página e acendem os LEDs da porta
NET. NDI → ILDA e Spout → ILDA são o FÓSFORO, o conversor que ainda não existe: ligar NDI ou Spout
hoje não converte nada, e o Pino diz isso.

**`net`.** A varredura de rede está na CLI e no registry (`spellcore net --timeout 2 [--json]`):
interfaces, nós Art-Net, sACN e Ether Dream, mais as sugestões de configuração.

## As outras páginas

Todas moram em `spellgui/web` e são servidas pelo mesmo `spellcore serve`. Cinco delas trazem a
barra `nav.js` no topo, com `Shift+1` a `Shift+6` e a tecla `?` para a ajuda. A página 3D do laser
**não** carrega essa barra: a navegação dela é o Pino.

- **TIMELINE** (`index.html`, `Shift+1`) — a timeline em canvas: tracks, keyframes, cues, loop no
  engine, gravação (`R`) e previz 2D na faixa de baixo (`Alt+M`).
- **PATCHBAY** (`patchbay.html`, `Shift+2`) — o editor do graph: criar nó (`Shift+A`), cabear,
  agrupar e desfazer, tudo por `show_patch`.
- **TEATRO** (`teatro.html`, `Shift+3`) — patch, cenas e cues do DMX, com GO e cenário clicável.
- **FACE** (`face.html?face=quatro`, `Shift+4`) — a mesa: os botões grandes do show, em kiosk.
  `Enter` = cue GO, `Esc` segurado = blackout, `Shift+F` = tela cheia.
- **LASER** (`Shift+5`) — a aba da barra aponta para `laser.html`, a página 2D do ILDA player (DAC,
  kpps, arquivo, play/stop, sliders, shutter, stats), não para a página 3D.
- **MIDI** (`midi.html`) — portas, abrir e fechar, a tabela `tecla → comando` do `.spell`, LEARN e
  a última tecla ao vivo. Não tem aba na barra; abra pela URL.
- **AJUDA** (`help.html`, `Shift+6`, ou `?` em qualquer página com a barra) — a tabela de atalhos
  de `design/SHORTCUTS.md` e o registry vivo, com um formulário por comando para executar.

Detalhes de cada arquivo: `spellgui/web/README.md`. Mapa de teclas do produto inteiro, com a
coluna de estado (feito / falta / n.a.): `design/SHORTCUTS.md`.

## CLI e MCP

Um processo toca o hardware; toda página e toda sessão de IA falam com ele.
```
spellcore play <show.spell> [--loop] [--osc-port N]   toca o show (sACN / Art-Net conforme "outputs")
spellcore net [--json] [--timeout N]                  varredura de rede + sugestões
spellcore commands                                    o registry em JSON (nome, doc, schema)
spellcore serve [--port N] [--dir D] [--show S]       barramento HTTP + WebSocket + MCP
spellcore mcp                                         servidor MCP em stdio
spellcore mcp install --target desktop|code [--yes]   registra o servidor no Claude
```

`serve` abre só em `127.0.0.1`, sem autenticação e sem TLS — e por isso não aceita `--host`. Porta
`0` escolhe uma livre e a linha `serve http://127.0.0.1:<porta>` sai no stderr. `--show` carrega o
show com o player parado em `t=0`; o comando `resume` solta. **MCP.** `spellcore mcp` sobe o servidor em stdio; `mcp install` grava a entrada no Claude Desktop
(`--target desktop`) ou um `.mcp.json` no diretório corrente (`--target code`), sempre mostrando o
que vai gravar, com backup `.bak`, e só depois de confirmar no console. O mesmo MCP também é
servido em `/mcp` pelo `serve`. Uma sessão de IA vê **uma tool por comando do registry** — são 59
hoje, incluindo `load`,
`show_get`, `resume`, `pause`, `locate`, `cue_go`, os de edição (`track_add`, `key_set`, `cue_set`,
`patch_add`, `show_patch`), os `midi_*`, `net`, `play_show` e os `laser_*` (`laser_dacs`,
`laser_open`, `laser_play`, `laser_stop`, `laser_close`, `laser_param`, `laser_stats`,
`laser_files`, `clip_frame`) — e quatro resources: `spell://show`, `spell://commands`,
`spell://graph` e `spell://face`. Com isso a sessão escaneia a rede, patcheia, edita a timeline e
dá play sem GUI. Tabela completa dos comandos e dos parâmetros: `spellcore/README.md`.

## Capacidades e limites

| Área | Estado |
|---|---|
| Engine, protocolos (sACN, Art-Net, OSC), `net` | pronto (R0) |
| Timeline, cues, `.spell`, `fx` em Rhai, Graph, player com transporte OSC | pronto (R1) |
| Pixel mapping (100 000 px) | pronto (R3), mas o crate é autônomo: ligar a fonte de frame ao player espera a mídia |
| Laser multi-feed: Ether Dream, IDN, safety, 4 feeds | pronto (R4) |
| MCP em stdio + `/mcp` no `serve` | pronto (R7) |
| Empacotamento: onedir Windows, Linux, Pi estático, CI com bench como gate | pronto (R8) |
| GUI: janela própria (`spellcaster.exe`) abrindo a página 3D do laser, ligada ao registry | base pronta (R5); faltam painéis, Theme e Face |
| Mídia: GStreamer, NDI, RTSP, Spout | não existe (R2) — bloqueado pelos SDKs não instalados |
| FÓSFORO (NDI/Spout → ILDA) | só o núcleo `laser::trace` (bitmap → contorno → ILDA, 4,4 ms com 20 objetos). Falta a fonte NDI da R2; os botões NDI/SPOUT da aba NET não convertem nada |
| Previz em Godot | não existe (R6) — Godot não instalado |
| Editores de Face e de Graph, painel Agent | não existem (R9) — dependem da GUI |
| Helios (DAC USB) | stub em qualquer build, não é limitação do binário estático |
| `face_patch`, `graph_patch`, `theme_set` | não existem: o `.spell` ainda não serializa Theme nem Graph. Quem edita é o `show_patch` |
| Exportar / render do show | não existe no registry; `Ctrl+M` fica reservado |

**Na página 3D, o que ainda não age no engine.** BUFFER e as curvas γ dos módulos são locais (não
há `laser_param` para eles, e o BUFFER hoje não muda nem o desenho na parede); o UNIVERSO da aba
DMX só aparece no display; o mapa de OSC, Art-Net e MQTT do interlock é guardado no navegador e não
vira comando — só tecla e MIDI acionam a entrada; ILDA OUT (encadeamento) é informativo.
