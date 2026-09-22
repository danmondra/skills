---
name: lesson-to-html
description: Build and evolve a "clase grabada interactiva" — an interactive HTML study page built from a recorded class (grabación + transcripción). Use when the user hands over a class recording or transcript to study from, or asks to extend the page with their own notes and backing data, tools realigned to the class timeline, per-block quizzes, or player changes.
---

# Clase grabada interactiva

One self-contained HTML page built from a recorded class and its transcript, with a collapsible audio player at the bottom. A study artifact, not a summary.

## Methodology

The page is cut into **blocks** — one per moment of the class, in the order the class happened. Every block carries the same three parts:

1. **Lo que se mencionó en clase** — near-verbatim transcript prose, not a summary: the professor's examples and phrasing, every claim carrying a timestamp **linked to the player** — click it and the audio seeks to that second. The page follows the audio as it plays: the current block and the line being spoken light up.
2. **El acompañamiento** — beside the class text, at that same moment: interactive or visual support (charts, simulators, formulas, explainers) and extra information about what was mentioned there. It may outgrow the class text; that's expected.
3. **Las notas del estudiante** — the student's own layer, in its own color, in two columns: their claims | the evidence backing them. Evidence is interactive or charted — assertions get *backed*, not decorated.

## Learning contract

- **The transcript is the source.** Content stays *this* class — the professor's ordering and emphases ("¡va en examen!", "no se casen con un ejemplo"). Trace every fact to a transcript line; label anything you add that the class didn't show as `Modelo didáctico …, no datos de la clase`.
- **Retrieval closes a block.** Two questions per block — one recall, one built on the professor's warning — answered in place, with a per-block score and reset. No global quiz.

## Build process

1. **Mine the transcript, never the memory.** Grep the VTT for each concept for its exact cue time; read cue buckets to hear the class's shape before writing. Done when block ranges and fact timestamps come from cue times. ([PIPELINE.md](PIPELINE.md) carries the snippets.)
2. **Cut the recording into blocks** — some 5–15 for an hour, one per topic, each with `data-start`/`data-end`, a range pill, and a "▶ escuchar bloque" link. Done when the blocks cover the recording end to end.
3. **Write the class text** as transcript prose in light markdown: bold concepts, `<em>` for the professor's phrasing, `<blockquote>` for verbatim, `data-ts` on every fact. Done when each fact-bearing item carries its timestamp.
4. **Place the accompaniment** beside its moment, even when it outgrows the class text. Done when no tool sits in a block that doesn't discuss it.
5. **Write the student's notes only when dictated** — claims | evidence, data labeled. Done when every claim has its evidence.
6. **Close each block with two quiz questions.** Done when every block scores itself.
7. **Verify before reporting.** Done when the extracted script passes `node --check` and headless Chrome renders a computed value.

## Editing safely

The page may carry a base64 audio blob (a 16 MB MP3 becomes a 22 MB HTML). A blind splice around it has silently killed the audio before, so every edit follows one pattern:

1. Back the page up beside the work.
2. Swap the blob for `__DATAURI__`.
3. Assert every anchor exists exactly once before replacing it; a failed assert leaves the page untouched.
4. Restore the blob last and assert its single presence.

## Verify

```bash
# syntax-check the script (strip the blob first so the extract stays small)
python3 -c "import re;h=re.sub(r'data:[a-z/]+;base64,[A-Za-z0-9+/=]+','DATAURI',open('clase.html',encoding='utf-8').read());open('/tmp/app.js','w').write(re.findall(r'<script>(.*?)</script>',h,re.S)[-1])"
node --check /tmp/app.js
# the rendered DOM proves the script ran to the end
google-chrome --headless=new --disable-gpu --no-sandbox --virtual-time-budget=8000 \
  --dump-dom http://localhost:8000/clase.html | grep -o 'id="simOut"[^>]*>[^<]*'
curl -s -o /dev/null -w "%{http_code} %{content_type}\n" http://localhost:8000/clase.vtt  # expect 200 text/vtt
```

A rendered computed value — a simulator output, a note's computed line — proves every listener bound; an empty one means the script died partway.

## Reference

- [PIPELINE.md](PIPELINE.md) — recording → assets: compress, embed, SRT→VTT, mine cue times, serve.
- [COMPONENTS.md](COMPONENTS.md) — style, then block, timeline, timestamps, tools, notes, quiz, player: markup shapes, runtime behavior, gotchas.
