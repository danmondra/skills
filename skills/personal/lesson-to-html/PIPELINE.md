# Pipeline: recording → assets

Run once per new class. Needs `ffmpeg` and `python3`; the page and its VTT are served by `python3 -m http.server 8000` from the class folder.

## Compress the audio

```bash
ffmpeg -y -i "<original>.m4a" -ar 22050 -ac 1 -b:a 32k /tmp/clase-32k.mp3
```

## SRT → VTT

Transcription tools emit SRT: comma timestamps, `Speaker N:` labels, cue indices. VTT wants dots and no indices — and the labels out so the cues read as plain prose.

```python
import re
txt = open("<transcript>.srt", encoding="utf-8").read()
txt = re.sub(r'(\d{2}:\d{2}:\d{2}),(\d{3})', r'\1.\2', txt)
txt = re.sub(r'^Speaker \d+:\s*', '', txt, flags=re.M)
txt = "\n".join(l for l in txt.split("\n") if not re.fullmatch(r'\d+', l.strip()))
txt = re.sub(r'\n{3,}', '\n\n', txt).strip()
open("clase-<fecha>.vtt", "w", encoding="utf-8").write("WEBVTT\n\n" + txt + "\n")
```

The page fetches the VTT, so page and VTT travel together and need the server; the VTT must come back as `text/vtt` (the Python server does).

## Embed or link the audio

Self-contained (default when the file must travel alone): base64 data-URI in the page — the HTML grows to ~22 MB and every later edit must protect the blob (see *Editing safely* in SKILL.md). Served: drop the mp3 next to the page and link it, and every edit stays small.

```python
import base64
h = open("clase-<fecha>.html").read()
b64 = base64.b64encode(open("/tmp/clase-32k.mp3", "rb").read()).decode()
h = h.replace('<audio id="au" preload="auto"></audio>',
              '<audio id="au" preload="auto" src="data:audio/mpeg;base64,%s"></audio>' % b64)
open("clase-<fecha>.html", "w").write(h)
```

## Mine cue times

Buckets show the class's shape (where topics start and end); greps give each fact its exact second. Both read the VTT cues.

```python
import re, collections, unicodedata
cues = re.findall(r'(\d+:\d+:\d+\.\d+) --> (\d+:\d+:\d+\.\d+)\n(.+?)(?=\n\n|\Z)', open("clase.vtt").read(), re.S)
def sec(s): h, m, r = s.split(':'); return int(h)*3600+int(m)*60+float(r)
def norm(s): return ''.join(c for c in unicodedata.normalize('NFD', s.lower()) if unicodedata.category(c) != 'Mn')

# shape: five-minute buckets of everything said
buckets = collections.defaultdict(list)
for s, e, t in cues: buckets[int(sec(s)//300)*5].append(t.replace('\n', ' '))
for b in sorted(buckets): print(b, ' '.join(buckets[b])[:800], '\n')

# locate: every mention of a concept, with its cue time
for k in ['isocuanta', 'fobaproa', 'gini', 'bienes libres']:
    for s, e, t in cues:
        if k in norm(t): print(k, s, t.replace('\n', ' ')[:90])
```

Write the grep keys unaccented — `norm` strips accents from the text. The bucket printout is also the fastest way to hear the professor's emphases before writing blocks.
