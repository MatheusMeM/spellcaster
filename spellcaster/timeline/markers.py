# Markers: video cuts from the ffmpeg scene detect (subprocess, ffmpeg on the PATH) and
# audio onsets from energy jumps, with wave + array only.
import array
import re
import subprocess
import sys
import wave

PTS = re.compile(r"pts_time:([0-9.]+)")


def video(path, threshold=0.3):
    """Instants (s) of the scene cuts. Returns [] if ffmpeg is not on the PATH."""
    cmd = ["ffmpeg", "-nostats", "-hide_banner", "-i", str(path),
           "-vf", f"select=gt(scene\\,{threshold}),showinfo", "-an", "-f", "null", "-"]
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, errors="replace")
    except OSError:
        return []
    return [float(m) for m in PTS.findall(p.stderr)]


def audio(path, threshold=3.0, hop=0.02, gap=0.1, window=8):
    """Onsets (s) of a 16-bit PCM WAV: window energy above threshold x the average of the
    previous `window` windows, with a refractory period of `gap` seconds."""
    with wave.open(str(path), "rb") as w:
        if w.getsampwidth() != 2:
            raise ValueError("markers.audio: use 16-bit PCM WAV")
        ch, sr, n = w.getnchannels(), w.getframerate(), w.getnframes()
        a = array.array("h", w.readframes(n))
    if sys.byteorder == "big":
        a.byteswap()
    step = max(1, int(sr * hop)) * ch
    # ponytail: RMS energy over a fixed window, no FFT and no spectral flux ; swap it if you need
    # to catch a low beat under a full mix (broadband energy confuses the kick with the horns).
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
    """Markers of a file: .wav by energy, everything else by the ffmpeg scene detect."""
    return audio(path) if str(path).lower().endswith(".wav") else video(path, threshold)
