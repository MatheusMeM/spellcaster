# Aprendiz — o menu principal (todos os temas)

O Aprendiz é a paleta de comandos, a ajuda contextual, o painel de avisos e a status bar, com uma voz. Não é um personagem que interrompe: é a superfície de texto que `PRINCIPIOS.md §4` exige de toda tela, e o lugar onde "o galvo não acompanha" (`TEMAS.md`) é dito.

| Item | Quem resolveu melhor | Por quê |
|---|---|---|
| Objetos e verbos | Blender, com a tabela de ajuda do Resolume | Busca que indexa rótulos de menu (não operadores), favoritos, "ajustar última operação", "repetir última"; ajuda contextual como dado, uma frase por elemento |
| Estados | Chataigne, com o TouchDesigner e o Capture | Avisos registrados por id com o culpado clicável; erro e aviso como contagem por nó; mensagem que nomeia a causa provável e põe o log a um clique |
| Zonas da tela | Blender | Status bar com quatro coisas em ordem fixa; a mensagem nunca some, só muda de lugar; Playback e Keying como popovers |
| Atalhos | Blender | `F3`, `Q`, `F9`, `Shift+R`, `F2`, `Shift+Ctrl+C`; qualquer menu vira busca ao digitar; ajuda com `Alt`+hover vem do TouchDesigner |
| Arquivo | Resolume, com o `help` do TouchDesigner e o `.c2l` do Capture | Corpus de ajuda como arquivo de dados, chaveado pelo elemento; dica é texto puro; tradução é `Seção/Chave`; tudo gerado do registry, não paralelo a ele |

## 1. Objetos e verbos

**Comando.** O `@command` do registry, com `label`, `help` (uma frase), `menu` (caminho `Painel/Menu/Item`), `shortcut`. Um comando só é encontrável se estiver em um menu: no Blender, "o `text=` é a chave de busca" [fontes/blender.md § Paleta de comandos]. Aqui o rótulo do menu **é** o nome do registry (regra 10), então a paleta indexa os dois de uma vez.

**Menu.** `View, Select, Add, <objeto do painel>` em todo painel (regra 6); barra do app com `Arquivo, Editar, Show, Janela, Ajuda` (o topbar do Blender é `Blender, File, Edit, Render, Window, Help`) [fontes/blender.md § Anatomia de um editor]. Menu com mais de 8 itens vira busca ao primeiro caractere (`SEARCH_ON_KEY_PRESS`). Os 16 tool panes do Capture em coluna sem busca são o contra-exemplo [fontes/capture.md § O que NÃO copiar].

**Paleta.** `F3`. O resultado mostra o rótulo, o caminho do menu onde mora, o atalho e a frase de ajuda. A busca por nome interno de operador fica atrás de "modo desenvolvedor" no Blender [fontes/blender.md § Paleta de comandos]; aqui não há diferença entre os dois nomes, então não há modo.

**Favoritos.** `Q`, o menu que o usuário monta botão a botão [fontes/blender.md § Paleta de comandos]. Fica no perfil, não no show.

**Última operação.** `F9` abre o painel HUD com os parâmetros do último comando, editáveis: "depois de um GO, o operador quer corrigir o fade sem repetir o cue" [fontes/blender.md § O que copiar]. `Shift+R` repete.

**Ajuda contextual.** Tabela `elemento → uma frase`: os 412 verbetes de `docs\help\English.xml` do Resolume são "o corpo do Aprendiz pronto" [fontes/resolume.md § O que copiar]. Cada parâmetro carrega `help` no esquema (TouchDesigner `TDJSON.py`) [fontes/touchdesigner.md § Arquivo]; `Alt`+hover sobre o rótulo mostra [fontes/touchdesigner.md § Parâmetros].

**Dica.** Texto puro, uma por linha, o `TouchDesignerTips.txt` [fontes/touchdesigner.md § Fontes lidas]. Aparece no splash e no slot de estatísticas quando não há aviso.

**Aviso.** `{alvo (endereço), id, mensagem, severidade}`, registrado por qualquer objeto com `setWarningMessage(msg, id)` e limpo por id [fontes/chataigne.md § Estados visuais]. A linha do painel resolve para o culpado. Exemplos, com a fonte de cada número:

| Frase do Aprendiz | Origem do critério |
|---|---|
| "FPS real 28 Hz: o galvo não acompanha. PPS 30 000 ÷ 1 070 pontos." | MadMapper, `ILDA FPS = PPS / Point Count`, pisca abaixo de 35 [fontes/madmapper.md § Laser] |
| "Perdendo quadros da fonte NDI (missed_frames subindo)." | TouchDesigner, canais de info do NDI In [fontes/touchdesigner.md § Laser e NDI] |
| "Universo 3 sem resposta há 4 s. Possivelmente bloqueado por firewall. Abrir log." | Capture `PotentiallyBlocked` + `OpenLogFolder` [fontes/capture.md § Estados e mensagens] |
| "Este patch consome 39 canais e transborda o universo 1 em 7. Continuar no 2?" | Capture `ChannelsRequired` + `OverflowWithContinue` [fontes/capture.md § O que copiar] |
| "Aparelhos 4 e 5 sobrepõem os canais 40–48." | Resolume, painel DMX Output [fontes/resolume.md § O que copiar] |
| "Taxa da saída Art-Net acima de 44 Hz." | TouchDesigner DMX Out [fontes/touchdesigner.md § Laser e NDI] |
| "1 830 caminhos vetorizados, limite 2 000." | MadMapper `Monitor/Info` [fontes/madmapper.md § Laser] |
| "Preset OSC 'OutputAllMessages' não existe; usando o padrão." | Resolume, referência por nome quebrada [fontes/resolume.md § O que NÃO copiar] |

O aviso chega quando o custo sobe, não quando o teto estoura (`TooManySmokeObjects` do Capture chega tarde) [fontes/capture.md § O que NÃO copiar].

**Log.** `{hora, fonte, severidade, texto}`, 2 000 entradas em memória, gravação em arquivo opcional [fontes/chataigne.md § Estados visuais]. O Error DAT do TouchDesigner registra erros ao longo do tempo "para caçar intermitente" [fontes/touchdesigner.md § Estados].

**Vigia.** "Vigie este parâmetro e plote o histórico" (Detective) [fontes/chataigne.md § Estados visuais]. Verbo de qualquer campo.

Verbos: buscar e executar, explicar, avisar, ir ao culpado, repetir, ajustar, copiar comando CLI, favoritar, abrir pasta de log, vigiar, dispensar.

## 2. Estados

As quatro tintas do bloco State do Blender, e só elas: `error`, `warning`, `info`, `success` [fontes/blender.md § Estados]. Um aviso é `warning`; o que para o show é `error`; "cue gravada" é `success` por dois segundos; dica é `info`.

- **Aviso ativo / limpo**: por id; quando o alvo conserta, o aviso some sozinho [fontes/chataigne.md § Estados visuais].
- **Contagem por nó**: `warnings` e `errors` como número em cada nó, lidos pela GUI, pela CLI e pelo Aprendiz da mesma fonte (regra 5).
- **Tarefa rodando**: barra de progresso no terceiro slot da status bar [fontes/blender.md § Estados].
- **Atividade não é funcionamento**: verde é tráfego; o Aprendiz só diz "funciona" quando há resposta [fontes/capture.md § Estados e mensagens].
- **Mensagem nunca some**: se a status bar está escondida, o banner de aviso e o de tarefa migram para o topo [fontes/blender.md § Estados].
- **Falha cedo, com nome**: o Capture separa sete falhas de inicialização (configuração, licença, rede, vídeo, conectividade, recursos, tempo real) [fontes/capture.md § Estados e mensagens]. `spell` reporta o mesmo no boot do Pi.

Não entra: diálogo que pede desculpa por bug (`DisableAdaptiveQualityWarning`) [fontes/capture.md § O que NÃO copiar]; modal de qualquer tipo para aviso (`SHORTCUTS.md`: diálogo só para abrir/salvar).

## 3. Zonas da tela

- **Status bar** (rodapé, sempre): quatro coisas, nesta ordem, e nada mais [fontes/blender.md § Estados]: (1) o que os botões do mouse fazem agora, no contexto; (2) a última mensagem; (3) tarefa rodando; (4) estatísticas ou dica. O item 1 é a linha de aprendizado do software e ensina a gramática de modificadores de `cenario-interativo.md § 4` gesto a gesto.
- **Painel Log (`Shift+6`)**: abas `Log | Avisos | Ajuda`, o canto inferior direito do Chataigne [fontes/chataigne.md § Anatomia da tela]. Avisos com o culpado clicável; Ajuda mostra a frase do elemento sob o mouse ou selecionado.
- **Paleta**: flutuante no centro, só enquanto `F3` está aberta; fecha com `Esc` ou ao executar.
- **HUD de última operação**: rodapé do painel focado, aparece com `F9`, some ao clicar fora (a região `HUD` do Blender) [fontes/blender.md § Anatomia de um editor].
- **Popovers no header**: toggle com seta em vez de janela; Playback e Keying no Timeline do Blender são popovers [fontes/blender.md § O que copiar].
- **Menu**: barra do app no topo; menus de painel no header de cada painel, colapsáveis.

O Aprendiz como personagem (o Clippy do `TEMAS.md`) mora no slot 2 da status bar e na aba Ajuda. Nunca cobre o viewer, nunca bloqueia.

## 4. Atalhos

| Ação | Tecla | Origem | Conflito com `SHORTCUTS.md` |
|---|---|---|---|
| Paleta de comandos | `F3` | Blender `wm.search_menu` [fontes/blender.md § Atalhos] | nenhum |
| Favoritos | `Q` | Blender `SCREEN_MT_user_menu` | nenhum |
| Ajustar última operação | `F9` | Blender `screen.redo_last` | nenhum; TD usa `F9` para mostrar a rede sob o cursor, irrelevante |
| Repetir última operação | `Shift+R` | Blender | nenhum; `R` é record arm |
| Renomear item ativo / em lote | `F2` / `Ctrl+F2` | Blender | nenhum |
| Copiar o comando CLI do campo sob o mouse | `Shift+Ctrl+C` | Blender `copy_data_path` | nenhum |
| Ajuda do elemento sob o mouse | `Alt`+hover; `F1` fixa a ajuda na aba | TD [fontes/touchdesigner.md § Parâmetros]; `F1` nosso | TD usa `F1` para Perform Mode; aqui a Face troca por `Tab` |
| Filtrar a lista sob o cursor | `Ctrl+F` | Blender | nenhum |
| Qualquer menu vira busca | digitar com o menu aberto | Blender `SEARCH_ON_KEY_PRESS` | gesto |
| Ir ao culpado do aviso | clique na linha, ou `Enter` com a linha selecionada no painel Avisos | Chataigne, TD Errors Dialog | `Enter` é GO fora do painel; painel focado decide |
| Fechar paleta / HUD | `Esc` toque | `SHORTCUTS.md` | segurado é blackout |
| Trocar o tipo de editor do painel | não entra | Blender `Shift+F1..F12` | Faces são presets nomeados, não painéis trocáveis |

Desabilitar um atalho: manter a linha em `config.json`, apagar o comando (regra 8). O mapa de teclas precisa de um teste que o compare com o comportamento: o `TouchShortcuts.txt` traz `forward left` e `backward right` invertidos em relação ao wiki "desde sempre" [fontes/touchdesigner.md § O que NÃO copiar]; `spell keys check` lista o mapa e falha se um comando não existe no registry.

## 5. Arquivo

Tudo do Aprendiz é gerado do registry ou é texto ao lado dele; nada é paralelo.

- **Ajuda**: o campo `help` de cada `@command` e de cada parâmetro é a fonte. `spell help --export` gera `help/pt-BR.json` no formato `{ "nome.do.registry": "uma frase" }`, o `elemento → frase` do Resolume [fontes/resolume.md § Fontes lidas], para revisão e tradução. Um verbete sem comando correspondente é erro de build.
- **Tradução**: um arquivo por idioma, chaveado por nome do registry, o `Seção/Chave` do `.c2l` [fontes/capture.md § Fontes lidas]. Atalho nunca é string traduzível: no Capture "nenhuma Phrase contém Ctrl, Shift ou F1" [fontes/capture.md § Atalhos].
- **Dicas**: `tips/pt-BR.txt`, uma por linha.
- **Favoritos, mapa de teclas, layout das Faces**: `config.json` do perfil, nunca no `.spell`.
- **Avisos**: não se salvam; renascem do estado ao abrir.
- **Log**: `logs/<data>.log` quando ligado; "Abrir pasta de log" é um comando.
- **Última operação**: memória de sessão; o `F9` edita o que ainda está na pilha de undo.

Corpus de origem para escrever as frases, já em disco: os 412 verbetes do Resolume, as 74 dicas do TouchDesigner, as mensagens nomeadas do Capture (`.c2l`, 1 885 linhas) e as páginas de Laser/NDI/DMX do wiki offline do TouchDesigner, todos citados em `fontes/`.
