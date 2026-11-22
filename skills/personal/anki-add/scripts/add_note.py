#!/usr/bin/env python3
"""Add a note to a deck, creating the deck first if needed.

Refuses to run while Anki is open. Prints what it did and verifies it after
committing, so a silent field-name mismatch cannot pass as success.
"""

import argparse
import subprocess
import sys
from pathlib import Path

DEFAULT_COLLECTION = Path.home() / ".local/share/Anki2/User 1/collection.anki2"


def ensure_anki_closed() -> None:
    preflight = Path(__file__).with_name("preflight.py")
    result = subprocess.run([sys.executable, str(preflight)], capture_output=True, text=True)
    if result.returncode != 0:
        sys.stderr.write(result.stdout + result.stderr)
        sys.exit(result.returncode or 1)


def parse_fields(pairs: list[str]) -> dict[str, str]:
    fields = {}
    for pair in pairs:
        if "=" not in pair:
            sys.exit(f"--field expects key=value, got {pair!r}")
        key, _, value = pair.partition("=")
        fields[key.strip()] = value
    return fields


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--deck", required=True, help="deck name; nested names like 'Historia::México' work")
    ap.add_argument("--notetype", default="Basic", help="notetype name (default: Basic)")
    ap.add_argument(
        "--field",
        action="append",
        default=[],
        metavar="NAME=VALUE",
        help="field assignment; repeat per field",
    )
    ap.add_argument("--tags", nargs="*", default=[], help="space-separated tags")
    ap.add_argument(
        "--keep-ord",
        type=int,
        default=None,
        metavar="N",
        help="keep only the card built from template index N; delete the others",
    )
    ap.add_argument("--collection", help="path to collection.anki2")
    ap.add_argument("--dry-run", action="store_true", help="show the plan without writing")
    args = ap.parse_args()

    if not args.dry_run:
        ensure_anki_closed()

    col_path = Path(args.collection).expanduser() if args.collection else DEFAULT_COLLECTION
    if not col_path.is_file():
        sys.exit(f"Collection not found: {col_path}")

    from anki.collection import Collection

    col = Collection(str(col_path))

    model = col.models.by_name(args.notetype)
    if model is None:
        available = [m.name for m in col.models.all_names_and_ids()]
        col.close()
        sys.exit(f"Unknown notetype {args.notetype!r}. Available: {', '.join(available)}")

    field_values = parse_fields(args.field)
    known = [f["name"] for f in model["flds"]]
    unknown = set(field_values) - set(known)
    if unknown:
        col.close()
        sys.exit(
            f"Unknown field(s) for notetype {args.notetype!r}: {', '.join(sorted(unknown))}\n"
            f"Valid fields: {', '.join(known)}"
        )
    # Filling no fields produces an invisible note; refuse instead.
    if not any(v.strip() for v in field_values.values()):
        col.close()
        sys.exit("All supplied fields are empty; refusing to create a blank note.")

    templates = [t["name"] for t in model["tmpls"]]
    would_create = len(templates)
    order = args.keep_ord if args.keep_ord is not None else None
    if order is not None and not 0 <= order < len(templates):
        col.close()
        sys.exit(f"--keep-ord {order} out of range; {args.notetype!r} has {len(templates)} template(s)")

    deck_before_exists = col.decks.id_for_name(args.deck) is not None

    print(f"Collection:  {col_path}")
    print(f"Deck:        {args.deck} ({'existing' if deck_before_exists else 'will be created'})")
    print(f"Notetype:    {args.notetype} — templates: {', '.join(templates)}")
    print(f"Cards:       {would_create}" + (f" -> keeping ord {order} only" if order is not None else ""))
    for key, value in field_values.items():
        preview = value if len(value) <= 100 else value[:100] + "…"
        print(f"  {key} = {preview}")
    if args.tags:
        print(f"Tags:        {' '.join(args.tags)}")

    if args.dry_run:
        print("\nDry run — nothing written.")
        col.close()
        return 0

    deck_id = col.decks.id(args.deck)
    note = col.new_note(model)
    for key, value in field_values.items():
        note[key] = value
    note.tags = args.tags

    col.add_note(note, deck_id)

    if order is not None:
        extra = [c.id for c in note.cards() if c.ord != order]
        if extra:
            col.remove_cards_and_orphaned_notes(extra)

    # Verify by reading back through the API, not by trusting add_note.
    stored = col.get_note(note.id)
    cards = [(c.id, c.ord, col.decks.name(c.did)) for c in stored.cards()]
    print(f"\nAdded note {stored.id}")
    for cid, cord, cdeck in cards:
        print(f"  card {cid} ord={cord} deck={cdeck}")

    blanks = [name for name in stored.keys() if not stored[name].strip()]
    if blanks:
        print(f"  WARNING: empty fields: {', '.join(blanks)}")

    print(f"Collection now: {col.note_count()} notes, {col.card_count()} cards")
    col.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
