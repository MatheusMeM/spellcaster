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

- Commits e pushes só na conta do Matheus, sem crédito ao Claude. Branch `design/0.1.2`.
- Agentes sempre em Opus.
- Ponytail full; Python só em `.py`; UTF-8 sem BOM; binários pesados fora da pasta do Drive.
- Função antes de UI; um material por tema; votar dentro do protótipo; ser wild; nada de software quadradão.

## 09/09/2026 · foco da câmera na splash

- [x] Na splash o foco da câmera **sai do ponto estático** em que ela está mirando e **vai para a tela (parede) mesmo**, ignorando o laser e a traseira dele. Só depois ela vai e foca em outra coisa: o menu (traseira) ou as preferências (tampa).
