# Rodada 2 — protótipo cracktro

Publicado em https://claude.ai/code/artifact/106e46a9-8070-4edd-9351-83ac4e7a5e2e (voto salvo em `moodboard/round2`).

- `cracktro.tpl.html` — fonte do protótipo: vidro sobre desktop falso, splash com jingle Web Audio, gels Lee/Rosco, fita, NFO, leitor `.wmz`, voto.
- `build_cracktro.py` — embute o `Revert.wmz` (skin da Microsoft, lido de `C:\Windows\WinSxS\...\Revert.wmz` ou da variável `REVERT_WMZ`) e gera `spellcaster-cracktro.html` ao lado do template. Gerar fora do repo.
- `wmzparse.py` — dissecção do formato `.wmz`: zip + `.wms` UTF-16 + BMPs; `VIEW/clippingColor`, `BUTTONGROUP/mappingImage` por cor.
- `moodboard.tpl.html` + `build_moodboard.py` — rodada 1 (reprovada); precisa das capturas `wmp/` e `ida/` que ficaram no scratchpad.

Rodar: `C:\Python313\python.exe build_cracktro.py` dentro desta pasta.
