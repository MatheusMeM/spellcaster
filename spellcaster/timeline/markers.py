# Markers: cortes de video pelo scene detect do ffmpeg (subprocess, ffmpeg no PATH) e
# onsets de audio por salto de energia, so com wave + array.
import array
import re
import subprocess
import sys
import wave

PTS = re.compile(r"pts_time:([0-9.]+)")


def video(path, threshold=0.3):
    """Instantes (s) dos cortes de cena. Devolve [] se o ffmpeg nao estiver no PATH."""
    cmd = ["ffmpeg", "-nostats", "-hide_banner", "-i", str(path),
           "-vf", f"select=gt(scene\\,{threshold}),showinfo", "-an", "-f", "null", "-"]
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, errors="replace")
    except OSError:
        return []
    return [float(m) for m in PTS.findall(p.stderr)]


def audio(path, threshold=3.0, hop=0.02, gap=0.1, window=8):
    """Onsets (s) de um WAV PCM 16 bits: energia da janela acima de threshold x a media das
    window janelas anteriores, com refratario de gap segundos."""
    with wave.open(str(path), "rb") as w:
        if w.getsampwidth() != 2:
            raise ValueError("markers.audio: use WAV PCM 16 bits")
        ch, sr, n = w.getnchannels(), w.getframerate(), w.getnframes()
        a = array.array("h", w.readframes(n))
    if sys.byteorder == "big":
        a.byteswap()
    step = max(1, int(sr * hop)) * ch
    # ponytail: energia RMS por janela fixa, sem FFT nem flux espectral ; trocar se precisar
    # pegar batida grave sob musica cheia (a energia larga confunde bumbo com naipe).
    e = [sum(x * x for x in a[i:i + step]) / step / 1073741824.0 for i in range(0, len(a) - step + 1, step)]
    out, last = [], -gap
    for i in range(window, len(e)):
        m = sum(e[i - window:i]) / window
        t = i * step / ch / sr
        if e[i] > 1e-5 and e[i] > threshold * m and t - last >= gap:
            out.append(round(t, 4))
            last = t
    return out


def find(path, threshold=0.3):
    """Markers de um arquivo: .wav pela energia, o resto pelo scene detect do ffmpeg."""
    return audio(path) if str(path).lower().endswith(".wav") else video(path, threshold)
