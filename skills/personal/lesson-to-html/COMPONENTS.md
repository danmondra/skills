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

Any element with `data-ts` becomes a seek button. Author with any clock form (`SS`, `MM:SS`, `HH:MM:SS` — with `.mmm` allowed); normalize at bind time, never trust the raw string at click time.

```html
<span data-ts="05:38">05:38</span>
<span data-ts="00:05:38">▶ escuchar bloque</span>
```

```js
function tsSec(s){
  s = String(s).trim().replace(',', '.');
  const p = s.split(':').map(x => parseFloat(x));
  if (p.some(isNaN)) return 0;
  let v = 0;
  for (const n of p) v = v * 60 + n;
  return v;
}
// Bind once, via delegation so later-injected marks work too:
document.querySelectorAll('[data-ts]').forEach(el => {
  el.classList.add('ts');
  el.dataset.sec = tsSec(el.dataset.ts || el.textContent);
  el.setAttribute('role', 'button');
  el.tabIndex = 0;
});
document.addEventListener('click', e => {
  const t = e.target.closest('[data-ts]');
  if (!t) return;
  seekTo(parseFloat(t.dataset.sec));
});
document.addEventListener('keydown', e => {
  if ((e.key === 'Enter' || e.key === ' ') && e.target.matches?.('[data-ts]')) {
    e.preventDefault();
    seekTo(parseFloat(e.target.dataset.sec));
  }
});
function seekTo(sec){
  const au = document.getElementById('au');
  if (!isFinite(sec)) return;
  sec = Math.max(0, Math.min(sec, au.duration || sec));
  au.currentTime = sec;
  au.play().catch(()=>{});
}
```

Rules: parse with `tsSec` (colon-split, `parseFloat` per segment). Never `Date.parse`, never `new Date`, never split-and-take-last-segment. Cache seconds in `dataset.sec` at bind time; the click handler reads `dataset.sec`, not `dataset.ts`. One delegated listener — per-button `onclick` breaks on duplicated marks inside `<details>`.

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

The control row carries, in order: **Prev / −10 s / play / +10 s / Next / CC / speed stepper**. Prev and Next walk the timestamps of the block under `currentTime`; they are not block skips (those stay on `Shift+N`/`Shift+P`). The speed stepper is the single home of playback rate: `−`/`+` step through `0.75×–3×` in `0.25×` increments, the middle button shows the current rate and resets to `1×` on click.

```html
<audio id="au" preload="metadata" src="clase.mp3"></audio>
<div id="player" aria-label="Reproductor de la clase">
  <div id="plHead" role="button" tabindex="0" aria-expanded="true">
    <span id="plTitle">…</span><span class="pl-t" id="tCur">00:00</span>
    <span class="spacer"></span><span id="plCaret">▴</span>
  </div>
  <div id="plBody">
    <div class="pl-top">
      <button class="pl-btn pl-nav" id="bPrev" title="Timestamp anterior del bloque (P)" aria-label="Timestamp anterior del bloque">‹ Prev</button>
      <button class="pl-btn pl-skip" id="bBack" title="-10 s (←/J)" aria-label="Retroceder 10 segundos">−10 s</button>
      <button class="pl-btn" id="bPlay" title="Reproducir o pausar (Espacio/K)" aria-label="Reproducir o pausar">▶</button>
      <button class="pl-btn pl-skip" id="bFwd" title="+10 s (→/L)" aria-label="Avanzar 10 segundos">+10 s</button>
      <button class="pl-btn pl-nav" id="bNext" title="Siguiente timestamp del bloque (N)" aria-label="Siguiente timestamp del bloque">Next ›</button>
      <button class="pl-btn" id="bCC" title="Subtítulos (C)" aria-label="Alternar subtítulos">CC</button>
      <div class="pl-rate" role="group" aria-label="Velocidad de reproducción">
        <button class="pl-btn" id="bRateDown" title="Bajar velocidad (↓)" aria-label="Bajar velocidad">−</button>
        <button class="pl-btn" id="bRate" title="Velocidad actual; clic para volver a 1×" aria-label="Velocidad de reproducción: 1×">1×</button>
        <button class="pl-btn" id="bRateUp" title="Subir velocidad (↑)" aria-label="Subir velocidad">+</button>
      </div>
    </div>
    <div class="pl-seek">
      <span class="pl-t" id="tCur2">00:00</span>
      <input id="seek" type="range" min="0" max="1000" value="0" step="1" aria-label="Avanzar o retroceder en el audio">
      <span class="pl-t" id="tEnd">00:00</span>
    </div>
  </div>
</div>
```

The control row styles (the rest of the skin comes from **Style**):

```css
.pl-top{display:flex;align-items:center;justify-content:center;flex-wrap:wrap;gap:8px 12px}
.pl-btn{background:none;border:0;color:#a1a1aa;font-size:1rem;cursor:pointer;padding:4px 6px;border-radius:4px;line-height:1}
.pl-btn:hover{color:#fff;background:#1f1f23}
.pl-btn.pl-nav{font-size:.66rem;font-weight:700;letter-spacing:.05em;text-transform:uppercase;padding:5px 7px}
.pl-btn.pl-skip{font-size:.7rem;font-weight:600;font-variant-numeric:tabular-nums;padding:5px 7px}
.pl-rate{display:flex;align-items:center;gap:1px;border:1px solid #3f3f46;border-radius:4px;padding:1px}
.pl-rate .pl-btn{font-size:.74rem;padding:2px 5px}
.pl-rate #bRate{font-size:.68rem;font-weight:600;padding:2px 5px;min-width:3.2em}
```

Seek, paint and drag:

```js
const au = document.getElementById('au');
const seek = document.getElementById('seek');
let scrubbing = false;
const fmt = s => {
  if (!isFinite(s) || s < 0) s = 0;
  s = Math.floor(s);
  const h = Math.floor(s / 3600), m = Math.floor(s % 3600 / 60), x = s % 60;
  const p = n => String(n).padStart(2, '0');
  return (h ? h + ':' + p(m) : p(m)) + ':' + p(x);
};
function paint(){
  const d = au.duration || 0, c = au.currentTime || 0;
  if (!scrubbing && d > 0) seek.value = Math.round(c / d * 1000);
  seek.style.setProperty('--p', (seek.value / 10) + '%');
  document.querySelectorAll('#tCur,#tCur2').forEach(el => el.textContent = fmt(c));
  const end = document.getElementById('tEnd');
  if (end) end.textContent = fmt(d);
}
au.addEventListener('loadedmetadata', paint);
au.addEventListener('timeupdate', paint);
au.addEventListener('seeked', paint);
// Drag: preview the fill on input, commit currentTime on change.
seek.addEventListener('pointerdown', () => scrubbing = true);
seek.addEventListener('input', () => seek.style.setProperty('--p', (seek.value / 10) + '%'));
seek.addEventListener('change', () => {
  if (au.duration) au.currentTime = (seek.value / 1000) * au.duration;
  scrubbing = false;
  paint();
});
```

Speed stepper — one array, one clamp, one label:

```js
const RATES = [0.75, 1, 1.25, 1.5, 1.75, 2, 2.25, 2.5, 2.75, 3];
const rateLabel = r => String(+r.toFixed(2)) + '×';
function applyRate(r) {
  r = Math.max(RATES[0], Math.min(RATES[RATES.length - 1], Math.round(r * 100) / 100));
  au.playbackRate = r;
  const b = document.getElementById('bRate');
  b.textContent = rateLabel(r);
  b.setAttribute('aria-label', 'Velocidad de reproducción: ' + rateLabel(r) + '. Clic para volver a 1×');
}
function setRate(dir) {
  const r = au.playbackRate || 1;
  let i = RATES.findIndex(x => Math.abs(x - r) < 0.001);
  if (i < 0) { i = 0; while (i < RATES.length - 1 && RATES[i] < r) i++; }
  i = Math.max(0, Math.min(RATES.length - 1, i + dir));
  applyRate(RATES[i]);
  toast('Velocidad ' + rateLabel(RATES[i]));
}
document.getElementById('bRateDown').addEventListener('click', () => setRate(-1));
document.getElementById('bRateUp').addEventListener('click', () => setRate(1));
document.getElementById('bRate').addEventListener('click', () => { applyRate(1); toast('Velocidad 1×'); });
```

Prev/Next inside the current block — the block owning `currentTime`, its `data-ts` marks sorted and deduped, then the nearest mark on the requested side:

```js
function blockAt(t) {
  let b = null;
  for (const el of blocks) if (t >= (+el.dataset.start || 0)) b = el;
  return b;
}
function blockMarks(b) {
  if (!b) return [];
  const out = $$('[data-ts]', b).map(el => parseFloat(el.dataset.sec))
    .filter(s => isFinite(s)).sort((a, c) => a - c);
  return out.filter((s, i) => i === 0 || s - out[i - 1] > 0.5);
}
function jumpTs(dir) {
  const t = au.currentTime || 0;
  const b = blockAt(t);
  if (!b) return;
  const marks = blockMarks(b);
  if (!marks.length) { toast('Este bloque no tiene marcas de tiempo'); return; }
  let target = null;
  if (dir > 0) target = marks.find(s => s > t + 0.5);
  else for (const s of marks) { if (s < t - 0.5) target = s; }
  if (target == null) {
    if (dir > 0) { toast('Último apunte del bloque'); return; }
    target = (+b.dataset.start || 0) + 0.1;
  }
  seekTo(target);
  toast((dir > 0 ? 'Siguiente' : 'Anterior') + ' · ' + fmt(target));
}
document.getElementById('bPrev').addEventListener('click', () => jumpTs(-1));
document.getElementById('bNext').addEventListener('click', () => jumpTs(1));
```

Keyboard — one handler, delegate to the same functions the buttons call:

| Key | Action |
|---|---|
| `Space` / `K` | Play-pause |
| `←` / `J` | −10 s |
| `→` / `L` | +10 s |
| `↑` / `↓` | Speed ±0.25× (clamped to `0.75×–3×`) |
| `N` / `P` | Next / previous timestamp **in the current block** |
| `Shift+N` / `Shift+P` | Next / previous block |
| `M` | Mute |
| `C` | Captions |
| `Shift+R` | Collapse the player |

```js
document.addEventListener('keydown', e => {
  if (e.target.matches('[data-ts]')) {
    if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); seekTo(parseFloat(e.target.dataset.sec)); }
    return;
  }
  if (e.target.matches('input,textarea,select,button') || e.target.isContentEditable) return;
  const k = e.key.toLowerCase();
  if (k === ' ' || k === 'k') { e.preventDefault(); togglePlay(); }
  else if (k === 'arrowleft' || k === 'j') { e.preventDefault(); skip(-10); }
  else if (k === 'arrowright' || k === 'l') { e.preventDefault(); skip(10); }
  else if (k === 'arrowup' || k === 'arrowdown') { e.preventDefault(); setRate(k === 'arrowup' ? 1 : -1); }
  else if ((k === 'n' || k === 'p') && e.shiftKey) { e.preventDefault(); jumpBlock(k === 'n' ? 1 : -1); }
  else if (k === 'n' || k === 'p') { e.preventDefault(); jumpTs(k === 'n' ? 1 : -1); }
  else if (k === 'm') { au.muted = !au.muted; toast(au.muted ? 'Silencio' : 'Con sonido'); }
  else if (k === 'c') toggleCC();
  else if (k === 'r' && e.shiftKey) { e.preventDefault(); setCollapsed(!player.classList.contains('collapsed')); }
});
```

`skip(sec)`, `togglePlay()`, `setCollapsed(v)`, `jumpBlock(dir)` and `toast(msg)` are the player's small helpers: skip = `seekTo(currentTime + sec)`; jumpBlock seeks to the adjacent block's `data-start` and toasts its `h2`. The transport buttons put their shortcut in `title`, so the keyboard map is discoverable without a legend elsewhere.

Gotchas:
- Audio `src` must be set (file or data-URI) and `preload="metadata"` at minimum — an empty `<audio>` makes every seek a silent no-op and `duration` stays `NaN`.
- Guard `duration` before dividing: `if (au.duration)` before `currentTime = …`. `NaN` seeks throw the bar to 0.
- The `scrubbing` flag is what makes dragging work: while the thumb is down, `timeupdate` must NOT rewrite `seek.value`, or the thumb snaps back under the finger.
- Commit on `change`, preview fill on `input`. Setting `au.currentTime` on every `input` event stutters and fires seeks faster than the decoder can follow.
- Times live outside the range input as `.pl-t` spans; the fill is `--p` on the input's background.
- `RATES` is the single source of allowed speeds; never hardcode a second list in the click handler and the keyboard handler.
- Prev/Next use `t + 0.5` / `t - 0.5` so the mark already speaking does not count as the next one. At the block edges: Next toasts *Último apunte del bloque*; Prev falls back to the block start.
- `blockMarks` includes `data-ts` inside `<details>`; hidden marks are still valid navigation targets.

Subtitles: fetch the VTT once, parse locally, render into a floating overlay above the player that tracks the collapsed state; `CC` toggles.

Gotchas:
- `const $` first: one thrown statement kills every listener after it, silently.
- Parse VTT times with `vttSec` (`parseFloat` of the last segment). The SRT comma parser yields `NaN` on them → subtitles that never match.
- Times live outside the range input as `.pl-t` spans; the fill is `--p` on the input's background.
- State via classes (`.on`, `.now`, `.speaking`), not inline styles.
- The page reserves bottom padding (~120px) so the fixed bar never covers content.
