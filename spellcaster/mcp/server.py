# Servidor MCP (Model Context Protocol) do Spellcaster.
#
# Metodos: initialize, notifications/initialized, ping, tools/list, tools/call,
#          resources/list, resources/read, prompts/list, prompts/get.
# Transportes: stdio (uma linha JSON por mensagem, stderr para log) e HTTP streamable
#          (POST /mcp com JSON-RPC -> JSON; GET /mcp -> SSE so com notificacoes).
#
# As tools saem do registry: nada de logica de produto aqui (mesmo contrato da GUI).
# Entradas: `spell mcp [--transport http] [--port 8765]` ou `python -m spellcaster.mcp.server`.
import contextlib
import io
import json
import sys
import threading
import time
from collections import deque
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse

from .. import __version__
from ..core import registry
from ..core.registry import command
from . import tools as spelltools

PROTOCOL = "2025-06-18"
SUPPORTED = ("2026-07-28", "2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05")
# ponytail: sem negociacao real, so eco da versao pedida quando conhecida ; implementar
# server/discover e o _meta de 2026-07-28 quando algum cliente exigir.

# ponytail: o decorador ja aceita mcp=False, mas marcar comando por comando obrigaria a
# editar player/fixtures/gui/cli ; a lista mora aqui ate o proximo passo nesses arquivos.
HIGH_LEVEL = {"play_show", "stop", "pause", "locate", "net", "markers", "patch_list",
              "show_summary", "monitor"}

# Comandos que bloqueiam ate Ctrl+C: rodam em thread e a tool volta na hora.
BACKGROUND = {"play_show", "play", "serve", "mcp", "calib_hold", "calib_sweep"}

_JSON_TYPE = {"str": "string", "int": "integer", "float": "number", "bool": "boolean"}
LOG = deque(maxlen=200)            # resource spell://log
_SSE = set()                       # wfile dos clientes SSE do transporte HTTP

INSTRUCTIONS = ("Spellcaster: show control (sACN, Art-Net, OSC, laser ILDA). Ordem util: `net` para achar "
                "os nos da rede, `show_summary` para ler o show aberto, `play_show` para tocar, "
                "`monitor` para conferir os 512 bytes de um universo. Comandos fora da lista de tools "
                "ficam em `run_command`; os resources spell://show, spell://patch, spell://net e "
                "spell://log dao o estado atual sem gastar uma chamada de tool.")


class MethodNotFound(Exception):
    pass


def log(msg):
    """Log em stderr (stdout e o canal JSON-RPC) e no buffer do resource spell://log."""
    line = time.strftime("%H:%M:%S ") + str(msg)
    LOG.append(line)
    print(line, file=sys.stderr, flush=True)


# ---------------------------------------------------------------- tools
def tool_list():
    """Tools MCP a partir do registry: uma por comando de HIGH_LEVEL + o generico run_command."""
    out, resto = [], []
    for c in registry.schema():
        if c["name"] not in HIGH_LEVEL:
            resto.append(c["name"])
            continue
        props, req = {}, []
        for p in c["params"]:
            s = props[p["name"]] = {"type": _JSON_TYPE.get(p["type"], "string")}
            if p["default"] is None:
                req.append(p["name"])
            else:
                s["default"] = p["default"]
        out.append({"name": c["name"], "description": c["doc"] or c["name"],
                    "inputSchema": {"type": "object", "properties": props, "required": req}})
    out.append({
        "name": "run_command",
        "description": "Roda qualquer comando de baixo nivel do registry. Disponiveis: "
                       + ", ".join(sorted(resto)) + ". Use `commands` para ver os parametros de cada um.",
        "inputSchema": {"type": "object",
                        "properties": {"name": {"type": "string", "description": "nome do comando"},
                                       "args": {"type": "object", "description": "argumentos por nome"}},
                        "required": ["name"]}})
    return out


def call_tool(name, args):
    """Executa a tool e devolve texto. stdout do comando e capturado (no stdio ele corromperia o JSON-RPC)."""
    if name == "run_command":
        name, args = args.get("name", ""), dict(args.get("args") or {})
    if name not in registry.REGISTRY:
        raise KeyError(f"comando {name!r} nao existe")
    log(f"tool {name} {args}")
    if name in BACKGROUND:
        threading.Thread(target=registry.call, args=(name,), kwargs=args, daemon=True).start()
        return f"{name} iniciado em background; use stop/pause/monitor para acompanhar"
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        r = registry.call(name, **args)
    # ponytail: quando o comando imprime, o texto impresso e a resposta e o valor de retorno e
    # descartado (os dois dizem a mesma coisa hoje) ; devolver os dois se algum comando divergir.
    return buf.getvalue().strip() or json.dumps(r, ensure_ascii=False, default=str)


# ---------------------------------------------------------------- resources
RESOURCES = [
    {"uri": "spell://show", "name": "show", "title": "Show aberto",
     "description": "Resumo do .spell aberto: fps, duracao, saidas, tracks, cues, patch.",
     "mimeType": "application/json"},
    {"uri": "spell://patch", "name": "patch", "title": "Patch",
     "description": "Fixtures do patch corrente: nome, perfil, universo, endereco, canais.",
     "mimeType": "application/json"},
    {"uri": "spell://net", "name": "net", "title": "Rede",
     "description": "Ultimo scan de rede: interfaces, nos Art-Net, fontes sACN, DACs Ether Dream.",
     "mimeType": "application/json"},
    {"uri": "spell://log", "name": "log", "title": "Log",
     "description": "Ultimas 200 linhas de log do servidor MCP.", "mimeType": "text/plain"},
]


def read_resource(uri):
    if uri == "spell://show":
        return json.dumps(spelltools.summary(), indent=1, ensure_ascii=False, default=str)
    if uri == "spell://patch":
        from ..fixtures import patch as fxpatch
        rows = fxpatch.PATCH.rows() if fxpatch.PATCH is not None else []
        return json.dumps(rows, indent=1, ensure_ascii=False)
    if uri == "spell://net":
        # ponytail: sem scan guardado, faz um curto na hora ; o `net` da CLI so imprime, nao cacheia.
        d = spelltools.LAST_NET or spelltools.net_json(1)
        return json.dumps(d, indent=1, ensure_ascii=False, default=str)
    if uri == "spell://log":
        return "\n".join(LOG)
    raise KeyError(f"resource {uri!r} nao existe")


def _mime(uri):
    return next((r["mimeType"] for r in RESOURCES if r["uri"] == uri), "text/plain")


# ---------------------------------------------------------------- prompts
PROMPTS = [
    {"name": "montar_show_do_video", "title": "Montar show a partir de um video",
     "description": "Passo a passo para transformar os cortes de um video em cues de um show.",
     "arguments": [{"name": "video", "description": "caminho do video ou .wav", "required": True}]},
    {"name": "calibrar_grupo", "title": "Calibrar um grupo de moving heads",
     "description": "Passo a passo para achar pan/tilt, roda de cor e gobo de um grupo.",
     "arguments": [{"name": "grupo", "description": "nome do grupo no patch", "required": True}]},
]

_P_VIDEO = """Monte um show do Spellcaster a partir do video {video}. Siga nesta ordem:

1. `net` (timeout 2) para ver interfaces, nos Art-Net e fontes sACN; confirme comigo a interface de saida.
2. `patch_list` para ver as fixtures ja patcheadas. Se estiver vazio, pergunte quais fixtures existem
   antes de continuar; nao invente perfil nem endereco.
3. `markers` com video={video} para pegar a lista de cortes de cena (segundos).
4. Monte o .spell: uma cue por corte, fade curto (0.2 s) em corte seco e fade longo em transicao lenta.
   Use `run_command` para os comandos de baixo nivel que faltarem, e `commands` para ver a assinatura deles.
5. `show_summary` para conferir tracks, cues e duracao antes de tocar.
6. `play_show` e, logo depois, `monitor` no universo principal para confirmar que esta saindo DMX.
7. `stop` ao terminar. Nunca deixe o show tocando sem me avisar."""

_P_GRUPO = """Calibre o grupo {grupo} do patch corrente. Siga nesta ordem:

1. `patch_list` e o resource spell://patch para ver quais unidades formam o grupo {grupo} e qual perfil usam.
2. `net` para confirmar que a saida esta na interface certa antes de mandar luz.
3. Pan/tilt: `run_command` com name=calib_hold e args {{"grupo": "{grupo}", "pan": ..., "tilt": ...}}.
   Va por bissecao, um eixo de cada vez, e me pergunte o que aparece no palco a cada passo.
4. Roda de cor e gobo: `run_command` com name=calib_sweep, channel=color (depois gobo) e um valor por
   unidade, ex. values="0,64,128,255". Anote qual valor deu qual cor/gobo.
5. `monitor` no universo do grupo para confirmar os bytes que estao saindo.
6. Feche com um resumo em tabela: unidade, pan, tilt, valores de cor e de gobo. Nao grave nada
   no .spell sem eu confirmar."""


def get_prompt(name, args):
    if name == "montar_show_do_video":
        txt = _P_VIDEO.format(video=args.get("video", "<video>"))
    elif name == "calibrar_grupo":
        txt = _P_GRUPO.format(grupo=args.get("grupo", "<grupo>"))
    else:
        raise KeyError(f"prompt {name!r} nao existe")
    d = next(p for p in PROMPTS if p["name"] == name)
    return {"description": d["description"],
            "messages": [{"role": "user", "content": {"type": "text", "text": txt}}]}


# ---------------------------------------------------------------- JSON-RPC
def method(name, p):
    if name == "initialize":
        v = p.get("protocolVersion")
        return {"protocolVersion": v if v in SUPPORTED else PROTOCOL,
                "capabilities": {"tools": {}, "resources": {}, "prompts": {}, "logging": {}},
                "serverInfo": {"name": "spellcaster", "title": "Spellcaster", "version": __version__},
                "instructions": INSTRUCTIONS}
    if name == "ping":
        return {}
    if name == "tools/list":
        return {"tools": tool_list()}
    if name == "tools/call":
        try:
            txt = call_tool(p.get("name", ""), dict(p.get("arguments") or {}))
        except Exception as e:                       # erro de execucao vai no resultado, nao no JSON-RPC
            log(f"tools/call: {type(e).__name__}: {e}")
            return {"content": [{"type": "text", "text": f"{type(e).__name__}: {e}"}], "isError": True}
        return {"content": [{"type": "text", "text": txt}], "isError": False}
    if name == "resources/list":
        return {"resources": RESOURCES}
    if name == "resources/read":
        uri = p.get("uri", "")
        return {"contents": [{"uri": uri, "mimeType": _mime(uri), "text": read_resource(uri)}]}
    if name == "prompts/list":
        return {"prompts": PROMPTS}
    if name == "prompts/get":
        return get_prompt(p.get("name", ""), dict(p.get("arguments") or {}))
    raise MethodNotFound(name)


def handle(m):
    """Uma mensagem JSON-RPC -> resposta (dict) ou None (notificacao / resposta do cliente)."""
    mid = m.get("id")
    name = m.get("method")
    if name is None:                                  # resposta do cliente a uma request nossa: ignorada
        return None
    if name.startswith("notifications/"):
        log(f"<- {name}")
        return None
    try:
        r = method(name, dict(m.get("params") or {}))
    except MethodNotFound:
        return None if mid is None else _err(mid, -32601, f"metodo desconhecido: {name}")
    except KeyError as e:
        return None if mid is None else _err(mid, -32602, str(e))
    except Exception as e:
        log(f"{name}: {type(e).__name__}: {e}")
        return None if mid is None else _err(mid, -32603, f"{type(e).__name__}: {e}")
    return None if mid is None else {"jsonrpc": "2.0", "id": mid, "result": r}


def _err(mid, code, msg):
    return {"jsonrpc": "2.0", "id": mid, "error": {"code": code, "message": msg}}


def notification(level, data):
    return {"jsonrpc": "2.0", "method": "notifications/message",
            "params": {"level": level, "logger": "spellcaster", "data": data}}


# ---------------------------------------------------------------- eventos do player
def watch(send, period=0.5, stop=None):
    """ponytail: o Player nao tem hooks de evento ; polling do estado a 2 Hz nesta thread.
    Trocar por callback quando Player expuser on_cue/on_end."""
    from ..player import player as pl
    last = None
    while stop is None or not stop.is_set():
        time.sleep(period)
        try:
            p = pl.CURRENT
            now = (p is not None, "" if p is None else p.clock.state, -1 if p is None else p.cues.index)
            if last is None or now == last:
                last = now
                continue
            if last[0] and not now[0]:
                send(notification("info", {"event": "show_end"}))
            elif now[2] != last[2] and now[2] >= 0:
                c = pl.CURRENT.cues.cues[now[2]]
                send(notification("info", {"event": "cue", "index": now[2], "name": c.name,
                                           "t": round(pl.CURRENT.clock.time, 3)}))
            elif now[1] != last[1]:
                send(notification("info", {"event": "transport", "state": now[1]}))
            last = now
        except Exception as e:                        # o watcher nunca derruba o servidor
            send(notification("error", {"event": "error", "text": f"{type(e).__name__}: {e}"}))
            last = None


# ---------------------------------------------------------------- transporte stdio
def serve_stdio(inp=None, out=None):
    """Uma linha JSON por mensagem em stdin/stdout; log em stderr. Volta em EOF."""
    if inp is None:
        inp = sys.stdin
    if out is None:
        out = sys.stdout
        with contextlib.suppress(Exception):
            out.reconfigure(encoding="utf-8", newline="\n")
    lock = threading.Lock()

    def send(msg):
        with lock:                                    # ensure_ascii: console cp1252 nao quebra o canal
            out.write(json.dumps(msg, default=str) + "\n")
            out.flush()

    threading.Thread(target=watch, args=(send,), daemon=True).start()
    log("MCP stdio pronto")
    for line in inp:
        line = line.strip()
        if not line:
            continue
        try:
            m = json.loads(line)
        except ValueError as e:
            send(_err(None, -32700, f"parse error: {e}"))
            continue
        r = handle(m)
        if r is not None:
            send(r)
    log("MCP stdio: EOF")


# ---------------------------------------------------------------- transporte HTTP streamable
def _local_origin(origin):
    """Origin ausente (cliente nao-browser) ou de loopback; barra DNS rebinding."""
    return not origin or urlparse(origin).hostname in ("127.0.0.1", "localhost", "::1")


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *a):
        pass

    def _send(self, code, body=b"", ctype="application/json"):
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        if body:
            self.wfile.write(body)

    def do_POST(self):
        if self.path.split("?")[0] != "/mcp":
            return self.send_error(404)
        if not _local_origin(self.headers.get("Origin")):
            return self.send_error(403, "Origin nao e loopback")
        n = int(self.headers.get("Content-Length") or 0)
        try:
            m = json.loads(self.rfile.read(n) or b"{}")
        except ValueError as e:
            return self._send(400, json.dumps(_err(None, -32700, f"parse error: {e}")).encode())
        r = handle(m)
        if r is None:                                 # notificacao aceita
            return self._send(202)
        self._send(200, json.dumps(r, default=str).encode())

    def do_GET(self):
        if self.path.split("?")[0] != "/mcp" or "text/event-stream" not in (self.headers.get("Accept") or ""):
            return self.send_error(405)
        if not _local_origin(self.headers.get("Origin")):
            return self.send_error(403, "Origin nao e loopback")
        self.close_connection = True                  # SSE: stream ate o cliente fechar
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        _SSE.add(self.wfile)
        try:
            while True:
                time.sleep(1)
                self.wfile.write(b": keepalive\n\n")
                self.wfile.flush()
        except OSError:
            pass
        finally:
            _SSE.discard(self.wfile)
    # ponytail: sem Mcp-Session-Id, sem resumo por Last-Event-ID e sem SSE na resposta do POST
    # ; um POST = um JSON. Adicionar sessao quando houver mais de um cliente por servidor.


def _sse_push(msg):
    data = ("data: " + json.dumps(msg, default=str) + "\n\n").encode()
    for w in list(_SSE):
        try:
            w.write(data)
            w.flush()
        except OSError:
            _SSE.discard(w)


def http_server(port=8765, host="127.0.0.1"):
    """ThreadingHTTPServer com /mcp; use .serve_forever() ou serve_http()."""
    return ThreadingHTTPServer((host, port), Handler)


def serve_http(port=8765, host="127.0.0.1"):
    srv = http_server(port, host)
    threading.Thread(target=watch, args=(_sse_push,), daemon=True).start()
    print(f"MCP HTTP em http://{host}:{srv.server_address[1]}/mcp  (Ctrl+C encerra)", flush=True)
    try:
        srv.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        srv.server_close()
    return srv


@command
def mcp(transport: str = "stdio", port: int = 8765):
    """Servidor MCP: transport stdio (Claude Desktop/Code) ou http (POST /mcp, SSE no GET /mcp)."""
    if transport == "stdio":
        return serve_stdio()
    if transport == "http":
        return serve_http(int(port))
    raise ValueError(f"transport {transport!r}: use stdio ou http")


if __name__ == "__main__":
    from .. import cli  # noqa: F401   (importar a CLI registra todos os comandos no registry)
    serve_stdio()
