---
name: voxtral-transcribe
description: Transcribe audio/video to JSON, SRT, VTT or TXT with Mistral Voxtral (offline transcription API) — including speaker diarization, context-biasing keywords, chunking of long files, resumable runs and merging chunks back. Use when the user wants a transcript, subtitles/captions, or asks to "get the srt", "transcribe this recording/class/meeting", or cut+transcribe audio.
---

# Transcribing with Mistral Voxtral

Mistral's `voxtral-mini-latest` offline transcription endpoint returns text, per-segment timestamps and speaker labels in a single JSON payload. Everything else (SRT, VTT, plain text) is derived from that JSON — **the JSON is the artifact; the subtitle file is a projection of it.** Never re-transcribe to produce a second format.

Docs: <https://docs.mistral.ai/studio/audio/speech_to_text/offline_transcription#context-biasing>

## Before anything: the credential

The API key must come from the environment. Never hardcode a key into a script, a skill, or a repo file.

```bash
[ -n "${MISTRAL_API_KEY:-}" ] || echo "MISTRAL_API_KEY not set"
```

If it is not set, ask the user to export it (`export MISTRAL_API_KEY=...`, ideally from their shell profile or a secret manager). Do not paste a key from a previous script into a new one: if a local script still has one baked in, that is a leak worth flagging to the user, not a convenience to inherit.

## The workflow

1. **Inspect the input.** Get the real duration before deciding anything:

   ```bash
   ffprobe -v error -show_entries format=duration -of default=nw=1:nokey=1 input.m4a
   ```

2. **Short file (≤ ~10 min):** one request via the JSON endpoint. See [api.md](api.md).

3. **Long file:** chunk it, transcribe each chunk, merge with offsets. Long requests hang or time out at the API, and chunking makes the run resumable and retryable per slice. See [chunking.md](chunking.md).

4. **Read the JSON, not the derived formats.** Inspect what was actually returned before converting:

   ```bash
   python3 -c "
   import json; d=json.load(open('out.json'))
   print(len(d.get('segments') or []), 'segments')
   print(json.dumps((d.get('segments') or [{}])[0], ensure_ascii=False, indent=1))"
   ```

5. **Derive SRT/VTT/TXT from the existing JSON.** `/home/quorra/scripts/json2srt.py input.json [srt|vtt|txt]` (or the copy in [json2srt.py](json2srt.py)). If a JSON already exists next to the audio, this step is all that is needed — transcribing again would be wasted money and time.

## Output shape

The response is a dict with `text` (full concatenation) and `segments`, each segment:

```json
{"type": "transcription_segment", "text": "…", "start": 3.6, "end": 21.3, "speaker_id": "speaker_1"}
```

`start`/`end` are seconds (float). `speaker_id` is present only when diarization is on. A failed call returns `{"object": "error", "message": "…"}` instead — always validate before trusting a response file.

## Practical notes

- **Diarization is coarse.** On single-mic classroom or lecture audio the whole file frequently comes back as `speaker_1`. A single distinct `speaker_id` in the JSON means diarization found nothing, not that one person spoke. Say so rather than presenting `[speaker_1]` tags as if they were meaningful — and offer to strip them.
- **Keep the raw JSON.** It is the only artifact carrying timings; a deleted JSON means paying for the audio twice.
- **Validate before merging.** A truncated chunk JSON silently shifts every later timestamp. The validity check in [chunking.md](chunking.md) exists for this reason.
- **Never re-encode when only the duration changes.** Cutting an audio file for transcription is `ffmpeg -t`, not a re-encode: `-c copy` keeps quality and runs in seconds.
- **Keywords bias spelling, not vocabulary.** `context_bias` fixes proper nouns and technical terms the model would otherwise invent lowercase or misspell (see the economics example in [chunking.md](chunking.md)). Values are `context_bias` form fields repeated per keyword — max 100.
- **Language:** pass `language` explicitly (`es`, `en`, …). Auto-detection on long multi-topic audio drifts.

## Files in this skill

| File | Contents |
| --- | --- |
| [api.md](api.md) | Endpoint, form fields, limits, curl and Python SDK examples |
| [chunking.md](chunking.md) | Chunked + resumable transcription, merging offsets |
| [json2srt.py](json2srt.py) | JSON → SRT / VTT / TXT converter |
| [merge_voxtral.py](merge_voxtral.py) | Merge chunk JSONs into one JSON with absolute timestamps |
