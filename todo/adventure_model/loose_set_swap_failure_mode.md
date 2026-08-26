# What exactly breaks when a loose `.set` is swapped in for a `.pak`'s embedded settings

Confirmed: a `.pak`'s own embedded `SettingsData` is authoritative, and a same-named loose `.set` file
found alongside it should be treated as stale/unrelated data - swapping the loose `.set` in wholesale
(on `Open Play Military 2`) breaks the colony transition (fails/crashes moving from the parent city into
the colony). `colony_episodes_used` was ruled out as the specific cause (flipping just that field alone
loads and plays fine, and doesn't even change how many colonies the episode-select screen offers).

**Still open**: which of the loose `.set`'s other divergent fields actually causes the failure, and
whether the failure is the same "old version and cannot be loaded" message the write-path bugs used to
produce or a different crash. Not blocking anything - the practical conclusion (ignore loose `.set`
files) already stands regardless of the exact mechanism - but worth chasing if someone wants to fully
understand loose-`.set` semantics rather than just avoiding them.

## Next steps

Binary-search the other fields that differ between the loose `.set` and the `.pak`'s embedded one
(flip them one at a time into an otherwise-untouched `.pak`, same method already used for
`colony_episodes_used`) until the failure reproduces, then note which field and what the actual failure
message/behavior is.
