# Chunked, resumable transcription

Above ~10 minutes, send the audio in slices. Each slice is an independent request, so a failure costs one 10-minute chunk instead of the whole recording, and a re-run skips whatever already succeeded.

## Workdir layout

```
recording.m4a
recording.chunks/
  chunk_000.mp3  out_000.json  err_0.log   # slice audio / its response / its stderr
  chunk_001.mp3  out_001.json  err_1.log
  ...
recording.json   # merged result: absolute timestamps
recording.srt    # projection of recording.json
```

Chunks are numbered with three digits so shell globs sort correctly up to 999 slices.

## Step 1 — cut the chunks

Cut with re-encoding to mono 16 kHz mp3: it is what the ASR pipeline wants, and the file is ~4.5 MB per 10 minutes.

```bash
START=0; CHUNK=600
ffmpeg -v error -ss "$START" -t "$CHUNK" -i input.m4a -ac 1 -ar 16000 -b:a 64k -y chunk_000.mp3
```

`-ss` before `-i` seeks fast and is accurate enough for chunking (the offset merge absorbs the millisecond drift).

**Exception — cutting for delivery, not transcription.** If the goal is a shorter audio file for the user, do not re-encode: `ffmpeg -i in.m4a -t 4320 -c copy out.m4a` keeps the original codec and runs in seconds.

## Step 2 — transcribe each chunk, with validation and retries

A truncated or error JSON must never reach the merge step: it would shift every later timestamp. Validate structure, not just HTTP success.

```bash
valid_json() {
  python3 - "$1" <<'PY'
import json, sys
try:
    data = json.load(open(sys.argv[1], encoding="utf-8"))
except Exception:
    sys.exit(1)
if not isinstance(data, dict) or data.get("object") == "error":
    sys.exit(1)
if not (data.get("text") or data.get("segments")):
    sys.exit(1)
PY
}
```

Loop per chunk: skip if `out_NNN.json` already validates, otherwise call the endpoint with `timeout`, retry up to 4 times with `sleep 30 × try`, and move the response into place only once it validates.

Result to match: the chunk files and `out_NNN.json` in this skill's example run — 8 chunks of 600 s covering 4320 s, merging to 797 segments.

## Step 3 — merge with offsets

Each chunk's timestamps are relative to its own start, so add the offset back:

```bash
python3 merge_voxtral.py recording.json 0:out_000.json 600:out_001.json 1200:out_002.json ...
```

`merge_voxtral.py` (in this skill) offsets every `start`/`end` and concatenates `text`. If chunks were produced from separately-measured audio files rather than fixed offsets, use measured durations accumulated with `ffprobe` instead of `i × chunk_secs` — the last chunk is usually shorter, and a truncated `-t` cut shifts the offsets by a few hundred ms.

## Step 4 — derive the subtitle file

```bash
python3 json2srt.py recording.json srt > recording.srt
python3 json2srt.py recording.json vtt > recording.vtt
```

Then clean up (`rm -rf recording.chunks`) once the merged output checks out — or keep it if more slices may need re-running.

## Context-bias keywords

A keyword file for a Mexican university macroeconomics course looks like this — comma-separated, newlines also accepted:

```
Keynes,Keynesiano,demanda_efectiva,eficiencia_marginal_del_capital,curva_IS,curva_LM,Banco_de_México,UNAM
```

**Known limit:** the real `keywords.txt` in `macro-keynes/` holds 177 terms; only the first 100 are sent, so 77 are silently dropped unless the script warns. Trim the file to the most important 100 rather than relying on truncation order.

Read keywords from a file, split on both commas and newlines, trim whitespace, drop empties, and stop at 100. Warn on the total when it exceeds the cap so the user knows what got dropped rather than silently losing the tail.
