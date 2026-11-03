# openmx MD57: far join relief in tools/stitch

The worlds' joins far from every map were flat blend. `tools/stitch/src/build.rs` now adds the nearest
maps' own relief past the seam gates, the same code as openmx's `crates/stitch` (full write-up in openmx
research/journal/2026-11-02-md57-far-join-relief). Each map's ground is smoothed and held to 0.25 ft/ft,
then folded into its keep and shared by inverse-square distance. It rises from nothing at the gates under
a 0.1 ft/ft cap, and is kept off blend too steep to hold it: the room comes from the coarse pass, with
local dents where the relief still came out too steep. No world widens for it. mod.json gains
`stitch.relief`. New test: tests/relief.rs.

## How it was proved

- `cargo test --release -p stitch`: all non-ignored tests pass, the new
  `a_join_far_from_every_map_has_the_nearest_maps_relief` included. A join 2000 ft from every map
  has a spread of 3.65 ft against its map's 4.11, and `stitch check` passes.
- Rebuilt all five worlds (world_mxu, world_mxu_free, world_mxvatv, world_untamed, world_ultra). All
  five pass `stitch check`, no clamped samples, far slopes 0.413–0.498, relief scale 1.0. Installed in
  out/mods/tracks and openmx's mods/tracks. Far-ground spread went from 0.1–2 ft to ~5 ft.

## Not verified

- world_ultra's new ground between whole worlds (`ultra.rs`) gets no relief (openmx MD70).
- The world_* mod.json files carry MD66's `"category": "World"` line, put in by hand. MD66's branch
  here is still unmerged, so a fresh build drops that line until it lands.
- Not ridden in the shell; checked by stitch check and openmx's world tests only.
