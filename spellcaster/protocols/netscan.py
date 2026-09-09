"""Analise de rede: interfaces, nos Art-Net (ArtPoll), fontes sACN (discovery), Ether Dream (beacon), sugestoes.
Os pacotes sao decodificados por artnet.parse e sacn.parse: aqui so mora a varredura.
Uso: python -m spellcaster.protocols.netscan [--json] [--timeout N]"""
import argparse
import json
import platform
import re
import socket
import struct
import subprocess
import sys
import threading
import time

from . import artnet, sacn
from .ilda.etherdream import BEACON, BEACON_PORT, STATUS, parse_beacon

ARTNET_PORT = artnet.PORT
ARTPOLL = artnet.artpoll(flags=0, priority=0)   # TalkToMe 0, prioridade 0: so quero o ArtPollReply
SACN_PORT, SACN_DISCOVERY = sacn.PORT, sacn.DISCOVERY_IP
IPV4 = r"(\d{1,3}(?:\.\d{1,3}){3})"
WINDOWS = platform.system() == "Windows"


# ---------------------------------------------------------------- interfaces
def _mask(prefix):
    return socket.inet_ntoa(struct.pack("!I", (0xFFFFFFFF << (32 - prefix)) & 0xFFFFFFFF))


def _net(ip, mask):
    a, m = struct.unpack("!I", socket.inet_aton(ip))[0], struct.unpack("!I", socket.inet_aton(mask))[0]
    return socket.inet_ntoa(struct.pack("!I", a & m))


def _bcast(ip, mask):
    a, m = struct.unpack("!I", socket.inet_aton(ip))[0], struct.unpack("!I", socket.inet_aton(mask))[0]
    return socket.inet_ntoa(struct.pack("!I", a | (~m & 0xFFFFFFFF)))


_HEADER = re.compile(r"^.*?\b(?:\w+ adapter|Adaptador (?:de Rede sem Fio|de T\S+|\S+))\s+(.+?):\s*$")


def parse_ipconfig(text):
    """Saida de `ipconfig` (pt-BR ou en, acentos podem vir quebrados) -> lista de interfaces com IPv4."""
    ifaces, cur, want_gw = [], None, False
    for line in text.splitlines():
        if line and not line[0].isspace() and line.rstrip().endswith(":"):
            m = _HEADER.match(line)
            cur = {"name": m.group(1) if m else line.strip()[:-1], "ip": None, "mask": None, "gateway": None}
            ifaces.append(cur)
            want_gw = False
            continue
        if cur is None:
            continue
        if want_gw and re.fullmatch(r"\s*" + IPV4 + r"\s*", line):  # gateway IPv4 na linha seguinte ao IPv6
            cur["gateway"] = line.strip()
            want_gw = False
            continue
        if ":" in line:
            want_gw = False
        if m := re.search(r"IPv4[^:]*:\s*" + IPV4, line):
            cur["ip"] = m.group(1)
        elif m := re.search(r"(?:M.scara|Mask)[^:]*:\s*" + IPV4, line):
            cur["mask"] = m.group(1)
        elif m := re.search(r"Gateway[^:]*:\s*(?:" + IPV4 + ")?", line):
            cur["gateway"] = m.group(1)
            want_gw = m.group(1) is None
    return [i for i in ifaces if i["ip"]]


def parse_ip_addr(text, route_text=""):
    """Fallback Linux: saida de `ip addr` (+ `ip route` para o gateway)."""
    ifaces, cur = [], None
    for line in text.splitlines():
        if m := re.match(r"^\d+:\s+([^:@]+)", line):
            cur = m.group(1)
        elif cur and (m := re.search(r"^\s+inet " + IPV4 + r"/(\d+)", line)):
            ifaces.append({"name": cur, "ip": m.group(1), "mask": _mask(int(m.group(2))), "gateway": None})
    for m in re.finditer(r"default via " + IPV4 + r" dev (\S+)", route_text):
        for i in ifaces:
            if i["name"] == m.group(2):
                i["gateway"] = m.group(1)
    return ifaces


def _run(*cmd):
    out = subprocess.run(cmd, capture_output=True).stdout
    try:
        return out.decode("utf-8")
    except UnicodeDecodeError:
        return out.decode("cp1252", "replace")


def interfaces():
    """[{name, ip, mask, gateway}] das interfaces IPv4 ativas (sem loopback)."""
    if WINDOWS:
        ifaces = parse_ipconfig(_run("ipconfig"))
    else:
        try:
            gws = {r.get("dev"): r.get("gateway") for r in json.loads(_run("ip", "-j", "route")) if r.get("dst") == "default"}
            ifaces = [{"name": d["ifname"], "ip": a["local"], "mask": _mask(a["prefixlen"]), "gateway": gws.get(d["ifname"])}
                      for d in json.loads(_run("ip", "-j", "addr")) for a in d.get("addr_info", []) if a.get("family") == "inet"]
        except (ValueError, KeyError, OSError):
            ifaces = parse_ip_addr(_run("ip", "addr"), _run("ip", "route"))
    return [i for i in ifaces if not i["ip"].startswith("127.")]


# ---------------------------------------------------------------- Art-Net
def scan_artnet(timeout=2, ifaces=None):
    """ArtPoll em broadcast (global + 2.x + 10.x + subrede de cada interface) e coleta ArtPollReply."""
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
    try:
        s.bind(("", ARTNET_PORT))  # replies vem em broadcast na 6454
    except OSError:
        s.bind(("", 0))  # porta ocupada: alguns nos respondem unicast ao remetente
    targets = {"255.255.255.255", "2.255.255.255", "10.255.255.255"}
    targets |= {_bcast(i["ip"], i["mask"]) for i in (ifaces or []) if i["mask"]}
    for dst in targets:
        try:
            s.sendto(ARTPOLL, (dst, ARTNET_PORT))
        except OSError:
            pass  # sem rota para essa subrede
    found, end = {}, time.monotonic() + timeout
    while (left := end - time.monotonic()) > 0:
        s.settimeout(left)
        try:
            data, addr = s.recvfrom(1024)
        except (socket.timeout, OSError):
            break
        r = artnet.parse(data)
        if r and r["op"] == "ArtPollReply" and r["ip"] not in found:
            r["from"] = addr[0]
            found[r["ip"]] = r
    s.close()
    return list(found.values())


# ---------------------------------------------------------------- sACN
def scan_sacn(timeout=3, ifaces=None):
    """Entra no multicast de discovery (239.255.250.214:5568) e lista fontes e universos anunciados."""
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("", SACN_PORT))
    for ip in [i["ip"] for i in (ifaces or [])] or ["0.0.0.0"]:
        try:
            s.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, socket.inet_aton(SACN_DISCOVERY) + socket.inet_aton(ip))
        except OSError:
            pass
    found, end = {}, time.monotonic() + timeout
    while (left := end - time.monotonic()) > 0:
        s.settimeout(left)
        try:
            data, addr = s.recvfrom(2048)
        except (socket.timeout, OSError):
            break
        r = sacn.parse(data)
        if r and r["kind"] == "discovery":
            cid = r["cid"].hex()
            src = found.setdefault(cid, {"cid": cid, "ip": addr[0], "source_name": r["name"], "universes": []})
            src["universes"] = sorted(set(src["universes"]) | set(r["universes"]))
    s.close()
    return list(found.values())


# ---------------------------------------------------------------- Ether Dream
def scan_etherdream(timeout=2):
    """Escuta beacons UDP 7654 (1 Hz por DAC)."""
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
    s.bind(("", BEACON_PORT))
    found, end = {}, time.monotonic() + timeout
    while (left := end - time.monotonic()) > 0:
        s.settimeout(left)
        try:
            data, addr = s.recvfrom(256)
        except (socket.timeout, OSError):
            break
        if len(data) >= BEACON.size + STATUS.size:
            d = parse_beacon(data)
            d["ip"] = addr[0]
            found[d["mac"]] = d
    s.close()
    return list(found.values())


# ---------------------------------------------------------------- sugestoes
def suggest(ifaces, windows=WINDOWS):
    """Regras: Art-Net prefere 2.x/8 ou 10.x/8; sACN qualquer; aviso de interfaces na mesma subrede.
    No Windows inclui o comando netsh pronto (texto; nao executa)."""
    out = []
    if not ifaces:
        return ["Nenhuma interface IPv4 ativa: conecte o cabo ou fixe um IP."]
    nets = {}
    for i in ifaces:
        name, ip, mask = i["name"], i["ip"], i["mask"] or "255.255.255.0"
        nets.setdefault(_net(ip, mask), []).append(name)
        if ip.split(".")[0] in ("2", "10"):
            out.append(f"{name} {ip}/{mask}: Art-Net ok (rede {ip.split('.')[0]}.x.x.x), sACN ok.")
        else:
            out.append(f"{name} {ip}/{mask}: sACN ok; Art-Net prefere 2.x.x.x/8 ou 10.x.x.x/8 "
                       "(nos de fabrica em 2.x nao enxergam esta placa).")
            if windows:
                out.append(f'  netsh interface ip set address name="{name}" static 2.0.0.{ip.split(".")[-1]} 255.0.0.0'
                           "   (como administrador; nao executado)")
    for net, names in nets.items():
        if len(names) > 1:
            out.append(f"Aviso: {', '.join(names)} na mesma subrede {net}: o sistema envia por uma so; "
                       "desligue a outra ou fixe o IP de origem.")
    if len(ifaces) > 1:
        out.append("sACN multicast sai pela interface da rota padrao (gateway); "
                   "para outra placa, fixe o IP de origem (IP_MULTICAST_IF).")
    return out


# ---------------------------------------------------------------- relatorio
def scan_all(timeout=2):
    """Dict JSON-serializavel com tudo; os tres scans rodam em paralelo (~timeout+1 s)."""
    ifaces = interfaces()
    res = {"interfaces": ifaces, "suggestions": suggest(ifaces), "artnet": [], "sacn": [], "etherdream": []}
    jobs = {"artnet": (scan_artnet, (timeout, ifaces)), "sacn": (scan_sacn, (timeout + 1, ifaces)),
            "etherdream": (scan_etherdream, (timeout,))}

    def run(key, fn, args):
        try:
            res[key] = fn(*args)
        except OSError as e:
            res[key] = {"error": str(e)}

    threads = [threading.Thread(target=run, args=(k, *v), daemon=True) for k, v in jobs.items()]
    for t in threads:
        t.start()
    for t in threads:
        t.join(timeout + 5)
    return res


def report(d):
    def section(title, key, fmt):
        ln.extend(["", title])
        items = d[key]
        if isinstance(items, dict):
            ln.append(f"  erro: {items['error']}")
        elif not items:
            ln.append("  (nada encontrado)")
        else:
            ln.extend("  " + fmt(x) for x in items)

    ln = ["Interfaces:"] if d["interfaces"] else ["Interfaces: (nenhuma)"]
    for i in d["interfaces"]:
        ln.append(f"  {i['name']}: {i['ip']} / {i['mask']}" + (f"  gw {i['gateway']}" if i.get("gateway") else ""))
    ln += ["", "Sugestoes:"] + [f"  {s}" for s in d["suggestions"]]
    section("Art-Net (ArtPollReply):", "artnet", lambda n: f"{n['ip']}  {n['short_name']!r}  {n['long_name']!r}  "
            f"[{', '.join(f'{p['dir']} U{p['universe']}' for p in n['ports']) or 'sem portas'}]")
    section("sACN (discovery):", "sacn", lambda s: f"{s['ip']}  {s['source_name']!r}  universos {s['universes']}")
    section("Ether Dream (beacon):", "etherdream", lambda e: f"{e['ip']}  mac {e['mac']}  hw {e['hw_rev']} sw {e['sw_rev']}  "
            f"buffer {e['buffer_capacity']}  max {e['max_point_rate']} pps  playback {e['status']['playback_state']}")
    return "\n".join(ln)


def main(argv=None):
    ap = argparse.ArgumentParser(description="Spellcaster: analise de rede")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--timeout", type=float, default=2)
    a = ap.parse_args(argv)
    d = scan_all(a.timeout)
    print(json.dumps(d, indent=2, ensure_ascii=True) if a.json else report(d))


if __name__ == "__main__":
    sys.exit(main())
