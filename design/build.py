r"""Inlines the <script src="local.js"> and <link href="local.css"> of a page and writes the HTML ready
to publish (artifact = a single file). CDN scripts (http...) are left as they are.
Usage: C:\Python313\python.exe design\build.py design\laser\app.html <output.html>
ponytail: regex over two tags; becomes a real bundler once there is import/export."""
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
