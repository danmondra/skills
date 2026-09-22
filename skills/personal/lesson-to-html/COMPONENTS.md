# Components

Markup shapes, runtime behavior, and gotchas. Artifact copy is Spanish. Libs from CDN: Chart.js 4.4.1, KaTeX 0.16.9 (only `$$…$$` is enabled — single-`$` inline math never renders), Inter + Newsreader.

## Style

Dry and minimal — flat, quiet, typographic. The page should read like a good textbook, not a dashboard.

- **One hue, many values.** Depth comes from gradients of a single color (the blue scale `--b100` … `--b900`), never from a new hue. The hero is the model: `linear-gradient(135deg,#0a1c4e,#14389f 60%,#2f6df6)`.
- **Color marks meaning.** A second color appears only to flag what matters: `mark` (with a `title` tooltip that explains the concept) for key terms, `mark.red` for what the class frames as a problem or trap, amber for the student's notes, green for the player's on-state. Everything else is ink on white.
- **Flat surfaces.** Cards are transparent with hairline rules (`border-top:1px solid var(--line)`) — no fills, no shadows. Backgrounds only where they carry weight: the hero, formula blocks, the player.
- **No chart junk.** Charts follow the same discipline: one-hue palettes, `borderWidth:0` on pies, legends at the bottom, no grid noise.
- **Type does the work.** Newsreader for headings, quotes, and notes; Inter for everything else; body text held to ~70ch.

## Block

```html
<div class="topic" id="t2" data-start="300" data-end="600">
  <div class="topic-head">
    <div class="num">Bloque 2 · Salario</div>
    <h2>Salario real y por qué no alcanza para ahorrar</h2>
    <span class="range">05:00 – 10:00</span>
    <span data-ts="00:05:10">▶ escuchar bloque</span>
  </div>
  <div class="topic-grid">
    <div class="tl">…timeline…</div>
    <div class="tools">…tools…</div>
  </div>
</div>
```

`.topic-grid` is five columns: `.tl` spans 1–2, `.tools` spans 3–5, single column under 960px. `data-start`/`data-end` are seconds; the block holding `currentTime` takes `.now` (filled range pill, blue timeline border) on `timeupdate`/`seeked`/`loadedmetadata`.

Order inside a block: head, grid, notes, quiz.

## Timeline — the class text

Transcript prose, not summary. `h4` subheads, lists, `<blockquote>` for verbatim, `<em>` to keep the professor's wording, `data-ts` on claims.

## Timestamp buttons — the player link

`<span data-ts="00:05:38">00:05:38</span>` becomes a `.ts` button at load that seeks the audio and plays; seconds are cached in `dataset.sec` when the button is created.

**Active line.** All marks sorted by time; on every tick the latest mark ≤ `currentTime + .15` gets `.on`, and its nearest `li/p/blockquote/summary` gets `.speaking` (blue wash, inset bar). Ties go to the first in document order — the visible timeline line, not a duplicated quote inside a `<details>`. A container with two marks clears only after both pass.

## Tools — the accompaniment

Charts, formulas, simulators, `<details>` explainers, stat cards (`.statrow`/`.stat`) — anything that supports or extends what was mentioned. Every tool sits in the block that mentions its subject.

Computed, not decorative: curves follow the real equation (`U = X·Y`, `Q = √(K·N)`), sliders move a live point, and an output line prints the computed value — that line doubles as the verification signal. Anything the class didn't derive gets `<p class="src">Modelo didáctico …, no datos de la clase</p>`.

## Notes — the student's channel

```html
<div class="nota" id="nota-<tema>">
  <span class="nota-tag">✎ Mi apunte · <tema></span>
  <div class="nota-grid">
    <div class="afirm"><h4>Lo que sostengo</h4>…claims in the user's voice…</div>
    <div class="datos"><h4>Datos que lo sostienen</h4>…chart, stats, .src…</div>
  </div>
</div>
```

Amber (`#fffbeb` on `#fcd34d`, text `#78350f`), after the block's grid, above its quiz. Claims left, evidence right; evidence is interactive or charted, per the user's standing instruction.

## Mini-quiz

```html
<div class="quiz mini" id="quiz-t2">
  <h4>☑ Comprueba este bloque</h4>
  <div id="t2q1">
    <p><b>1. …</b></p>
    <label><input type="radio" name="t2q1"> distractor</label>
    <label><input type="radio" name="t2q1" data-ok="1"> correct</label>
    <label><input type="radio" name="t2q1"> distractor</label>
  </div>
  <p class="score"></p>
  <button class="btn ghost" onclick="…clear this block only…">Reiniciar</button>
</div>
```

Two questions per block, one of them from the professor's own warning. The `change` handler scores inside `e.target.closest('.quiz')` (`Llevas n/2 ✓`); reset clears only its own block.

## Player

Fixed bottom, collapsible from anywhere on the head bar: title left, elapsed time as a white pill hugging it, spacer, caret right. Spotify-flavored skin: near-black bar, centered controls, big white circular play, flat gray buttons that brighten on hover, slim progress bar with times at both ends, `#1db954` accents for CC-on and the live dot.

Keyboard: `Space`/`K` play-pause, `←`/`J` and `→`/`L` ∓10 s, `↑`/`↓` speed, `N`/`P` next/previous timestamped section (a toast announces the destination), `M` mute, `C` captions, `Shift+R` collapse. Shortcuts delegate to the buttons and skip typing targets; each button's `title` carries its hint.

Subtitles: fetch the VTT once, parse locally, render into a floating overlay above the player that tracks the collapsed state; `CC` toggles.

Gotchas:
- `const $` first: one thrown statement kills every listener after it, silently.
- Parse VTT times with `vttSec` (`parseFloat` of the last segment). The SRT comma parser yields `NaN` on them → subtitles that never match.
- Times live outside the range input as `.pl-t` spans; the fill is `--p` on the input's background.
- State via classes (`.on`, `.now`, `.speaking`), not inline styles.
- The page reserves bottom padding (~120px) so the fixed bar never covers content.
