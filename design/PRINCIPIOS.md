# Spellcaster — princípios de interface

Uma página. Cada princípio tem o que ele proíbe. Revisão de qualquer tela, widget ou theme passa por esta lista antes de virar código.

Pessoa de referência: o operador na mesa às 23h do dia da montagem, sala escura, mão no GO, sem tempo para procurar.

## 1. O graph é a interface

Todo widget é um nó visível. Uma Face é uma vista do Graph; o Graph vive no engine e roda igual no Pi sem tela.

Proíbe: botão que faz algo que não está no Graph; lógica na GUI; "atalho mágico" sem nó correspondente.

## 2. Um accent, e ele significa estado

Tudo parado é cinza. Cor aparece só quando algo acontece: armado, ao vivo, GO, ensaio, erro. Se a tela está colorida, o show está acontecendo. Isso vem da mesa de luz, não do Material Design.

Proíbe: cor decorativa; accent em ícone inativo; gradiente; mais de um accent por Theme; marca da firma (verde-lima) como accent do produto.

## 3. Densidade de console

Grade de 8 px. Widgets em tamanhos discretos (1×1 = 48 px, 2×1, 4×4, ...). Nada elástico, nada que muda de lugar sozinho. O operador acha o botão de olhos fechados.

Proíbe: layout responsivo que reorganiza; cantos arredondados por padrão; sombra; animação de layout; alvo de toque menor que 44 px.

## 4. Texto primeiro

Paleta de comandos com busca (o F3 do Blender). Todo comando mostra o nome que a CLI e o MCP usam. A interface ensina o próprio vocabulário: quem aprende a GUI já sabe operar por SSH.

Proíbe: ícone sem rótulo em ação destrutiva; nome na GUI diferente do nome no registry; menu com mais de 8 itens sem busca.

## 5. Themes são looks, Faces são superfícies

O selvagem fica no Theme: forma da janela por SVG, bisel, LCD, scope, cromo. O funcional fica na Face: quais widgets, onde, em que views. Um Theme veste qualquer Face. É isso que faz um Headspace de 2001 e um flat de 2026 operarem a mesma tela.

Proíbe: Theme que muda posição ou tamanho de widget; Face que fixa cor; widget que só existe em um Theme.

## Filtro anti-slop (checklist de revisão)

- Sem gradiente decorativo. Sem sombra Material. Sem blur de vidro.
- Sem cantos arredondados por padrão (raio 0; 2 px em chips, e só).
- Sem ícone genérico de biblioteca na tela final; placeholder é aceito se estiver marcado.
- Sem cor sem significado declarado em `tokens/spellcaster.css`.
- Sem Inter, Roboto, Arial. Voz do produto é mono; rótulos em condensada.
- Sem emoji. Sem "✨". Sem varinha, faísca, roxo místico. "Spell" é comando escrito.
- Sem texto de exemplo genérico em mockup: nomes reais de show, fixture, universo.
- Toda tela nova mostra, em algum canto, o comando equivalente da CLI.
