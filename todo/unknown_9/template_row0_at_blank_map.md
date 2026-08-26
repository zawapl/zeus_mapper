# Is `unknown_9` row0's value on a fresh blank map a fixed template?

A genuinely fresh "New Map" (size 226) observed row0 as `[22462, 0, 3, 27, 57, 34, 63, 43, 0]` once on a
colony slot. A separate "clean replacement" of a *parent* map (different test, "Youngest 2") did **not**
reproduce this - current best explanation is that the first sighting may just have been whatever the
colony's content already was before that investigation session's edits started, not something "New
Map"/replace actively stamps in. Unconfirmed either way.

Side finding from the same tests, unrelated to `unknown_9` itself but useful context: two independently
created "clean" size-226 maps produced byte-identical `elevation` (100% uniform `0`), `terrain` (exactly
3 distinct values), and `root_offset` nonzero counts (25,764 tiles both times) - so the "New Map"
generator is deterministic per size, not randomized. A "blank map at size 226" has a recognizable,
reproducible signature independent of this specific question.

## Next steps

Create a *new* blank colony map from scratch in a fresh test adventure (in-editor) and check whether
`unknown_9` row0 is `[22462, 0, 3, 27, 57, 34, 63, 43, 0]` again:

- Same value again -> strong constant-template result (like `MapData.constant_3`), worth then varying
  map size/civilization/tropical to see if/how the template value depends on them.
- Different value -> check whether it varies with `map_size` specifically, which would still indicate a
  template (just parameterized) rather than ruling the idea out.
