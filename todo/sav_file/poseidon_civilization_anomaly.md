# Two `.sav` files with an unexplained combination of flags

`SavData::civilization()` derives `Civilization` from `poseidon_marker` (`1` iff the loaded episode's
`adventure_type` is Poseidon-published) and `poseidon_flag` (`0` iff Poseidon-published *and*
`civilization == Greek`) - `Atlantean` iff both are `1`, `Greek` otherwise. This resolves correctly on
all 28 `Save Tests/*.sav` files that have a matching `.pak` to check against.

`Save Tests/Proteus and Bellerophon.sav` and `Save Tests/Two Worlds Collide.sav` are the only 2 files
(of 28) where the underlying `.pak` is `PoseidonCampaign` (Poseidon-published) but ships with
`civilization == Greek` - a real, rare, user-confirmed-correct combination, not a bug in the resolution
logic. Both are also the only 2 non-tutorial files in the full 62-file sample with `poseidon_flag == 0`
(every other non-tutorial file has `poseidon_flag == 1`).

**Open question**: is there anything else distinctive about these two specific episodes/saves that
explains why they're the only ones in this corner case, or is it purely coincidental that both examples
of "Poseidon-published but Greek" happen to be the only two in the sample? Not blocking anything -
`SavData::civilization()`/`PakData::civilization()` already resolve both correctly - but worth a closer
look before assuming the derived-`civilization` rule generalizes safely beyond these two data points
(only 2 positive examples of this corner case exist to test against).

## Next steps

- Check whether `field_1`/`version_1` or any other `manifest`/settings-level field singles these two
  out beyond the already-checked `adventure_type`/`civilization` pair.
- If more Poseidon-published-but-Greek episodes turn up in a larger sample, re-run the
  `poseidon_marker`/`poseidon_flag` derivation against them to firm up confidence beyond n=2.
