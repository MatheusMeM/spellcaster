/* laser3d ↔ engine: o mapa estado → comando do registry, e nada mais.
   `cmdFor(id, value, st)` e' pura: `id` e' o caminho do estado do aparelho (o mesmo nome que o
   binding e o slider usam), `value` e' o valor absoluto ja' aplicado em `S`, `st` e' o contexto do
   feed aberto ({feed, dac, host, kpps, file, fps, show}). Devolve `{cmd, args}` do registry, ou
   `null` quando aquele estado nao tem comando hoje — a pagina segue local e o desenho na parede e'
   o mesmo. Quem chama e' `app.js`; quem executa e' `bus.call`. */
(function () {
  "use strict";

  function cmdFor(id, v, st) {
    st = st || {};
    var feed = st.feed;
    switch (id) {
      // a chave arma o aparelho: abre (ou fecha) o DAC escolhido
      case "key":
        if (v) return { cmd: "laser_open", args: { dac: st.dac || "etherdream", host: st.host || "", kpps: (st.kpps || 30000) / 1000 } };
        return feed == null ? null : { cmd: "laser_close", args: { feed: feed } };
      case "play":
        if (feed == null) return null;
        if (!v) return { cmd: "laser_stop", args: { feed: feed } };
        return st.file ? { cmd: "laser_play", args: { feed: feed, file: st.file, fps: st.fps || 30, loop: true } } : null;
      // o mesmo play move o transporte do show, quando ha' show carregado
      case "transport":
        return st.show ? { cmd: v ? "resume" : "pause", args: {} } : null;
      // interlock fechado = obturador aberto; `shutter` 1 fecha
      case "lock":
        return feed == null ? null : { cmd: "laser_param", args: { feed: feed, path: "shutter", value: v ? 0 : 1 } };
      case "size":
        return feed == null ? null : { cmd: "laser_param", args: { feed: feed, path: "geo/scale", value: +v } };
      case "lim.r": case "lim.g": case "lim.b":
        return feed == null ? null : { cmd: "laser_param", args: { feed: feed, path: "limit/" + id.slice(4), value: +v } };
      default:
        // ponytail: kpps, gam.*, dmx, buffer, speed, net.* e power ficam so' locais ate' o
        // `laser_param` aceitar dev/pps, curve/r|g|b, dmx/addr ; entao cada um vira mais um case
        // aqui, sem tocar em app.js.
        return null;
    }
  }

  var api = { cmdFor: cmdFor };
  if (typeof window !== "undefined") window.LaserEngine = api;
  if (typeof module !== "undefined") module.exports = api;
})();
