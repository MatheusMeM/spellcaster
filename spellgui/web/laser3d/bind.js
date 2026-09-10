/* Action registry + key binding + MIDI learn (in and out), MadMapper/Resolume style.
   def(id, label, fn, {key, type:"btn"|"cc", get}) registers; run(id, v) runs it and sends the MIDI feedback.
   LEARN: learn(id, "key"|"midi") and the next key / MIDI message becomes the binding. Persisted in localStorage.
   ponytail: MIDI note/cc only, channel included in the key ("cc:1:7"); no NRPN, no MIDI clock. */
window.Bind = (function () {
  "use strict";
  var A = {}, order = [], DEF = { key: {}, midi: {} }, USR = { key: {}, midi: {} }, learn = null, outs = [], ins = 0, api, onChange = function () {};
  try { USR = JSON.parse(localStorage.getItem("sc-laser-bind") || "null") || USR; } catch (e) {}
  function save() { try { localStorage.setItem("sc-laser-bind", JSON.stringify(USR)); } catch (e) {} }
  function table(src) { var t = Object.assign({}, DEF[src]); Object.keys(USR[src]).forEach(function (k) { var id = USR[src][k]; Object.keys(t).forEach(function (kk) { if (t[kk] === id) delete t[kk]; }); if (id) t[k] = id; else delete t[k]; }); return t; }
  function keyOf(id, src) { var t = table(src); return Object.keys(t).filter(function (k) { return t[k] === id; })[0] || ""; }
  function def(id, label, fn, o) { o = o || {}; A[id] = { label: label, fn: fn, type: o.type || "btn", get: o.get }; order.push(id); if (o.key) DEF.key[o.key] = id; if (o.midi) DEF.midi[o.midi] = id; }
  function run(id, v) { var a = A[id]; if (!a) return false; a.fn(v); feedback(id); onChange(); return true; }
  // "Shift+" also applies to a single-character key: without this "Shift+Z" never arrived (keyStr
  // returned "Z" and the binding was dead). `idFor` falls back to the key without Shift when the
  // combination is not mapped, so "S" keeps arming with Shift or CapsLock held down.
  function keyStr(e) { var k = e.key; if (k === " ") k = "Space"; else if (k.length === 1) k = k.toUpperCase(); return (e.ctrlKey || e.metaKey ? "Ctrl+" : "") + (e.shiftKey ? "Shift+" : "") + (e.altKey ? "Alt+" : "") + k; }
  function idFor(ks) { var t = table("key"); return t[ks] || (ks.indexOf("Shift+") === 0 ? t[ks.slice(6)] : undefined); }
  // Only the focused control keeps the keys IT uses (arrows and Home/End in a fader); the rest of the
  // keyboard still belongs to the device. Before, any focused <input> killed the whole keyboard, and
  // since clicking a fader in the panel focuses it, after touching the LIMIT neither "2" nor "B"
  // answered until you clicked on empty space.
  document.addEventListener("keydown", function (e) { var tg = e.target, T = tg.tagName;
    if (T === "TEXTAREA" || tg.isContentEditable || (T === "INPUT" && tg.type !== "range")) return;
    if (T === "INPUT" && /^(Arrow|Home|End|Page)/.test(e.key)) return;
    var ks = keyStr(e);
    // Escape cancels any learn (key OR MIDI). Before, the MIDI learn ignored Escape: the key went
    // straight through, closed the panel, and the learn stayed armed with nothing on screen counting.
    if (learn) { if (ks === "Escape" || learn.src === "key") { if (ks !== "Escape") { USR.key[ks] = learn.id; save(); } learn = null; onChange(); } e.preventDefault(); return; }
    var id = idFor(ks); if (id) { e.preventDefault(); run(id); } });
  function onMidi(e) { var d = e.data, ty = d[0] >> 4, ch = (d[0] & 15) + 1; if (ty !== 9 && ty !== 8 && ty !== 11) return; var k = (ty === 11 ? "cc" : "note") + ":" + ch + ":" + d[1];
    if (learn && learn.src === "midi") { if (ty === 8 || (ty === 9 && d[2] === 0)) return; USR.midi[k] = learn.id; learn = null; onChange(); return; }
    var id = table("midi")[k], a = A[id]; if (!a) return; if (a.type === "cc") { a.fn(d[2] / 127); onChange(); } else if (ty === 11 ? d[2] > 63 : ty === 9 && d[2] > 0) run(id); }
  function feedback(id) { var a = A[id], t = table("midi"); if (!outs.length || !a) return; Object.keys(t).forEach(function (k) { if (t[k] !== id) return; var p = k.split(":"), ch = +p[1] - 1, v = a.get ? a.get() : 1, val = typeof v === "number" ? Math.round(Math.max(0, Math.min(1, v)) * 127) : v ? 127 : 0; outs.forEach(function (o) { try { o.send([(p[0] === "cc" ? 0xB0 : 0x90) | ch, +p[2], val]); } catch (e) {} }); }); }
  function syncAll() { order.forEach(feedback); }
  function connect() { if (!navigator.requestMIDIAccess) { api.midi = "no Web MIDI"; onChange(); return; }
    navigator.requestMIDIAccess({ sysex: false }).then(function (acc) { function wire() { ins = 0; outs = []; acc.inputs.forEach(function (i) { i.onmidimessage = onMidi; ins++; }); acc.outputs.forEach(function (o) { outs.push(o); }); api.midi = ins + " in · " + outs.length + " out"; onChange(); syncAll(); } acc.onstatechange = wire; wire(); }, function () { api.midi = "MIDI denied"; onChange(); }); }
  function html() { var tk = table("key"), tm = table("midi"), inv = function (t, id) { return Object.keys(t).filter(function (k) { return t[k] === id; })[0] || "—"; };
    return "<table>" + order.map(function (id) { var a = A[id], lk = learn && learn.id === id && learn.src === "key", lm = learn && learn.id === id && learn.src === "midi"; return "<tr><td class='k'>" + a.label + (a.type === "cc" ? " <small>cc</small>" : "") + "</td><td class='b'><button class='lb" + (lk ? " learn" : "") + "' data-learn='key' data-id='" + id + "'>" + (lk ? "KEY…" : inv(tk, id)) + "</button></td><td class='b'><button class='lb" + (lm ? " learn" : "") + "' data-learn='midi' data-id='" + id + "'>" + (lm ? "MIDI…" : inv(tm, id)) + "</button></td><td class='b'><button class='lb' data-clear='" + id + "'>×</button></td></tr>"; }).join("") + "</table>"; }
  function click(e) { var b = e.target.closest("[data-learn],[data-clear]"); if (!b) return false; if (b.dataset.clear) { var id = b.dataset.clear; ["key", "midi"].forEach(function (src) { Object.keys(table(src)).forEach(function (k) { if (table(src)[k] === id) USR[src][k] = null; }); }); save(); learn = null; } else learn = learn && learn.id === b.dataset.id && learn.src === b.dataset.learn ? null : { id: b.dataset.id, src: b.dataset.learn }; onChange(); return true; }
  /// The action table as data: `[{id, label, key, midi, type}]`, in the order they were declared. It
  /// is what the MIDI front of the engine consumes to map a controller without reimplementing the
  /// list; the source is still a single one (the `Bind.def` calls in `app.js`).
  function manifest() { return order.map(function (id) { var a = A[id]; return { id: id, label: a.label, key: keyOf(id, "key"), midi: keyOf(id, "midi"), type: a.type }; }); }
  api = { def: def, run: run, has: function (id) { return !!A[id]; }, keyOf: keyOf, manifest: manifest, learnState: function () { return learn; }, connect: connect, html: html, click: click, feedback: feedback, syncAll: syncAll, midi: "off", onChange: function (f) { onChange = f; }, reset: function () { USR = { key: {}, midi: {} }; save(); onChange(); } };
  return api;
})();
if (typeof module !== "undefined") module.exports = window.Bind;
