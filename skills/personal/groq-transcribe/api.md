# Groq Whisper API reference

Base URL: `https://api.groq.com/openai/v1` — OpenAI-compatible.

## Endpoint

`POST /audio/transcriptions`, `multipart/form-data`.

| Field | Notes |
| --- | --- |
| `file` | Required. Audio/video file, **≤ 25 MB**. |
| `model` | `whisper-large-v3`, `whisper-large-v3-turbo`, or `distil-whisper-large-v3-en` (English). |
| `language` | ISO-639-1 (`es`, `en`, `pt`). Optional; omitting it lets Whisper detect, which drifts on long audio. |
| `prompt` | Free text to bias spelling/vocabulary. Max ~224 tokens. |
| `response_format` | `json` (default), `verbose_json`, `text`, `srt`, `vtt`. Use `verbose_json` for timings. |
| `temperature` | `0` makes runs reproducible. |
| `timestamp_granularities[]` | `word` and/or `segment`; requires `verbose_json`. Word timings cost nothing extra. |

`response_format=srt` returns SRT directly, which is convenient for one-shot use but throws away the JSON — transcribing twice to get both formats is wasted money. Prefer `verbose_json` and project.

## Limits

Chunk length is not a fixed duration — it is whatever fits in the byte budget. 24 MB at 48 kbps mono is ~67 minutes; at 40 kbps, ~80; at 32 kbps, ~100. Mono 16 kHz mp3 (MPEG-2 layer III) only has the rates 8/16/24/32/40/48/56/64… kbps — libmp3lame snaps any other value (36 → 32, 39 → 40), so size arithmetic on an in-between rate is wrong.

| Limit | Value |
| --- | --- |
| File size | 25 MB per request (decimal; see the sizing note in SKILL.md) | 
| Min billable duration | 10 seconds |
| Rate limit | Model-dependent; free tier is tight (requests/minute *and* audio-seconds/hour). |

Going over 25 MB is a `413`; hitting a rate limit is `429` with a `retry-after` header. Both mean "chunk smaller" or "fewer workers", not "retry harder".

## Models

| Model | Speed | Cost | Use |
| --- | --- | --- | --- |
| `whisper-large-v3` | fast | baseline | Default. Best accuracy on accented Spanish, technical vocabulary. |
| `whisper-large-v3-turbo` | ~2× faster | ~8× cheaper | Long recordings where clean audio and rough fidelity is enough. |
| `distil-whisper-large-v3-en` | fast | cheap | English only; do not use on other languages. |

## Errors

```json
{"error": {"message": "…", "type": "invalid_request_error", "code": "…"}}
```

| HTTP | Meaning |
| --- | --- |
| 400 | Bad field, unsupported format, or file rejected (often size). |
| 401 | Bad/missing key. Check `GROQ_API_KEY`. |
| 413 | File over 25 MB. |
| 429 | Rate limited. Back off, reduce `--workers`. |
| 5xx | Transient; retry with backoff. |

A response that parses as JSON but has `error` is a failure — validate for `segments`, not for HTTP 200.

## Curl, single request

```bash
curl -s "$GROQ/api/audio/transcriptions" \
  -H "Authorization: Bearer ${GROQ_API_KEY}" \
  -F "model=whisper-large-v3" \
  -F "file=@${FILE}" \
  -F "language=es" \
  -F "temperature=0" \
  -F "response_format=verbose_json" \
  -X POST > out.json
```

