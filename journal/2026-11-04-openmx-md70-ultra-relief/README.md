# openmx MD70: ultra relief in tools/stitch

world_ultra's own new ground between whole worlds (`tools/stitch/src/ultra.rs`) was flat blend. It now
carries the nearest worlds' own relief past the seam gates, the same way MD57 did it for the per-game
worlds in build.rs (full write-up in openmx research/journal/2026-11-04-md70-ultra-relief). Each
world's ground is read at 16 ft nodes over its whole rectangle, from its tiles and its joins. It is
smoothed and held to 0.25 ft/ft, then folded into the world's rectangle and shared by inverse-square
distance over all wrapped images. It rises from nothing at the gates under a 0.1 ft/ft cap, and is
kept off blend too steep to hold it: the room comes from the ultra coarse lattice, with local dents
where the relief still came out too steep. No gap widens for it. build.rs only gained `pub(crate)` and
two splits (`Relief::of_nodes`, `Room::of_cells`). mod.json gains `stitch.relief`. `stitch ultra`
gains `--wrap`. New test: tests/ultra_relief.rs.

## How it was proved

- `cargo test --release -p stitch`: every non-ignored test passes except the two in tests/wrap.rs
  (see below), and the new `new_ground_far_from_every_world_has_the_nearest_worlds_relief` passes.
  A 600 ft window 2000 ft from both worlds has a spread of 3.69 ft against its world's 4.30 ft. The
  files match `UltraPlan::ground_at` to 0.05 ft, and the relief is 0 at the gates.
- Rebuilt world_ultra from the installed worlds: same layout as before (widen 3.815, 77 tiles, 2265
  joins, 625 new pieces), relief scale 1.0 after 14 dents, new-ground slope 0.498 far and 0.448 near.
  World reliefs: mxu 38.3 ft spread, mxvatv 64.9 ft, untamed 105.1 ft, all at gain 1.00.
- `stitch check --mods out/mods --iso iso` on it: ok in 76 min. 60,726 points where new ground meets a
  world's edge, worst 0.00354 ft; wraps worst 0.00351 ft; join slope 0.500; every tile byte for byte
  its disc. Installed in out/mods/tracks and openmx's mods/tracks; openmx's
  `every_installed_worlds_tiles_read_as_alone` and `the_stitched_worlds_have_no_gap_along_a_seam` pass.
- Far new ground (195 windows of 300 ft, each ≥1000 ft from every world; ignored test
  `the_installed_ultra_maps_far_new_ground_has_relief`): mean spread 0.31 ft before and 3.89 after;
  median 0.27 before and 3.72 after.

## Not verified

- tests/wrap.rs: `a_wrapped_world_*` fail with "elev_c12_a: at -144 is not a whole number of feet",
  and they fail the same way on untouched origin/main. MD61's game-sim half is not yet in openmx.
- The relief is rolling ground only: no tracks, jumps or objects are carried into the gaps.
- The installed world_ultra/mod.json's `"category": "World"` line (MD66) was put back by hand.
- Not ridden in the shell.
