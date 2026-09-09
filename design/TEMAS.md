# Funções e temas do Spellcaster

Regra do Matheus (09/09/2026): **não se desenha UI para software sem função.** Cada tela nasce de uma função concreta; o tema é o material que essa função veste; splash, tela de trabalho e info têm a mesma cara. Tudo o que foi feito nas rodadas 1 e 2 (vidro CSS, SDF 2D, cubo raymarched) está jogado fora como UI e guardado como técnica (ver "Espólio").

Regra que amarra: **um tema é um material, e tudo na skin obedece ao material** (forma, o que vive dentro, o que o GO faz, como o estado aparece, que som faz).

## As cinco funções

| # | Função | O que faz (e o comando) | Tema (material) | Rodada |
|---|---|---|---|---|
| 1 | **ILDA player** | Abre `.ild` (formatos 0/1/4/5), toca frames, controla kpps, tamanho, cor, shutter. `spell ilda play show.ild --kpps 30`. Saída Ether Dream / Helios. | **LASER**: a UI é o modelo 3D fotorrealista do próprio projetor 10 W numa sala com névoa. Traseira = menu (portas, VFD, botões); tampa aberta = preferências (diodos, galvos, placa, fonte). A parede mostra o frame com física de verdade: feixes saem da abertura, o galvo tem atraso, o frame pisca se kpps ÷ pontos cair. | 3 → 4 |
| 2 | **NDI → ILDA** | Recebe vídeo NDI, extrai contornos (Canny + simplificação), vira frame ILDA em tempo real. `spell ilda from-ndi "RESOLUME (out)" --kpps 25`. | **FÓSFORO**: rack de broadcast dos anos 70. À esquerda um CRT raster (o NDI), à direita um osciloscópio vetorial (o ILDA). Fósforo verde, knobs Tektronix, ruído de linha. | 4 |
| 3 | **Orquestrador** | O modo Chataigne: módulos (sACN, Art-Net, OSC, MIDI, NDI, ILDA), estados, sequências, mapeamentos. É o Graph do `PRINCIPIOS.md §1` visto de frente. `spell graph`. | **PATCHBAY**: central telefônica de 1960. Baquelite, jacks de latão, cabos de pano com física, etiquetas Dymo. Mapear = plugar cabo. Estado = lâmpada de válvula. | 5 |
| 4 | **Cenas e cues DMX + cenário interativo** | Programa cenas (valores por fixture), cues (cena + fade + follow), e um menu de cenário onde se clica no aparelho na maquete. `spell cue`, `spell scene`, `spell patch`. | **TEATRO DE PAPEL**: maquete de palco de papelão. Aparelhos são recortes que acendem; cenas são bastidores que deslizam; cues são páginas do libreto. GO vira a página. | 6 |
| 5 | **Pino** (companion) | O Clippy do Spellcaster e o **menu principal** de todas as skins. Um cabo DMX com plugue XLR-5 na cabeça: os cinco pinos são botões (1 ILDA, 2 NDI→ILDA, 3 orquestrador, 4 cenas e cues, 5 info), a trava manda ele embora. Vetor (`design/pino.js`), olhos que seguem o mouse, balão Win98. Sabe o contexto: avisa quando o galvo não acompanha, quando o frame pisca, quando um universo não responde. Lembra a última sessão. Substituiu o Aprendiz pixel no voto da rodada 3. | Não tem tema: é o mesmo em todas, como o Clippy era o mesmo em todos os Office. | 4 em diante |

Ordem de build: 3 → 4 (o projetor 3D) → 5 → 6, uma função por rodada, cada uma com voto. O Pino cresce a cada rodada. O NDI → ILDA (FÓSFORO) aparece na porta ETHER do projetor.

## As cinco perguntas por tema

| | LASER | FÓSFORO | PATCHBAY | TEATRO DE PAPEL |
|---|---|---|---|---|
| **Matéria** | Feixe em névoa; parede como tela; vetor com glow e persistência | Fósforo P31 em vidro curvo; alumínio escovado; serigrafia | Baquelite preta; latão; cabo de pano; feltro | Papelão, papel kraft, tinta guache, luz de velas |
| **Fenômeno** | Ponto parado do feixe; névoa se movendo; cantos arredondados pelo galvo | Ruído de linha 60 Hz; retrace; burn-in da imagem antiga | Cabos balançam; lâmpadas piscam quando passa sinal | Papel ondula com o ar; sombra de vela |
| **GO** | Shutter abre, frame aparece com o traço correndo | Trigger dispara: o CRT congela, o vetor desenha | Relé estala, a lâmpada da rota acende | A página do libreto vira; o bastidor desliza |
| **Estado** | Armado = ponto parado; ao vivo = traço correndo; erro = SCAN FAIL, shutter fecha | Armado = tela verde vazia; erro = tela cheia de neve | Armado = lâmpada âmbar; erro = fusível queimado | Armado = cortina fechada; erro = a vela apaga |
| **Som** | Chip com onda quadrada, arpejo rápido (o "canto" do galvo) | Chip com senoide + hum 60 Hz | Chip com pulse + estalos de relé | Chip com triângulo + papel amassando |

## Arco 5E em cada função

Mesma sequência sempre, veste-se de tema: **Excitement** = ligar (o material acorda: feixe parado, CRT aquecendo, lâmpadas de teste, cortina fechada) · **Entry** = splash com o logo desenhado pelo material + jingle no timbre do tema · **Engagement** = a tela de trabalho · **Exit** = o material apaga (shutter, tela para um ponto, cabos desligam, cortina) com o acorde final · **Extension** = o Aprendiz lembra e comenta na próxima abertura.

## Espólio das rodadas 1 e 2 (técnica reaproveitável, UI descartada)

- Splash cracktro com jingle Web Audio sintetizado (zero mídia): aprovado, vira padrão de Entry.
- Fragment shader WebGL para material físico (raymarching, refração, dispersão, absorção): guardado para FÓSFORO (vidro do CRT) e PATCHBAY (latão).
- Face em CSS 3D com a mesma matriz do shader: guardado para qualquer tela com objeto 3D.
- Leitor `.wmz`: descartado (voto). Fica em `rodada2/` como referência de formato.
- Gels Lee/Rosco como filtro físico: guardado para TEATRO DE PAPEL (gelatina de verdade na frente do recorte).
- Voto dentro do protótipo com `db`: padrão de todas as rodadas.
