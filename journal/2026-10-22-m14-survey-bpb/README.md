# M14: survey reads Untamed's scenes

`tools/survey` now decodes a map's `<MAP>.scn.bpb` with `tools/bpb` (`bpb::parse`, then `to_ini`) and
reads the text with openmx's `Scene::parse`, the same path a text `.scn` takes. So the scene columns of
Untamed's table (scene name, terrain keys, models, collision placements and where the placed `.col`
are) are filled like MX vs. ATV Unleashed's. The scene cell says `(BPB, n B)` when the scene came from a
BPB. A BPB that `bpb::parse` refuses is a refused scene (`refused: bpb: ...`), not a missing one.
`SceneRow::Bpb` ("not read (M10)") is gone, and `SceneRow::Ini` has a new `bpb: Option<usize>`.

survey now depends on bpb. bpb already dev-depends on survey (its disc test uses `res_view`). Cargo
allows that dev-dependency cycle, and `./check.sh` builds both.

## Proof

- `an_untamed_bpb_scene_is_decoded_and_gives_the_same_row` (runs in the gate): the generated map of
  `tests/row.rs` with its INI scene swapped for the same sections built as a `bpb::Bpb` and written by
  `bpb::write`, in a `.res` with a padded directory, gives the same row as the INI map, apart from the
  scene cell. A file that is not a BPB is reported as refused.
- `on_untamed_every_scene_is_read_from_its_bpb` (ignored, the disc): all 48 archives and the loose
  SM_IP. Each scene is read from its BPB, and each map's models and collision placements equal the
  `[ModelN]` with an `SLT` and the `[ObjectN]` with a `COL` in the decoded sections. The total is
  77,542, M10's count over the loose scenes. It also looks for each placed `.col` that the row reports
  missing in every other map archive. None is there.
  `DISCX_ISO=/home/rycs/repos/openmx-mods/iso cargo test -p survey --release -- --ignored`: 3 passed.
- `survey untamed` was run again, and its map table replaced the one in `notes/mxvatv-later.md`. A
  script checked that the terrain and `.col` columns of every row were unchanged before the paste.
  Only the four scene columns differ.

## Found

In `notes/mxvatv-later.md`, *Survey tables*:

- Seven placed `.col` (12 placements over six maps) are nowhere on the disc as `.col`. Only their
  `Geo\*.psg` exist.
- 43 scenes name `TerrainFile=<MAP>.trn`, and SX_F–K name `<MAP>.ps2.trn`.
- Nat_OR's scene scale is 4.5, where its terrain file says 4.499999.
- SM_IP and TR_A place nothing.

## Not verified

- What the game does with the seven missing `.col` (skip, or no collision), and how it resolves
  `TerrainFile=<MAP>.trn` to `<MAP>.ps2.trn`. Neither was looked up in SLUS_217.01. Both are outside
  this task.
- That the running game places what `Scene::parse` counts. That needs PCSX2 (T7).
- The "loose" `.col` count only checks that the name exists in the PSR's top-level `Col\` (as for the
  other discs). The loose files' contents are read in the PSR `.col` table, but they are not matched
  to the placements.
- `survey res FILE.res` on an Untamed archive on disk was not run separately. It goes through the same
  `map_res` that the disc run uses.
