/* Registro de ações + key binding + MIDI learn (in e out), à la MadMapper/Resolume.
   def(id, label, fn, {key, midi, type:"btn"|"cc", get, addr, ctx, arg, t, range, unit, needs}) registra; run(id, v) executa e manda o feedback MIDI.
   LEARN: learn(id, "key"|"midi") e a próxima tecla / mensagem MIDI vira o binding. Persistido em localStorage.
   Rodada 6: addr é o endereço do registry (laser/1/kpps); ctx é o CommandContext do Chataigne (action|mapping|both),
   mais value (somente leitura, vira values do manifesto) e view (só do previz, vira o bloco view do .spell).
   manifest() gera o module.json das definições; graph() gera o trecho graph do .spell com os bindings atuais como rotas.
   ponytail: MIDI só note/cc, canal incluído na chave ("cc:1:7"); sem NRPN, sem MIDI clock.
   ponytail: cc ligado a ação de disparo (btn) vira rota sem filtro; entra um filtro de limiar quando o PRD §10 tiver um. */
window.Bind = (function () {
  "use strict";
  var A = {}, order = [], DEF = { key: {}, midi: {} }, USR = { key: {}, midi: {} }, learn = null, outs = [], ins = 0, api, onChange = function () {}, last = null;
  try { USR = JSON.parse(localStorage.getItem("sc-laser-bind") || "null") || USR; } catch (e) {}
  function save() { try { localStorage.setItem("sc-laser-bind", JSON.stringify(USR)); } catch (e) {} }
  function table(src) { var t = Object.assign({}, DEF[src]); Object.keys(USR[src]).forEach(function (k) { var id = USR[src][k]; Object.keys(t).forEach(function (kk) { if (t[kk] === id) delete t[kk]; }); if (id) t[k] = id; else delete t[k]; }); return t; }
  function keyOf(id, src) { var t = table(src); return Object.keys(t).filter(function (k) { return t[k] === id; })[0] || ""; }
  function def(id, label, fn, o) { o = o || {}; A[id] = { label: label, fn: fn, type: o.type || "btn", get: o.get, addr: o.addr, ctx: o.ctx, arg: o.arg, t: o.t, range: o.range, unit: o.unit, needs: o.needs }; order.push(id); if (o.key) DEF.key[o.key] = id; if (o.midi) DEF.midi[o.midi] = id; }
  function fire(id, v) { var a = A[id]; last = id; a.fn(v); onChange(); last = null; }
  function run(id, v) { var a = A[id]; if (!a || !a.fn) return false; last = id; a.fn(v); feedback(id); onChange(); last = null; return true; }
  function keyStr(e) { var k = e.key; if (k === " ") k = "Space"; else if (k.length === 1) k = k.toUpperCase(); return (e.ctrlKey || e.metaKey ? "Ctrl+" : "") + (e.shiftKey && k.length > 1 ? "Shift+" : "") + (e.altKey ? "Alt+" : "") + k; }
  document.addEventListener("keydown", function (e) { var tg = e.target.tagName; if (tg === "TEXTAREA" || tg === "INPUT") return; var ks = keyStr(e);
    if (learn && learn.src === "key") { if (ks !== "Escape") { USR.key[ks] = learn.id; save(); } learn = null; onChange(); e.preventDefault(); return; }
    var id = table("key")[ks]; if (id) { e.preventDefault(); run(id); } });
  function onMidi(e) { var d = e.data, ty = d[0] >> 4, ch = (d[0] & 15) + 1; if (ty !== 9 && ty !== 8 && ty !== 11) return; var k = (ty === 11 ? "cc" : "note") + ":" + ch + ":" + d[1];
    if (learn && learn.src === "midi") { if (ty === 8 || (ty === 9 && d[2] === 0)) return; USR.midi[k] = learn.id; learn = null; onChange(); return; }
    var id = table("midi")[k], a = A[id]; if (!a || !a.fn) return; if (a.type === "cc") fire(id, d[2] / 127); else if (ty === 11 ? d[2] > 63 : ty === 9 && d[2] > 0) run(id); }
  function feedback(id) { var a = A[id], t = table("midi"); if (!outs.length || !a) return; Object.keys(t).forEach(function (k) { if (t[k] !== id) return; var p = k.split(":"), ch = +p[1] - 1, v = a.get ? a.get() : 1, val = typeof v === "number" ? Math.round(Math.max(0, Math.min(1, v)) * 127) : v ? 127 : 0; outs.forEach(function (o) { try { o.send([(p[0] === "cc" ? 0xB0 : 0x90) | ch, +p[2], val]); } catch (e) {} }); }); }
  function syncAll() { order.forEach(feedback); }
  function connect() { if (!navigator.requestMIDIAccess) { api.midi = "sem Web MIDI"; onChange(); return; }
    navigator.requestMIDIAccess({ sysex: false }).then(function (acc) { function wire() { ins = 0; outs = []; acc.inputs.forEach(function (i) { i.onmidimessage = onMidi; ins++; }); acc.outputs.forEach(function (o) { outs.push(o); }); api.midi = ins + " in · " + outs.length + " out"; onChange(); syncAll(); } acc.onstatechange = wire; wire(); }, function () { api.midi = "MIDI negado"; onChange(); }); }
  function html() { var tk = table("key"), tm = table("midi"), inv = function (t, id) { return Object.keys(t).filter(function (k) { return t[k] === id; })[0] || "—"; };
    return "<table>" + order.filter(function (id) { return A[id].fn; }).map(function (id) { var a = A[id], lk = learn && learn.id === id && learn.src === "key", lm = learn && learn.id === id && learn.src === "midi"; return "<tr><td class='k'>" + a.label + (a.type === "cc" ? " <small>cc</small>" : "") + (a.addr ? " <small class='ad'>" + a.addr + "</small>" : "") + "</td><td class='b'><button class='lb" + (lk ? " learn" : "") + "' data-learn='key' data-id='" + id + "'>" + (lk ? "TECLA…" : inv(tk, id)) + "</button></td><td class='b'><button class='lb" + (lm ? " learn" : "") + "' data-learn='midi' data-id='" + id + "'>" + (lm ? "MIDI…" : inv(tm, id)) + "</button></td><td class='b'><button class='lb' data-clear='" + id + "'>×</button></td></tr>"; }).join("") + "</table>"; }
  function click(e) { var b = e.target.closest("[data-learn],[data-clear]"); if (!b) return false; if (b.dataset.clear) { var id = b.dataset.clear; ["key", "midi"].forEach(function (src) { Object.keys(table(src)).forEach(function (k) { if (table(src)[k] === id) USR[src][k] = null; }); }); save(); learn = null; } else learn = learn && learn.id === b.dataset.id && learn.src === b.dataset.learn ? null : { id: b.dataset.id, src: b.dataset.learn }; onChange(); return true; }
  /* ---------- orquestrador: manifesto e graph a partir das mesmas definições ---------- */
  function name(a) { return a.addr.replace(/^laser\/1\//, ""); }
  function val(a) { var v = a.get ? a.get() : undefined; if (typeof v === "number" && a.range) { v = a.range[0] + v * (a.range[1] - a.range[0]); if (a.t === "int") v = Math.round(v); else v = +v.toFixed(3); } return v; }
  function ptype(a) { if (a.t === "file") return "value"; if (a.type === "cc") return "value"; return typeof (a.get && a.get()) === "boolean" ? "toggle" : "trigger"; }
  function manifest() { var m = { name: "laser", type: "ilda", version: "0.1.3", hasInput: true, hasOutput: true, parameters: {}, values: {}, commands: {}, dependency: [] };
    order.forEach(function (id) { var a = A[id]; if (!a.addr || !a.ctx || a.ctx === "view") return; var n = name(a), v = val(a);
      if (a.ctx === "value") m.values[n] = { type: a.t || (typeof v === "boolean" ? "bool" : "float"), readOnly: true, unit: a.unit };
      else if (a.ctx === "mapping") m.parameters[n] = { type: a.t || "float", default: v, norm: [0, 1], min: a.range ? a.range[0] : undefined, max: a.range ? a.range[1] : undefined, unit: a.unit };
      else { var c = m.commands[n] || (m.commands[n] = { type: ptype(a), context: a.ctx, parameters: {} }); if (c.context !== a.ctx) c.context = "both"; if (a.type === "cc") c.type = "value"; if (a.t === "file") c.parameters.file = "file"; if (a.arg) Object.keys(a.arg).forEach(function (k) { c.parameters[k] = typeof a.arg[k]; }); if (a.unit) c.unit = a.unit; }
      if (a.needs) m.dependency.push({ source: a.needs, check: "equals", value: true, action: "enable", target: n }); });
    return m; }
  function graph(view) { var nodes = [{ uid: "key", type: "in.key" }, { uid: "midi", type: "in.midi" }], wires = [], pos = { key: [0, 0], midi: [0, 120] }, lagN = 0, P = {};
    order.forEach(function (a) { a = A[a]; if (a.addr && a.ctx === "mapping") P[name(a)] = val(a); });
    nodes.push({ uid: "laser/1", type: "laser", params: P, mute: false, lock: false, enabled: true }); pos["laser/1"] = [520, 60];
    ["key", "midi"].forEach(function (src) { var t = table(src); Object.keys(t).sort().forEach(function (k) { var a = A[t[k]]; if (!a || !a.addr || !a.ctx || a.ctx === "view" || a.ctx === "value") return; var w = { from: src + "/" + k, to: a.addr };
      if (src === "midi" && k.slice(0, 2) === "cc" && a.type === "cc") { var u = "lag/" + (++lagN); nodes.push({ uid: u, type: "filter.lag", params: { ms: 80 }, mute: false, lock: false, enabled: true }); pos[u] = [260, 20 + lagN * 40]; w.filter = u; }
      if (a.arg) w.args = a.arg; w.muted = false; wires.push(w); }); });
    var V = {}; Object.keys(pos).forEach(function (u) { V[u] = { x: pos[u][0], y: pos[u][1], collapsed: false }; }); if (view) Object.assign(V["laser/1"], view);
    return { nodes: nodes, wires: wires, states: [], view: V }; }
  api = { def: def, run: run, has: function (id) { return !!A[id]; }, keyOf: keyOf, learnState: function () { return learn; }, last: function () { return last; }, connect: connect, html: html, click: click, feedback: feedback, syncAll: syncAll, manifest: manifest, graph: graph, midi: "desligado", onChange: function (f) { onChange = f; }, reset: function () { USR = { key: {}, midi: {} }; save(); onChange(); } };
  return api;
})();
