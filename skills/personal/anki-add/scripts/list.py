#!/usr/bin/env python3
"""Show decks, notetypes (with exact field names and templates), or existing notes."""

import argparse
import sys
from pathlib import Path

DEFAULT_COLLECTION = Path.home() / ".local/share/Anki2/User 1/collection.anki2"


def resolve(path_arg: str | None) -> Path:
    col = Path(path_arg).expanduser() if path_arg else DEFAULT_COLLECTION
    if not col.is_file():
        sys.exit(f"Collection not found: {col}")
    return col


def strip_html(text: str, limit: int = 160) -> str:
    import re

    text = re.sub(r"<[^>]+>", "", text or "")
    text = text.replace("&nbsp;", " ").replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
    text = " ".join(text.split())
    return text[:limit] + ("…" if len(text) > limit else "")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--collection", help="path to collection.anki2")
    ap.add_argument("--notes", metavar="DECK", help="list notes in this deck")
    ap.add_argument("--limit", type=int, default=20, help="max notes to print (default 20)")
    args = ap.parse_args()

    from anki.collection import Collection

    col = Collection(str(resolve(args.collection)))

    if args.notes:
        deck_id = col.decks.id_for_name(args.notes)
        if deck_id is None:
            col.close()
            sys.exit(f"No deck named {args.notes!r}")
        query = f'deck:"{args.notes}"'
        nids = col.find_notes(query)
        print(f"{len(nids)} note(s) in {args.notes}\n")
        for nid in nids[: args.limit]:
            note = col.get_note(nid)
            type_name = note.note_type()["name"]
            fields = " | ".join(
                f"{name}={strip_html(note[name], 80)}"
                for name in note.keys()
                if note[name]
            )
            ords = sorted({c.ord for c in note.cards()})
            print(f"[{nid}] {type_name} (cards: {ords})")
            print(f"    {fields}")
            if note.tags:
                print(f"    tags: {' '.join(note.tags)}")
        if len(nids) > args.limit:
            print(f"\n… {len(nids) - args.limit} more (raise --limit)")
        print(f"\nCollection: {col.note_count()} notes, {col.card_count()} cards")
        col.close()
        return 0

    print("DECKS")
    for name_id in col.decks.all_names_and_ids():
        n = len(col.find_cards(f'deck:"{name_id.name}"'))
        print(f"  {name_id.name:<40} {n:>5} card(s)")

    print("\nNOTETYPES")
    for name_id in col.models.all_names_and_ids():
        model = col.models.get(name_id.id)
        fields = [f["name"] for f in model["flds"]]
        templates = [t["name"] for t in model["tmpls"]]
        print(f"  {name_id.name}")
        print(f"      fields:    {' | '.join(fields)}")
        print(f"      templates: {' | '.join(templates)}")
        if len(templates) > 1:
            print(f"      NOTE: {len(templates)} templates -> {len(templates)} cards per note")

    print(f"\nCollection: {col.note_count()} notes, {col.card_count()} cards")
    col.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
