/* ILDA: leitor/escritor (mesmo da rodada 3), frames demo, render 2D da parede e o contorno da splash.
   ponytail: formato 2 (paleta) é pulado; 0/1/4/5 lidos. */
window.ILDA = (function () {
  "use strict";
  function write(frames) { var recs = 0, i; for (i = 0; i < frames.length; i++) recs += frames[i].length; var buf = new Uint8Array((frames.length + 1) * 32 + recs * 8), o = 0, dv = new DataView(buf.buffer);
    function head(n, fi, total) { buf.set([73, 76, 68, 65, 0, 0, 0, 5], o); buf.set([83, 80, 69, 76, 76, 32, 32, 32, 70, 69, 73, 84, 73, 67, 65, 82], o + 8); dv.setUint16(o + 24, n); dv.setUint16(o + 26, fi); dv.setUint16(o + 28, total); o += 32; }
    for (i = 0; i < frames.length; i++) { var f = frames[i]; head(f.length, i, frames.length); for (var j = 0; j < f.length; j++) { var p = f[j]; dv.setInt16(o, p.x); dv.setInt16(o + 2, p.y); buf[o + 4] = (p.bl ? 64 : 0) | (j === f.length - 1 ? 128 : 0); buf[o + 5] = p.b; buf[o + 6] = p.g; buf[o + 7] = p.r; o += 8; } }
    head(0, frames.length, frames.length); return buf; }
  function pal(i) { if (i === 0) return [255, 0, 0]; if (i >= 56) return [255, 255, 255]; var h = (i / 56) * 6, k = Math.floor(h), f = Math.round((h - k) * 255), c = [[255, f, 0], [255 - f, 255, 0], [0, 255, f], [0, 255 - f, 255], [f, 0, 255], [255, 0, 255 - f]]; return c[k % 6]; }
  function parse(ab) { var dv = new DataView(ab), o = 0, frames = [], name = "";
    while (o + 32 <= ab.byteLength) { if (String.fromCharCode(dv.getUint8(o), dv.getUint8(o + 1), dv.getUint8(o + 2), dv.getUint8(o + 3)) !== "ILDA") break; var fmt = dv.getUint8(o + 7), n = dv.getUint16(o + 24); if (!name) name = String.fromCharCode.apply(null, new Uint8Array(ab, o + 8, 8)).replace(/\0/g, "").trim(); o += 32; if (n === 0) break; if (fmt === 2) { o += n * 3; continue; }
      var f = [], sz = fmt === 0 ? 8 : fmt === 1 ? 6 : fmt === 4 ? 10 : 8; for (var j = 0; j < n; j++) { var x = dv.getInt16(o), y = dv.getInt16(o + 2), st, c; if (fmt === 0) { st = dv.getUint8(o + 6); c = pal(dv.getUint8(o + 7)); } else if (fmt === 1) { st = dv.getUint8(o + 4); c = pal(dv.getUint8(o + 5)); } else if (fmt === 4) { st = dv.getUint8(o + 6); c = [dv.getUint8(o + 9), dv.getUint8(o + 8), dv.getUint8(o + 7)]; } else { st = dv.getUint8(o + 4); c = [dv.getUint8(o + 7), dv.getUint8(o + 6), dv.getUint8(o + 5)]; } f.push({ x: x, y: y, r: c[0], g: c[1], b: c[2], bl: !!(st & 64) }); o += sz; } frames.push(f); }
    return { name: name, frames: frames }; }
  function P(x, y, c, bl) { return { x: Math.max(-32768, Math.min(32767, Math.round(x * 32767))), y: Math.max(-32768, Math.min(32767, Math.round(y * 32767))), r: c[0], g: c[1], b: c[2], bl: !!bl }; }
  function poly(pts, c, out) { out.push(P(pts[0][0], pts[0][1], c, true)); for (var i = 0; i < pts.length; i++) out.push(P(pts[i][0], pts[i][1], c)); }
  function demo() { var F = [], n = 90; for (var i = 0; i < n; i++) { var t = i / n, f = [], j, k;
      for (j = 0; j < 4; j++) { var s = .28 + ((t * 2 + j / 4) % 1) * .72, a = t * Math.PI * 2 + j * .2, cc = [[255, 40, 40], [40, 255, 60], [60, 90, 255], [255, 255, 255]][j], q = []; for (k = 0; k <= 4; k++) { var an = a + k * Math.PI / 2 + Math.PI / 4; q.push([Math.cos(an) * s, Math.sin(an) * s]); } poly(q, cc, f); }
      var st = [], r = .34; for (k = 0; k <= 5; k++) { var an2 = -Math.PI / 2 + t * Math.PI * 2 + k * Math.PI * 4 / 5; st.push([Math.cos(an2) * r, Math.sin(an2) * r]); } poly(st, [56, 255, 92], f);
      for (k = 0; k < 90; k++) { var th = k / 90 * Math.PI * 2; f.push(P(Math.sin(3 * th + t * Math.PI * 2) * .82, Math.sin(4 * th) * .22 - .62, [Math.round(128 + 127 * Math.sin(th)), Math.round(128 + 127 * Math.sin(th + 2.1)), Math.round(128 + 127 * Math.sin(th + 4.2))], k === 0)); }
      F.push(f); } return F; }

  /* contorno de texto: rasteriza, marching squares (ponto médio, grade de 3 px), encadeia em laços.
     É o que o galvo desenha na splash: laços ordenados da esquerda para a direita. */
  function outlines(lines, W, H) { var c = document.createElement("canvas"); c.width = W; c.height = H; var x = c.getContext("2d"); x.fillStyle = "#fff"; x.textAlign = "center"; x.textBaseline = "middle";
    lines.forEach(function (l) { x.font = l[1]; x.fillText(l[0], W / 2, l[2]); });
    var d = x.getImageData(0, 0, W, H).data, s = 3, gw = Math.floor(W / s), gh = Math.floor(H / s), bit = function (i, j) { return i < 0 || j < 0 || i >= gw || j >= gh ? 0 : d[((j * s) * W + i * s) * 4 + 3] > 128 ? 1 : 0; };
    var segs = [], i, j; for (j = -1; j < gh; j++) for (i = -1; i < gw; i++) { var k = bit(i, j) * 8 + bit(i + 1, j) * 4 + bit(i + 1, j + 1) * 2 + bit(i, j + 1); if (k === 0 || k === 15) continue;
      var T = [i + .5, j], R = [i + 1, j + .5], B = [i + .5, j + 1], L = [i, j + .5], e = { 1: [L, B], 2: [B, R], 3: [L, R], 4: [T, R], 5: [L, T, B, R], 6: [T, B], 7: [L, T], 8: [T, L], 9: [T, B], 10: [T, R, L, B], 11: [T, R], 12: [R, L], 13: [R, B], 14: [B, L] }[k]; for (var q = 0; q < e.length; q += 2) segs.push([e[q], e[q + 1]]); }
    var key = function (p) { return p[0] + "," + p[1]; }, at = {}; segs.forEach(function (sg, n) { [sg[0], sg[1]].forEach(function (p) { (at[key(p)] = at[key(p)] || []).push(n); }); });
    var used = [], loops = []; segs.forEach(function (sg, n) { if (used[n]) return; used[n] = 1; var loop = [sg[0], sg[1]], cur = sg[1]; for (var guard = 0; guard < 20000; guard++) { var nb = at[key(cur)].filter(function (m) { return !used[m]; })[0]; if (nb == null) break; used[nb] = 1; var ns = segs[nb], nx = key(ns[0]) === key(cur) ? ns[1] : ns[0]; loop.push(nx); cur = nx; if (key(cur) === key(loop[0])) break; } if (loop.length > 6) loops.push(loop.map(function (p) { return [(p[0] + .5) * s, (p[1] + .5) * s]; })); });
    loops.sort(function (a, b) { var ma = Math.min.apply(null, a.map(function (p) { return p[0]; })), mb = Math.min.apply(null, b.map(function (p) { return p[0]; })); return ma - mb || a[0][1] - b[0][1]; });
    var len = 0; loops.forEach(function (l) { l.len = 0; for (var n = 1; n < l.length; n++) l.len += Math.hypot(l[n][0] - l[n - 1][0], l[n][1] - l[n - 1][1]); len += l.len; });
    return { loops: loops, len: len }; }
  return { write: write, parse: parse, demo: demo, outlines: outlines };
})();
