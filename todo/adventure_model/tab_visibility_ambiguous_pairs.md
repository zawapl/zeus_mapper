# `SettingsData.tab_visibility`: 3 ambiguous byte-to-tab assignments

`tab_visibility: [u8; 11]` gates which of the game's 11 building tabs are visible. 3 of 11 byte-to-tab
assignments are uniquely confirmed (offsets `2`/`7`/`10` -> tabs `3`/`8`/`11`, each independently
matched with no ambiguity). The other 8 rest on the "byte `i` = tab `i+1`" positional hypothesis: 3
ambiguous pairs whose two members always toggled together in every sample seen (offsets `{1,3}`,
`{5,8}`, `{6,9}` -> tabs `2`/`4`, `6`/`9`, `7`/`10`), plus 2 tabs that never varied at all because
they're apparently always visible.

**Parked** (per user direction - don't push on this unprompted, wait for it to come back up): the 16
known tutorial `.pak` files never split any of the 3 ambiguous pairs, so disambiguating them needs
either (a) an in-game observation of an adventure/tutorial where exactly one tab of a pair is visible
without the other, or (b) finding `.sav`'s own tab-visibility representation and cross-referencing
against checkpoints there - confirmed *not* a byte-for-byte copy of the `.pak` array (exact-match byte
search across all 16 `tutorial00-15.sav` files against every known `tab_visibility` value found zero
hits), so this would need genuinely new decoding, not a lookup.

Test adventures for manual in-game disambiguation were generated at one point (copies of a known
"all tabs visible" `.pak` with one offset at a time flipped, `adventure_type` set to `ZeusCustom` so
they show up outside the tutorial menu) but results were never reported back.

## Next steps

Revisit once `.sav` gets more general-purpose investigation (see `sav_file/` in this todo folder) -
tab visibility itself hasn't been searched for there yet even though `SavData` now has much more
structure decoded than when this was last touched.
