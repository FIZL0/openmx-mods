# M23: stitch cuts each map at its ridge

openmx MD27 (the user, 2026-10-22 and 2026-10-23): a big world's maps should meet peak to peak, not
across two tiles' shootoff plateaus. `Tile::find_keep` stopped at the shootoff band on MX Unleashed's
freeride maps, so every join of world_mxu_free was ~675 ft of flat 190 ft plateau with a ridge on
either side. Now those maps are cut at their ring's ridge, the layout keeps every `at` at 0 or more,
world_mxu_free's joins are 135 ft wide, and openmx rides across every join and is thrown only at the
world's outer edge. Details in notes/stitch.md, *Ridge to ridge (M23)*.

## Log

- Imported MX Unleashed's 27 maps (`import-mxvatv --disc ".../MX Unleashed (USA).iso"`, 8 s) and read
  the edge profiles (`stitch profile`). All five freeride maps share one ring: 189.9 ft flat to d≈69
  samples (621 ft), the peak at d 88/87/87/88 (x0, z0, x1, z1; 239–251 ft), a valley near 110 ft
  by d 112.
- **First try: the brief's prototype rule** (the highest point within 1200 ft past the berm that
  stands more than 10 ft over it, and, my addition, falls more than 10 ft again inward). It gives the
  freeride maps `[88, 87, 681, 680]`, but it also moved the keep of 65 more of the 123 maps (every
  map in out/mods plus the 27 MX Unleashed ones), up to a quarter of the map into the track
  (mxvatv_nat07 `[250, 250, 1030, 1030]`, untamed_am_cs `[246, 263, 1030, 1038]`). That would cut
  tracks, not shootoff ground. Measured the flat run past each side's berm: the freeride maps have
  53–54 samples (477 ft) on every side. The other long ones are freestyle, supercross and
  Untamed's MT_A arenas (no world takes them) and mxu_nat04 side 0 (351 ft, but no ridge behind it).
- **The rule kept**: a ridge counts only behind a plateau, the profile within 1 ft of the berm's
  height for 300 ft or more (`RIDGE_PLATEAU`, `RIDGE_FLAT`). Only the five freeride maps change
  (`Tile::find_cut` gives the keep and which sides were ridge cuts; `find_keep` is its keep).
- **Negative `at`.** `plan` started the first row/column at `pad` (300 ft), so a keep 792 ft in put
  the origin at −432. Now the first line's cursor starts at the deepest cut among its tiles (centred
  as before), and each tile's `want` is never below its own cut. world_mxu_free: all `at` ≥ 0. The
  joins cover 0..size as always, so the outer band has no holes (the prototype's 39 holes came from
  moving the tiles but not the joins).
- **Ridge-top join width.** At the default `min_gap` 300 the gap was 423 ft (300 plus leaf rounding:
  each `at` is a whole 144 ft leaf). `gap_for`'s floor without `min_gap` is the two rooms,
  2 × (9 + 48) = 114 ft. `worlds/world_mxu_free.json` now sets `min_gap` 120, and the gaps come to
  135 ft. I did not change the default: the other worlds' gaps are set by height differences, and
  their layout is not this task's.
- **Seam slope 2.27.** Probed the built world through `Tiled::at` (a throwaway test, deleted): the
  steepest join cells are 32 ft outside free07/free09's x1 edge, one every 1152 ft along z. The
  heights show it is the extension carrying the crest's bumps outward: the edge's x-slopes alternate
  along z, so over the 48 ft fade neighbouring rows part and the join gets a z-slope the tile lacks
  there. It is C1's design (the first join cell has the tile's own slope), it is inside the seam
  gates, and the tiles' own cells within 200 ft of the cut reach 3.29. Left as is; noted.
- **Drops.** 45 models on the ring past the ridge (1 before). Their `why` now says "in the ring past
  the ridge it is cut at" where the side was a ridge cut, and the world's `todo` names each ridge-cut
  tile and its sides.
- Compared against HEAD's stitch (built from a temporary detached worktree under `target/`, removed
  after): world_mxvatv and world_untamed come out with the same piece counts, drops and slopes;
  world_mxu's 46.26 near-seam slope is the same before and after (not the freeride maps').
- Tests: `tests/ridge.rs` (in the gate, ~21 s debug): a generated ringed map is cut at its peak on all
  sides, a map that only climbs inward is not, and two ringed maps make a world with `at` ≥ 0, the gap
  within a leaf of `min_gap`, the todo lines, `stitch check` ok, no holes (3 ft) or strips (1 ft) as
  openmx loads it, and no dip across the join. `tests/openmx_worlds.rs`
  `world_mxu_free_meets_ridge_to_ridge` (ignored, the disc): the five keeps, `at` ≥ 0, the C11 ground
  check, holes and strips, one ridge across every join, and openmx rides.
- Rebuilt every world (`stitch --mods out/mods --out target/m23/stage`, 2 min 17 s) and replaced the
  five `world_*` in this slot's `out/mods/tracks/`, which held builds made before C11's fix.

## How it was proved

- `world_mxu_free_meets_ridge_to_ridge`, run with `DISCX_ISO` = the main checkout's iso/, the openmx
  binary in `extracted/openmx-target` (built 2026-10-23 14:43) and `OPENMX_WORLDS=target/m23/openmx-worlds`:
  - 4,800 tile points and 23,996 join points, 0 differ from the imported maps and the joins' files;
    11,758 join samples against stitch's field, 0 past the quantum; 0 of 373,272 edge points owned by
    nothing (plus `assert_no_strips` at 1 ft: 0).
  - Across the 5 joins (each 135 ft), on lines 50 ft apart and 300 ft clear of the ring's corners,
    the ground between the two ridges dips at most 5.8–6.7 ft under the lower one.
  - 10 rides on the gas across every join both ways: 10 reach the far tile, 0 thrown. 5 have one
    slow wreck (57–61 ft/s), each on the far tile's own ground 75–230 ft past the seam.
  - 10 rides straight out from 150 ft inside every outer side: 10 thrown, at 151–153 ft from the
    world's edge.
- `tests/ridge.rs` in `./check.sh`.
- `stitch check --iso` (the discs) on the five rebuilt worlds in out/mods/tracks: all ok.
  world_mxu_free 23,760 seam points (worst 0.00224 ft, slope step 0.000492), join slope 0.356 away
  from the seams; world_mxu 247,736 (0.0106 ft), 0.473; world_mxvatv 258,412, 0.469; world_untamed
  213,608, 0.493; world_ultra 719,756 seam points (0.0106 ft), 0.497. world_ultra took about 25 min,
  against ~5.5 min in the notes, while other runs shared the machine.
- One `--shot-pose` from 579 ft inside mxu_free06 towards mxu_free10 (Global sky, 85° fov): one dark
  ridge band with the next map's ground beyond. Looked at by eye; the picture is not kept.

## Not verified

- **openmx's own `crates/openmx/tests/world_joins.rs` (MD27) was not run.** The brief says MD27
  landed, but at this session it is not in openmx's main or `plan/DONE.md`. It is uncommitted work
  in openmx's slot s4 (`task/MD27`, its journal still a stub). I did not build another agent's live
  worktree. Equivalent rides were run with openmx main's binary in `world_mxu_free_meets_ridge_to_ridge`.
  When MD27 lands, run it on `OPENMX_WORLDS` filled by that test (or on out/mods).
- **openmx's `world holes:` line and MD27's cutting of shapes and models outside a keep** were not
  seen, for the same reason. Holes were counted with game-sim's `Tiled::owner` (`stitch::openmx::holes`
  and `strips`).
- **The main checkout's out/mods (the 84-hole world_mxu_free) was not rebuilt.** This session may
  not touch the main checkout. Only this slot's out/mods/tracks/world_* were replaced. After the merge,
  `./build.sh` on main (or the stitch step alone) rebuilds it; build.sh also installs into openmx/mods.
- **Not checked against MX Unleashed in PCSX2 (T7):** the cut changes only what the world leaves out
  (the ring past the ridge); the kept ground is the disc's, proved bit for bit.
- The 5 slow wrecks were not looked into beyond where they begin (on the far tile's own ridge face).
  Whether the original's ridge throws a rider the same way was not tried in PCSX2.
- world_ultra was rebuilt and passes `stitch check`, but it was not ridden in openmx this
  session. C11's `every_world_of_the_three_discs_in_openmx`, `rides_in_openmx` and
  `shots_at_every_gate` were not re-run.
- The join's look (`.blend`) over the ridge top was seen in one shot only, under Global sky.
