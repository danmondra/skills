#!/usr/bin/env python3
"""Convierte la respuesta JSON de Mistral Voxtral a subtítulos (SRT, VTT o texto).

Uso:
  json2srt.py respuesta.json [srt|vtt|txt] > salida

- srt (por defecto): subtítulos SubRip con marcas de hablante.
- vtt: WebVTT.
- txt: solo el texto plano.
"""
import json
import sys


def fmt_time(seconds, vtt=False):
    if seconds is None:
        seconds = 0.0
    ms_total = int(round(float(seconds) * 1000))
    h, rem = divmod(ms_total, 3600_000)
    m, rem = divmod(rem, 60_000)
    s, ms = divmod(rem, 1000)
    sep = "." if vtt else ","
    return f"{h:02d}:{m:02d}:{s:02d}{sep}{ms:03d}"


def label(seg):
    speaker = seg.get("speaker_id")
    text = (seg.get("text") or "").strip()
    return f"[{speaker}] {text}" if speaker else text


def main():
    if len(sys.argv) < 2:
        print("Uso: json2srt.py respuesta.json [srt|vtt|txt]", file=sys.stderr)
        sys.exit(1)

    fmt = (sys.argv[2] if len(sys.argv) > 2 else "srt").lower()
    with open(sys.argv[1], encoding="utf-8") as fh:
        data = json.load(fh)

    segments = data.get("segments") or []

    if fmt == "txt":
        print(data.get("text", "").strip())
        return

    if not segments:
        # Sin granulación de segmentos: un único bloque con todo el texto
        text = data.get("text", "").strip()
        if not text:
            return
        # Sin timestamps: duración desconocida; cue con fin > inicio para que los reproductores lo muestren
        segments = [{"start": 0.0, "end": 1.0, "text": text}]

    out = [] if fmt != "vtt" else ["WEBVTT", ""]
    for i, seg in enumerate(segments, 1):
        if fmt == "vtt":
            out.append(f"{fmt_time(seg.get('start'), vtt=True)} --> {fmt_time(seg.get('end'), vtt=True)}")
            out.append(label(seg))
        else:
            out.append(str(i))
            out.append(f"{fmt_time(seg.get('start'))} --> {fmt_time(seg.get('end'))}")
            out.append(label(seg))
        out.append("")

    try:
        sys.stdout.write("\n".join(out))
    except BrokenPipeError:
        # Tubería cerrada por el consumidor (p. ej. `| head`); salir sin traza.
        sys.stderr.close()


if __name__ == "__main__":
    main()
