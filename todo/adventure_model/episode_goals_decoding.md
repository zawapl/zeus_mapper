# Does episode progression actually depend on `episode_goals` being met?

`episode_goals` itself is fully decoded and wired up: every real `goal_type` value seen across the
full corpus (`{0,1,2,3,4,5,6,7,8,9,10,14,15,16}`, 65 adventures) resolves via `EpisodeGoal::from_raw_fields`,
and resolved `EpisodeGoal` counts match `parent_episode_goal_counts`/`colony_episode_goal_counts` exactly
everywhere a colony/parent episode actually exists (see `episode_goals.rs`, `DATA_MAPPING.md`).

What's still open is the reason this mattered in the first place: a Tier-3 finding elsewhere established
that raising/lowering `SettingsData.parent_episodes` has **no effect** on how many parent episodes
actually appear in-game. The leading hypothesis is per-episode chaining via `next_episode`/whether that
episode's goals were met - now testable since `episode_goals` is decoded, but not yet tested.

## Next steps

Find or construct a case (in-game save/editor) where a parent episode's goal is/isn't met and see if
`next_episode` resolution differs. This needs live-game observation, not a file survey.
