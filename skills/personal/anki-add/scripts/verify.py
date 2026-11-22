#!/usr/bin/env python3
"""Read the collection back and prove what is actually stored.

Use after every write. Reports the deck, its notes with rendered-ish fields,
card ordinals and owning decks, and collection-wide counts.
"""

import argparse
import re
import sys
from pathlib import Path

DEFAULT_COLLECTION = Path.home() / ".local/share/Anki2/User 1/collection.anki2"


def plain(html: str, limit: int = 300) -> str:
    text = re.sub(r"<br\s*/?>", "\n", html or "")
    text = re.sub(r"<[^>]+>", "", text)
    for entity, char in (("&nbsp;", " "), ("&amp;", "&"), ("&lt;", "<"), ("&gt;", ">"), ("&quot;", '"')):
        text = text.replace(entity, char)
    text = "\n".join(line.strip() for line in text.splitlines() if line.strip())
    return text[:limit] + ("…" if len(text) > limit else "")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--deck", help="deck to inspect; omit for a collection summary")
    ap.add_argument("--note", type=int, help="specific note id")
    ap.add_argument("--collection", help="path to collection.anki2")
    args = ap.parse_args()

    col_path = Path(args.collection).expanduser() if args.collection else DEFAULT_COLLECTION
    if not col_path.is_file():
        sys.exit(f"Collection not found: {col_path}")

    from anki.collection import Collection

    col = Collection(str(col_path))

    if args.note:
        notes = [col.get_note(args.note)]
    elif args.deck:
        if col.decks.id_for_name(args.deck) is None:
            col.close()
            sys.exit(f"No deck named {args.deck!r}")
        notes = [col.get_note(nid) for nid in col.find_notes(f'deck:"{args.deck}"')]
    else:
        print(f"Collection: {col_path}")
        print(f"Notes: {col.note_count()}  Cards: {col.card_count()}")
        print(f"Decks: {len(col.decks.all_names_and_ids())}  Notetypes: {len(col.models.all_names_and_ids())}")
        col.close()
        return 0

    label = f"note {args.note}" if args.note else f"deck {args.deck}"
    print(f"{len(notes)} note(s) in {label}\n")

    for note in notes:
        print(f"[{note.id}] notetype={note.note_type()['name']}")
        for name in note.keys():
            value = note[name]
            if not value:
                print(f"  {name}: <empty>")
                continue
            rendered = plain(value)
            if "\n" in rendered:
                print(f"  {name}:")
                for line in rendered.splitlines():
                    print(f"      {line}")
            else:
                print(f"  {name}: {rendered}")
        print(f"  tags: {' '.join(note.tags) or '<none>'}")
        for card in note.cards():
            due = card.due
            state = card.type
            print(f"  card {card.id}: ord={card.ord} deck={col.decks.name(card.did)} type={state} due={due}")
        print()

    print(f"Collection now: {col.note_count()} notes, {col.card_count()} cards")
    col.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
