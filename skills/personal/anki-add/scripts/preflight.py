#!/usr/bin/env python3
"""Check that Anki is closed and locate the collection file.

Exits 1 if Anki is running (writing now would lose or corrupt data).
"""

import os
import platform
import subprocess
import sys
from pathlib import Path

# Match the Anki executable, not any command line that merely mentions anki.
ANKI_EXECUTABLES = (
    "anki",
    "anki-bin",
    "anki.x86_64",
    "aqt",
    "anki-launcher",
)


def anki_running() -> list[str]:
    """Return running Anki processes, ignoring our own Python scripts.

    pgrep -f is too eager: a script living in a directory called `anki-add`
    matches its own command line. So inspect the executable name instead, and
    skip every python process (ours, and Anki's own bundled python if any).
    """
    try:
        out = subprocess.run(
            ["ps", "-eo", "pid=,comm=,args="],
            capture_output=True,
            text=True,
            timeout=10,
        ).stdout
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return []

    hits = []
    for line in out.splitlines():
        parts = line.split(None, 2)
        if len(parts) < 2:
            continue
        comm = os.path.basename(parts[1]).lower()
        if comm.startswith("python") or "/python" in comm:
            continue
        if comm in ANKI_EXECUTABLES:
            hits.append(line.strip())
    return hits


def anki2_base() -> Path | None:
    override = os.environ.get("ANKI2_BASE")
    if override:
        return Path(override).expanduser()
    system = platform.system()
    if system == "Darwin":
        return Path.home() / "Library" / "Application Support" / "Anki2"
    if system == "Windows":
        appdata = os.environ.get("APPDATA")
        return Path(appdata) / "Anki2" if appdata else None
    return Path.home() / ".local" / "share" / "Anki2"


def find_collections() -> list[tuple[Path, Path]]:
    """Return (profile_dir, collection_path) for every profile found."""
    base = anki2_base()
    if base is None or not base.is_dir():
        return []
    found = []
    for child in sorted(base.iterdir()):
        if not child.is_dir():
            continue
        col = child / "collection.anki2"
        if col.is_file():
            found.append((child, col))
    return found


def main() -> int:
    running = anki_running()
    if running:
        print("ERROR: Anki appears to be running:", file=sys.stderr)
        for line in running:
            print("  " + line, file=sys.stderr)
        print(
            "\nClose Anki completely, then re-run this script.\n"
            "Writing while Anki is open loses or corrupts the changes.",
            file=sys.stderr,
        )
        return 1

    print("Anki is not running. Safe to write.")
    collections = find_collections()
    if not collections:
        base = anki2_base()
        print(f"\nNo collection found under {base}", file=sys.stderr)
        return 2

    if len(collections) == 1:
        profile, col = collections[0]
        print(f"Profile: {profile.name}")
        print(f"Collection: {col}")
        print(f"Size: {col.stat().st_size:,} bytes")
        return 0

    print("\nMultiple profiles found — pass the right path to the other scripts:")
    for profile, col in collections:
        print(f"  {profile.name:<12} {col}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
