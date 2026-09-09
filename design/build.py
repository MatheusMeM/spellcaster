r"""Inlina os <script src="local.js"> e <link href="local.css"> de uma pagina e grava o HTML pronto
para publicar (artifact = um arquivo so). Scripts de CDN (http...) ficam como estao.
Uso: C:\Python313\python.exe design\build.py design\laser\app.html <saida.html>
ponytail: regex sobre duas tags; vira bundler de verdade quando houver import/export."""
import re, sys, pathlib
src = pathlib.Path(sys.argv[1]); out = pathlib.Path(sys.argv[2])
html = src.read_text(encoding="utf-8")
def js(m):
    p = m.group(1)
    if p.startswith("http"): return m.group(0)
    return "<script>\n" + (src.parent / p).read_text(encoding="utf-8") + "\n</script>"
def css(m):
    p = m.group(1)
    if p.startswith("http"): return m.group(0)
    return "<style>\n" + (src.parent / p).read_text(encoding="utf-8") + "\n</style>"
html = re.sub(r'<script src="([^"]+)"></script>', js, html)
html = re.sub(r'<link rel="stylesheet" href="([^"]+)">', css, html)
out.write_text(html, encoding="utf-8")
print("ok", out, out.stat().st_size)
