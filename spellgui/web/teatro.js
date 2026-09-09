// TEATRO DE PAPEL: patch, cues e o cenário clicável. Cliente do barramento (`spellcore serve`):
// tudo que a página faz é `Registry::call` por WebSocket — nenhuma regra de show mora aqui.
//
// A metade de cima do arquivo é pura (frame DMX + patch + perfil -> cor e intensidade) e roda no
// `node --test`; a metade de baixo é DOM e só existe no navegador.
"use strict";

// --------------------------------------------------------------- parte pura

// Canais nomeados de uma fixture: {nome do perfil: valor 0..255}.
// `dmx` = {universo: array de 512 bytes}; canal fora do que chegou vale 0.
function canais(profile, base, universe, dmx) {
  const buf = (dmx && dmx[universe]) || [];
  const out = {};
  for (const c of (profile && profile.channels) || []) {
    if (typeof c.offset !== "number") continue;
    out[c.name] = buf[base - 1 + c.offset] || 0;
  }
  return out;
}

// ponytail: cor vem de r/g/b/w ; roda de cor (`wheel`) e gel viram cor quando o perfil declarar
// o tipo do canal (design/FUNCOES/cenas-cues-dmx.md secao 1, "Perfil").
function cor(ch) {
  const v = k => ch[k] || 0;
  const w = v("w");
  const tem = "r" in ch || "g" in ch || "b" in ch;
  if (!tem) return [255, 255, 255]; // dimmer, elipsoidal: lâmpada branca
  return [v("r") + w, v("g") + w, v("b") + w].map(x => Math.min(255, x));
}

// Intensidade 0..1: dimmer manda; sem dimmer, o mais aceso dos canais de cor; senão o `on`.
function intensidade(ch) {
  if ("dim" in ch) return ch.dim / 255;
  const c = ["r", "g", "b", "w"].filter(k => k in ch).map(k => ch[k]);
  if (c.length) return Math.max(...c) / 255;
  if ("on" in ch) return ch.on / 255;
  return 0;
}

// O que cada fixture do patch está mostrando AGORA, a partir do que saiu na rede.
// `patch` = [{name, profile, universe, address, pos}], `profiles` = {nome do arquivo: perfil}.
function look(patch, profiles, dmx) {
  return (patch || []).map((f, i) => {
    const p = profiles[f.profile] || null;
    const u = f.universe || 1;
    const ch = canais(p, f.address || 1, u, dmx);
    return {
      i,
      name: f.name,
      profile: f.profile,
      universe: u,
      address: f.address || 1,
      pos: Array.isArray(f.pos) ? f.pos : null,
      channels: ch,
      rgb: cor(ch),
      intensity: intensidade(ch),
    };
  });
}

// Posição padrão de quem ainda não foi arrastado: uma fileira, como uma vara de luz.
function posicao(f, n) {
  return f.pos || [0.08 + (0.84 * (f.i + 0.5)) / Math.max(1, n), 0.3];
}

const TEATRO = { canais, cor, intensidade, look, posicao };
if (typeof module !== "undefined") module.exports = TEATRO;

// -------------------------------------------------------------- barramento

// ponytail: cliente WS de 40 linhas ; trocar por `bus.js` (F5) quando ele existir — mesma
// assinatura `call(cmd, args)` / `on(evento, fn)`.
TEATRO.bus = {
  ws: null,
  n: 0,
  pend: new Map(),
  ev: {},
  dmx: {},
  on(e, f) { (this.ev[e] = this.ev[e] || []).push(f); },
  emit(e, d) { for (const f of this.ev[e] || []) f(d); },
  open() {
    // ponytail: file:// nao tem host e o WebSocket estoura ; a pagina segue montada, so' sem barramento.
    let ws;
    try {
      ws = new WebSocket(`ws://${location.host}/ws`);
    } catch (e) {
      return this.emit("fechado");
    }
    ws.binaryType = "arraybuffer";
    this.ws = ws;
    ws.onopen = () => this.emit("aberto");
    ws.onclose = () => { this.emit("fechado"); setTimeout(() => this.open(), 2000); };
    ws.onmessage = m => {
      if (typeof m.data !== "string") return this.frame(m.data);
      const v = JSON.parse(m.data);
      if (v.event) return this.emit(v.event, v.data);
      const p = this.pend.get(v.id);
      if (!p) return;
      this.pend.delete(v.id);
      v.error === undefined ? p.ok(v.result) : p.no(new Error(v.error));
    };
  },
  // topic:u8 | universe:u16 LE | 512 bytes  (topic 1 = dmx de saída)
  frame(buf) {
    const d = new DataView(buf);
    if (buf.byteLength < 515 || d.getUint8(0) !== 1) return;
    this.dmx[d.getUint16(1, true)] = new Uint8Array(buf, 3, 512);
    this.emit("dmx");
  },
  call(cmd, args) {
    if (!this.ws || this.ws.readyState !== 1) return Promise.reject(new Error("sem barramento"));
    const id = ++this.n;
    this.ws.send(JSON.stringify({ id, cmd, args: args || {} }));
    return new Promise((ok, no) => this.pend.set(id, { ok, no }));
  },
};

// -------------------------------------------------------------------- página

TEATRO.mount = function (el) {
  const bus = TEATRO.bus;
  const q = id => el.querySelector("#" + id);
  const st = {
    show: { patch: [], cues: [] },   // show inteiro (show_get full)
    grade: [],                       // linhas de patch_check
    erro: null,                      // erro do patch_check (na linha grade.length)
    perfis: {},                      // nome do arquivo -> perfil (profile_get)
    cue: -1,                         // cue corrente do transporte
    sel: -1,                         // fixture aberta no cenário
    drag: null,
  };
  const diga = t => { q("msg").textContent = t; };
  const erro = e => diga(String((e && e.message) || e));

  // -------- carga
  async function recarrega() {
    st.show = await bus.call("show_get", { full: true });
    st.show.patch = st.show.patch || [];
    st.show.cues = st.show.cues || [];
    const g = await bus.call("patch_check", {});
    st.grade = g.rows;
    st.erro = g.error;
    for (const f of st.show.patch) {
      if (f.profile && !(f.profile in st.perfis)) {
        st.perfis[f.profile] = await bus.call("profile_get", { name: f.profile }).catch(() => null);
      }
    }
    patch();
    cues();
  }

  // -------- patch
  function patch() {
    const t = q("patch");
    t.innerHTML = "";
    st.show.patch.forEach((f, i) => {
      const r = st.grade[i];
      const tr = document.createElement("tr");
      const cols = r
        ? [r.name, r.profile, r.universe, r.address, r.channels]
        : [f.name, f.profile, f.universe || 1, f.address || 1, "?"];
      for (const c of cols) {
        const td = document.createElement("td");
        td.textContent = c;
        tr.appendChild(td);
      }
      const td = document.createElement("td");
      const b = document.createElement("button");
      b.textContent = "x";
      b.title = "tirar do patch";
      b.onclick = () => bus.call("patch_del", { name: f.name }).then(recarrega, erro);
      td.appendChild(b);
      tr.appendChild(td);
      if (i === st.grade.length && st.erro) {
        tr.className = "alerta";
        tr.title = st.erro;
      }
      tr.onclick = () => abre(i);
      t.appendChild(tr);
    });
    q("patch-erro").textContent = st.erro || "";
  }

  q("add").onclick = () => {
    bus
      .call("patch_add", {
        name: q("f-nome").value,
        profile: q("f-perfil").value,
        universe: +q("f-uni").value,
        address: +q("f-end").value,
      })
      .then(recarrega, e => { erro(e); q("patch-erro").textContent = e.message; });
  };

  // -------- cues
  function cues() {
    const t = q("cues");
    t.innerHTML = "";
    st.show.cues.forEach((c, i) => {
      const tr = document.createElement("tr");
      if (i === st.cue) tr.className = "viva";
      const campo = (k, tipo) => {
        const td = document.createElement("td");
        const inp = document.createElement("input");
        inp.type = tipo;
        if (tipo === "checkbox") inp.checked = !!c[k];
        else inp.value = c[k] === undefined ? "" : c[k];
        inp.onchange = () => {
          const v = tipo === "checkbox" ? inp.checked : tipo === "number" ? +inp.value : inp.value;
          bus
            .call("cue_set", {
              index: i,
              name: c.name || "",
              fade: c.fade || 0,
              wait: c.wait || 0,
              follow: !!c.follow,
              values: c.values || {},
              [k]: v,
            })
            .then(recarrega, erro);
        };
        td.appendChild(inp);
        tr.appendChild(td);
      };
      const n = document.createElement("td");
      n.textContent = i;
      tr.appendChild(n);
      campo("name", "text");
      campo("fade", "number");
      campo("wait", "number");
      campo("follow", "checkbox");
      const nv = document.createElement("td");
      nv.textContent = Object.keys(c.values || {}).length + " ch";
      tr.appendChild(nv);
      const td = document.createElement("td");
      const b = document.createElement("button");
      b.textContent = "x";
      b.onclick = e => { e.stopPropagation(); bus.call("cue_del", { index: i }).then(recarrega, erro); };
      td.appendChild(b);
      tr.appendChild(td);
      tr.ondblclick = () => bus.call("cue_go", { index: i }).catch(erro);
      t.appendChild(tr);
    });
  }

  q("go").onclick = () => bus.call("cue_go", {}).then(() => diga("GO"), erro);
  q("capturar").onclick = () =>
    bus.call("cue_capture", { name: q("cena-nome").value || "cena" }).then(recarrega, erro);
  q("solta").onclick = () => bus.call("level_clear", {}).then(n => diga(n + " ch soltos"), erro);

  // -------- cenário
  const cv = q("cenario");
  const ctx = cv.getContext("2d");

  function desenha() {
    const w = (cv.width = cv.clientWidth);
    const h = (cv.height = cv.clientHeight);
    const css = getComputedStyle(document.documentElement);
    ctx.fillStyle = css.getPropertyValue("--sc-well") || "#070707";
    ctx.fillRect(0, 0, w, h);
    ctx.strokeStyle = css.getPropertyValue("--sc-hair") || "#222";
    ctx.beginPath();
    ctx.moveTo(0, h * 0.3);
    ctx.lineTo(w, h * 0.3);
    ctx.stroke();
    const fs = look(st.show.patch, st.perfis, bus.dmx);
    fs.forEach(f => {
      const [x, y] = posicao(f, fs.length);
      const [px, py] = [x * w, y * h];
      const [r, g, b] = f.rgb;
      const a = f.intensity;
      if (a > 0.01) {
        const grd = ctx.createRadialGradient(px, py, 4, px, py, 46);
        grd.addColorStop(0, `rgba(${r},${g},${b},${0.55 * a})`);
        grd.addColorStop(1, "rgba(0,0,0,0)");
        ctx.fillStyle = grd;
        ctx.fillRect(px - 46, py - 46, 92, 92);
      }
      ctx.beginPath();
      ctx.arc(px, py, 11, 0, 6.2832);
      ctx.fillStyle = `rgba(${r},${g},${b},${0.12 + 0.88 * a})`;
      ctx.fill();
      ctx.lineWidth = f.i === st.sel ? 2 : 1;
      ctx.strokeStyle = f.i === st.sel
        ? css.getPropertyValue("--sc-accent") || "#FFB000"
        : css.getPropertyValue("--sc-fg-3") || "#888";
      ctx.stroke();
      ctx.fillStyle = css.getPropertyValue("--sc-fg-3") || "#888";
      ctx.font = "10px monospace";
      ctx.textAlign = "center";
      ctx.fillText(f.name || "?", px, py + 26);
    });
  }

  function acha(e) {
    const b = cv.getBoundingClientRect();
    const p = [(e.clientX - b.left) / b.width, (e.clientY - b.top) / b.height];
    const fs = look(st.show.patch, st.perfis, bus.dmx);
    let melhor = -1;
    let d = 0.04;
    fs.forEach(f => {
      const [x, y] = posicao(f, fs.length);
      const dd = Math.hypot((x - p[0]) * b.width, (y - p[1]) * b.height);
      if (dd < d * b.width) { d = dd / b.width; melhor = f.i; }
    });
    return [melhor, p];
  }

  cv.onpointerdown = e => {
    const [i, p] = acha(e);
    if (i < 0) return;
    st.drag = { i, p, moveu: false };
    cv.setPointerCapture(e.pointerId);
  };
  cv.onpointermove = e => {
    if (!st.drag) return;
    const b = cv.getBoundingClientRect();
    const p = [(e.clientX - b.left) / b.width, (e.clientY - b.top) / b.height];
    st.drag.moveu = st.drag.moveu || Math.hypot(p[0] - st.drag.p[0], p[1] - st.drag.p[1]) > 0.005;
    st.show.patch[st.drag.i].pos = p;
    desenha();
  };
  cv.onpointerup = () => {
    const d = st.drag;
    st.drag = null;
    if (!d) return;
    if (!d.moveu) return abre(d.i);
    // posição é do show: vai por show_patch (JSON Patch), o único caminho de edição parcial
    bus
      .call("show_patch", {
        ops: [{ op: "add", path: `/patch/${d.i}/pos`, value: st.show.patch[d.i].pos }],
      })
      .catch(erro);
  };

  // clique na fixture: os canais do perfil viram widgets -> fixture_set
  async function abre(i) {
    st.sel = i;
    const f = st.show.patch[i];
    const box = q("canais");
    box.innerHTML = "";
    if (!f) return;
    if (!st.perfis[f.profile]) {
      st.perfis[f.profile] = await bus.call("profile_get", { name: f.profile }).catch(() => null);
    }
    const p = st.perfis[f.profile];
    const cab = document.createElement("div");
    cab.className = "over";
    cab.textContent = `${f.name} — ${(p && p.name) || f.profile} — u${f.universe || 1}/${f.address || 1}`;
    box.appendChild(cab);
    const vivo = canais(p, f.address || 1, f.universe || 1, bus.dmx);
    for (const c of (p && p.channels) || []) {
      const l = document.createElement("label");
      l.textContent = c.name;
      const s = document.createElement("input");
      s.type = "range";
      s.min = 0;
      s.max = 255;
      s.value = vivo[c.name] || 0;
      const n = document.createElement("span");
      n.textContent = s.value;
      s.oninput = () => {
        n.textContent = s.value;
        bus.call("fixture_set", { name: f.name, channel: c.name, value: +s.value }).catch(erro);
      };
      l.appendChild(s);
      l.appendChild(n);
      box.appendChild(l);
    }
    desenha();
  }

  // -------- barramento vivo
  bus.on("aberto", () => {
    diga("ligado");
    bus.call("profiles", {}).then(v => {
      q("f-perfil").innerHTML = "";
      for (const n of v) {
        const o = document.createElement("option");
        o.value = o.textContent = n;
        q("f-perfil").appendChild(o);
      }
    }, erro);
    recarrega().catch(erro);
  });
  bus.on("fechado", () => diga("sem barramento — spellcore serve"));
  bus.on("show", () => recarrega().catch(erro));
  bus.on("transport", d => {
    if (d.cue !== st.cue) { st.cue = d.cue; cues(); }
    q("cue-viva").textContent = st.cue < 0 ? "—" : `${st.cue} ${(st.show.cues[st.cue] || {}).name || ""}`;
  });
  bus.on("dmx", () => { st.pinta = true; });

  (function laco() {
    if (st.pinta) { st.pinta = false; desenha(); }
    requestAnimationFrame(laco);
  })();
  addEventListener("resize", desenha);
  addEventListener("keydown", e => {
    if (e.key === "Enter" && e.target.tagName !== "INPUT") q("go").click();
  });
  desenha();
  bus.open();
  // o estado vai de volta com os tres desenhadores: e' por onde se redesenha sem barramento
  return Object.assign(st, { patch, cues, desenha });
};
