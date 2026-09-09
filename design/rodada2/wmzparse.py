import zipfile, re, base64
from PIL import Image
import io
z = zipfile.ZipFile("wmz/Revert.wmz")
x = z.read("netgen.wms").decode("utf-16")
# only tags of interest, collapsed
for m in re.finditer(r"<(VIEW|BUTTONGROUP|BUTTONELEMENT|TEXT|slider|SLIDER|SUBVIEW|subview|BUTTON|PLAYLIST|EFFECTS|effects|VIDEO|video)\b[^>]*>", x, re.I):
    s = re.sub(r"\s+", " ", m.group(0))
    print(s[:400])
for n in ["player_up.bmp", "player_map.bmp", "player_hover.bmp"]:
    Image.open(io.BytesIO(z.read(n))).convert("RGB").resize((512, 260), Image.NEAREST).save("wmz/" + n.replace(".bmp", ".png"))
open("wmz/revert.b64", "w").write(base64.b64encode(open("wmz/Revert.wmz", "rb").read()).decode())
print("b64 KiB", len(open("wmz/revert.b64").read()) // 1024)
