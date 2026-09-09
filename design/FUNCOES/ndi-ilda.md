# NDI → ILDA — `spell ilda from-ndi` (tema FÓSFORO)

Receber vídeo por NDI, vetorizar cada quadro em caminhos e entregar frames ILDA à mesma saída laser do `ilda-player.md`. Tudo que é saída (PPS, blanking, cor, segurança, armar, shutter) está lá e não se repete aqui; este arquivo cobre a entrada e a conversão.

| Item | Quem resolveu melhor | Por quê |
|---|---|---|
| Objetos e verbos | MadMapper | Os dois algoritmos (contornos vs esqueleto), o filtro de comprimento, o limite "fique com os N mais longos" e o campo de erro da vetorização são o conjunto completo, com chaves reais no projeto salvo |
| Estados | TouchDesigner | O NDI In não tem parâmetro de FPS: tem seis canais de leitura que são o painel de saúde do link; e `updatemethod` nomeia o flicker |
| Zonas da tela | MadMapper, com o Monitor do Resolume | Um preview só, com quatro modos (fonte, processada, processada + polilinhas, só polilinhas); preview e saída são o mesmo widget |
| Atalhos | Blender | Toggle com seta para os parâmetros de Canny; nada modal |
| Arquivo | MadMapper, com o `failovername` do TouchDesigner | A saída vetorizada é publicada como mídia interna (loopback), consumível pelo resto do graph; a fonte NDI tem nome de contingência |

## 1. Objetos e verbos

**Fonte NDI.** Parâmetros, do NDI In TOP [fontes/touchdesigner.md § Laser e NDI]:

| Parâmetro | Tipo | Nota |
|---|---|---|
| Fonte/Nome | enum auto-populado pelos streams descobertos | descoberta por mDNS, "costuma ser limitada a redes locais" |
| Fonte/IPs extras | lista de IP | para fontes fora do multicast |
| Fonte/Banda | enum alta, baixa | dois valores só; baixa é o modo de ensaio no Pi |
| Fonte/Contingência | nome no formato `MÁQUINA (Fonte)` | do NDI Out `failovername`, lido aqui do lado do receptor: para onde migrar se a fonte cair |
| Fonte/Grupos | lista | filtra as fontes listadas |

Sem parâmetro de FPS: o frame rate chega como leitura (ver Estados). Decodificação por hardware só vale para NDI|HX, e "soluções em software só enviam NDI nativo", então o toggle fica escondido até a fonte ser HX. Multicast configura-se fora, no NDI Access Manager; jumbo frames de 9014 bytes resolveram queda de frames em gigabit, e isso é frase do Aprendiz, não parâmetro.

**Vetorizador.** Dois algoritmos, com as chaves do grupo `Process` do `Laser Example.mad` [fontes/madmapper.md § Laser]:

- **Contornos** (Canny): `Limiar`, `Tamanho`, `Desfoque`. Bordas de forma cheia.
- **Caminhos** (esqueleto): `Limiar`, `Espessura` (do traço no material de origem), `Redução de ruído`, `Resolução máxima`; por baixo, thinning Zhang-Suen e recomposição por tolerância de ângulo, cor e distância. Traço de linha.

O TouchDesigner tem um terceiro caminho, raster: o Scan CHOP converte a imagem em varredura de osciloscópio, com `largura`, `altura`, `níveis` de brilho, redução automática para manter o frame rate, ordem por brilho e entrelaçamento `sweep | evenodd | max` "para minimizar o flicker" [fontes/touchdesigner.md § Laser e NDI]. Está depreciado lá, mas é o único raster→vetor documentado nos seis apps. Fica como terceiro valor do enum `Algoritmo`, para quando a fonte é texto ou logotipo cheio.

**Filtro de caminhos.** `Comprimento mínimo` e `máximo` em porcentagem da maior dimensão da mídia; `Limite: manter os N mais longos` [fontes/madmapper.md § Laser]. O engine desiste acima de 2 000 caminhos; esse teto é constante visível, não surpresa.

**Suavização.** Entre o contorno e o frame entram filtros da cadeia de mapping do Chataigne: `Damping`, `Lag`, `OneEuro`, `Speed`, `CurveMap`, cada um devolvendo `CHANGED | UNCHANGED | STOP_HERE` [fontes/chataigne.md § Mappings e Actions]. São nós do graph (`orquestrador.md`), não parâmetros escondidos do vetorizador; aqui só se declara que a saída do vetorizador é uma porta de tipo `frame`.

**Frame de saída.** O que sai do vetorizador é o mesmo objeto que o player entrega: lista de pontos com `id` de forma. O Laser CHOP agrupa pontos por canal `id`, e sem ele "cada ponto é solto e desconectado" [fontes/touchdesigner.md § Laser e NDI]; o MadMapper faz igual com `shapeNumber`, "toda vez que muda, começa um path novo" [fontes/madmapper.md § Laser]. Um caminho = um `id`.

Verbos: conectar, desconectar, escolher algoritmo, congelar quadro, gravar (o snapshot do Capture grava DMX, vídeo, laser e câmera juntos, e na reprodução sobrepõe a entrada externa) [fontes/capture.md § O que copiar], publicar a saída como mídia interna (MadMapper loopback, `Dispatch Count` reparte em N mídias, uma por projetor) [fontes/madmapper.md § Laser].

## 2. Estados

**Saúde do link**, seis leituras do NDI In, sempre visíveis quando a fonte está escolhida [fontes/touchdesigner.md § Laser e NDI]:

| Leitura | Estado que gera |
|---|---|
| `connected` | desconectado / conectado |
| `receive_fps` | FPS recebido; comparado ao FPS real do laser dá "o galvo não acompanha" |
| `num_source` | zero = "nenhuma fonte na rede", frase do Aprendiz |
| `queue_size` | fila crescendo = latência subindo |
| `received_frames` | contador |
| `missed_frames` | subindo = `warning`, a frase "perdendo quadros" |

Mais os do Capture, `Requesting` e `Receiving`: a mídia diz o que pediu e o que está recebendo, dois estados antes de "conectado" [fontes/capture.md § Estados e mensagens].

**Vetorização**: caminhos encontrados / limite; pontos necessários / `PPS ÷ FPS`; `Monitor/Info` do MadMapper como campo de erro em texto [fontes/madmapper.md § Laser]. Quando pontos necessários > pontos disponíveis, o estado é o mesmo `warning` "galvo não acompanha" do player, com a causa "vetorização" em vez de "clipe".

**Congelado**: o quadro atual fica, a fonte continua chegando. Independente do shutter e do transporte (MadMapper separa congelar engine de congelar saída) [fontes/madmapper.md § Estados].

**Pendente**: mudar de algoritmo aplica no próximo quadro; vermelho até lá [fontes/touchdesigner.md § Estados].

**Latência**: `queue_size` do NDI mais `Delay` por saída (Resolume: 0–150 ms por dispositivo, "hardware real chega fora de fase") [fontes/resolume.md § O que copiar]. O delay é da saída DMX e da saída laser separadamente; é assim que o laser vetorizado se alinha com o DMX que veio do mesmo vídeo.

## 3. Zonas da tela

- **Esquerda**: lista de fontes NDI descobertas, com `receive_fps` ao lado do nome. Lista longa vira busca.
- **Centro, viewer**: um só, com o seletor de modo do MadMapper: `Imagem fonte`, `Imagem processada`, `Processada + polilinhas`, `Só polilinhas` [fontes/madmapper.md § Laser]. Mais o modo diagnóstico por ponto do player. Máscaras e área segura por cima em todos os modos.
- **Direita, Inspector**: `Fonte/`, `Vetorização/`, `Filtro/`, `Saída/` (o mesmo grupo do player). O toggle `Vetorizar` com seta: liga e abre os parâmetros do algoritmo escolhido; trocar de algoritmo troca o conteúdo do popover [fontes/blender.md § Anatomia de um editor].
- **Embaixo**: sem timeline própria; a régua é a do show. O painel de saúde (as seis leituras + caminhos + pontos + FPS real) fica na status bar, no slot de estatísticas [fontes/blender.md § Estados].

Menu do painel: `View, Select, Add, Fonte`.

## 4. Atalhos

Tudo do player vale. O que entra:

| Ação | Tecla | Origem | Conflito |
|---|---|---|---|
| Ciclar modo do viewer | `V` | (nosso; MadMapper tem o seletor sem tecla) | nenhum em `SHORTCUTS.md` |
| Congelar quadro | `F` | (nosso) | nenhum; `Shift+F` é tela cheia |
| Ligar/desligar vetorização | `Shift+V` | (nosso) | nenhum |
| Parâmetros de Canny | popover pela seta ao lado do toggle | Blender | gesto |
| Limiar por escada de valor | segurar botão do meio | TD Value Ladder | gesto |

Regra: nenhum atalho de fonte muda estado de saída. Armar, shutter e blackout são os do player.

## 5. Arquivo

No `.spell`:

- `sources[]`: `{uid, kind: "ndi", name, extra_ips[], bandwidth, failover, groups[]}`. Nome de fonte é identidade fraca (muda com a máquina que emite); `failover` é a segunda tentativa declarada, nunca heurística.
- `vectorizer`: `{source, algorithm (id estável: contours | paths | raster), params completos dos três algoritmos, filter: {min_len, max_len, keep_longest}, publish_as: "media/<nome>"}`. Guardar os parâmetros dos três algoritmos, mesmo o inativo, para trocar sem perder ajuste; é o que o TouchDesigner faz com os quatro modos de parâmetro guardados ao mesmo tempo [fontes/touchdesigner.md § O que copiar].
- `outputs[]`: os do player, com `delay_ms` por saída.

Fora do `.spell`: lista de fontes vista na rede, `receive_fps`, contadores, quadro congelado.

O que a saída publica: um `frame` no graph, endereçável como `media/<nome>` (o loopback do MadMapper publica os paths de um output sem destino como Live Input, e `Dispatch Count` reparte em N mídias) [fontes/madmapper.md § O que copiar]. Assim o orquestrador roteia o mesmo vetor para dois lasers sem duplicar o vetorizador.

Gravação: um só gravador para os quatro fluxos (DMX, vídeo, laser, câmera), e "Recall DMX" para congelar um estado quando não há sinal [fontes/capture.md § O que copiar]; o arquivo gravado é o do `ilda-player.md § 5` no modo "a FPS fixo".
