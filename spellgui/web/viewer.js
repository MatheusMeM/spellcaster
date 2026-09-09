"use strict";
// viewer.js — previz 2D da timeline: o que sai no tempo `t`, na faixa de baixo do canvas (Alt+M).
//
// Tres colunas, cada uma um pedaco da saida no playhead:
//   dmx    512 barras do ultimo frame binario do universo da lane focada (topic 1 do barramento);
//   ilda   o quadro do .ild de cada track laser em `t`, com `scale` e `rot` do track aplicados;
//   patch  uma bolinha por fixture do patch, pintada com o frame de saida pelo perfil.
//
// Nao desenha: 3D, feixe, thumbnail por lane, video (ROADMAP R6 e' o Godot, outra coisa).
//
// Arquivo separado de proposito: a timeline so' chama `VW.draw(ctx, rect, t)`, e o previz cresce
// sem tocar em timeline.js. Precisa de `window.TL` e da parte pura de `teatro.js` (`TEATRO`) —
// index.html carrega os dois antes deste script.

// IIFE porque script classico compartilha o escopo global: `const TL` ja' e' de timeline.js.
(function (window, TEATRO) {
  const TL = window.TL;

  const VW = {
    clip: new Map(),     // "arquivo|indice" -> pontos do quadro, ou 1 enquanto o pedido esta' em voo
    n: new Map(),        // arquivo -> quantos quadros tem (so' depois da primeira resposta)
    prof: new Map(),     // nome do perfil -> canais, ou 1 em voo
    call: (cmd, args) => TL.call(cmd, args),
    // ponytail: teto de 512 quadros em cache, esvaziado inteiro ; virar LRU se um .ild longo piscar
    cap: 512,
  };
  window.VW = VW;

  // ---- modelo -------------------------------------------------------------

  // Valor de um parametro do track (`scale`, `rot`) em `t`, direto das keys do .spell. Mesma
  // matematica de `TL.valueAt`: a curva vale para o segmento que CHEGA no keyframe.
  VW.param = function (keys, t, def) {
    if (!Array.isArray(keys) || !keys.length) return def;
    if (t <= keys[0][0]) return +keys[0][1];
    let i = 0;
    while (i + 1 < keys.length && keys[i + 1][0] <= t) i++;
    if (i + 1 >= keys.length) return +keys[i][1];
    const a = keys[i], b = keys[i + 1], dt = b[0] - a[0];
    if (dt <= 0) return +b[1];
    return +a[1] + (+b[1] - +a[1]) * TL.ease(b[2])((t - a[0]) / dt);
  };

  // Quadro do clipe em `t`: a conta do player (`spellcaster/player/player.py::laser_frame`).
  // Antes da primeira resposta nao se sabe quantos quadros o arquivo tem; o engine tira o resto
  // de qualquer jeito, entao o indice cru so' custa uma entrada a mais no cache.
  VW.index = function (clip, t, fps) {
    const i = Math.max(0, Math.floor(t * (fps > 0 ? fps : 30)));
    const n = VW.n.get(clip);
    return n ? i % n : i;
  };

  // Pontos do quadro, ou null enquanto nao chegam. Um pedido por (clipe, quadro).
  VW.frame = function (clip, index) {
    const k = clip + "|" + index;
    const v = VW.clip.get(k);
    if (v !== undefined) return v === 1 ? null : v;
    if (VW.clip.size >= VW.cap) VW.clip.clear();
    VW.clip.set(k, 1);
    VW.call("clip_frame", { clip: clip, index: index }).then(r => {
      VW.n.set(clip, r.frames);
      VW.clip.set(k, r.points || []);
      if (TL.k) TL.k.invalidate();
    }, () => VW.clip.set(k, []));      // erro fica cacheado: nao adianta pedir de novo todo frame
    return null;
  };

  // Canais do perfil, ou null enquanto nao chegam.
  VW.perfil = function (name) {
    const v = VW.prof.get(name);
    if (v !== undefined) return v === 1 ? null : v;
    VW.prof.set(name, 1);
    VW.call("profile_get", { name: name }).then(p => {
      VW.prof.set(name, p.channels || []);
      if (TL.k) TL.k.invalidate();
    }, () => VW.prof.set(name, []));
    return null;
  };

  // Cor da fixture no frame de saida: a MESMA conta do teatro de papel (`teatro.js`, parte pura,
  // coberta por test/teatro.test.js). Copia propria discordava dele na mesma fixture: ignorava o
  // canal `w` (a barra WLED em branco puro saia preta aqui e branca la') e o canal `on`.
  // Apagada (intensidade 0) devolve null: a bolinha fica so' com o contorno.
  VW.cor = function (chans, data, address) {
    if (!chans || !chans.length || !data) return null;
    const ch = TEATRO.canais({ channels: chans }, address, 1, { 1: data });
    const k = TEATRO.intensidade(ch);
    return k ? TEATRO.cor(ch).map(x => Math.round(x * k)) : null;
  };

  // ---- desenho ------------------------------------------------------------

  function barras(c, b) {
    const sel = TL.lanes[TL.cur];
    const u = sel ? +(sel.spec.universe || 1) : 1, d = TL.dmx.get(u);
    if (!d) return "dmx u" + u + "  sem frame";
    const bw = b.w / 512;
    c.fillStyle = TL.col.accent;
    for (let i = 0; i < 512; i++) {
      const v = d[i];
      if (v) c.fillRect(b.x + i * bw, b.y + b.h - v / 255 * b.h, Math.max(1, bw - 0.4), v / 255 * b.h);
    }
    return "dmx u" + u;
  }

  function quadro(c, b, t) {
    const tracks = ((TL.show && TL.show.tracks) || []).filter(s => s.type === "laser" && s.clip);
    if (!tracks.length) return "sem laser";
    const s = Math.min(b.w, b.h) / 2, cx = b.x + b.w / 2, cy = b.y + b.h / 2;
    let lab = "";
    for (const sp of tracks) {
      const i = VW.index(sp.clip, t, +sp.fps || TL.fps());
      const pts = VW.frame(sp.clip, i);
      lab = sp.clip + " #" + i;
      if (!pts) { lab += " ..."; continue; }
      // scale e rot como no player: escala, gira em graus, e o Y do ILDA cresce para cima
      const k = VW.param(sp.scale, t, 1), a = VW.param(sp.rot, t, 0) * Math.PI / 180;
      const ca = Math.cos(a) * k * s, sa = Math.sin(a) * k * s;
      let cur = "", aberto = false, lx = 0, ly = 0;
      const fecha = () => { if (aberto) { c.strokeStyle = cur; c.stroke(); aberto = false; } };
      c.lineWidth = 1;
      for (let j = 0; j < pts.length; j++) {
        const p = pts[j], x = cx + p[0] * ca - p[1] * sa, y = cy - (p[0] * sa + p[1] * ca);
        const on = !p[5] && (p[2] | p[3] | p[4]);
        const q = on ? "rgb(" + p[2] + "," + p[3] + "," + p[4] + ")" : "";
        if (j && on) {
          if (q !== cur) { fecha(); cur = q; }
          if (!aberto) { c.beginPath(); c.moveTo(lx, ly); aberto = true; }
          c.lineTo(x, y);
        } else fecha();
        lx = x; ly = y;
      }
      fecha();
    }
    return lab;
  }

  function planta(c, b) {
    const ps = (TL.show && TL.show.patch) || [];
    if (!ps.length) return "sem patch";
    // Grade por ordem de endereco: o patch nao tem x/y (design/DECISOES.md, aguarda voto).
    const fx = ps.slice().sort((x, y) =>
      (+x.universe || 1) - (+y.universe || 1) || (+x.address || 1) - (+y.address || 1));
    const cols = Math.ceil(Math.sqrt(fx.length)), rows = Math.ceil(fx.length / cols);
    const dx = b.w / cols, dy = b.h / rows, rad = Math.max(2, Math.min(dx, dy) / 2 - 3);
    for (let i = 0; i < fx.length; i++) {
      const f = fx[i];
      const x = b.x + dx * (i % cols) + dx / 2, y = b.y + dy * Math.floor(i / cols) + dy / 2;
      const rgb = VW.cor(VW.perfil(f.profile), TL.dmx.get(+f.universe || 1), +f.address || 1);
      c.beginPath();
      c.arc(x, y, rad, 0, Math.PI * 2);
      if (rgb) { c.fillStyle = "rgb(" + rgb + ")"; c.fill(); }
      c.strokeStyle = TL.col.fg3; c.lineWidth = 1; c.stroke();
    }
    return fx.length + " fixtures";
  }

  // `rect` = {x, y, w, h}: a faixa que a timeline reserva no rodape do canvas.
  VW.draw = function (c, r, t) {
    const col = TL.col, w = Math.floor(r.w / 3);
    c.fillStyle = col.panel;
    c.fillRect(r.x, r.y, r.w, r.h);
    c.strokeStyle = col.line; c.lineWidth = 1;
    c.beginPath();
    c.moveTo(r.x, r.y + 0.5); c.lineTo(r.x + r.w, r.y + 0.5);
    for (let i = 1; i < 3; i++) {
      c.moveTo(Math.round(r.x + i * w) + 0.5, r.y + 1);
      c.lineTo(Math.round(r.x + i * w) + 0.5, r.y + r.h);
    }
    c.stroke();
    c.font = "10px " + col.mono;          // textBaseline "middle" ja' vem de draw() em timeline.js
    const fs = [barras, quadro, planta];
    for (let i = 0; i < 3; i++) {
      const b = { x: r.x + i * w + 8, y: r.y + 20, w: w - 16, h: r.h - 26 };
      c.save();
      c.beginPath();
      c.rect(b.x, b.y, b.w, b.h);
      c.clip();
      const lab = fs[i](c, b, t);
      c.restore();
      c.fillStyle = col.fg3;
      c.fillText(lab, r.x + i * w + 8, r.y + 10);
    }
  };

})(window, TEATRO);
