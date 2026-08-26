# `WorldMapElementData`/`TradeRoutePointData`: remaining unidentified bytes

Byte-density survey done across every real `.pak` (65 files).

## `WorldMapElementData` (2066 real `variant != 0` elements)

`sprite_id`, `data_a`, `data_d`, `unknown_1`, `unknown_2` have **zero inert bytes** - every single byte
column carries real, varying content across the full corpus (6-29 distinct values per column, several
spanning the full `0..=255` range). This rules out "mostly filler, a few real bytes" for this struct; a
byte-density pass can't narrow it further, it needs real correlation work. One correction to the
existing code comment: `data_a[0]` is **not** constant `2` - it's `2` in 75% of samples (1559/2066), 0 in
199, and 15 other scattered values (204, 3, 205, 29, ...) in the rest.

`TradeRoutePointData.unknown_1` (50 points x however many real routes exist, `exists != 0`; 9806 routes,
490300 points surveyed): resolved to a specific, non-random pattern. 99.98% of values are 0
(490191/490300). Of the 9806 routes, 9697 (99%) have `unknown_1 == 0` on every point. Of the 109 routes
where it varies, **every single one** has its only nonzero value at a point index `>= points_count` -
i.e. strictly in the route's unused trailing slots, never among the route's actual defined points. This
is consistent with `unknown_1` being uninitialized/leftover editor memory in slots the game never reads,
not real per-point path data. Distinct nonzero values seen: `211`, `7084`, `7087`, `7093`, `7096` - all
close together, which fits "stale leftover value from some other editor operation" better than
structured per-point data.

## Next steps

- `sprite_id`/`data_a`/`data_d`/`unknown_1`/`unknown_2`: since density surveying is exhausted here, next
  useful move is cross-referencing against fields already decoded on the same struct (`variant`,
  `region_name`, `label_position`) or against the sprite-sheet asset files named in the existing code
  comment, rather than more corpus-wide stats.
- `TradeRoutePointData.unknown_1`: treat as very likely inert/leftover rather than real per-point data
  unless a counterexample turns up; if pursued further, the interesting question is what leaves those
  specific values (`7084`-`7096`, `211`) behind in unused point slots, not what they mean per-point.
