# brand/ — a marca Feitiçaria Industrial

Vetores oficiais de `4-Marketing/ID VISUAL 2026/Logos/SVG (vetor)/`, com o preenchimento trocado de
`#333` para `#F2F1EA` (a marca é clara sobre o chassi preto e sobre o HUD escuro).

| arquivo | o que é | onde usar |
|---|---|---|
| `symbol.svg` | o símbolo (boneco em órbita) sozinho, recortado do `stacked.svg` (não existe como arquivo oficial) | ícone, favicon, botão de voltar o Pino, canto do display |
| `submark.svg` | FEITIÇARIA + símbolo, sem INDUSTRIAL (horizontal 1009×305) | tampa, etiqueta, onde o nome inteiro não cabe |
| `wordmark.svg` | FEITIÇARIA INDUSTRIAL em texto | serigrafia da traseira, rodapé do HUD |
| `main.svg` | FEITIÇARIA + símbolo (horizontal) | HUD, splash |
| `stacked.svg` | símbolo em cima do nome | telas quadradas, bench |

Para desenhar em textura de canvas: `img = new Image(); img.src = "brand/wordmark.svg"; img.onload → ctx.drawImage(img, x, y, w, h)` e `texture.needsUpdate = true` depois do load. A serigrafia da traseira (`body.js`) faz isso.
