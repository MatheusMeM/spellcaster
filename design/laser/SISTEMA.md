# Spellcaster LASER — design system

Sistema da ferramenta de laser e ILDA. Não é o design system da Feitiçaria Industrial: vale só aqui, dentro do módulo de laser. Fonte de verdade dos valores: `tokens.css` nesta pasta. Este documento diz o que cada token significa e quando usar.

---

## 1. Princípio

O programa **é** o modelo 3D fotorrealista do próprio projetor RGB 10 W. Não é uma janela com um viewport 3D dentro: a interface é o aparelho.

- **Traseira do aparelho = menu.** Todo controle vive num conector, botão, chave ou display que existe no equipamento real. Clicar no painel traseiro é operar o painel traseiro.
- **Tampa aberta = preferências.** Ajuste fino, calibração, kpps, limites de scan, ILDA in/out: coisas que num laser de verdade se mexe com a tampa fora.
- **A parede é o output.** O que sai do aparelho aparece projetado na parede da sala, em névoa. Preview de ILDA não é um retângulo de UI: é feixe.
- Nada aparece na tela por conveniência de software. Se não tem função no aparelho, não existe.

Consequência prática: **função antes de UI**. Cada elemento novo precisa responder "que peça do laser é essa e o que ela faz". Se a resposta é "é um card", corta.

## 2. Material único

Um material, um acento, dois semáforos.

- **Alumínio anodizado preto** — todo o corpo, todo painel, todo fundo. `--bg:#050608`, `--panel:#07090C`, arestas `--line:#1A2026` e `--line-2:#2A333C`. Superfície fosca, sem gradiente decorativo, sem vidro por cima da UI.
- **Feixe em névoa** — a única fonte de luz da cena. Todo glow do sistema vem do feixe, nunca de sombra de card.
- **Acento verde do feixe `#38FF5C` (`--las`)** — ativo, LIVE, armado, selecionado, valor numérico corrente. Verde na tela significa "isto está emitindo ou pronto para emitir".
- **Âmbar `#FFB000` (`--amb`)** — aviso, ação pendente, LEARN em curso, CLI ecoado. Âmbar nunca é um estado final.
- **Vermelho `#FF2A1A` (`--red`)** — perigo, desarmado, e-stop, interlock aberto, SCAN FAIL.
- **OLED `#9FF5D0` (`--oled`)** — e só — o displayzinho do painel traseiro. Nenhum outro elemento usa essa cor; ela identifica "isto é a tela física do aparelho".
- **Azul `#3A6BFF` (`--blue`)** — reservado para ILDA/DMX externo (sinal que vem de fora). Uso raro; não é um segundo acento.

Texto: `--fg:#E6EBEF` para leitura, `--dim:#7C8791` para rótulo e metadado.

## 3. Tokens (`tokens.css`)

Ler o arquivo, não decorar valores. Resumo do contrato:

**Cor** — `--bg --panel --fg --dim --line --line-2 --las --amb --red --oled --blue --pino-on`.

**Tipografia** — `--font-disp: Michroma` (título, HUD de título, `h1/h2/h3`, botão de salvar) e `--font-mono: Share Tech Mono` (todo o resto: rótulo, valor, CLI, tabela, botão). Duas famílias, sem terceira.

**Escala** — `--fs-0:10px --fs-1:11px --fs-2:12px --fs-3:13px --fs-4:15px --fs-5:18px --fs-6:34px`. 10 e 11 são para rótulo de painel e HUD de aviso; 12–13 é o corpo da UI do aparelho; 15 é texto lido de longe; 34 só para título de tela.

**Espaçamento** — `--s1:4 --s2:8 --s3:12 --s4:16 --s6:24 --s8:32`. Nada de 6, 10, 18 improvisados.

**Chanfro** — `--chamfer:14px` em painel (`clip-path` cortando cantos opostos) e `--chamfer-room:30px` na sala. Painel retangular está errado; é peça usinada.

**Glow** — `--glow` (repouso do acento), `--glow-on` (ativo), `--glow-red` (perigo). Glow é do material, não é `box-shadow` de elevação.

**Tempo** — `--t-fast:120ms` (feedback de botão), `--t-ui:220ms` (abrir/fechar painel, troca de estado), `--t-cam:700ms` (movimento de câmera). Três tempos, sem quarto.

## 4. Componentes

**HUD `.hud`** — texto sobre a cena, `pointer-events:none`, `text-shadow:0 0 8px currentColor`. Variantes: `.title` (Michroma, cor de texto), `.cli` (âmbar, ecoa o comando `spell` equivalente), `.warn` (10px, âmbar), `.danger` (vermelho). HUD nunca recebe clique — quem recebe clique é a peça do aparelho.

**Botão laser `.lb`** — contorno do acento, fundo quase preto, preenchimento só quando ativo.
- padrão: borda e texto verdes, `--glow`;
- `.on`: preenchido `rgba(56,255,92,.18)`, `--glow-on` — o estado ligado é visível de longe;
- `.red` / `.red.on`: perigo e perigo acionado;
- `.amb`: aviso/pendente;
- `:disabled`: `opacity:.35`, sem glow, `cursor:not-allowed` — desabilitado não brilha;
- `:focus-visible`: outline branco 2px, offset 2px;
- `small` dentro do botão = atalho ou unidade, `opacity:.6`.

**Painel `.panel`** — opaco (nunca translúcido), borda do acento, chanfro 14px, `z-index:5`. Estrutura fixa: `h3` (Michroma) + `.sub` (11px, dim) + linhas `.row` no grid **rótulo · controle · valor** (`96px 1fr 58px`), valor alinhado à direita, verde, `tabular-nums`. `.x` fecha no canto superior direito. `.btns` agrupa `.lb` menores no rodapé do painel. `pre` dentro do painel = bloco de CLI.

**Tooltip `.tip`** — fundo verde sólido, texto preto, sem borda, sem seta, 11px, `display:none` até o hover. É etiqueta de bancada, não balão.

**Balão do Pino `.bal`** — Win98 literal: `#FFFFE1`, borda preta 1px, sombra dura `2px 2px 0 #000`, Tahoma 12px, seta em `::after`, `x` no canto. É o único elemento "de software" da cena, e isso é proposital — o Pino é o mascote-menu, não faz parte do aparelho. Lista de opções em `ul/li` com hover `#0A246A` invertido, `small` para o atalho. Voz curta.

**Tabela de bindings** — dentro de `.panel`, `td.k` (ação, cor de texto), `td.b` (binding atual, dim, `nowrap`) e um `.lb.learn` âmbar com `animation:pulse 1s infinite`. Enquanto pulsa, o próximo evento de teclado ou MIDI captura. `prefers-reduced-motion` desliga o pulso — o botão continua âmbar.

**Sala `.room`** — fundo preto puro `#000`, `overflow:hidden`, `clip-path` **não retangular**: chanfro de 30px nos quatro cantos e um recorte trapezoidal na base central (a marca do aparelho no chão). Tudo que é cena vive dentro dela.

## 5. Estados do aparelho

Quatro estados, e nada entre eles. O estado é lido simultaneamente no feixe, no OLED e no LED de emissão.

| Estado | Feixe | OLED (`--oled`) | LED de emissão | UI |
|---|---|---|---|---|
| **DESLIGADO** | ausente | apagado | apagado | painéis `disabled`; sala escura; só a chave de força responde |
| **STANDBY** | ausente | `STANDBY / INTERLOCK OK` | âmbar piscando lento | controles editáveis, ARM disponível; HUD âmbar "sem emissão" |
| **SCAN FAIL** | cortado imediatamente | `SCAN FAIL` invertido | vermelho fixo | `.lb.red.on` no ARM, todo output travado até reset; balão do Pino explica |
| **LIVE** | visível, verde/RGB na névoa | `LIVE · 30 kpps` | verde fixo | HUD verde, CLI ecoando, e-stop sempre alcançável |

Regras: nunca ir de DESLIGADO direto a LIVE; SCAN FAIL só sai por reset explícito; qualquer perda de interlock cai para SCAN FAIL, não para STANDBY.

## 6. Câmera — padrão SolidWorks

A órbita é a do SolidWorks, sem invenção:

- **MMB arrasta** — gira em torno do **ponto clicado** (não do centro da cena).
- **Ctrl + MMB** — pan.
- **Shift + MMB** — zoom.
- **Roda** — zoom no cursor, **direção invertida por padrão**, com toggle nas preferências.
- **Setas** — gira 15°; **Shift + setas** — 90°; **Ctrl + setas** — pan.
- **F** — enquadra a seleção (ou a cena, se nada selecionado).
- **Ctrl + 1..7** — vistas padrão (frente, trás, esquerda, direita, topo, base, isométrica).
- **Arrastar com o botão esquerdo no vazio** também gira — quem não tem botão do meio não fica de fora.

Todo movimento de câmera usa `--t-cam` (700 ms) com ease-out. Sem inércia infinita, sem "flutuar".

## 7. Key binding

Qualquer ação do sistema vira tecla **ou** evento MIDI, in e out.

- Botão **LEARN** por linha da tabela; ao pulsar, captura o próximo evento (tecla, note on, CC).
- **MIDI out** existe: o estado do controle volta para a superfície (LED do pad acende quando ARM está ligado). Binding é bidirecional por padrão.
- Persistência em `localStorage`, chave **`sc-laser-bind`**, JSON `{ acao: {key, midi} }`.
- Ação sem binding é válida; binding sem ação não existe.
- Toda ação bindável tem CLI equivalente — é o mesmo verbo do registry.

## 8. Movimento

Splash, uma vez, na abertura:

1. O foco da câmera **sai do ponto estático** onde estava e **vai para a parede** — o output — ignorando o laser e a traseira dele. Sala em névoa, aparelho fora de quadro (ou desfocado em primeiro plano).
2. O laser **contorna** `SPELLCASTER LASER` na parede, em movimento dinâmico e contínuo — sem ILDA, sem show, só o outline.
3. As letras vão sendo **reveladas** conforme o feixe passa e **ficam** na parede.
4. Todas **brilham** juntas ao fechar o contorno.
5. Só então a câmera **voa para a traseira** (o menu) — ou para a tampa aberta (preferências), `--t-cam`.
6. O laser **apaga** e a luz da sala **baixa** um pouco. A partir daqui, interatividade.

Fora da splash: transição de painel em `--t-ui`, feedback de botão em `--t-fast`. `prefers-reduced-motion` corta a splash para o quadro final e remove pulsos.

## 9. Som

- **Jingle** de onda quadrada na splash — curto, chiptune, sem sample.
- **Blip** ao revelar cada letra e ao capturar um LEARN.
- **Clique** seco de botão físico em `.lb`.
- **Acorde descendente** ao desarmar / entrar em SCAN FAIL.
Som é confirmação de ação física, nunca trilha. Mudo é padrão respeitado e persistido.

## 10. Texto

Voz do **Pino**: curto, técnico, humor seco. Frase de uma linha. Sem "Ops!", sem exclamação dupla, sem tutorial.

- Bom: `Interlock aberto. Sem interlock, sem feixe.`
- Ruim: `Ops! Parece que algo deu errado com o seu interlock :(`

**CLI sempre visível.** Todo controle mostra o comando equivalente em HUD `.cli` ou em `pre` dentro do painel:

```
spell ilda play show.ild --kpps 30
```

Quem aprende a GUI aprende o CLI de graça. Nomes na tela = nomes no registry.

## 11. Acessibilidade

- **Foco visível** obrigatório: `:focus-visible` com outline branco 2px e offset 2px em todo controle, inclusive dentro da cena 3D. Ordem de tab segue a leitura do painel.
- **`prefers-reduced-motion`**: sem splash animada, sem pulso do LEARN, sem transição de painel; a informação continua toda presente por cor e texto.
- **Contraste** — pares aprovados sobre `--bg`/`--panel`: `--fg`, `--las`, `--amb`, `--oled` e `--dim` (só para rótulo, nunca para informação crítica). `--red` sobre preto é aprovado para ícone e borda; texto longo em vermelho, não. Estado **nunca** é comunicado só por cor: sempre cor + rótulo (LIVE, STANDBY, SCAN FAIL) + posição.
- Alvo de clique mínimo 24px na cena, mesmo quando a peça modelada é menor.

## 12. O que NÃO fazer

- **Software quadradão** — grid de cards, sidebar, barra de título de app. Se parece com um dashboard, está errado.
- **Painel transparente** — vidro, blur, translucidez na UI. O único material que transmite luz é a névoa. Painel é alumínio: opaco.
- **Elemento sem função** — dois interlocks, fusível decorativo, LED que não indica nada, conector que não conecta. Um interlock. Sem fusível.
- **Cartoon** — traço grosso, cor pastel, ícone arredondado, mascote fofo. O Pino é Win98 seco, não é bichinho.
- Terceira fonte, quarta cor de acento, valor de espaçamento fora da escala, tempo fora dos três.
- Glow como elevação de card. Glow é feixe.
- Cor sozinha carregando estado.
