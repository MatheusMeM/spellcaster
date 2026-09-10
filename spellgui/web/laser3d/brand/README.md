# brand/ — the Feitiçaria Industrial brand

Official vectors from `4-Marketing/ID VISUAL 2026/Logos/SVG (vetor)/`, with the fill swapped from
`#333` to `#F2F1EA` (the brand is light over the black chassis and over the dark HUD).

| file | what it is | where to use it |
|---|---|---|
| `symbol.svg` | the symbol (figure in orbit) on its own, cut out of `stacked.svg` (it does not exist as an official file) | icon, favicon, the button that brings the Pino back, corner of the display |
| `submark.svg` | FEITIÇARIA + symbol, without INDUSTRIAL (horizontal 1009×305) | lid, label, wherever the whole name does not fit |
| `wordmark.svg` | FEITIÇARIA INDUSTRIAL as text | silkscreen of the rear panel, HUD footer |
| `main.svg` | FEITIÇARIA + symbol (horizontal) | HUD, splash |
| `stacked.svg` | symbol on top of the name | square screens, bench |

To draw onto a canvas texture: `img = new Image(); img.src = "brand/wordmark.svg"; img.onload → ctx.drawImage(img, x, y, w, h)` and `texture.needsUpdate = true` after the load. The silkscreen of the rear panel (`body.js`) does exactly that.
