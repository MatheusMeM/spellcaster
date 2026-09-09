/* Pino: o companion do Spellcaster. Um cabo DMX com plugue XLR-5 na cabeça; os cinco pinos são o menu
   principal (1 ILDA, 2 NDI→ILDA, 3 Orquestrador, 4 Cenas e cues, 5 Info) e a trava é "some". Vetor, não
   pixel: contorno escuro, chapa flat, olhos de Clippy que seguem o mouse. É o mesmo em todas as skins.
   Uso: var pino = Pino.mount(stageEl, { on: function (key) {}, current: "ilda" }); pino.say("texto", [["k","rótulo","sub"]]);
   ponytail: copiado inline em cada rodadaN/*.html porque o artifact é um arquivo só; este é o canônico. */
(function () {
  "use strict";
  var PINS = [["ilda", "ILDA player", "LASER"], ["ndi", "NDI → ILDA", "FÓSFORO"], ["orq", "Orquestrador", "PATCHBAY"], ["cues", "Cenas e cues", "TEATRO DE PAPEL"], ["nfo", "Info", "N"]];
  var CSS = "#pino{position:absolute;right:18px;bottom:64px;width:144px;height:210px;z-index:5;user-select:none}" +
    "#pino svg{width:144px;height:210px;display:block;overflow:visible;filter:drop-shadow(0 8px 8px rgba(0,0,0,.7))}" +
    "#pino .plug{transform-origin:60px 60px;animation:pinobob 3s ease-in-out infinite}" +
    "#pino.talk .plug{animation:pinotalk .28s ease-in-out infinite}" +
    "@keyframes pinobob{50%{transform:translateY(-3px) rotate(-1.5deg)}}@keyframes pinotalk{50%{transform:rotate(3deg)}}" +
    "#pino .pin{cursor:pointer}#pino .pin circle.k{fill:#D9DDE2;stroke:#2A2E33;stroke-width:1.6;transition:fill .12s}" +
    "#pino .pin:hover circle.k,#pino .pin.on circle.k{fill:var(--pino-on,#38FF5C)}#pino .pin.on circle.k{filter:drop-shadow(0 0 5px var(--pino-on,#38FF5C))}" +
    "#pino .pin text{font:700 7px Tahoma,Verdana,sans-serif;fill:#111;pointer-events:none}" +
    "#pino .latch{cursor:pointer}#pino .latch:hover rect{fill:#FF2A1A}" +
    "#pino .bal{position:absolute;right:132px;bottom:110px;width:250px;background:#FFFFE1;color:#000;border:1px solid #000;border-radius:8px;padding:9px 11px 8px;font:12px Tahoma,Verdana,'Segoe UI',sans-serif;line-height:1.4;box-shadow:2px 2px 0 #000;display:none;cursor:default}" +
    "#pino .bal.on{display:block}#pino .bal::after{content:'';position:absolute;right:-11px;bottom:14px;border:6px solid transparent;border-left-color:#000}" +
    "#pino .bal b{display:block;margin-bottom:4px;font-weight:700}#pino .bal ul{list-style:none;margin:6px 0 0;padding:6px 0 0;border-top:1px dotted #888}" +
    "#pino .bal li{padding:3px 4px;cursor:pointer}#pino .bal li:hover{background:#0A246A;color:#fff}#pino .bal li small{color:#666}#pino .bal li:hover small{color:#cfd6ff}" +
    "#pino .bal .x{position:absolute;right:7px;top:4px;font-weight:bold;cursor:pointer}#pino .bal .hint{color:#555;font-size:11px;margin-top:5px}" +
    "@media(prefers-reduced-motion:reduce){#pino .plug,#pino.talk .plug{animation:none}}";
  function pinXY(i) { var a = -Math.PI / 2 + i * Math.PI * 2 / 5; return [60 + Math.cos(a) * 15, 62 + Math.sin(a) * 15]; }
  function svg() { var s = '<svg viewBox="0 0 120 180" xmlns="http://www.w3.org/2000/svg">' +
    // cabo: borracha preta com brilho, dobra à la Clippy e rabo enrolado
    '<path class="cable" d="M60 96 C 60 120, 22 122, 22 146 S 70 176, 84 152 S 60 128, 44 146" fill="none" stroke="#111" stroke-width="12" stroke-linecap="round"/>' +
    '<path d="M60 96 C 60 120, 22 122, 22 146 S 70 176, 84 152 S 60 128, 44 146" fill="none" stroke="#3A3F45" stroke-width="4" stroke-linecap="round" stroke-dasharray="8 10"/>' +
    '<g class="plug">' +
    // bota de borracha e carcaça de alumínio
    '<rect x="42" y="86" width="36" height="16" rx="5" fill="#1E2226" stroke="#0B0D0F" stroke-width="2"/>' +
    '<rect x="26" y="16" width="68" height="76" rx="12" fill="#B9BFC7" stroke="#2A2E33" stroke-width="2.2"/>' +
    '<rect x="31" y="20" width="10" height="68" rx="5" fill="#E4E8EC" opacity=".7"/>' +
    // trava (some, Pino)
    '<g class="latch"><rect x="90" y="46" width="12" height="20" rx="3" fill="#8E959D" stroke="#2A2E33" stroke-width="2"/><title>solta o Pino</title></g>' +
    // face do conector com os 5 pinos = menu
    '<circle cx="60" cy="62" r="26" fill="#15181B" stroke="#2A2E33" stroke-width="2.2"/><circle cx="60" cy="62" r="22" fill="none" stroke="#3A3F45" stroke-width="1"/>';
    PINS.forEach(function (p, i) { var c = pinXY(i); s += '<g class="pin" data-k="' + p[0] + '"><circle class="k" cx="' + c[0] + '" cy="' + c[1] + '" r="6.4"/><text x="' + c[0] + '" y="' + (c[1] + 2.6) + '" text-anchor="middle">' + (i + 1) + '</text><title>' + (i + 1) + ' · ' + p[1] + '</title></g>'; });
    // olhos de Clippy: brancos grandes, pupila que segue o mouse, pálpebra que pisca, sobrancelha
    s += '<g class="eyes">' + [46, 74].map(function (x) { return '<ellipse cx="' + x + '" cy="30" rx="8" ry="10" fill="#fff" stroke="#2A2E33" stroke-width="2"/><circle class="pupil" cx="' + x + '" cy="31" r="3.4" fill="#111"/><rect class="lid" x="' + (x - 9) + '" y="19" width="18" height="0" fill="#B9BFC7"/><path class="brow" d="M' + (x - 8) + ' 16 q 8 -6 16 0" fill="none" stroke="#2A2E33" stroke-width="2.4" stroke-linecap="round"/>'; }).join("") + '</g>' +
    '</g></svg>'; return s; }
  window.Pino = { PINS: PINS, mount: function (parent, o) { o = o || {}; var st = document.createElement("style"); st.textContent = CSS; document.head.appendChild(st);
    var el = document.createElement("div"); el.id = "pino"; el.innerHTML = '<div class="bal"></div>' + svg(); parent.appendChild(el);
    var bal = el.querySelector(".bal"), pupils = el.querySelectorAll(".pupil"), lids = el.querySelectorAll(".lid"), talking = 0, hover = null, api;
    function setCur(k) { el.querySelectorAll(".pin").forEach(function (p) { p.classList.toggle("on", p.dataset.k === k); }); }
    function say(t, items, hint) { bal.innerHTML = '<span class="x">×</span><b>' + t + "</b>" + (items ? "<ul>" + items.map(function (it) { return "<li data-a=\"" + it[0] + "\">" + it[1] + (it[2] ? " <small>" + it[2] + "</small>" : "") + "</li>"; }).join("") + "</ul>" : "") + (hint === false ? "" : '<div class="hint">' + (hint || "os pinos são o menu: 1 laser · 2 fósforo · 3 patchbay · 4 teatro · 5 info · trava = some") + "</div>"); bal.classList.add("on"); talking = 10; el.classList.add("talk"); }
    function hide() { bal.classList.remove("on"); }
    setInterval(function () { if (talking > 0 && --talking === 0) el.classList.remove("talk"); }, 120);
    setInterval(function () { lids.forEach(function (l) { l.setAttribute("height", 22); }); setTimeout(function () { lids.forEach(function (l) { l.setAttribute("height", 0); }); }, 110); }, 3600 + Math.random() * 800);
    document.addEventListener("pointermove", function (e) { var r = el.getBoundingClientRect(), dx = e.clientX - (r.left + 62), dy = e.clientY - (r.top + 32), d = Math.max(1, Math.hypot(dx, dy)), k = Math.min(3, d / 60); pupils.forEach(function (p, i) { p.setAttribute("cx", (i ? 74 : 46) + dx / d * k); p.setAttribute("cy", 31 + dy / d * k); }); });
    el.addEventListener("pointerover", function (e) { var p = e.target.closest(".pin"); if (p && p !== hover) { hover = p; var i = PINS.findIndex(function (x) { return x[0] === p.dataset.k; }); say("pino " + (i + 1) + " · " + PINS[i][1] + (PINS[i][2] ? " · " + PINS[i][2] : ""), null, false); talking = 0; el.classList.remove("talk"); } });
    el.addEventListener("click", function (e) { var p = e.target.closest(".pin"), li = e.target.closest("li"); if (p) { o.on && o.on(p.dataset.k, api); return; } if (li) { o.on && o.on(li.dataset.a, api); return; } if (e.target.closest(".latch")) { say("Tá, me solta. Puxa pelo cabo se precisar.", null, false); setTimeout(hide, 1600); o.on && o.on("bye", api); return; } if (e.target.classList.contains("x")) { hide(); return; } if (e.target.closest(".bal")) return; if (bal.classList.contains("on")) hide(); else say(o.greet || "Sou o Pino. Cabo DMX, cinco pinos, zero paciência com software quadrado. Que rota você quer?"); });
    setCur(o.current); api = { el: el, say: say, hide: hide, current: setCur }; return api; } };
})();
