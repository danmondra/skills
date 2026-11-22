---
name: anki-add
description: Add decks and flashcards to the user's local Anki collection safely, through the official Python `anki` library. Use when the user asks to create cards, add notes to a deck, make a new deck or subdeck, or import content into Anki.
---

# Anki: add decks and cards

Add decks and notes to the user's local Anki collection using the **official `anki` Python library**. Never hand-write SQLite rows: Anki needs `csum`, `sfld`, `cards`, `col` and `revlog` bookkeeping in sync, and getting it wrong leaves the collection inconsistent.

## Hard rules

1. **Anki must be closed.** It keeps the collection in memory and flushes on quit, so writing while it runs loses or corrupts changes. Always run `scripts/preflight.py` first; if Anki is running, stop and ask the user to close it. Do not "try anyway".
2. **Never edit the collection with `sqlite3`.** If you need to inspect, either use the library or copy the file to a temp dir and read the copy.
3. **Never name a script `inspect.py`** (or any other stdlib module name: `json.py`, `types.py`, `random.py`). Python puts the script's directory first on `sys.path`, so `anki/collection.py` importing `inspect` picks up your script and dies with a circular-import `ImportError`. This costs real debugging time every single time.
4. **Write, then verify.** Never report success from `add_note` alone — reopen the collection and read the note back (see step 5).

## Paths

Default collection (single profile):

```
~/.local/share/Anki2/User 1/collection.anki2
```

`prefs21.db` in `~/.local/share/Anki2/` lists profiles; multiple profiles mean multiple `User N/` folders. `preflight.py` resolves this and reports what it found. On macOS the base is `~/Library/Application Support/Anki2`.

## Environment

Use a dedicated venv, not the system Python and not a pipx app:

```bash
VENV="$HOME/.local/share/anki-skill/venv"
[ -d "$VENV" ] || python3 -m venv "$VENV"
"$VENV/bin/pip" -q install --upgrade anki
```

`anki` is a large wheel (~200 MB with deps). Installing it once in a stable location pays for itself across sessions.

## Workflow

### 1. Preflight

```bash
python3 scripts/preflight.py
```

Exits non-zero if Anki is running, and prints the resolved collection path. Do that first, every time.

### 2. Inspect before writing

```bash
"$VENV/bin/python" scripts/list.py            # decks + notetypes + fields + templates
"$VENV/bin/python" scripts/list.py --notes "Historia::México"   # existing notes in a deck
```

Read the output before choosing a notetype. Two things bite:

- **Field names must match exactly** (`Spanish Translation`, `Short Description`, `q1`). A typo silently creates an empty field rather than failing.
- **Templates decide card count.** A notetype with two templates produces two cards per note. The user's `Basic` has both `Card 1` and a second reversed-style `Card 2`, so adding one note yields two cards unless you say otherwise.

Reuse an existing deck when it fits. Create a new one only when the content has no home. Nested names create parents automatically: `col.decks.id("Historia::México")` also creates `Historia`.

### 3. Add the note

```bash
"$VENV/bin/python" scripts/add_note.py \
  --deck "Historia::México" \
  --notetype Basic \
  --field 'Front=¿Cómo se llamaba…?' \
  --field 'Back=El <b>Palacio de Axayácatl</b>…' \
  --tags Historia México Conquista \
  --keep-ord 0
```

`--keep-ord 0` deletes the extra cards when the notetype generates more than one, keeping only the card built from template index 0. Omit it when every template's card is wanted.

For anything beyond a few lines of HTML, pass the content via a Python heredoc calling `col.new_note()` / `col.add_note()` directly — shell quoting around `<b>`, `«»` and `&nbsp;` is where mistakes happen.

### 4. Deleting stray cards

`Collection` has no `rem_cards` (the attribute is looked up through `__getattr__` and raises `AttributeError`). The method is:

```python
col.remove_cards_and_orphaned_notes([card_id, ...])
```

It only removes orphaned notes, so the parent note survives as long as one card remains.

### 5. Verify

```bash
"$VENV/bin/python" scripts/verify.py --deck "Historia::México"
```

Confirm deck name, Front/Back, tags, the cards' ordinals and owning deck, and the collection-wide `note_count` / `card_count` before and after. Report the actual numbers to the user.

## Content conventions for this collection

- Front asks the question; Back answers it. Do not put the answer in Front.
- Write in Spanish unless the user says otherwise.
- Formatting matches the existing cards: `<b>`, `<br>`, `<u>`. No Markdown — Anki renders it literally inside HTML fields.
- Tags are flat, no nesting, in Spanish: `Historia`, `México`, `Conquista`.
- Cloze notetype fields are `Text` and `Back Extra`; cloze syntax is `{{c1::answer}}`.

## When the user supplies a URL as the source

Fetch it, extract the relevant prose, and then:

1. **Cite the source in the Back**, at the end, as a link: `<br><br><i>Fuente: <a href="URL">infobae</a></i>`.
2. Only state what the source actually says. If the source hedges ("posiblemente el 29 de junio de 1520"), keep the hedge in the answer — do not upgrade it to a fact.
3. If the source disagrees with what the user asked for, flag the discrepancy and ask before writing.

```bash
curl -sL --max-time 30 -A "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36" "$URL" -o /tmp/anki_source.html
```

Then strip `<p>` tags in Python (`re` + `html.unescape`) rather than trying to read the raw HTML by eye.

## Failure modes seen in practice

| Symptom | Cause | Fix |
| --- | --- | --- |
| `ImportError: cannot import name 'Collection' from partially initialized module 'anki.collection'` | Script named `inspect.py` | Rename the script |
| `AttributeError: 'Collection' object has no attribute 'rem_cards'` | API renamed | `remove_cards_and_orphaned_notes` |
| Two cards per note, only one wanted | Notetype has multiple templates | Delete the extra ords |
| Changes vanish after opening Anki | Anki was running during the write | Preflight; redo with Anki closed |
| Card exists but fields empty | Field name mismatch | `list.py` shows exact names |

## Not covered

- AnkiWeb sync and AnkiDroid.
- Editing note types or templates (this skill consumes them, never rewrites them).
- Media/audio files (`[sound:...]`). The collection uses them; adding new files needs `col.media.add_file()` plus a matching field reference, which is a separate job.
- `.apkg` export. If the user wants to review before importing, generate one with `genanki` instead of touching the collection.
