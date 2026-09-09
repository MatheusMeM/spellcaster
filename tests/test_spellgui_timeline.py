# Teste da base da R5 (spellgui/web): canvaskit.js + timeline.js num Chrome headless.
# A pagina de teste carrega os dois scripts, injeta o shows/medgrupo.spell inline, roda as asseracoes
# em JS e escreve o resultado num <pre id=RESULT>. Aqui so lemos o DOM e falhamos se houver FAIL,
# se faltar o DONE final ou se window.onerror tiver disparado. Saida so em ASCII.
# ponytail: --dump-dom em vez de webdriver ; trocar por CDP se um dia precisarmos de eventos reais.
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WEB = os.path.join(ROOT, "spellgui", "web")
SHOW = os.path.join(ROOT, "shows", "medgrupo.spell")

PAGE = """<!doctype html>
<meta charset="utf-8">
<style>html,body{margin:0;background:#000;color:#ccc}#cv{width:1200px;height:500px;display:block}</style>
<canvas id="cv" tabindex="0"></canvas>
<pre id="RESULT">SEM RESULTADO</pre>
<script src="canvaskit.js"></script>
<script src="timeline.js"></script>
<script>
"use strict";
var LOG = [], SPELL = __SPELL__;
function dump() { document.getElementById("RESULT").textContent = LOG.join("\\n"); }
function ok(name, cond, detail) { LOG.push((cond ? "OK   " : "FAIL ") + name + (cond ? "" : " : " + detail)); }
window.onerror = function (m, s, l) { LOG.push("FAIL onerror : " + m + " linha " + l); dump(); };
try {
  var k = TL.mount(document.getElementById("cv"), {});
  var n = TL.load(SPELL);
  ok("lanes do medgrupo.spell", n === 5, "lanes=" + n);
  ok("gutter = cabecalho", k.gutter === TL.headW, "gutter=" + k.gutter);

  var rt = true, worst = 0;
  [0, 1.5, 42.75, 85.9].forEach(function (t) {
    [4, 40, 400, 3000].forEach(function (z) {
      k.view.zoom = z; k.view.x = 3.25;
      var d = Math.abs(k.toWorld(k.toScreen(t)) - t);
      if (d > worst) worst = d;
      if (d > 1e-9) rt = false;
    });
  });
  ok("world<->screen ida e volta", rt, "erro=" + worst);

  k.view.zoom = 40; k.view.x = 0;
  var px = 600, tb = k.toWorld(px);
  k.zoomAt(px, 2.5);
  ok("zoom ancorado no cursor", Math.abs(k.toWorld(px) - tb) < 1e-9, "t=" + tb + " -> " + k.toWorld(px));

  var ts = new Float64Array([0, 1, 2, 5, 9]);
  ok("bisect", CK.bisect(ts, 5, 2) === 2 && CK.bisect(ts, 5, 2.5) === 3 &&
               CK.bisect(ts, 5, -1) === 0 && CK.bisect(ts, 5, 99) === 5, "bisect errado");
  ok("near", CK.near(ts, 5, 5.2, 0.5) === 3 && CK.near(ts, 5, 7, 0.5) === -1, "near errado");

  k.sel.clear(); k.sel.add(0, 1); k.sel.add(0, 2); k.sel.toggle(0, 2);
  ok("selecao", k.sel.count() === 1 && k.sel.has(0, 1) && !k.sel.has(0, 2), "count=" + k.sel.count());

  TL.fit();
  var li = -1;
  for (var i = 0; i < TL.lanes.length; i++) if (TL.lanes[i].param === "scale") li = i;
  var L = TL.lanes[li], sc = (TL.rowH - 8) / ((L.vmax - L.vmin) || 1);
  var x = k.toScreen(L.ts[1]);
  var y = TL.rulerH + li * TL.rowH - k.view.y + TL.rowH - 4 - (L.vs[1] - L.vmin) * sc;
  ok("lane de parametro laser", li === 3 && L.n === 2, "li=" + li + " n=" + L.n);
  ok("laneAt", TL.laneAt(y) === li, "laneAt=" + TL.laneAt(y));
  ok("keyAt no keyframe", TL.keyAt(li, x, y) === 1, "keyAt=" + TL.keyAt(li, x, y));
  ok("keyAt no vazio", TL.keyAt(li, x - 60, y) === -1, "keyAt=" + TL.keyAt(li, x - 60, y));

  TL.show.markers = [10, 20];
  TL.buildSnaps();
  var tol = 8 / k.view.zoom;
  ok("snap no marcador", TL.snapT(10 + tol * 0.5) === 10, "t=" + TL.snapT(10 + tol * 0.5));
  ok("sem snap longe do marcador", TL.snapT(10 + tol * 5) !== 10, "grudou");
  TL.snap = false;
  ok("snap desligado", TL.snapT(10 + tol * 0.5) !== 10, "grudou com snap off");
  TL.snap = true;

  k.redraw();
  ok("lanes desenhadas", TL.drawn === 5, "drawn=" + TL.drawn);
  k.view.y = 2 * TL.rowH;
  k.redraw();
  ok("lanes desenhadas com rolagem", TL.drawn === 3, "drawn=" + TL.drawn);
  k.view.y = 0;

  var m = TL.load({ name: "curvas", fps: 30, duration: 5, markers: [],
                    tracks: [{ type: "dmx", universe: 1, address: 1,
                               keys: [[0, 0], [1, 10], [3, 110, "inout"], [5, 210, "hold"]] }] });
  var C = TL.lanes[0];
  ok("show de curvas", m === 1, "lanes=" + m);
  ok("valor antes do primeiro key", TL.valueAt(C, -1) === 0, "v=" + TL.valueAt(C, -1));
  ok("curva linear em t=0.5", Math.abs(TL.valueAt(C, 0.5) - 5) < 1e-9, "v=" + TL.valueAt(C, 0.5));
  ok("curva inout em t=2", Math.abs(TL.valueAt(C, 2) - 60) < 1e-9, "v=" + TL.valueAt(C, 2));
  ok("curva hold em t=4", TL.valueAt(C, 4) === 110, "v=" + TL.valueAt(C, 4));
  ok("valor depois do ultimo key", TL.valueAt(C, 9) === 210, "v=" + TL.valueAt(C, 9));
  LOG.push("DONE");
} catch (e) {
  LOG.push("FAIL excecao : " + e.message);
}
dump();
</script>
"""


def find_chrome():
    """Caminho do chrome.exe, ou None. Ordem: env, Program Files, LOCALAPPDATA, registro, PATH."""
    env = os.environ.get("SPELL_CHROME")
    if env and os.path.exists(env):
        return env
    cands = [
        os.path.join(os.environ.get("PROGRAMFILES", r"C:\Program Files"),
                     "Google", "Chrome", "Application", "chrome.exe"),
        os.path.join(os.environ.get("PROGRAMFILES(X86)", r"C:\Program Files (x86)"),
                     "Google", "Chrome", "Application", "chrome.exe"),
        os.path.join(os.environ.get("LOCALAPPDATA", ""), "Google", "Chrome", "Application", "chrome.exe"),
    ]
    for p in cands:
        if p and os.path.exists(p):
            return p
    if sys.platform == "win32":
        try:
            import winreg
            for root in (winreg.HKEY_LOCAL_MACHINE, winreg.HKEY_CURRENT_USER):
                try:
                    with winreg.OpenKey(root, r"SOFTWARE\Microsoft\Windows\CurrentVersion"
                                              r"\App Paths\chrome.exe") as kk:
                        p = winreg.QueryValue(kk, None)
                        if p and os.path.exists(p):
                            return p
                except OSError:
                    pass
        except ImportError:
            pass
    for name in ("chrome", "google-chrome", "chromium"):
        p = shutil.which(name)
        if p:
            return p
    return None


class TestSpellguiTimeline(unittest.TestCase):
    def test_canvaskit_e_timeline_no_chrome(self):
        chrome = find_chrome()
        if not chrome:
            self.skipTest("chrome.exe nao encontrado (defina SPELL_CHROME)")
        with open(SHOW, encoding="utf-8") as f:
            spell = json.load(f)
        tmp = tempfile.mkdtemp(prefix="spellgui_")
        try:
            for name in ("canvaskit.js", "timeline.js"):
                shutil.copy(os.path.join(WEB, name), os.path.join(tmp, name))
            page = os.path.join(tmp, "page.html")
            with open(page, "w", encoding="utf-8", newline="\n") as f:
                f.write(PAGE.replace("__SPELL__", json.dumps(spell)))
            cmd = [chrome, "--headless=new", "--disable-gpu", "--no-first-run", "--no-sandbox",
                   "--window-size=1400,900", "--virtual-time-budget=5000",
                   "--user-data-dir=" + os.path.join(tmp, "profile"),
                   "--dump-dom", "file:///" + page.replace("\\", "/")]
            p = subprocess.run(cmd, capture_output=True, timeout=120)
            dom = p.stdout.decode("utf-8", "replace")
            self.assertIn("RESULT", dom, "chrome nao devolveu o DOM: " + p.stderr.decode("utf-8", "replace")[-500:])
            i = dom.find('<pre id="RESULT">')
            self.assertGreater(i, 0, "sem <pre id=RESULT> no DOM")
            out = dom[i + len('<pre id="RESULT">'):dom.find("</pre>", i)]
            print(out)
            self.assertNotIn("FAIL", out, "asseracoes JS falharam")
            self.assertIn("DONE", out, "a pagina nao chegou ao fim")
            self.assertGreaterEqual(out.count("OK"), 20, "poucas asseracoes rodaram")
        finally:
            shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    unittest.main()
