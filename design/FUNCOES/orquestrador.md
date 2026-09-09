# Orquestrador — `spell graph` (tema PATCHBAY, "modo Chataigne")

Ligar fontes (OSC, MIDI, NDI vetorizado, DMX de entrada, teclado, tempo) a destinos (universos, lasers, cues, player) por rotas com filtros, sem código. É o painel Graph de `SHORTCUTS.md` (`Shift+3`) e a realização de `PRINCIPIOS.md §1`: o graph é a interface.

| Item | Quem resolveu melhor | Por quê |
|---|---|---|
| Objetos e verbos | Chataigne | Módulo, rota, ação, multiplex e lista de cues são o mesmo `Processor` em quatro sabores; um comando aceita disparo, valor ou os dois por `CommandContext` |
| Estados | Chataigne, com o cook do TouchDesigner | Dois piscas de atividade por módulo, alimentados por evento; `WarningReporter` central com o culpado clicável; o cabo tracejado animado diz que o dado está passando; erro e aviso são contagem por nó |
| Zonas da tela | Chataigne | O layout padrão é exatamente Premiere/Resolve: lista à esquerda, canvas no centro, Inspector à direita, sequências e log embaixo |
| Atalhos | Blender (editor de nós) | Entrar/sair de grupo, apagar religando, mutar cabo sem apagar, menu Add com busca ao digitar |
| Arquivo | Chataigne `module.json`, com a externalização do TouchDesigner | Módulo declarado em texto (`parameters, values, commands, context, dependency`); referência externa com cópia de segurança e `relpath` declarado |

## 1. Objetos e verbos

Toda a hierarquia desce de dois tipos: **container** (nó com filhos) e **controlável** (folha com valor), e todo controlável tem endereço [fontes/chataigne.md § Objetos e verbos]. No Spellcaster o endereço é o nome do registry (regra 2).

**Módulo** — uma fonte, um destino, ou os dois (`hasInput`/`hasOutput`). Tem `parâmetros` (configuração), `valores` (o que recebe, somente leitura), `comandos` (o que aceita), dois gatilhos de atividade (entrada, saída) e um estado de conexão [fontes/chataigne.md § Objetos e verbos]. Verbos: adicionar pelo menu, habilitar/desabilitar, logar entrada/saída, testar comando, rotear todos os valores para outro módulo (o Module Router: fonte, destino, "route all") [fontes/chataigne.md § Mappings e Actions]. O módulo DMX do Chataigne é multi-universo com `thru`, `sendRate` e `sendOnChangeOnly`; o DMX Out do TouchDesigner acrescenta o aviso "Rate ≤ 44 Hz" e, para sACN, `cid`, nome e **prioridade** para merge com outras fontes [fontes/touchdesigner.md § Laser e NDI].

**Comando** — o `@command`. Declara `context: action | mapping | both`: se aceita GO, valor contínuo, ou ambos [fontes/chataigne.md § Objetos e verbos]. O mesmo objeto serve "dispare isto" e "mande este valor para isto"; saída de rota e consequência de ação são a mesma classe, só muda o contexto.

**Rota** (mapping) — cadeia de quatro estágios: `entradas` (qualquer endereço; várias, só uma dispara o recálculo), `filtros`, `saídas` (comandos em contexto mapping), mais `modo` (`ao mudar | manual | timer`) e `reemitir ao ativar` [fontes/chataigne.md § Mappings e Actions]. O Resolume acrescenta ao mesmo objeto o que o Chataigne não tem: **escopo do alvo** (`Selecionado | Este | Por posição`), faixa de entrada e saída separadas do parâmetro (`in/out` vs `min/max`), e o **caminho de retorno** para LED de controlador no mesmo objeto (`OutputPath`, `NamedValues` = tabela de cor) [fontes/resolume.md § Atalhos e mapeamento]. Faixa útil separada da física é o que evita um nó de escala em cada rota [fontes/resolume.md § O que copiar].

**Filtro** — nó de um só input e um só output que devolve `CHANGED | UNCHANGED | STOP_HERE`. Lista do Chataigne: Delay, Script, Time; ColorRemap, ColorShift; Condition; Conversion, Merge, SimpleConversion; Crop, CurveMap, Damping, Freeze, Inverse, Lag, Math, OneEuro, SimpleRemap, SimpleSmooth, Speed; String [fontes/chataigne.md § Mappings e Actions]. Mais os geradores do MadMapper, que são filtros entre controle e valor: `time_base` (integra velocidade; existe porque `sin(speed*TIME)` salta quando se mexe na velocidade), `damper`, `adsr`, `ease`, `incrementer`, `pass_thru` (lê qualquer endereço do app) [fontes/madmapper.md § Parâmetros de material/módulo]. `Condition` com `STOP_HERE` é o gate: não existe nó "if".

**Ação** — condições + consequências para verdadeiro e para falso + `papel: ao ativar | ao desativar` [fontes/chataigne.md § Mappings e Actions]. Condições: comparação por tipo, grupo E/OU, manual, script, índice de multiplex, ativação.

**Multiplex** — `count` + índice: uma rota ou ação instanciada N vezes. "Um mapeamento para 24 aparelhos em vez de 24 mapeamentos" [fontes/chataigne.md § O que copiar].

**Lista de cues** (Conductor) — cue atual, próximo, loop, gatilhos anterior/atual; cada cue pode amarrar uma sequência com `autoStart`, `forceStartFrom0`, `autoStop`, `autoNext` [fontes/chataigne.md § Objetos e verbos]. É a lista de GO de mesa de luz escrita como ação; detalhada em `cenas-cues-dmx.md`.

**Estado** (State) — container de rotas e ações com `ativo`, `ao carregar: restaurar | ativar | desativar`, `checar transições ao ativar`. Transição é uma ação com origem e destino; desliga a origem antes de ligar o destino; vários estados ativos ao mesmo tempo [fontes/chataigne.md § State Machine]. **O catálogo de nós do PRD §10 não tem nó de estado.** Sem ele, "quando entrar no segundo ato, este conjunto de rotas passa a valer e aquele para" exige gambiarra de condição em cada rota. Ponto aberto para `DECISOES.md`; a semântica proposta é a do Chataigne, inteira.

**Cabo** — só liga portas de tipo compatível. O TouchDesigner só cabeia dentro da mesma família e usa Link/Export para cruzar [fontes/touchdesigner.md § Objetos e verbos]; aqui as famílias são os tipos de porta (`trigger`, `bool`, `number`, `color`, `xy`, `frame`, `dmx`) e a conversão é um nó de filtro visível, nunca coerção implícita.

**Grupo** — container navegável. `Tab` entra e sai no Blender (meta-strip, node group) [fontes/blender.md § O que copiar]; aqui é `Ctrl+]`/`Ctrl+[` (ver Atalhos).

Verbos de autoria que nascem no widget, não no painel: botão direito em qualquer parâmetro oferece "Adicionar e ligar a uma sequência" (cria a camada, o output e copia a faixa) e "Adicionar e ligar a uma variável" [fontes/chataigne.md § Mappings e Actions]; qualquer controlável sabe virar item de dashboard (`createDashboardItem()`) [fontes/chataigne.md § Objetos e verbos]; arrastar canal do CHOP até o parâmetro cria o export, e pousar sobre a aba troca de página no meio do arrasto [fontes/touchdesigner.md § Parâmetros].

Parâmetro dirigido por cabo guarda a constante: o TouchDesigner guarda constante, expressão, export e bind ao mesmo tempo, com um quadradinho no botão do modo inativo que tem conteúdo [fontes/touchdesigner.md § Parâmetros]. Aqui: mutar o cabo (não apagar) devolve o parâmetro à constante, e o campo mostra o quadradinho "tem cabo". O que não entra: os quatro modos do Chataigne escondidos no menu de contexto, "lógica invisível no graph" [fontes/chataigne.md § O que NÃO copiar]; expressão é nó.

## 2. Estados

- **Módulo**: habilitado/desabilitado; conectado/desconectado; pisca de entrada e de saída **por evento, não por polling** [fontes/chataigne.md § Estados visuais]. Capture: verde é atividade, "atividade não garante funcionamento", e a causa provável vem nomeada (`Potentially blocked by firewall`) [fontes/capture.md § Estados e mensagens].
- **Cabo**: tracejado animado enquanto o dado passa (TouchDesigner: "wire com tracejado animado = a origem está cozinhando") [fontes/touchdesigner.md § Objetos e verbos]; botão do meio no cabo mostra o valor que está passando. Cabo mutado: apagado, presente.
- **Nó**: `mute` (entrada passa direto, o Bypass do TD), `lock` (congela o valor de saída, o Lock do TD, salvo no arquivo), `solo` (só este emite; apaga o mute dos outros sem removê-los) [fontes/blender.md § O que copiar]. Um verbo por conceito (regra 9).
- **Erro e aviso como número**: cada nó expõe `warnings` e `errors` como contagem [fontes/touchdesigner.md § Estados]; o painel de avisos lista todos e cada linha leva ao culpado (`WarningReporter`, `warningResolveInspectable`) [fontes/chataigne.md § Estados visuais]. Erro no campo do parâmetro, não só no nó (`parms.err.bg`).
- **Estado ativo** (State): aceso; a última ação disparada pode rolar a vista até ela (`focusOnLastActionTriggered`) [fontes/chataigne.md § State Machine].
- **Vigia** (Detective): "vigie este parâmetro e plote o histórico" [fontes/chataigne.md § Estados visuais]. Depurador de sinal, não de código.
- **Pendente**: rota editada e não aplicada; vermelho [fontes/touchdesigner.md § Estados].

Não entra: cor por família de nó (sete matizes dessaturados no TouchDesigner: "quando tudo é colorido, nada é estado") [fontes/touchdesigner.md § O que NÃO copiar]; família é forma ou rótulo. Nem `itemColor` gravado no item [fontes/chataigne.md § O que NÃO copiar].

## 3. Zonas da tela

O `default.chalayout` do Chataigne, que já é o desenho de `SHORTCUTS.md § Interface` [fontes/chataigne.md § Anatomia da tela]:

- **Esquerda**: lista de módulos (com os dois piscas por linha), e abaixo as variáveis do show.
- **Centro**: canvas do graph. Abas do centro no Chataigne: State Machine, Dashboard, Router, Morpher; aqui uma só, o Graph, com estados como containers dentro dele.
- **Direita**: Inspector genérico, "cada objeto responde `getEditor`", edição de N itens ao mesmo tempo é nativa [fontes/chataigne.md § Anatomia da tela]. Um Inspector, um lugar; não os três do TouchDesigner [fontes/touchdesigner.md § O que NÃO copiar].
- **Embaixo**: sequências à esquerda, timeline no meio, abas `Ajuda | Log | Avisos` à direita. Log com 2 000 entradas e gravação opcional em arquivo [fontes/chataigne.md § Estados visuais].

Menu do painel: `View, Select, Add, Módulo`. `Add` abre com busca ao primeiro caractere (o Tab menu do TouchDesigner acende os tipos que casam enquanto se digita; o Blender faz o mesmo com `SEARCH_ON_KEY_PRESS`) [fontes/touchdesigner.md § Anatomia da tela], [fontes/blender.md § Paleta de comandos]. Nada de palette acoplável.

Canvas: clique no vazio faz pan sem mudar zoom; `Shift`+arrasto é seleção em caixa; roda é zoom; botão do meio no nó abre o popup de info [fontes/touchdesigner.md § Atalhos]. Mapa de bordas da rede não é necessário; `Shift+Z` enquadra.

Face performance: sem canvas. Módulos como lista com piscas, avisos e o estado ativo.

## 4. Atalhos

`SHORTCUTS.md` vale (`Shift+3` foca o Graph, `Shift+Z` enquadra, `Ctrl+C/V/X`, `Delete`, `Shift+D` mute, `Shift+S` solo, `Ctrl+A`). O que entra, do editor de nós do Blender [fontes/blender.md § Atalhos] e do Network Editor do TouchDesigner [fontes/touchdesigner.md § Atalhos]:

| Ação | Tecla | Origem | Conflito |
|---|---|---|---|
| Adicionar nó (menu com busca) | `Shift+A` | Blender | nenhum |
| Entrar no grupo / subir um nível | `Ctrl+]` / `Ctrl+[` | (nosso) | Blender usa `Tab`, que em `SHORTCUTS.md` é a troca de Face; TD usa `Enter`/`u`, e `Enter` é GO |
| Apagar religando os dois lados | `Shift+Delete` | Blender `delete_reconnect` | Blender usa `Ctrl+X`, que é recortar |
| Mutar cabo sem apagar | `Ctrl+Alt` + arrasto com botão direito sobre o cabo | Blender `links_mute` | gesto |
| Cortar cabos | `Ctrl` + arrasto com botão direito | Blender `links_cut` | gesto |
| Novo ramo de uma saída já ligada | botão do meio na saída | TD | gesto |
| Inserir nó no meio do cabo | botão direito no cabo | TD | gesto |
| Criar vários nós já cabeados em série | segurar `Shift` ao criar | TD | gesto |
| Trocar a entrada de um nó | arrastar da saída de outro nó sobre o cabo existente | TD | gesto |
| Lock do nó | `Shift+L` | (nosso) | nenhum |
| Renomear | `F2` | Blender | nenhum |
| Mostrar valor no cabo | botão do meio no cabo | TD | gesto |
| Ligar/desligar cooking global | `Ctrl+Space` | TD | Blender usa `Ctrl+Space` para maximizar; `SHORTCUTS.md` maximiza com `` Ctrl+` ``, então livre |

Gramática dos modificadores no canvas, do Blender: sem modificador = a ação; `Shift` = a mesma sobre o complemento; `Alt` = o inverso ou limpar; `Ctrl` = a variante forte [fontes/blender.md § Atalhos]. Bate com `SHORTCUTS.md § Gramática`.

## 5. Arquivo

**Manifesto de módulo**, texto, um por pasta, o `module.json` do Chataigne [fontes/chataigne.md § Arquivo]:

```json
{ "name": "...", "type": "osc", "version": "1.0.0",
  "hasInput": true, "hasOutput": true,
  "parameters": { "porta": { "type": "int", "default": 9001 } },
  "values":     { "nível": { "type": "float", "readOnly": true } },
  "commands":   { "go": { "context": "action", "parameters": {} } },
  "dependency": [ { "source": "modo", "check": "equals", "value": "avançado", "action": "show" } ] }
```

`dependency` mostra ou habilita um parâmetro conforme outro, sem script. Módulo cifrado (`main.ldat` do MadMapper) é o contra-exemplo: "quem compra o software não consegue ler nem versionar o que roda no show dele" [fontes/madmapper.md § O que NÃO copiar].

**No `.spell`**, chave `graph`:

- `nodes[]`: `{uid, type, name, params (completos), mute, lock, enabled}`. Referência entre nós por `uid`, nunca por nome curto (`sourceState`/`destState` do Chataigne quebram ao renomear) [fontes/chataigne.md § O que NÃO copiar].
- `wires[]`: `{from: uid.port, to: uid.port, muted}`.
- `states[]`: `{uid, name, active, on_load: restore | activate | deactivate, check_on_activate, children[]}`.
- `view`: `{uid: {x, y, collapsed}}`, em bloco separado dos nós, para o diff da lógica não carregar o diff da posição. Cor nunca; é do Theme (`PRINCIPIOS.md §5`).
- Módulo externo: `{path, relpath: show | module, backup: <cópia embutida>}`. "Um show que referencia módulos externos precisa carregar uma cópia de segurança de cada um, senão morre num pendrive que não tem a pasta" [fontes/touchdesigner.md § O que copiar].

Lock salva o valor congelado dentro do arquivo, como o Lock flag do TouchDesigner [fontes/touchdesigner.md § Arquivo].

Fora do `.spell`: log, avisos, valores recebidos, lista de dispositivos, estado dos piscas, layout dos docks (o Chataigne embute `layout` no `.noisette`; não) [fontes/chataigne.md § Arquivo].

Desempate de fontes concorrentes: o Cue Scheduler do MadMapper confere o relógio a 1 Hz e "o último módulo da lista vence" [fontes/madmapper.md § O que copiar]. Aqui: para trigger, o último da lista; para valor DMX, a política de merge declarada no patch (`cenas-cues-dmx.md § 1`).
