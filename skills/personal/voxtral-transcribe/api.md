# Voxtral offline transcription API

```
POST https://api.mistral.ai/v1/audio/transcriptions
Authorization: Bearer $MISTRAL_API_KEY
Content-Type: multipart/form-data
```

## Form fields

| Field | Value | Notes |
| --- | --- | --- |
| `file` | audio file | mp3/m4a/wav/flac/opus/ogg/mp4 — mono 16 kHz mp3 at 64 kbps is a good chunk format |
| `model` | `voxtral-mini-latest` | the offline transcription model |
| `language` | `es`, `en`, … | pass explicitly; do not rely on detection |
| `diarize` | `true` / `false` | adds `speaker_id` to segments |
| `timestamp_granularities` | `segment` | omit for a single flat text blob |
| `context_bias` | one keyword | repeat the field once per keyword; **100 max** |

`context_bias` is a repeated **form field** in curl, not a JSON array — the array form only applies to the Python/TS SDKs. Sending a comma-joined string in one field does not work.

## curl

```bash
curl --silent --show-error \
  --location 'https://api.mistral.ai/v1/audio/transcriptions' \
  --header "Authorization: Bearer $MISTRAL_API_KEY" \
  --form "file=@recording.m4a" \
  --form 'model="voxtral-mini-latest"' \
  --form 'language="es"' \
  --form 'diarize=true' \
  --form 'timestamp_granularities="segment"' \
  --form 'context_bias="demanda_efectiva"' \
  --form 'context_bias="eficiencia_marginal_del_capital"' \
  > recording.json
```

Check the first byte before anything else — an error also comes back as HTTP 200 with a JSON body:

```bash
head -c 1 recording.json | grep -q '{' || echo "not JSON / API error"
```

## Python SDK

```python
import os
from mistralai import Mistral

client = Mistral(api_key=os.environ["MISTRAL_API_KEY"])
with open("recording.m4a", "rb") as fh:
    resp = client.audio.transcriptions.complete(
        model="voxtral-mini-latest",
        file={"content": fh, "file_name": "recording.m4a"},
        language="es",
        diarize=True,
        timestamp_granularities=["segment"],
        context_bias=["demanda_efectiva", "eficiencia_marginal_del_capital"],
    )
print(resp.text)
for seg in resp.segments or []:
    print(seg.start, seg.end, seg.speaker_id, seg.text)
```

In the SDK `context_bias` **is** a list; in curl it is repeated fields.

## Response

```json
{
  "text": "full concatenated transcript",
  "segments": [
    {"type": "transcription_segment", "text": "…", "start": 3.6, "end": 21.3, "speaker_id": "speaker_1"}
  ]
}
```

Error response:

```json
{"object": "error", "message": "…"}
```

## Limits and failure modes

- **Long audio hangs.** Multi-hour files frequently stall or time out on a single request. Above ~10 minutes, chunk (see [chunking.md](chunking.md)).
- **Transient failures** — 429, capacity, gateway timeouts. Retry with growing backoff (30 s, 60 s, 90 s…), a few attempts per chunk, and treat each chunk as independently resumable.
- **Cost scales with audio length.** Cached chunk JSONs are the only defence against paying twice; keep them until the merged output is verified.
