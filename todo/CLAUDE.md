# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with files in this folder.

## What this is

A generic punch list for the project: open items, one file per item, grouped into subfolders by area.
Everything in here right now happens to be a reverse-engineering question about `zeus_mapper`'s file
formats, since that's been the project's focus so far, but this folder isn't RE-specific - a build
task, a refactor, a doc gap, etc. belongs here the same way.

This is **not** a history log. Confirmed findings/completed work belong wherever they actually live
(e.g. `zeus_mapper/src/adventure/DATA_MAPPING.md`, or as `///` docs / `validate()` checks on the
relevant struct for format-decoding items), not here.

## Conventions

- **When you resolve a todo, delete its file.** Promote whatever you learned/built to wherever it
  actually belongs (naming a field, adding a `validate()` check, updating `DATA_MAPPING.md`, landing
  the code change, etc.), then remove the file from this folder. Don't leave a "RESOLVED" stub behind
  - git history already records that.
- **When you make partial progress**, rewrite the file in place with the new, smaller open question
  and whatever evidence/context is now relevant - don't keep appending a narrative of every attempt.
- **Don't point to a file in this folder from outside it** (code comments, `DATA_MAPPING.md`, other
  docs, memory). These files are meant to disappear once resolved, so a path reference to one is a
  dangling link waiting to happen. If you need to note that something is blocked on an open item,
  describe the open question itself instead of the filename - e.g. "waiting on `field_12`'s meaning"
  or "pending where `X` is stored," not "see `todo/foo/bar.md`."
- Field names cited in the format-investigation files can drift as investigations elsewhere rename
  `unknown_N`/`field_N` fields to real names. If a file references a field that no longer exists under
  that name, grep the relevant struct before trusting the file - it may be stale, or may itself have
  just been resolved.
- One-off investigation scripts (`examples/*.rs` used to survey real game files) are disposable and
  routinely deleted once an investigation session ends - don't assume a script named in one of these
  files still exists; treat it as "this is the kind of survey that answered this before," not a
  runnable reference.
- New folders should follow the same pattern as below: group by area/topic, not by when the item was
  added.

## Folders

- `sav_file/` - the `.sav` (live save) format, `SavData`/`UnitData`/`BuildingData`.
- `adventure_model/` - the `Adventure`/`CityMap` domain model built from `PakData`/`SettingsData`/
  `MapData`, and remaining unidentified bytes in those `file_data` structs generally.
- `unknown_9/` - `MapData.unknown_9`, a 10-row packed record with partially-decoded columns; kept as
  its own folder since it's a single field with enough sub-structure to need several linked todos.
