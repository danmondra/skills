---
name: groq-transcribe
description: Transcribe audio/video to subtitles (SRT) with Groq's Whisper large-v3 API — one request for up to ~100 min of audio, byte-sized chunking beyond that, resumable runs, and correct absolute timestamps. Use when the user wants a transcript, subtitles, or asks to transcribe with Groq/Whisper.
---

# Transcribing with Groq Whisper

Groq serves Whisper `large-v3` (and `large-v3-turbo`) on an OpenAI-compatible endpoint. It is fast and cheap, but it has a hard **25 MB per request** limit and no speaker diarization. Everything else (SRT, VTT, plain text) is a projection of the JSON.

Docs: <https://console.groq.com/docs/speech-to-text>

## Before anything: the credential

The key lives in `~/.config/zsh/.zshenv` as `GROQ_API_KEY`. Never hardcode it in a script or repo file.

```bash
[ -n "${GROQ_API_KEY:-}" ] || echo "GROQ_API_KEY not set"
```

If it is missing, ask the user to add it — do not paste a key you saw in a chat into a file.

## The fast path

```bash
~/.agents/skills/groq-transcribe/bin/groq-transcribe input.mp4
```

**Run it in the foreground with a tool timeout of ~900 s.** Do not background it and poll with `sleep`: an 88-minute talk finishes in about a minute, and polling turned that into five. Do not `ffprobe` the input first either — the binary probes, and prints everything you need on stderr.

Outputs `input.srt` next to the input. Language defaults to Spanish (`es`); pass `--language` for anything else, or `--language auto` to let the API detect it.

What it does, and what it prints:

1. Probes the true audio duration **without decoding** (packet count × codec frame size, ~1 s).
2. Picks the highest bitrate ≤ 48 kbps that fits the whole file in one request, never below 32 kbps.
3. Encodes once. If it fits — up to ~100 min at 32 kbps — that is **the only request**. Longer audio is split with `-c copy` (no re-encode) into budget-sized chunks sent in parallel.
4. Merges, prints the segments at every chunk boundary (`boundary @ …`), clamps zero/negative-length segments, flags hallucination loops, trims the trailing 1–2 word filler (`y`, `gracias`), writes the SRT, and reports total time.

Read the stderr; it already contains the checks you would otherwise do by hand (boundaries, trimmed tail, duration mismatch, boilerplate share, loops).

A `WARNING: … hallucination loop` line is a suspect, not a verdict. Do not re-run yet, and do not read the SRT. Isolate the flagged cues plus one cue before and after each (a few lines, not the file). A loop is the same text repeating, or a stretch that does not continue the lines around it. Different short phrases sitting between normal lines are slow speech — board work, a pause — and the transcript stands. Re-run the whole file with no `--prompt` and `--language auto` only when that local check shows a loop.

### After the run: fix names, don't prompt for them

Send no prompt. Then check proper nouns in the output:

1. Collect the names you know from real sources: the user's message, a programme, slides or a syllabus next to the file. A filename or directory name is not a source of names.
2. Grep the SRT for each one and for close misspellings (`Catlán` for `Acatlán`, `Reyez` for `Reyes`).
3. Fix them with targeted replacements in the SRT and tell the user what you changed.

A bad replacement only affects the words it targets. A bad prompt can wreck the whole transcript (see Practical notes).

Measured on an 88.1-min talk: 1 request, 21.1 MB, 59 s wall (encode ~30 s, transcription ~21 s). The previous Python version made 2 requests at 48 kbps and spent a further ~9 s decoding the file only to measure it.

| Flag | Meaning |
| --- | --- |
| `--language es` | Default is **`es`**. Use `--language auto` for code-switched audio (a translator relaying a speaker), where any single hint forces the wrong language on half the file. Don't use it together with `--prompt` (see Practical notes). |
| `--model whisper-large-v3-turbo` | ~8× cheaper, slightly worse on accents and proper nouns. |
| `--bitrate auto` | Default. Or a fixed kbps; values are snapped to valid 16 kHz mp3 rates (8/16/24/32/40/48/…). |
| `--budget-mb 24` | Target MB per request against the 25 MB (decimal) limit. |
| `--workers 3` | Parallel requests when chunked. Keep ≤ 3; the free tier rate-limits. |
| `--prompt "…"` | Whisper prompt. **Not recommended**: measured harmful even with names only. Fix names after the run instead. No prompt is ever derived automatically. |
| `--keep-tail` | Keep the trailing 1–2 word segments instead of trimming them. |
| `--keep-work` | Keep the mp3, per-chunk JSON, and a merged `transcript.json`. |
| `--outdir DIR` | Write the output somewhere other than next to the input. |
| `--format srt` | `srt`, `vtt` or `txt`. |

**Resumable.** The work dir is `/tmp/groq-asr-<hash>`, keyed on input path, size, mtime, model, language and bitrate. A re-run of the same command after a failure reuses the encoded audio and every chunk JSON that validates; nothing paid for is paid twice (a cached re-run takes 0.3 s). The dir is removed on success unless `--keep-work`.

**Converting a kept JSON** (no network):

```bash
bin/groq-transcribe --from-json /tmp/groq-asr-…/transcript.json --format vtt > out.vtt
```

### Rebuilding the binary

Source is in `rust/` (deps: clap, serde_json, regex; HTTP goes through `curl`). After editing:

```bash
cd ~/.agents/skills/groq-transcribe/rust && cargo build --release && cp target/release/groq-transcribe ../bin/
```

Rust is not where the speed comes from — Python startup was 0.06 s. The time is ffmpeg encoding and Groq; the gains are fewer requests, one encode, and no decode pass just to measure.

## Sizing: one request when it fits

The limit is in bytes, so request length follows from bitrate: 24 MB is ~67 min at 48 kbps, ~80 at 40, ~100 at 32. The binary works backwards from the duration to the highest rate that fits in one request.

**32 kbps costs no measurable accuracy.** Same 88.1-min talk: 48 kbps in 2 requests gave 8,933 words; 32 kbps in 1 request gave 8,907 (the difference is mostly the trimmed tail filler), with `Bolaños` recognised 70 times vs 63 and no `Speaker N` filler in either. Whisper resamples to 16 kHz mono anyway. Below 32 kbps is untested, so longer files are chunked at 32 instead of squeezed further.

**Only some bitrates exist.** Mono 16 kHz mp3 is MPEG-2 layer III: 8/16/24/32/40/48/56/64… kbps. libmp3lame silently snaps anything else — asking for 36k produced exactly the 32k file, 39k produces 40k and overshoots. Arithmetic on an in-between rate is wrong.

**MB means 25,000,000, not 25 MiB.** A 24 MiB chunk reads as "25.2 MB" and can trip a limit enforced in decimal.

**Encoded size is not exactly budget/bitrate.** The binary measures the encoded file, and re-encodes once at a lower rate only when that turns two requests into one.

## When to do it by hand instead

For a file already under 25 MB that needs no encoding, plain curl is fine:

```bash
curl -s "https://api.groq.com/openai/v1/audio/transcriptions" \
  -H "Authorization: Bearer ${GROQ_API_KEY}" \
  -F "model=whisper-large-v3" \
  -F "file=@clase.mp3" \
  -F "temperature=0" \
  -F "language=es" \
  -F "response_format=verbose_json" \
  -X POST > whisper-large-v3.json
```

`response_format=verbose_json` is what returns `segments` with `start`/`end`. Ask for `json` and you get only `text`, with no timings and therefore no subtitles.

## Output shape

```json
{"segments": [{"id": 0, "start": 0.0, "end": 4.2, "text": "…"}],
 "language": "es", "chunk_languages": ["Spanish"], "duration": 5283.6,
 "model": "whisper-large-v3", "chunks": 1}
```

`start`/`end` are seconds (float). An error response is `{"error": {...}}` — always validate before trusting a response file.

## Practical notes

- **No diarization.** Groq Whisper returns no `speaker_id`. If the user needs speaker labels, that is Voxtral (`voxtral-transcribe` skill) or a separate diarization pass — say so up front, not after the run.
- **Duration can lie twice.** A file whose header reported 81.5 min carried 88.1 min of audio, and its packet timestamps summed to the same wrong 81.5 (ffmpeg prints thousands of `non monotonically increasing dts` lines on it). The packet *count* was right: 247,677 AAC packets × 1024 / 48 kHz = 88.1 min. The binary uses the count, warns on a mismatch, and its SRT follows the audio — a player trusting the header may drift against it. Codecs without a fixed frame size (opus, vorbis, flac, pcm) fall back to the header; the post-encode measurement catches it.
- **Chunk timestamp drift.** When chunked, merge offsets come from `ffprobe` on each chunk file, not from the API's reported `duration`, which can exceed the real slice length.
- **Boundary artifacts.** Whisper occasionally starts a chunk mid-sentence or invents a phrase at a slice edge. One request means no boundaries; with several, read the `boundary @` lines.
- **`temperature=0`** keeps repeated runs comparable.
- **Hallucination on silence.** Long silence yields filler or boilerplate ("Gracias por ver el video", a run of `y`). Trailing 1–2 word segments are trimmed by default and listed on stderr; a legitimate closing "muchas gracias" goes with them — use `--keep-tail` if that matters.
- **Prompts are a gamble; the default is none.** On one 300s sample, a names-only prompt recovered `Tewchitlán`, and a topic prompt dropped 483 to 464 words. On a 69-min conference, a hint built automatically from the path (`Conferencia Economía Digital conferences economics`) made Whisper write `Económica Digital` in 30 s cues for minutes at a time. Its 300s sample fell from 82 cues to 14. The same sample with real speaker names (`Raúl Marín, Manuel Reyes, Jorge Alonso Bustamante`) still lost 37% of words (664 → 416) and inserted a name mid-sentence, with `es` or `auto` alike. Path-derived prompting was removed for this reason. Correct names in the output instead.
- **`--language` plus a written `--prompt` is the failure mode to avoid.** Measured on a 300s Spanish sample: `language=en` alone returned 484 words; the prompt alone 393; **both together 162 words with `Speaker 1` filler**. The binary refuses that pair unless `--allow-language-prompt`, and warns when >5% of segments are boilerplate.
- **Loop detection.** The binary warns when ≥3 cues of ≥10 s carry under 0.3 words/s (speech runs 2–3), or when one text makes up over 10% of the cues (and appears at least 5 times). The second test is a loop. The first also fires on a lecture: one word held for 15 s while someone writes on the board, three times in an hour. Both fired on the prompted sample above. Neither fired on the clean 69-min transcript. Judge from the isolated cues, as in the fast path — a warning alone is not a reason to re-run.
- **Level is not the problem, so don't normalize reflexively.** -34 dB mean transcribed identically to -18 dB (483 vs 493 words over 300s); attenuation to -50 dB still produced no `Speaker 1`.
- **A second request is the test after the local check fails.** When the isolated cues show a loop, re-run with `--language auto` and no `--prompt` before changing anything else.

## Files in this skill

| File | Contents |
| --- | --- |
| [bin/groq-transcribe](bin/groq-transcribe) | Release binary: probe → encode once → 1 request (or copy-split chunks) → merge → SRT; `--from-json` converts kept JSON |
| [rust/](rust/) | Source (`src/main.rs`) and `Cargo.toml` |
| [api.md](api.md) | Endpoint, fields, limits, valid bitrates, error codes, model comparison |
