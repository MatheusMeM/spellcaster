# Browser e drag and drop — o painel da esquerda (`Alt+5`)

Pedido do Matheus (09/09/2026): *"Quero drag n drop de elementos e saídas e mídias e laser e áudio e vídeo"*, e antes disso *"não tenho como inputar ou gravar DMX novo ou ILDA ou vídeo"*. Hoje não existe **nenhuma** entrada de mídia: nenhum `drop`/`dragover`/`dataTransfer` em `spellgui/web/*`, nenhuma rota de escrita em `spellcore/serve/src/lib.rs:338-345`, nenhum comando de arquivo entre os 30 do registry.

Fontes: **Ableton Live 12** §4 (Working with the Browser), §4.10 (Adding Content), §5.3 (Live Clips), §41.22 (atalhos), citado por seção. **DaVinci Resolve 20**, Media Pool, citado por página do manual local. **Resolume Arena**, manual online (Decks, Clips), para o que acontece ao soltar em cima do que já existe.

| Item | Quem resolveu melhor | Por quê |
|---|---|---|
| Estrutura do painel | Ableton §4 | Três seções nomeadas — Collections, Library, Places — e Places é onde entram as pastas do usuário. O Media Pool do Resolve é um bin por projeto, e a nossa "biblioteca" é a pasta do show mais o que o operador apontar |
| Onde o drop cria track | Ableton §4.10 | *"into the space to the right of Session View tracks or below Arrangement View tracks will create a new track"* — regra única, nas duas vistas |
| Drop vindo do sistema de arquivos | ambos | Ableton §4.10: *"Files can also be dropped directly into Live from the Explorer (Win)/Finder (Mac)"*; Resolve p.690: *"You can also drag a clip directly from your file system to the Timeline"* |
| Drop em cima de algo que já existe | Resolume (Decks) | É o único manual que declara os três casos: sobrepor troca de posição, soltar entre dois insere, `Ctrl` ao soltar copia. O Ableton não descreve, o Resolve manda usar "Set Path" |
| Preview antes de usar | Ableton §4.9 | Toggle de preview, `Shift+Enter` audiciona sem ligar o toggle, e um botão `Raw` que separa "tocar no tempo do show" de "tocar como o arquivo é" |
| Saídas no browser | nenhum | Nenhum dos três trata saída como item arrastável. É invenção nossa, justificada em §4 |

## 1. Objetos do browser

Uma árvore, três raízes, na ordem em que aparecem:

| Raiz | Contém | De onde vem |
|---|---|---|
| **Show** | o que este `.spell` usa: mídia referenciada por `clips[]`, perfis do patch, módulos do graph, faces | lido do show aberto |
| **Pasta** | a pasta do show e as pastas que o operador acrescentar | `Places` do Ableton §4.7: *"you must first add them to the browser, either by dropping them directly into the Places section from the Explorer (Win)/Finder (Mac), or by using the Add Folder option"*; §4.7.8: *"Adding a user folder does not actually move the folder to a new location"* |
| **Saídas** | universos sACN/Art-Net, alvos OSC, DACs laser, janelas de vídeo | lido de `show.outputs[]` |

Tipos de item, com o que cada um vira ao ser solto:

| Item | Extensão | Vira |
|---|---|---|
| Clipe laser | `.ild` | track `laser` + clipe |
| Áudio | `.wav`, `.mp3`, `.flac` | track `audio` + clipe (`audio-video.md §1`) |
| Vídeo | `.mp4`, `.webm` | track `video` + clipe |
| Efeito | `.rhai` | track `fx` |
| Perfil de aparelho | `profiles/*.json` | `patch_add {name, profile, universe, address}` |
| Módulo do graph | `modules/*.json` | nó `module` no PATCHBAY (`DECISOES.md`, aguarda voto) |
| Face | `faces/*.json` | abre a Face |
| Saída | — | ver §4 |

Cor por tipo é a mesma regra de `daw-arranjo.md` B7: derivada do tipo, nunca livre.

## 2. Preview

Ableton §4.9 tem três coisas e as três valem:

1. **Toggle de preview** ao lado da aba. Ligado, clicar no item audiciona.
2. **`Shift+Enter` audiciona mesmo com o toggle desligado** (*"You can preview files even when the Preview toggle is not enabled by pressing ShiftEnter or the right arrow key"*). É o atalho que salva quem desligou o preview para não estourar a PA no meio do show.
3. **Botão `Raw`**: desligado, o preview espera o próximo compasso e roda em loop; ligado, toca no tempo original, sem loop, sem scrub. Nosso equivalente: desligado = o preview respeita o transporte (entra no próximo marcador); ligado = toca já. **O `Raw` entra**, porque é a diferença entre ouvir um arquivo e ensaiar uma entrada.

Preview de `.ild` é o mesmo gesto no viewer de previz (frente `previz`); preview de `.rhai` não existe (script não se audiciona).

## 3. O que cada drop faz

Esta é a tabela que a implementação segue. `alvo` = onde o mouse solta.

| # | Item | Alvo | Efeito | Origem |
|---|---|---|---|---|
| 1 | mídia | área vazia abaixo do último track | **cria track do tipo da mídia** e põe o clipe no tempo do ponto de soltura | Ableton §4.10 |
| 2 | mídia | track compatível, num tempo | põe um clipe ali. Compatível = o tipo do track bate com a extensão | Ableton §4.10 ("Items can be dragged and dropped from the browser into tracks") |
| 3 | mídia | track incompatível | recusa, com o cursor de recusa. Não converte, não cria track escondido | (nosso; a alternativa silenciosa é o defeito que `PRINCIPIOS.md` chama de mentira de widget) |
| 4 | mídia | em cima de um clipe existente | **insere ao lado**, empurrando: soltar na metade esquerda insere antes, na direita insere depois | Resolume, Decks: *"You can also drag a clip just to the right or left of another clip. This will insert the dragged clip next to it, and shift over the others to make room for it."* Substituir por drop **não entra**: o Resolume também não tem (o caminho lá é o Media Manager, "Set Path") |
| 5 | clipe já na timeline | outro lugar | move (é `daw-arranjo.md` C2) |
| 6 | clipe já na timeline | outro lugar, com `Alt` | copia | `timeline-daw.md` item 15; o Resolume usa `Ctrl` ao soltar, nós já fixamos `Alt` |
| 7 | múltiplos arquivos | qualquer alvo | **um track só**, empilhados no tempo, salvo se `Ctrl` estiver segurado ao soltar, que espalha em tracks | Ableton §7.4: *"Live defaults to arranging them in one track... Hold down Ctrl (Win) / Cmd (Mac) prior to dropping them so as to lay the clips out in multiple tracks instead"* |
| 8 | perfil de aparelho | painel Patch | `patch_add` com o nome do arquivo como `name` | (nosso; `cenas-cues-dmx.md`) |
| 9 | saída | cabeçalho de um track | **atribui a saída ao track**: escreve `universe`/`address` (sACN/Art-Net), `feed` (laser), `address` (OSC) | ver §4 |
| 10 | saída | área vazia | **cria a saída no show** (`show.outputs[]`) | ver §4 |
| 11 | arquivo do Explorer | qualquer alvo | idem 1–4, depois de subir o arquivo | ver §5 |
| 12 | pasta do Explorer | raiz **Pasta** do browser | acrescenta a pasta ao browser, **sem copiar nada** | Ableton §4.7.8 |

Regra transversal do drop, tirada do Ableton §4.10 e válida para todos: **duplo-clique ou `Enter` no item faz a mesma coisa que soltar no track selecionado.** Sem isso, o browser é inoperável por teclado.

## 4. Saída arrastável: por que, e o contrato

Nenhum dos três manuais tem isso — no Resolume a saída é um `screen` no Advanced Output (`Ctrl+Shift+A` lá), no Ableton é o roteamento no mixer, no Resolve é a página Deliver. Mas o pedido é literal (*"drag n drop de elementos e **saídas** e mídias"*) e a nossa saída **é** um objeto do show: `show.outputs[]` já existe e já tem forma (`shows/medgrupo.spell`: `{"type":"sacn","universes":[1],"priority":100,"source_name":"Spellcaster"}` e `{"type":"laser","dac":null,"port":7765,"pps":25000,"safety":{...}}`).

Contrato:

- **Item de saída no browser = uma entrada de `show.outputs[]`**, mais os candidatos descobertos na rede que ainda não estão no show (DAC EtherDream que respondeu, nó Art-Net anunciado). Candidato aparece apagado; arrastar para a área vazia é o que o adiciona.
- **Soltar saída em cabeçalho de track** escreve no track o que liga os dois. Para `dmx`/`artnet`: `universe` e `address`. Para `laser`: o `feed`. Para `osc`: o `address` base. É um `show_patch` de uma linha.
- **Soltar saída em área vazia** acrescenta a `show.outputs[]`.
- **Uma saída física serve um alvo só.** Resolume, Screens: *"Every output can only have a single screen associated with it"* — quando se escolhe uma saída já usada, o Resolume devolve a outra para virtual. Aqui: universo já atribuído a outro track é aviso, não erro (dois tracks no mesmo universo é uso legítimo com merge declarado, `cenas-cues-dmx.md § Universo`); DAC laser já aberto por outro track **é erro**, porque um DAC toca um fluxo.
- **Um botão de pânico.** Resolume, Screens: `Ctrl+Shift+D` desabilita todas as saídas. `SHORTCUTS.md` já tem `Ctrl+Shift+Enter` (armar saídas reais) e `Ctrl+Shift+R` (modo ensaio); o par que falta é desarmar tudo, e é a mesma tecla de armar batida de novo.

**Faltam comandos no registry.** Saída hoje só se edita por `show_patch` com caminho `/outputs/0/pps`, o que obriga a GUI a saber o índice e o esquema:

| Comando | Argumentos | Faz |
|---|---|---|
| `output_add` | `kind, ...` | Acrescenta a `show.outputs[]`; devolve o índice |
| `output_set` | `index, <campos>` | Edita |
| `output_del` | `index` | Remove |
| `outputs` | — | Lista as do show **e** as descobertas na rede, com estado |

`outputs` é o que enche o browser. Sem ele o painel de saídas é uma leitura de `/show` sem estado, que é o defeito 24 de `pontos-falhos.md` (o painel do laser tem oito sliders e nenhum DAC).

## 5. Arquivo do Explorer: o contrato de `POST /files`

O drop de arquivo do sistema chega por HTML5: o evento `drop` traz `dataTransfer.files`, uma `FileList` de objetos `File`. **Na janela wry não há caminho** — `File.name` é o nome, `File.path` é extensão do Electron e não existe ali. Portanto o conteúdo tem de subir.

Rota nova em `spellcore/serve/src/lib.rs`, ao lado das quatro que existem (`/commands`, `/show`, `/ws`, `/mcp`, `:338-345`):

```
POST /files/<nome>
  corpo: os bytes do arquivo
  200 -> {"path": "media/medgrupo_laser.ild", "bytes": 41232}
  400 -> nome recusado
  413 -> maior que o limite
  409 -> ja existe
```

Regras, todas obrigatórias:

1. **Destino é a pasta do show, subpasta `media/`.** Nunca `--dir`, nunca caminho vindo do cliente.
2. **`<nome>` passa pelo mesmo filtro de `estatico()`** (`serve/src/lib.rs:102-108`): recusa `:`, `\`, segmento vazio, `.` e `..`, e não faz percent-decode. O filtro já existe e já está testado; reusar, não reescrever.
3. **Extensão em lista branca**: `ild`, `wav`, `mp3`, `flac`, `mp4`, `webm`, `rhai`, `json`, `spell`. Fora da lista, 400. Sem lista branca, `POST /files/x.exe` na pasta do show é um vetor pronto.
4. **Limite de tamanho**, com o valor no comentário `ponytail:`. Proposta: 512 MB, que cobre um vídeo de show e não cobre um disco. Acima, 413.
5. **Já existe = 409**, e a GUI pergunta. Sobrescrever mídia usada por outro clipe é perda silenciosa.
6. **Só `127.0.0.1`.** O `serve` já é local (`serve/src/lib.rs`), mas a rota de escrita é a primeira que torna isso uma decisão de segurança e não um acaso; um comentário `ponytail:` diz que ela cai quando o `--host` e o token existirem, junto com a leitura da raiz do repo que já está declarada em `:99-101`.
7. **A resposta devolve o caminho relativo ao show**, que é exatamente o que vai em `clips[].src` (`daw-arranjo.md §4.1`).

**Bloqueio a resolver antes:** a GUI não sabe onde mora o show. `GET /show` chama `show_get {full:true}`, e esse ramo devolve só `serde_json::to_value(sh)` — sem o caminho (`registry.rs:221-223`; só o ramo não-`full` passa por `resumo(f, sh)`). Sem o caminho não há "pasta do show" para o `POST` nem para resolver `src`. Correção mínima: `show_get {full:true}` devolver `{"file": "<caminho>", "show": {...}}`, ou `GET /show` mandar o caminho num cabeçalho. É `pontos-falhos.md` item 16, e é pré-requisito desta função.

**Segundo bloqueio, menor:** `mime()` conhece quatro tipos (`serve/src/lib.rs:87-96`) e devolve `application/octet-stream` para todo o resto. Um `<audio src="media/x.mp3">` ou `<video src="media/x.mp4">` servido como octet-stream não toca em navegador nenhum. `mime()` ganha `mp3`, `wav`, `flac`, `mp4`, `webm` — cinco linhas, e o próprio `ponytail:` do arquivo já prevê ("imagem e fonte entram quando alguma pagina trouxer uma").

## 6. O que fica de fora

- **Hot-swap** (Ableton §23.2.4, tecla `Q`). Trocar o arquivo de um clipe sem soltá-lo é útil, mas o gesto é "audicionar e trocar ao vivo", que só faz sentido com preview instantâneo de vídeo e áudio já rodando. Volta depois de `audio-video.md`.
- **Favoritos e cores de coleção** (Ableton §4.5, teclas `1`–`7`). As teclas `1`–`7` são caras e `PRINCIPIOS.md §2` não deixa cor decorativa. Um item favorito é um item numa pasta.
- **Media Manager / "Set Path"** (Resolume) e relink de mídia perdida. Entra quando existir show que viajou de máquina; hoje o caminho é relativo à pasta do show e a pasta viaja junto.
- **Combinar áudio e vídeo num clipe só** (Resolume, Decks: soltar áudio sobre um slot com vídeo transpõe o vídeo para a duração do áudio). Nós temos dois tracks; combinar é sincronizar, e sincronizar é `audio-video.md §3`.
- **Aritmética nos campos** (Resolume, Input Selection: digitar `/3` num campo de largura). Bom, e é do Inspector, não do browser.

## 7. Atalhos

| Ação | Tecla | Origem | Conflito |
|---|---|---|---|
| Mostrar / esconder o browser | `Alt+5` | Ableton §41.2 ("Move Focus to the Browser \| Alt5") | nenhum; `SHORTCUTS.md` usa `Shift+1..7` para painéis e `Alt+Shift+1..9` para workspaces |
| Buscar no browser | `Ctrl+F` | Ableton §41.22 | nenhum |
| Carregar o item selecionado no track selecionado | `Enter` | Ableton §41.22 | `Enter` é GO na Face performance; aqui é painel focado |
| Audicionar o item selecionado | `Shift+Enter` | Ableton §41.22 | nenhum |
| Abrir/fechar pasta, ir para o conteúdo | `←` / `→` | Ableton §4.8 | painel focado |
| Espalhar em vários tracks ao soltar | segurar `Ctrl` antes de soltar | Ableton §7.4 | gesto |

## 8. Testes

Funções puras, `node`, um `assert` cada:

| Função | Entrada | Saída esperada |
|---|---|---|
| `BR.tipoDe("x.ILD")` | — | `"laser"` (extensão sem diferenciar caixa) |
| `BR.tipoDe("x.exe")` | — | `null` |
| `BR.alvoDrop(y, tracks)` | `y` abaixo do último track | `{acao:"novo-track"}` |
| `BR.alvoDrop(y, tracks)` | `y` sobre track `audio`, item `.ild` | `{acao:"recusa"}` (regra 3) |
| `BR.alvoDrop` sobre clipe, x na metade esquerda | — | `{acao:"insere", lado:"antes"}` (regra 4) |
| `BR.espalha(files, ctrl)` | 3 arquivos, `ctrl=false` | um track, três clipes em sequência |
| `BR.espalha(files, ctrl)` | 3 arquivos, `ctrl=true` | três tracks |

Em Rust, ao lado dos testes de `serve`: `POST /files/../x.ild` → 400; `POST /files/x.exe` → 400; `POST /files/a.ild` duas vezes → 200 e 409; arquivo acima do limite → 413.
