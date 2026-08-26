# `SettingsData.{parent,colony}_episode_goals` (`EpisodeGoalData`): only `SetAsideGoods` is decoded

`Adventure`'s `episode_goals: Vec<EpisodeGoal>` is always `vec![]` today - not wired up at all. Source
struct: `EpisodeGoalData.resource_id`/`amount` are confirmed only for `goal_type == 14`
(`SetAsideGoods`) - cross-checking an old-vs-new-format resave pair (`Test` vs `The Odyssey`, see
`DATA_MAPPING.md`) confirms this both ways: for `goal_type == 14` slots, `resource_id`'s old-vs-new byte
delta always matches `ResourceType::try_resolve_old_format`'s shift table exactly; for every other
`goal_type` seen (`0`, `1`, `2`, `7`), the delta on that same byte offset does *not* match the table at
all (e.g. raw `9` -> `8`, `1500` -> `2000`, `5000` -> `7500`, `7` -> `8`) - confirming those bytes aren't
a `ResourceType` id for those goal kinds, but not yet enough signal to say what they *are* instead.

This is also the concrete reason `episode_goals` matters beyond itself: a Tier-3 finding elsewhere
established that raising/lowering `SettingsData.parent_episodes` has **no effect** on how many parent
episodes actually appear in-game - the leading hypothesis for what actually drives episode progression
is per-episode chaining via `next_episode`/whether that episode's goals were met, which needs
`episode_goals` decoded to test.

## Next steps

- Survey `EpisodeGoalData` fields across every `goal_type` value seen in the real corpus, the same way
  `goal_type == 14`'s `resource_id`/`amount` were originally isolated - group by `goal_type`, look for
  a value at a consistent offset that varies plausibly for that goal kind (population target, building
  count, favour threshold, etc.).
- Once decoded, test the "episode progression is goal-driven, not count-driven" hypothesis directly:
  find or construct a case where a parent episode's goal is/isn't met and see if `next_episode`
  resolution differs.
