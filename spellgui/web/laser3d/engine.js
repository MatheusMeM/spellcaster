/* laser3d ↔ engine: as tres funcoes puras da pagina do laser — o que cada peca do aparelho e'
   (`CONTROLS`), o mapa estado → comando do registry (`cmdFor`) e o que o display mostra
   (`oledLines`). Nenhuma delas toca DOM, THREE nem rede: e' o que o teste em
   `spellgui/web/test/laser3d.test.js` roda sem navegador.

   `cmdFor(id, value, st)`: `id` e' o caminho do estado do aparelho (o mesmo nome que o binding e o
   slider usam), `value` e' o valor absoluto ja' aplicado em `S`, `st` e' o contexto do feed aberto
   ({feed, dac, host, kpps, file, fps, show}). Devolve `{cmd, args}` do registry, ou `null` quando
   aquele estado nao tem comando hoje — a pagina segue local e o desenho na parede e' o mesmo.
   Quem chama e' `app.js`; quem executa e' `bus.call`. */
(function () {
  "use strict";

  // ---------------------------------------------------------------- CONTROLS
  // Um controle, uma funcao. `[familia, rotulo]` por `userData.key` das pecas de `body.js` e
  // `optics.js`; e' daqui que sai o tooltip e e' daqui que `app.js` decide o que o clique faz.
  // familia: "toggle" (inverte estado) | "momentary" (age enquanto apertado) | "valor" (muda um
  // numero) | "navegacao" (abre a aba daquela peca na gaveta) | "mapear" (a peca e' uma ENTRADA do
  // aparelho: o clique abre onde se escolhe QUEM a aciona) | "peca" (chapa, aleta, plugue,
  // ventoinha: nao acende, nao tem tooltip e o clique nao faz nada — o dono nao quer menu de hover
  // nessas pecas). Nenhuma chave pode ter duas familias, e nenhuma funcao em duas chaves.
  // Rotulo vazio = sem tooltip. O rotulo e' texto de tela: vai acentuado, como o resto da pagina.
  // Sao os nomes das PAGINAS do display (`oledLines`) que ficam em ASCII, porque aquilo e' display.
  var CONTROLS = {
    // painel traseiro — energia e seguranca
    power:     ["toggle", "POWER: liga e desliga o aparelho"],
    keyswitch: ["toggle", "chave: arma a emissão"],
    interlock: ["mapear", "interlock: é uma entrada — clique para mapear quem aciona"],
    acin:      ["peca", ""],
    // painel traseiro — display e navegacao
    enc:       ["navegacao", "encoder: gira navega, aperta entra"],
    back:      ["navegacao", "BACK: volta uma página do display"],
    // painel traseiro — portas
    ilda:      ["navegacao", "ILDA IN: carrega o .ild"],
    ildathru:  ["navegacao", "ILDA OUT: encadeia o próximo projetor"],
    dmxin:     ["navegacao", "DMX IN: endereço e modo"],
    dmxout:    ["navegacao", "DMX OUT: repete o universo"],
    rj45:      ["navegacao", "NET: sACN, Art-Net, NDI, Spout e os DACs da rede"],
    fan:       ["peca", ""],
    // corpo — a tampa continua abrindo no clique (e' navegacao), mas sem tooltip
    lid:       ["navegacao", ""],
    front:     ["peca", ""],
    aperture:  ["peca", ""],
    side:      ["peca", ""],
    // dentro (optics.js)
    bench:     ["peca", ""],
    r:         ["navegacao", "módulo vermelho 638 nm · 2,5 W: limite e curva"],
    g:         ["navegacao", "módulo verde 520 nm · 3 W: limite e curva"],
    b:         ["navegacao", "módulo azul 445 nm · 4,5 W: limite e curva"],
    dichro:    ["peca", ""],
    fold:      ["peca", ""],
    shutter:   ["navegacao", "obturador: fecha sem chave ou sem interlock"],
    galvo:     ["navegacao", "galvos X/Y: kpps"],
    galvodrv:  ["navegacao", "driver dos galvos: buffer e velocidade"],
    pcb:       ["peca", ""],
    dac:       ["peca", ""],
    psu:       ["peca", ""],
    // Pino
    pino:      ["navegacao", "Pino: cabo DMX, cinco pinos, zero paciência"]
  };
  function labelOf(k) { return (CONTROLS[k] && CONTROLS[k][1]) || ""; }
  function kindOf(k) { return (CONTROLS[k] && CONTROLS[k][0]) || ""; }
  // Peca inerte: nao acende, nao tem tooltip, cursor normal, clique nao faz nada. Chave que nao
  // esta' na tabela cai aqui tambem (peca de modelo sem funcao declarada — e' assim que a USB
  // apagada some da interface). Chave com namespace (`pino.<pino>`) e' de outro dono: nao e' peca.
  function inert(k) { return String(k).indexOf(".") < 0 && (kindOf(k) === "" || kindOf(k) === "peca"); }

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

  // -------------------------------------------------------------- oledLines
  // O display do painel traseiro e' o instrumento do aparelho, nao enfeite: sete paginas, e cada
  // uma responde a uma pergunta que o operador faz de longe. `oledLines(S, eng)` e' pura e devolve
  // as linhas ja' prontas: [0] cabecalho, [1] linha grande (a que se le' do outro lado da sala),
  // [2..] ate' quatro linhas de detalhe. Uma linha que comeca com ">" e' o campo selecionado.
  // Tudo em ASCII: e' um display de equipamento, nao uma pagina web.
  var PAGES = ["STATUS", "SHOW", "DMX", "NET", "TEMP/ILK", "ENGINE", "ERRO"];
  // Os campos que o encoder edita em cada pagina, na ordem em que ele passa por eles.
  var FIELDS = { STATUS: ["kpps"], DMX: ["addr", "univ"], NET: ["sacn", "artnet", "ndi", "spout"], "TEMP/ILK": [], SHOW: [], ENGINE: [], ERRO: ["limpar"] };
  function n3(v) { return ("00" + Math.round(v)).slice(-3); }
  function mmss(t) { t = Math.max(0, Math.round(t || 0)); return Math.floor(t / 60) + ":" + ("0" + (t % 60)).slice(-2); }
  function cut(s, n) { s = String(s == null ? "" : s).toUpperCase(); return s.length > n ? s.slice(0, n - 1) + "+" : s; }

  function oledLines(S, eng) {
    eng = eng || {}; S = S || {};
    var page = PAGES[S.page] || PAGES[0], f = FIELDS[page] || [];
    var head = page + "   " + ((S.page || 0) + 1) + "/" + PAGES.length + (S.edit ? "  EDITA" : "");
    var sel = function (i, s) { return (S.edit && S.field === i ? ">" : " ") + s; };
    if (!S.power) return [head, "DESLIGADO", " SEM ALIMENTACAO", " O ROCKER POWER LIGA"];
    var armed = !!(S.power && S.key), live = armed && S.lock;
    var fr = (S.show && S.show[S.frame]) || [], pts = fr.length, fps = pts ? Math.round(S.kpps / pts) : 0;
    var out = [head];
    if (page === "STATUS") {
      out.push(live ? "LIVE" : !S.key ? "DESARMADO" : !S.lock ? "SCAN FAIL" : "STANDBY");
      out.push(" CHAVE " + (S.key ? "ARMADA" : "ABERTA") + "  ILK " + (S.lock ? "OK" : "ABERTO") + "  OBTURADOR " + (live ? "ABERTO" : "FECHADO"));
      out.push(" DAC " + cut(eng.dac || "-", 12) + " " + (eng.feed != null ? "FEED " + eng.feed : eng.err ? "ERRO" : eng.host ? cut(eng.host, 15) : "SEM HOST"));
      out.push(sel(0, "KPPS " + Math.round((S.kpps || 0) / 1000) + "  " + pts + " PTS  " + fps + " FPS"));
    } else if (page === "SHOW") {
      var tr = eng.tr || null;
      out.push(cut(eng.show || "SEM SHOW", 18));
      out.push(" REV " + (eng.rev || 0) + "  " + (tr ? tr.state.toUpperCase() : "SEM PLAYER") + "  T " + mmss(tr && tr.t) + " / " + mmss(tr && tr.duration));
      // ponytail: o evento `transport` nao publica loop ; o unico loop que a pagina conhece e' o
      // do `laser_play --loop`, entao e' esse que aparece. Sai daqui quando o evento trouxer loop.
      out.push(" ILDA " + cut(S.name || "-", 22));
      out.push(" FRAME " + ((S.frame || 0) + 1) + "/" + ((S.show && S.show.length) || 0) + "  " + (S.play ? "PLAY" : "PAUSA") + "  LOOP " + (eng.feed != null && eng.file ? "ON" : "-"));
    } else if (page === "DMX") {
      out.push("ADDR " + n3(S.dmx));
      out.push(sel(0, "ENDERECO " + n3(S.dmx)));
      out.push(sel(1, "UNIVERSO " + (S.univ || 1) + "   MODO 16CH"));
      out.push(" BUS " + (S.dmxIn ? "U" + S.dmxIn.universe + " CH" + n3(S.dmx) + " = " + S.dmxIn.value : "SEM FRAME"));
    } else if (page === "NET") {
      var nets = f, on = nets.filter(function (k) { return S.net && S.net[k]; });
      out.push(on.length ? on.join(" ").toUpperCase() : "TUDO OFF");
      nets.forEach(function (k, i) { if (i < 4) out.push(sel(i, k.toUpperCase() + (S.net && S.net[k] ? "  ON" : "  OFF") + (i === 0 ? "        DAC " + cut(eng.dac || "-", 10) : ""))); });
    } else if (page === "TEMP/ILK") {
      out.push(Math.round(S.temp) + " C");
      out.push(" DIODOS " + Math.round(S.temp) + "C  GALVOS " + Math.round(S.temp - 6) + "C  FONTE " + Math.round(S.temp + 4) + "C");
      out.push(" VENTOINHA " + (S.power ? "GIRANDO" : "PARADA") + "   DESLIGA A 65 C");
      out.push(" INTERLOCK " + (S.lock ? "FECHADO" : "ABERTO: SCAN FAIL"));
    } else if (page === "ENGINE") {
      out.push(eng.on ? "OK  REV " + (eng.rev || 0) : "OFFLINE");
      out.push(" PORTA " + cut(eng.port || "-", 22) + "   SPELL " + cut(eng.ver || "0.1.2", 8));
      out.push(" FEED " + (eng.feed != null ? eng.feed + "  " + ((eng.stats && eng.stats["stat/sent"]) || 0) + " PT" : "FECHADO") + "   DAC " + cut(eng.dac || "-", 10));
      out.push(" LOG " + cut(eng.log || "-", 30));
    } else {
      out.push(S.err ? "ERRO" : "SEM ERRO");
      if (S.err) { out.push(" " + cut(S.err.msg, 40)); if (S.err.msg.length > 40) out.push(" " + cut(S.err.msg.slice(39), 40)); out.push(" " + S.err.when); }
      else out.push(" NENHUM ERRO DESDE QUE LIGOU");
      out.push(sel(0, "LIMPAR"));
    }
    return out;
  }

  var api = { cmdFor: cmdFor, CONTROLS: CONTROLS, labelOf: labelOf, kindOf: kindOf, inert: inert, PAGES: PAGES, FIELDS: FIELDS, oledLines: oledLines };
  if (typeof window !== "undefined") window.LaserEngine = api;
  if (typeof module !== "undefined") module.exports = api;
})();
