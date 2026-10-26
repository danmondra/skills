#!/usr/bin/env python3
"""Une varias respuestas JSON de Mistral Voxtral en una sola.

Uso:
  merge_voxtral.py salida.json offset1:chunk1.json offset2:chunk2.json ...

Cada offset es el desplazamiento (en segundos) del inicio del chunk dentro del
audio original. Ajusta start/end de cada segmento y concatena el texto.
"""
import json
import sys


def main():
    if len(sys.argv) < 3:
        print("Uso: merge_voxtral.py salida.json offset:chunk.json [...]", file=sys.stderr)
        sys.exit(1)

    out_path = sys.argv[1]
    args = sys.argv[2:]

    merged_text = []
    merged_segments = []
    for arg in args:
        try:
            offset_s, path = arg.split(":", 1)
            offset = float(offset_s)
        except ValueError:
            print(f"Error: argumento inválido '{arg}' (esperado offset:archivo)", file=sys.stderr)
            sys.exit(1)

        with open(path, encoding="utf-8") as fh:
            data = json.load(fh)

        if data.get("object") == "error":
            print(f"Error: {path} contiene un error de la API: {data.get('message')}", file=sys.stderr)
            sys.exit(1)

        text = (data.get("text") or "").strip()
        if text:
            merged_text.append(text)

        for seg in data.get("segments") or []:
            seg = dict(seg)
            if seg.get("start") is not None:
                seg["start"] = float(seg["start"]) + offset
            if seg.get("end") is not None:
                seg["end"] = float(seg["end"]) + offset
            merged_segments.append(seg)

    merged = {"text": " ".join(merged_text), "segments": merged_segments}
    with open(out_path, "w", encoding="utf-8") as fh:
        json.dump(merged, fh, ensure_ascii=False)
    print(f"Unido: {out_path} ({len(args)} chunks, {len(merged_segments)} segmentos)", file=sys.stderr)


if __name__ == "__main__":
    main()
