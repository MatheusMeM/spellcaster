# Rodada 2 — protótipo cracktro

Publicado em https://claude.ai/code/artifact/106e46a9-8070-4edd-9351-83ac4e7a5e2e (voto salvo em `moodboard/round2`).

- `cracktro.tpl.html` — o protótipo, standalone: cubo de gelo raymarched (tema GELO de `../TEMAS.md`) sobre desktop falso, splash com jingle Web Audio, gels Lee/Rosco, fita, NFO, voto. Publicar este arquivo direto.
- `build_cracktro.py` + `wmzparse.py` — leitor de `.wmz` (skins do WMP) que era injetado no protótipo. **Desligado** depois do voto (`wmz: não`); as âncoras podem não bater mais com o template. Fica como referência do formato.
- `moodboard.tpl.html` + `build_moodboard.py` — rodada 1 (reprovada); precisa das capturas `wmp/` e `ida/` que ficaram no scratchpad.
