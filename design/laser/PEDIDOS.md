# Pedidos do Matheus para a ferramenta LASER (rodada 5 em diante)

Registro literal do que foi pedido, na ordem em que chegou. Nada aqui é opcional; cada item vira um `[x]` quando estiver no protótipo publicado.

## 09/09/2026 · sobre a rodada 4 ("gostei muito do programa como um todo, vamos iterar em cima dele")

- [x] **Traseira realista** como um laser show de verdade (referências: painel traseiro Kvant Clubmax; diagrama de laser RGB chinês). "Se inspire, não copie tudo." Sem elemento sem função: **um** interlock só, **sem fusível**. A ventoinha pode ficar, mas "você modelou ela toda errada" (ventoinha axial real: aro, cubo, 7 pás curvas, grade de arame).
- [x] Procurar imagens de laser na internet, modelos 3D e assets para montar o nosso (assets externos não carregam no artifact; o modelo é procedural, inspirado nas fotos).
- [x] **Parte de dentro "está péssima"**: usar **GLSL** para modelar o feixe do laser; colocar os **diodos** e os **espelhos dicroicos**; modelar o **optical path trace** de forma eficiente e eye-candy.
- [x] **Mascote-menu (Pino)** "não ficou muito bom ainda".
- [x] **Fluxo de abertura**: primeiro a tela de settings, só depois dá para navegar para a tela de show.
- [x] Resultado **triple AAA** no modelo, na interface e na usabilidade.
- [x] Gerar isto como **um design system só desta ferramenta de laser e ILDA** (`tokens.css` + `SISTEMA.md` + página `sistema.html`).
- [x] **Key binding magic** como MadMapper / Resolume: qualquer coisa vira evento MIDI, **in e out** (MIDI learn, feedback de saída).
- [x] **Câmera**: a órbita atual é desconfortável; usar o **padrão do SolidWorks** de manipulação (MMB gira em torno do ponto clicado, Ctrl+MMB pan, Shift+MMB zoom, roda dá zoom no cursor, setas 15°, Shift+setas 90°, Ctrl+setas pan, F enquadra, Ctrl+1..7 vistas padrão).

## 09/09/2026 · splash e câmera

- [x] A splash é a **câmera mirada direto no output** (a parede). Quando a splash termina, a câmera **foca na traseira do laser**; depois disso vêm menus e interatividade.
- [x] Detalhe da splash: na parede, e **só nela** (sem ILDA, sem laser de show no output), o laser faz o **contorno do nome SPELLCASTER LASER**. Enquanto o laser desenha o outline num **movimento dinâmico**, as letras vão sendo **reveladas** e ficam na parede; **brilham todas**; então a câmera vai **direto para o menu** (traseira) e o laser **se apaga**, apagando um pouco a luz do ambiente em volta.

## 09/09/2026 · parte de dentro (com print da rodada 4)

- [x] **Nada pode estar voando ou fora de contexto.** Procurar imagens de laser shows abertos.
- [x] **Modelar os galvos** (bloco X/Y com os dois motores a 90°, espelhos, cabos).
- [x] **Modelar o optical table system** (chapa base com furação, módulos, suportes cinemáticos).
- [x] Fazer o conjunto óptico **parecer um bloco só**, diferente dos demais componentes.
- [x] **Simular PCB melhor** do que isso, **mas não deixar em evidência** (placa de driver e fonte ficam de lado, escuras, discretas).

## Regras permanentes (já em memória, repetidas aqui por segurança)

- Commits e pushes só na conta do Matheus, sem crédito ao Claude. Branch `design/0.1.3` (rodada 6; antes `design/0.1.2`).
- Agentes sempre em Opus.
- Ponytail full; Python só em `.py`; UTF-8 sem BOM; binários pesados fora da pasta do Drive.
- Função antes de UI; um material por tema; votar dentro do protótipo; ser wild; nada de software quadradão.

## 09/09/2026 · foco da câmera na splash

- [x] Na splash o foco da câmera **sai do ponto estático** em que ela está mirando e **vai para a tela (parede) mesmo**, ignorando o laser e a traseira dele. Só depois ela vai e foca em outra coisa: o menu (traseira) ou as preferências (tampa).

## 09/09/2026 · Rodada 6 · integração com o orquestrador

Pedido literal (o bloco "Contexto:" que veio junto é de uma tarefa anterior, o merge de `design/funcoes-referencia`, e ficou de fora desta rodada):

> Você é o agente de design do Spellcaster. Ponytail full, protocolo-padrão, resposta em português, veredito antes de explicação. Agentes só em `opus`. Commits e pushes só na conta do Matheus: proibido `Co-Authored-By`, "Generated with Claude" ou qualquer crédito ao Claude. Python só em arquivo `.py` (`C:\Python313\python.exe`), nunca `python -c`. UTF-8 sem BOM. Nada de binário pesado na pasta do Drive: gere no scratchpad.
> 
> ## O que existe
> 
> - **Rodada 5 (branch `design/0.1.2`, commit `b90d1b0`)**: o programa é o projetor laser 10 W em 3D. Fontes em `design/laser/`: `app.html` + `app.js` (estado `S`, parede, splash, painéis, OLED, tick), `body.js` (case e traseira: portas com `userData.key` = power, keyswitch, interlock, ilda, ildathru, dmxin, dmxout, rj45, usb, oled, enc, back, fan), `optics.js` (mesa óptica, módulos r/g/b, dicroicos, obturador, galvos, PCBs, DAC, fonte), `beam.js` (feixe GLSL), `cam.js` (câmera SolidWorks), `bind.js` (registro de ações: `Bind.def(id, label, fn, {key, midi, type: "btn"|"cc", get})`, MIDI learn in/out, feedback, `localStorage sc-laser-bind`), `pino3d.js` (mascote-menu: pino 3 = "Orquestrador", hoje só abre o painel NET com uma mensagem), `tokens.css` + `SISTEMA.md` + `sistema.html` (design system), `PEDIDOS.md` (log literal dos pedidos). `design/build.py` inlina os módulos num HTML só. Protótipo publicado: https://claude.ai/code/artifact/8a913f8b-8ea7-4621-b57a-88d7738dbafd (voto em `moodboard/round5`; não mexa no voto).
> - **Funções (working tree da branch `design/funcoes-referencia`, `design/FUNCOES/`)**: `README.md` (11 regras transversais: parâmetro tipado gera widget; **um endereço textual é a identidade de tudo**; trigger/toggle/valor são tipos distintos; estado é tinta; erro é dado; um verbo por conceito...), `orquestrador.md` (`spell graph`, tema PATCHBAY: módulo com `hasInput/hasOutput`, parâmetros, valores, comandos com `context: action|mapping|both`; rota = entradas → filtros → saídas + escopo + faixa in/out + caminho de retorno para LED; filtros Lag/Damping/OneEuro/CurveMap...; multiplex; estado; cabo só entre portas de tipo compatível `trigger|bool|number|color|xy|frame|dmx`; manifesto `module.json`; chave `graph` do `.spell` com `nodes[]`, `wires[]`, `states[]`, `view`), `ilda-player.md` (tabela completa de parâmetros da saída laser com nomes agrupados por `/`, estados armado/shutter/DAC, atalhos), `fontes/` (auditorias com `path:linha`). Se `design/FUNCOES/` ainda estiver sem commit, é de outra sessão: **leia, não edite, não commite**.
> 
> ## O que fazer
> 
> **Objetivo**: o projetor da rodada 5 vira um **módulo do orquestrador**, e as bindings da rodada 5 viram **rotas** do graph. Nada de UI nova do PATCHBAY ainda: a integração é de contrato e de dado, com o mínimo de interface para ela ser visível no protótipo.
> 
> 1. **`design/FUNCOES/integracao-laser.md`** (novo; se não puder escrever em `FUNCOES/`, use `design/laser/INTEGRACAO.md`). Tabela: cada porta, componente e ação da rodada 5 (todos os ids de `Bind.def` em `app.js` e todos os `userData.key` de `body.js`/`optics.js`) → endereço do registry (`laser/1/kpps`, `laser/1/shutter`, `laser/1/limit/r`, `laser/1/dmx/addr`, `laser/1/net/sacn`, `laser/1/cam/view`...) → tipo (`trigger|toggle|value`) → `context` (`action|mapping|both`) → tipo de porta do graph → caminho de retorno (LED/fader). Use os nomes agrupados por `/` de `ilda-player.md §1` onde já existem; não invente nome novo para parâmetro que a tabela já nomeia. Diga o que da rodada 5 **não** vira endereço (câmera, splash, Pino) e por quê.
> 2. **Manifesto do módulo** `design/laser/module.json`, no formato de `orquestrador.md §5` (`parameters`, `values` somente leitura como temperatura, fps real, pontos por frame; `commands` com `context`; `dependency` para o que só aparece armado).
> 3. **`bind.js`**: `Bind.def` ganha `addr` (o endereço do registry) e `ctx`; a tabela de bindings (`Bind.html()`) mostra o endereço ao lado do rótulo, porque o endereço é a identidade (regra 2). `Bind.manifest()` gera o `module.json` a partir das definições; `Bind.graph()` devolve o trecho `graph` do `.spell` com o nó `laser/1` e as bindings atuais como `wires[]` de `midi/<porta>` → filtro → `laser/1/...` (CC contínuo passa por `Lag`; nota é `trigger` direto), com `view` em bloco separado. Sem duplicar a lista de ações: uma fonte só, em `app.js`.
> 4. **No protótipo**: o pino 3 do Pino ("Orquestrador") abre um painel `ORQUESTRADOR` (wide) com três blocos: o manifesto, as rotas ativas (uma linha por binding: entrada → filtro → saída, com o pisca de atividade acendendo quando a ação roda) e o `graph` do `.spell` num `<pre>` com botão COPIAR (download é bloqueado no artifact). O comando equivalente no canto: `spell graph add laser/1` e `spell graph wire midi/cc:1:7 laser/1/kpps --filter lag`. Nada mais de UI.
> 5. **Registro**: `design/laser/PEDIDOS.md` ganha a seção "Rodada 6 · integração com o orquestrador" com este pedido literal; `design/DECISOES.md` ganha a entrada da data com o que foi decidido (endereços, o que ficou fora, ponto aberto: nó de estado não existe no PRD §10). `design/TEMAS.md` só se algo mudar de tema.
> 
> ## Como verificar
> 
> Build: `C:\Python313\python.exe design\build.py design\laser\app.html <scratchpad>\spellcaster-laser.html`. Sintaxe: `node <scratchpad>\jscheck3.js` (lê `spellcaster-laser.html`, `new Function` por `<script>`). Runtime: `errwrap.py` no scratchpad gera `err-laser.html` com hook de `onerror`; rode o Chrome headless com `--dump-dom` e procure `id="ERR"`:
> 
> ```
> "C:/Program Files/Google/Chrome/Application/chrome.exe" --headless=new --no-first-run --user-data-dir=<tmp> --use-angle=swiftshader --enable-unsafe-swiftshader --hide-scrollbars --window-size=1240,1200 --virtual-time-budget=12000 --screenshot=<png> "file:///<scratchpad>/spellcaster-laser.html#tras"
> ```
> 
> Hashes `#tras`, `#dentro`, `#laser` pulam a splash. Uma passada visual de correção, não um loop. `module.json` e o `graph` gerado precisam de **um** teste `unittest` em `tests/` que valida: todo `wire` liga portas de tipo compatível e todo endereço existe no manifesto.
> 
> ## Entrega
> 
> Republique o protótipo **no mesmo artifact** (`url` = o link acima, `capabilities` omitido). Commit na branch **`design/0.1.3`** criada a partir de `design/0.1.2`, via `git worktree` temporário no scratchpad (a pasta do projeto está em outra branch com outra sessão viva: nunca troque a branch dela). Mensagem em português, uma linha de assunto, sem crédito ao Claude. Push. No relatório: veredito, o que virou endereço e o que ficou fora, link, commit, e o que não foi feito.

- [x] `design/laser/INTEGRACAO.md` (a branch não tem `FUNCOES/`): tabela porta/componente/ação → endereço → tipo → context → porta do graph → retorno; o que fica fora e por quê.
- [x] `design/laser/module.json` no formato de `orquestrador.md §5`, gerado por `Bind.manifest()`.
- [x] `bind.js`: `addr`, `ctx` (+ `arg`, `t`, `range`, `unit`, `needs`), endereço ao lado do rótulo, `manifest()`, `graph()`; fonte única em `app.js`.
- [x] Pino 3 abre o painel ORQUESTRADOR (manifesto, rotas com pisca, `graph` com COPIAR); comando `spell graph add laser/1` no canto.
- [x] `tests/test_laser_graph.py`; `DECISOES.md`; `TEMAS.md` não mudou.
