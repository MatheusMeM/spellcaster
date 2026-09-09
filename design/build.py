"""Inlina design/pino.js numa rodada e grava o HTML pronto para publicar (artifact = um arquivo so).
Uso: C:\\Python313\\python.exe design\\build.py design\\rodada4\\projetor.html <saida.html>
ponytail: replace de uma tag; vira template de verdade se houver mais de um include."""
import sys, pathlib
src = pathlib.Path(sys.argv[1]); out = pathlib.Path(sys.argv[2])
pino = (src.parent.parent / "pino.js").read_text(encoding="utf-8")
html = src.read_text(encoding="utf-8")
tag = '<script src="../pino.js"></script>'
assert tag in html, "tag do pino nao encontrada"
out.write_text(html.replace(tag, "<script>\n" + pino + "\n</script>"), encoding="utf-8")
print("ok", out, out.stat().st_size)
