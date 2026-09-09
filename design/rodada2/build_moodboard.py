import base64, io, re, os
from PIL import Image

ROOT = os.path.dirname(os.path.abspath(__file__))

def b64(path, w=360, q=78):
    im = Image.open(path).convert("RGB")
    if im.width > w:
        im = im.resize((w, int(im.height * w / im.width)), Image.LANCZOS)
    buf = io.BytesIO(); im.save(buf, "JPEG", quality=q, optimize=True)
    return "data:image/jpeg;base64," + base64.b64encode(buf.getvalue()).decode()

IMG = {}
for n in "0040 0041 0042 0043 0044 0045 0046 0047 0048 0049 0050 0051 0052 0053 0054 0055 0056 0057 0058 0059".split():
    IMG[n] = b64(os.path.join(ROOT, "wmp", n + ".png"))
IMG["batman"] = b64(os.path.join(ROOT, "wmp", "tsf_showcase.png"), 900)
for k, f in {"makhina": "makhina_ui.jpg", "makhina2": "makhina_ui2.jpg", "clear": "clearwater.png", "shaga": "shaga.png", "club": "club.png", "p0": "p0_ui.png", "rumor": "rumor_ui.jpg", "berlin": "berlin.png"}.items():
    IMG[k] = b64(os.path.join(ROOT, "ida", f), 420)

tpl = open(os.path.join(ROOT, "moodboard.tpl.html"), encoding="utf-8").read()
out = re.sub(r"\{\{img:(\w+)\}\}", lambda m: IMG[m.group(1)], tpl)
dst = os.path.join(ROOT, "spellcaster-moodboard.html")
open(dst, "w", encoding="utf-8").write(out)
print(dst, len(out) // 1024, "KiB")
