import os
ROOT = os.path.dirname(os.path.abspath(__file__))
src = open(os.path.join(ROOT, "cracktro.tpl.html"), encoding="utf-8").read()
import base64, glob
WMZ = os.environ.get("REVERT_WMZ") or glob.glob(r"C:\Windows\WinSxS\amd64_microsoft-windows-mediaplayer-skins_*\Revert.wmz")[0]
b64 = base64.b64encode(open(WMZ, "rb").read()).decode()  # ponytail: skin da Microsoft lida do Windows; nao entra no repo


def rep(old, new, count=1):
    global src
    assert src.count(old) == count, (src.count(old), old[:60])
    src = src.replace(old, new)


# CSS
rep("  #winb.drop { animation:drop .55s cubic-bezier(.2,1.4,.4,1); }",
"""  #winb.drop { animation:drop .55s cubic-bezier(.2,1.4,.4,1); }
  #skin { position:absolute; left:160px; top:120px; z-index:6; display:none; filter:drop-shadow(0 18px 30px rgba(0,0,0,.6)); }
  #skin.on { display:block; }
  #skinc { display:block; image-rendering:pixelated; cursor:grab; touch-action:none; }
  #skinc.hot { cursor:pointer; }
  .sb { display:flex; gap:12px; align-items:center; margin-top:6px; font-family:var(--mono); font-size:11px; color:var(--fg2); background:rgba(0,0,0,.7); padding:5px 10px; border-radius:6px; width:max-content; }
  .sb b { color:var(--amb); font-weight:400; }
  .sb label, .sb span.k { cursor:pointer; color:var(--fg); text-decoration:underline dotted; }
  #stage.dz::after { content:"solta o .wmz"; position:absolute; inset:0; display:grid; place-items:center; font-family:var(--px); font-size:18px; color:var(--amb); background:rgba(0,0,0,.6); z-index:9; pointer-events:none; }""")

# markup
rep('<span class="tg" id="nfobtn">NFO</span>', '<span class="tg" id="nfobtn">NFO</span>\n        <span class="tg" id="wmzbtn" title="skin do Windows Media Player">WMZ</span>')
rep('    <div id="strip"><span class="tc" id="tc2">',
"""    <div id="skin"><canvas id="skinc" width="512" height="260"></canvas>
      <div class="sb"><b id="skname">Revert.wmz</b><label>abrir .wmz<input type="file" id="wmzfile" accept=".wmz,.zip" hidden></label><span class="k" id="skoff">voltar ao vidro</span><span>ou arraste um .wmz para a cena</span></div></div>
    <div id="strip"><span class="tc" id="tc2">""")
rep('<span><b>N</b> NFO</span>', '<span><b>N</b> NFO</span><span><b>W</b> skin .wmz</span>')

# doc + vote
rep('        <li><b>Performance.</b>',
"""        <li><b>Skins do WMP de verdade.</b> O rodapé tem <b>WMZ</b>: abre o Revert.wmz da Microsoft (o zip real, embutido) e qualquer .wmz que você arrastar para a cena. O leitor entende <code>VIEW</code> com <code>clippingColor</code> (silhueta), <code>BUTTONGROUP</code> com mapa de clique por cor, hover e down repintados, <code>TEXT</code> e a área de <code>EFFECTS</code>; sliders e JScript ficam de fora. Play e Next viram GO, Prev volta um cue, fechar devolve o vidro, e o gel tinge a skin. No produto é o mesmo leitor dentro do WebView; .wmz é só zip + XML + BMP.</li>
        <li><b>Performance.</b>""")
rep('<div class="q"><span class="eyebrow">fita sobre o Resolume</span>',
"""<div class="q"><span class="eyebrow">skins do WMP</span><div class="opts" data-q="wmz"><button data-v="wmz">LER .WMZ DE VERDADE</button><button data-v="formato">SÓ O FORMATO, BITMAPS NOSSOS</button><button data-v="nao">NÃO PRECISA</button></div></div>
        <div class="q"><span class="eyebrow">fita sobre o Resolume</span>""")
rep('fita: null, notas: ""', 'fita: null, wmz: null, notas: ""')

# scripts
rep('<script>\n(function () {', '<script src="https://cdnjs.cloudflare.com/ajax/libs/jszip/3.10.1/jszip.min.js"></script>\n<script>\n(function () {')
rep('$("#gelname").textContent = g[1];', 'gelRGB = g[2]; gelA = g[3]; if (SK.skin) drawSkin(); $("#gelname").textContent = g[1];')
rep('    requestAnimationFrame(tick); }\n  requestAnimationFrame(tick);', '    if (SK.on) drawSkin();\n    requestAnimationFrame(tick); }\n  requestAnimationFrame(tick);')
rep('else if (e.key.toLowerCase() === "n") { $("#nfo").classList.toggle("on"); }', 'else if (e.key.toLowerCase() === "n") { $("#nfo").classList.toggle("on"); } else if (e.key.toLowerCase() === "w") { toggleSkin(); }')

LOADER = r"""
  /* ---------- skin .wmz de verdade ---------- */
  var SK = { on: false, skin: null, hot: null, down: null }, gelRGB = "27,15,143", gelA = 0.34;
  var sc = $("#skinc"), sx = sc.getContext("2d");
  var REVERT = "%%B64%%";
  function lower(n) { return (n || "").toLowerCase(); }
  function base(n) { return lower(n).replace(/^.*[\\\/]/, ""); }
  function zipFile(zip, name) { if (!name) return null; var b = base(name), k = Object.keys(zip.files).filter(function (f) { return base(f) === b; })[0]; return k ? zip.files[k] : null; }
  function loadImg(zip, name) { var f = zipFile(zip, name); if (!f) return Promise.resolve(null); var ext = base(name).split(".").pop(); return f.async("uint8array").then(function (u8) { return new Promise(function (res) { var im = new Image(), url = URL.createObjectURL(new Blob([u8], { type: "image/" + (ext === "jpg" ? "jpeg" : ext) })); im.onload = function () { URL.revokeObjectURL(url); res(im); }; im.onerror = function () { URL.revokeObjectURL(url); res(null); }; im.src = url; }); }); }
  function pix(im) { var c = document.createElement("canvas"); c.width = im.width; c.height = im.height; var x = c.getContext("2d"); x.drawImage(im, 0, 0); return x.getImageData(0, 0, im.width, im.height); }
  function hex(h) { h = (h || "").replace("#", ""); return h.length === 6 ? [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)] : null; }
  function toCanvas(d) { var c = document.createElement("canvas"); c.width = d.width; c.height = d.height; c.getContext("2d").putImageData(d, 0, 0); return c; }
  function keyed(im, clip) { var d = pix(im), p = d.data, k = hex(clip); if (k) for (var i = 0; i < p.length; i += 4) if (p[i] === k[0] && p[i + 1] === k[1] && p[i + 2] === k[2]) p[i + 3] = 0; return toCanvas(d); }
  function masked(im, map, col) { if (!im || !map) return null; var d = pix(im), p = d.data, m = pix(map).data, k = hex(col); if (!k) return null; for (var i = 0; i < p.length; i += 4) if (!(m[i] === k[0] && m[i + 1] === k[1] && m[i + 2] === k[2])) p[i + 3] = 0; return toCanvas(d); }
  function num(el, a, d) { var v = el.getAttribute(a); return v == null || v === "" || isNaN(+v) ? d : +v; }
  function parseSkin(zip, label) {
    var xmlf = Object.keys(zip.files).filter(function (f) { return /\.(wms|xml)$/i.test(f); })[0];
    if (!xmlf) throw new Error("sem skin.xml ou .wms dentro do zip");
    return zip.files[xmlf].async("uint8array").then(function (u8) {
      var utf16 = (u8[0] === 0xFF && u8[1] === 0xFE) || (u8[0] === 0xFE && u8[1] === 0xFF);
      var txt = new TextDecoder(utf16 ? "utf-16" : "utf-8").decode(u8);
      var doc = new DOMParser().parseFromString(txt, "text/html"); /* ponytail: parser HTML tolera o XML sujo do WMP; tags e atributos viram minúsculas */
      var view = doc.querySelector("view"); if (!view) throw new Error("sem <VIEW> no skin");
      var clip = view.getAttribute("clippingcolor");
      var S = { name: label, groups: [], texts: [], fx: null, W: num(view, "width", 0), H: num(view, "height", 0), bg: null }, jobs = [];
      function off(el) { var x = 0, y = 0, p = el.parentElement; while (p && p !== view) { if (p.tagName.toLowerCase() === "subview") { if (lower(p.getAttribute("visible")) === "false") return null; x += num(p, "left", 0); y += num(p, "top", 0); } p = p.parentElement; } return [x, y]; }
      jobs.push(loadImg(zip, view.getAttribute("backgroundimage")).then(function (im) { if (im) { S.bg = keyed(im, clip); S.W = S.W || im.width; S.H = S.H || im.height; } }));
      Array.prototype.forEach.call(view.querySelectorAll("buttongroup"), function (bg) {
        var o = off(bg); if (!o) return;
        var G = { x: o[0] + num(bg, "left", 0), y: o[1] + num(bg, "top", 0), els: [], map: null, img: null };
        S.groups.push(G);
        jobs.push(Promise.all([loadImg(zip, bg.getAttribute("mappingimage")), loadImg(zip, bg.getAttribute("hoverimage")), loadImg(zip, bg.getAttribute("downimage")), loadImg(zip, bg.getAttribute("image"))]).then(function (r) {
          if (!r[0]) return; G.map = pix(r[0]); G.w = r[0].width; G.h = r[0].height; if (r[3]) G.img = keyed(r[3], clip);
          Array.prototype.forEach.call(bg.children, function (el) { var tag = el.tagName.toLowerCase(); if (!/element$/.test(tag)) return; var col = el.getAttribute("mappingcolor"); if (!col) return;
            var oc = lower(el.getAttribute("onclick")), role = tag.replace("element", "");
            if (role === "button") role = /close/.test(oc) ? "close" : /minimize/.test(oc) ? "mini" : /mediacenter/.test(oc) ? "back" : /mute/.test(oc) ? "mute" : /menu/.test(oc) ? "menu" : /eq/.test(oc) ? "eq" : /playlist|togglepl/.test(oc) ? "pl" : "other";
            G.els.push({ col: hex(col), role: role, hover: masked(r[1], r[0], col), down: masked(r[2], r[0], col) }); });
        }));
      });
      Array.prototype.forEach.call(view.querySelectorAll("text"), function (t) { var o = off(t); if (!o) return; S.texts.push({ x: o[0] + num(t, "left", 0), y: o[1] + num(t, "top", 0), w: num(t, "width", 100), h: num(t, "height", 12), size: num(t, "fontsize", 8), face: t.getAttribute("fontface") || "Tahoma", col: t.getAttribute("foregroundcolor") || "#000000", just: lower(t.getAttribute("justification") || "left"), val: lower(t.getAttribute("value") || "") }); });
      var fx = view.querySelector("effects"); if (fx) { var fo = off(fx) || [0, 0]; S.fx = { x: fo[0] + num(fx, "left", 0), y: fo[1] + num(fx, "top", 0), w: num(fx, "width", 60), h: num(fx, "height", 30) }; }
      return Promise.all(jobs).then(function () { if (!S.W || !S.H) throw new Error("skin sem tamanho"); return S; });
    });
  }
  function drawSkin() {
    var S = SK.skin; if (!S) return; if (sc.width !== S.W * 2) { sc.width = S.W * 2; sc.height = S.H * 2; }
    sx.setTransform(2, 0, 0, 2, 0, 0); sx.imageSmoothingEnabled = false; sx.clearRect(0, 0, S.W, S.H);
    if (S.bg) sx.drawImage(S.bg, 0, 0);
    S.groups.forEach(function (G) { if (G.img) sx.drawImage(G.img, G.x, G.y); G.els.forEach(function (E) { var st = SK.down === E ? E.down : SK.hot === E ? E.hover : null; if (st) sx.drawImage(st, G.x, G.y); }); });
    if (S.fx) { sx.fillStyle = "#000"; sx.fillRect(S.fx.x, S.fx.y, S.fx.w, S.fx.h); var n = 12, bw = S.fx.w / n; for (var i = 0; i < n; i++) { var v = Math.min(1, vuv[i]); sx.fillStyle = v > 0.8 ? "#FF2D1F" : v > 0.55 ? "#FFB000" : "#A8E05E"; sx.fillRect(S.fx.x + i * bw + 1, S.fx.y + S.fx.h - v * S.fx.h, bw - 2, v * S.fx.h); } }
    var vals = [cues[cueI][0] + " · " + cues[cueI][1], "MED GRUPO RJ · PLENÁRIA", $("#tc").textContent];
    S.texts.forEach(function (T, i) { var v = /position|time|duration/.test(T.val) ? vals[2] : /name|title/.test(T.val) ? vals[0] : vals[i % 3]; sx.font = T.size + "px " + T.face + ", Tahoma, sans-serif"; sx.fillStyle = T.col; sx.textBaseline = "top"; sx.textAlign = T.just === "center" ? "center" : T.just === "right" ? "right" : "left"; var x = T.just === "center" ? T.x + T.w / 2 : T.just === "right" ? T.x + T.w : T.x; sx.save(); sx.beginPath(); sx.rect(T.x, T.y, T.w, T.h); sx.clip(); sx.fillText(v, x, T.y + 1); sx.restore(); });
    sx.globalCompositeOperation = "source-atop"; sx.fillStyle = "rgba(" + gelRGB + "," + gelA + ")"; sx.fillRect(0, 0, S.W, S.H); sx.globalCompositeOperation = "source-over";
  }
  function hit(e) { var S = SK.skin; if (!S) return null; var r = sc.getBoundingClientRect(), px = (e.clientX - r.left) / 2, py = (e.clientY - r.top) / 2; for (var g = S.groups.length - 1; g >= 0; g--) { var G = S.groups[g]; if (!G.map) continue; var x = px - G.x, y = py - G.y; if (x < 0 || y < 0 || x >= G.w || y >= G.h) continue; var i = ((y | 0) * G.w + (x | 0)) * 4, d = G.map.data; for (var k = 0; k < G.els.length; k++) { var c = G.els[k].col; if (c && d[i] === c[0] && d[i + 1] === c[1] && d[i + 2] === c[2]) return G.els[k]; } } return null; }
  function act(role) { blip(role !== "close"); ({ play: doGo, next: doGo, prev: function () { if (cueI > 0) { cueI--; renderCues(); toast("Voltou um cue. Ninguém viu."); } }, stop: function () { toast("Stop no meio do show? Coragem. A última cena fica no ar."); }, pause: function () { toast("Pausa. O DMX segura a última cena."); }, close: function () { setSkin(false); toast("Voltou pro vidro."); }, back: function () { setSkin(false); toast("Voltou pro vidro."); }, mini: function () { setSkin(false); toggleStrip(); }, mute: function () { if (playing) stopJingle(); toast("Mudo. O jingle respeita."); }, menu: function () { toast("Menu: no produto isso abre o patch."); }, eq: function () { toast("EQ vira o mixer de universos."); }, pl: function () { toast("Playlist vira a lista de cues."); }, other: function () { toast("Esse botão vai virar alguma coisa. Sugira."); } }[role] || function () {})(); }
  var skDrag = null;
  sc.addEventListener("pointermove", function (e) { if (skDrag) { var b = $("#skin"); b.style.left = (e.clientX - skDrag[0]) + "px"; b.style.top = (e.clientY - skDrag[1]) + "px"; return; } var h = hit(e); if (h !== SK.hot) { SK.hot = h; sc.classList.toggle("hot", !!h); drawSkin(); } });
  sc.addEventListener("pointerdown", function (e) { var h = hit(e); if (h) { SK.down = h; drawSkin(); } else { var b = $("#skin"); skDrag = [e.clientX - b.offsetLeft, e.clientY - b.offsetTop]; } sc.setPointerCapture(e.pointerId); });
  sc.addEventListener("pointerup", function (e) { skDrag = null; var h = hit(e); if (SK.down && h === SK.down) act(h.role); SK.down = null; drawSkin(); });
  sc.addEventListener("pointerleave", function () { SK.hot = null; drawSkin(); });
  function setSkin(on) { SK.on = on; $("#skin").classList.toggle("on", on); $("#wmzbtn").classList.toggle("on", on); if (on) { $("#winb").classList.add("hide"); $("#strip").classList.remove("on"); drawSkin(); } else if (!$("#strip").classList.contains("on")) $("#winb").classList.remove("hide"); }
  function useZip(p, label) { toast("lendo " + label + "…"); return p.then(function (zip) { return parseSkin(zip, label); }).then(function (S) { SK.skin = S; $("#skname").textContent = label + " · " + S.W + "×" + S.H + " · " + S.groups.reduce(function (n, g) { return n + g.els.length; }, 0) + " botões"; if (!shown) start(); setSkin(true); toast(label + ": silhueta, mapa de clique e hover lidos do zip. Play = GO."); }).catch(function (e) { toast("não deu: " + (e && e.message || e)); }); }
  function toggleSkin() { if (SK.skin) setSkin(!SK.on); else useZip(JSZip.loadAsync(REVERT, { base64: true }), "Revert.wmz"); }
  $("#wmzbtn").addEventListener("click", toggleSkin); $("#skoff").addEventListener("click", function () { setSkin(false); });
  function fromFile(f) { if (!f) return; useZip(JSZip.loadAsync(f), f.name); }
  $("#wmzfile").addEventListener("change", function (e) { fromFile(e.target.files[0]); e.target.value = ""; });
  var stage = $("#stage");
  stage.addEventListener("dragover", function (e) { e.preventDefault(); stage.classList.add("dz"); }); stage.addEventListener("dragleave", function () { stage.classList.remove("dz"); });
  stage.addEventListener("drop", function (e) { e.preventDefault(); stage.classList.remove("dz"); fromFile(e.dataTransfer.files[0]); });
"""
rep("\n  /* ---------- voto ---------- */", LOADER.replace("%%B64%%", b64) + "\n  /* ---------- voto ---------- */")

dst = os.path.join(ROOT, "spellcaster-cracktro.html")
open(dst, "w", encoding="utf-8").write(src)
print(dst, len(src) // 1024, "KiB")
