/* Menu de VÍDEO do visualizador: a TABELA das opções, e só ela. Sem three.js aqui dentro.
   Cada linha é `{id, g, label, type, ...}` e o valor de cada predefinição. A aba VÍDEO da gaveta se
   desenha a partir desta tabela, `app.js` traduz cada `id` para o three.js no `applyVideo`, e o
   `Bind.def("video.<id>")` sai do mesmo lugar — a identidade de cada opção é o endereço textual
   `video.<id>` (FUNCOES/README regra 2), o mesmo em tecla, MIDI e localStorage.
   O nome da predefinição não é guardado: ele é DERIVADO dos valores (`nameOf`), então mexer numa
   linha vira PERSONALIZADO sozinho e voltar o valor na mão volta ao nome da predefinição. */
window.VIDEO = (function () {
  "use strict";
  var KEY = "sc-laser-video", PRESETS = ["baixo", "medio", "alto", "ultra"];
  var NOME = { baixo: "BAIXO", medio: "MÉDIO", alto: "ALTO", ultra: "ULTRA", custom: "PERSONALIZADO" };
  var SIMNAO = [["nao", "NÃO"], ["sim", "SIM"]];
  function sel(id, g, label, vals, b, m, a, u, note) { return { id: id, g: g, label: label, type: "select", vals: vals, p: { baixo: b, medio: m, alto: a, ultra: u }, note: note }; }
  function rng(id, g, label, min, max, step, unit, b, m, a, u, note) { return { id: id, g: g, label: label, type: "range", min: min, max: max, step: step, unit: unit || "", p: { baixo: b, medio: m, alto: a, ultra: u }, note: note }; }
  function bool(id, g, label, b, m, a, u, note) { return sel(id, g, label, SIMNAO, b, m, a, u, note); }

  var GRUPOS = ["TELA", "QUALIDADE", "PÓS-PROCESSAMENTO", "LASER", "CENA"];
  var SUB = {
    TELA: "quantos pixels o programa desenha, e quantas vezes por segundo",
    QUALIDADE: "serrilhado, sombra e textura · é aqui que a GPU sofre",
    "PÓS-PROCESSAMENTO": "o que o composer faz depois da cena, em tela cheia",
    LASER: "o visualizador em si: a parede, o rastro, os feixes e a névoa",
    CENA: "o aparelho, o cabo do Pino e o que se mexe sozinho"
  };
  /* A tabela. Ordem = ordem na tela. Nada aqui é decorativo: cada id tem um caso no `applyVideo`. */
  var OPTS = [
    sel("escala", "TELA", "ESCALA DE RESOLUÇÃO", [[".5", "50 %"], [".75", "75 %"], ["1", "100 %"], ["1.5", "150 %"], ["2", "200 %"]], ".5", ".75", "1", "2",
      "% da resolução nativa; 100 % = devicePixelRatio, com teto 2×"),
    rng("fov", "TELA", "CAMPO DE VISÃO", 30, 90, 1, "°", 42, 42, 42, 42, "só a vista SHOW é livre; as fixas se reenquadram"),
    sel("fpsMax", "TELA", "LIMITE DE FPS", [["30", "30"], ["60", "60"], ["120", "120"], ["0", "ILIMITADO"]], "0", "0", "0", "0"),
    bool("hudFps", "TELA", "CONTADOR NA HUD", "nao", "nao", "nao", "nao"),

    sel("aa", "QUALIDADE", "ANTI-ALIASING", [["off", "DESLIGADO"], ["fxaa", "FXAA"], ["msaa4", "MSAA 4×"]], "off", "fxaa", "fxaa", "msaa4",
      "MSAA = alvo multiamostrado no composer (exige WebGL 2); FXAA age no resultado do composer"),
    sel("sombras", "QUALIDADE", "SOMBRAS", [["0", "DESLIGADAS"], ["1024", "1024"], ["2048", "2048"], ["4096", "4096"]], "0", "1024", "2048", "4096"),
    sel("sombraTipo", "QUALIDADE", "FILTRO DA SOMBRA", [["basic", "BÁSICO"], ["pcf", "PCF"], ["pcfsoft", "PCF SUAVE"], ["vsm", "VSM"]], "basic", "pcf", "pcfsoft", "vsm"),
    sel("aniso", "QUALIDADE", "ANISOTROPIA", [["1", "1×"], ["2", "2×"], ["4", "4×"], ["8", "8×"], ["16", "16×"]], "1", "4", "8", "16", "limitado pelo máximo da GPU"),
    bool("reflexo", "QUALIDADE", "REFLEXOS DO AMBIENTE", "nao", "sim", "sim", "sim"),
    rng("reflexoInt", "QUALIDADE", "INTENSIDADE DO REFLEXO", 0, 2, .05, "×", 1, 1, 1, 1),

    bool("bloom", "PÓS-PROCESSAMENTO", "BLOOM", "nao", "sim", "sim", "sim"),
    rng("bloomForca", "PÓS-PROCESSAMENTO", "FORÇA DO BLOOM", 0, 2, .05, "", .5, .5, .5, .5),
    rng("bloomRaio", "PÓS-PROCESSAMENTO", "RAIO DO BLOOM", 0, 1, .01, "", .45, .45, .45, .45),
    rng("bloomLimiar", "PÓS-PROCESSAMENTO", "LIMIAR DO BLOOM", 0, 1, .01, "", .82, .82, .82, .82),
    sel("bloomRes", "PÓS-PROCESSAMENTO", "RESOLUÇÃO DO BLOOM", [[".25", "¼ DA TELA"], [".5", "½ DA TELA"], ["1", "1× DA TELA"]], ".25", ".5", ".5", "1"),
    sel("tone", "PÓS-PROCESSAMENTO", "TONE MAPPING", [["nenhum", "NENHUM"], ["linear", "LINEAR"], ["reinhard", "REINHARD"], ["cineon", "CINEON"], ["aces", "ACES"]], "aces", "aces", "aces", "aces"),
    rng("exposicao", "PÓS-PROCESSAMENTO", "EXPOSIÇÃO", .2, 3, .05, "", .95, .95, .95, .95),

    sel("parede", "LASER", "RESOLUÇÃO DA PAREDE", [["512x320", "512 × 320"], ["1024x640", "1024 × 640"], ["2048x1280", "2048 × 1280"]], "512x320", "1024x640", "1024x640", "2048x1280"),
    rng("rastro", "LASER", "PERSISTÊNCIA DO RASTRO", .3, .95, .01, "", .7, .7, .7, .7, "quanto sobra do quadro anterior a cada 1/60 s"),
    rng("halo", "LASER", "HALO DO TRAÇO", 0, .4, .01, "", 0, .12, .12, .2),
    rng("haloPx", "LASER", "TAMANHO DO HALO", 4, 16, 1, "px", 9, 9, 9, 12),
    sel("feixes", "LASER", "FEIXES EXTERNOS", [["40", "40"], ["80", "80"], ["160", "160"], ["320", "320"]], "40", "80", "160", "320"),
    bool("poeira", "LASER", "POEIRA NO FEIXE", "nao", "sim", "sim", "sim"),
    rng("nevoa", "LASER", "NÉVOA", 0, 1, .01, "", .85, .85, .85, .85),
    sel("puffs", "LASER", "PARTÍCULAS DE NÉVOA", [["0", "NENHUMA"], ["14", "14"], ["28", "28"]], "0", "14", "14", "28"),

    bool("corda", "CENA", "CABO DO PINO (VERLET)", "nao", "sim", "sim", "sim"),
    bool("movimento", "CENA", "ANIMAÇÕES (VENTOINHA, LED)", "sim", "sim", "sim", "sim", "desligado por padrão com prefers-reduced-motion")
  ];
  var byId = {}; OPTS.forEach(function (o) { byId[o.id] = o; });

  var V = {}, onChange = function () {};
  /// Validação na entrada: fora de faixa ou fora da lista é RECUSADO (devolve null), não clampeado —
  /// valor que o menu não oferece não pode virar estado, venha do localStorage ou do MIDI.
  function ok(o, v) {
    if (!o || v === undefined || v === null) return null;
    if (o.type === "range") { var n = +v; return (typeof v !== "boolean" && isFinite(n) && n >= o.min && n <= o.max) ? n : null; }
    var s = String(v); for (var i = 0; i < o.vals.length; i++) if (o.vals[i][0] === s) return s;
    return null;
  }
  function apply(name) { OPTS.forEach(function (o) { V[o.id] = o.p[name]; }); }
  /// O nome da predefinição é derivado, nunca guardado: se todos os valores batem com uma
  /// predefinição, é ela; senão é PERSONALIZADO.
  function nameOf() {
    for (var i = 0; i < PRESETS.length; i++) { var p = PRESETS[i], all = true;
      for (var j = 0; j < OPTS.length; j++) if (String(V[OPTS[j].id]) !== String(OPTS[j].p[p])) { all = false; break; }
      if (all) return p; }
    return "custom";
  }
  function reduced() { try { return typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches; } catch (e) { return false; } }
  // máquina fraca começa em MÉDIO: quatro núcleos ou menos é quase sempre laptop de escritório
  function defaultPreset() { try { return (navigator.hardwareConcurrency || 8) <= 4 ? "medio" : "alto"; } catch (e) { return "alto"; } }
  function save() { try { localStorage.setItem(KEY, JSON.stringify(V)); } catch (e) {} }
  /// JSON quebrado, chave desconhecida ou valor fora de faixa não derrubam nada: cada linha que
  /// não valida fica com o padrão. Uma linha ruim não custa o menu inteiro.
  function load() {
    apply(defaultPreset());
    if (reduced()) { V.movimento = "nao"; V.rastro = .4; }   // era o `reduced` solto do app.js
    var raw = null, d = null;
    try { raw = localStorage.getItem(KEY); } catch (e) {}
    if (raw) { try { d = JSON.parse(raw); } catch (e) { d = null; } }
    if (d && typeof d === "object") OPTS.forEach(function (o) { var c = ok(o, d[o.id]); if (c !== null) V[o.id] = c; });
    return V;
  }
  function set(id, v) { var o = byId[id], c = ok(o, v); if (c === null) return false;
    if (String(V[id]) === String(c)) return true;
    V[id] = c; save(); onChange(id); return true; }
  function preset(name) { if (PRESETS.indexOf(name) < 0) return false; apply(name); save(); onChange(null); return true; }

  var api = {
    OPTS: OPTS, GRUPOS: GRUPOS, SUB: SUB, PRESETS: PRESETS, NOME: NOME,
    opt: function (id) { return byId[id]; },
    load: load, save: save, reset: function () { return preset(defaultPreset()); },
    get: function (id) { return V[id]; },
    set: set, preset: preset, presetName: nameOf, defaultPreset: defaultPreset,
    onChange: function (f) { onChange = f || function () {}; }
  };
  load();
  return api;
})();
if (typeof module !== "undefined") module.exports = window.VIDEO;
