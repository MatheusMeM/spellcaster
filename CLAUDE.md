# Spellcaster — regras do repositório

Show-control portátil: timeline + sACN / Art-Net / OSC / ILDA, GUI web com skins, MCP embutido, player standalone, Lite para Raspberry Pi (CLI por SSH). Plano em `ROADMAP.md`; arquitetura em `ARCHITECTURE.md`.

## Modo de trabalho: ponytail (sempre)

- Menor código que funciona. Stdlib antes de dependência; dependência já instalada antes de nova; uma linha antes de cinquenta.
- Sem abstração com uma implementação só, sem factory, sem config para valor que não muda, sem scaffolding "para depois".
- Simplificação deliberada leva comentário `# ponytail: <limite> ; <quando trocar>`.
- Toda lógica não trivial deixa UM teste `unittest` em `tests/`. Sem pytest, sem fixtures, sem mocks pesados: loopback UDP em 127.0.0.1 é o mock.
- Correção de bug = causa raiz no ponto que todos os chamadores usam.

## Contratos fixos

- Pacote `spellcaster`, CLI `spell` (`spellcaster.cli:main`). Arquivo de show `.spell` (JSON).
- Universos numerados a partir de 1 (sACN). Art-Net converte para port-address internamente.
- Toda saída de protocolo expõe `send(universe: int, data: bytes)` e `close()`.
- Todo comando do produto passa pelo `spellcaster.core.registry` (`@command`). CLI, OSC-API, GUI e MCP são clientes do registry; nunca implementam lógica própria.
- O engine não conhece GUI. Nada em `spellcaster/core`, `protocols`, `fixtures`, `timeline` importa de `gui` ou `mcp`.
- Python 3.13, sem venv. Rodar com `C:\Python313\python.exe` no Windows, `python3` no Pi.

## Ambiente (Windows, pasta no Google Drive)

- Python só em arquivos `.py`; nunca `python -c` com aspas aninhadas.
- Arquivos UTF-8 sem BOM. Console é cp1252: testes não imprimem caracteres fora de ASCII.
- Não gravar binário pesado (exe, mp4, imagem de SD) direto nesta pasta; gerar em `%TEMP%` e mover.
- Testes: `C:\Python313\python.exe -m unittest discover -s tests -v`.

## Git

- Branch `main`, remote `origin` = github.com/MatheusMeM/spellcaster. Commits em português, verbo no infinitivo ou substantivo, uma linha de assunto.
- Não commitar `build/`, `dist/`, `*.log`, `__pycache__/`.

- Commits e pushes só na conta do Matheus. Proibido `Co-Authored-By`, "Generated with Claude" ou qualquer crédito ao Claude no histórico git.
