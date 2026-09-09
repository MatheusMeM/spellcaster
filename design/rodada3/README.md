# Rodada 3 — ILDA player, tema LASER

Primeira rodada com função antes de UI (ver `../TEMAS.md`). Um arquivo só, sem build: `ilda.html`.

- Lê ILDA 0/1/4/5 (`ildaParse`), escreve formato 5 (`ildaWrite`); o demo é gerado em JS como bytes ILDA e passa pelo parser.
- Física: taxa de frame = kpps ÷ pontos (pisca abaixo de 25 fps); galvo como passa-baixa acima de ~32 kpps; feixes desde o projetor com névoa.
- Aprendiz: sprite 14×18 em `SPR`, balão Win98, menu das cinco funções, avisos com contexto, memória em `localStorage` (`sc-ap-last`).
- Voto em `moodboard/round3` (db do artefato) + `localStorage` (`sc-moodboard-r3`). `#laser` no hash pula o splash.
