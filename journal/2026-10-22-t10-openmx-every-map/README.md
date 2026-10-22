# T10: every imported map loads and rides in openmx

## What was done

- Built openmx at `2e369b3` (MD2, MD3, MD4, MD5, MD10, MD12 in) into this slot's
  `extracted/openmx-target` (`CARGO_TARGET_DIR`; openmx's own `target/release` is older, and nothing was
  written in its checkout).
- `tools/accuracy/src/ride.rs` and `accuracy ride --mod DIR [--ram IMAGE] --openmx BIN`: one openmx run a
  map, headless, `--frames 180` (6 s of its clock) with a `--shot-pose` shot from the map's overview pose
  on the first frame, so the shot and the ride are one run (7 s). Then four verdicts from what it printed
  and from the mod read the way the shell reads it:
  - **loads**: exit 0, the `mods: <id>` line, no `left out` line, nothing on stderr; a collision mesh
    `solid:` could not load must be named in the mod's `todo`.
  - **heights**: `Shell::load` copies the shell's own load (`track_mods::check` + `main.rs`): the folder
    through `Res::folder`, the one `.scn`, the terrain by `terrain_bytes`' suffix lookup (`gen.trn` →
    `gen.ps2.trn`), `mod.json`'s `spawn` through the scene's reader. Its ground, at the harness's 2,000
    points (`heights::points`, the same seed and box as `accuracy heights`), against the original's own
    query on the RAM image in all four modes (8,000), and `World::ground` — the ride's own call — against
    the harness's `look::height_at`.
  - **start**: openmx's printed "dirt under the start line" is the shell ground's (to 0.05 ft); after
    the ride the bike is within 3 ft of the ground under it; no wreck.
  - **edge**: openmx's `map: W x D` line against the scene's four numbers (MD3), with the file's box and
    `TerrainBorderWidth` beside it; a cannonball (wreck 13) fails it.
  - A second shot (`--frames 2`) at the first `[AnimatedToy<n>]` of the `.toy` with
    `HasCollision=TRUE`, for MD5's windmill.
- RAM images: MX vs. ATV Unleashed's 56 from slot s2's `extracted/mxvatv/ram` (read only; this slot has
  none), Untamed's 40 from this slot's. All 96 maps have one.
- Ran it on all 96 mods (4 at a time, about 3 minutes), read the 96 overview shots on six contact sheets
  and the toy shots of MC01, Nat04, Nat08 and Nat10 by eye. The table is in `notes/accuracy.md`,
  *Every map in openmx (T10)*; what failed is in `notes/openmx-asks.md`.
- Tests: `tools/accuracy/tests/ride.rs` (generated mod folders and logs in openmx's words): the command
  line, the log reader, the loads verdict with `todo`, the shell's load (suffix + `.ps2`, spawn, two
  scenes refused), the last sample strip, the start and edge verdicts, the toy pick.

## What it found

72 of 96 pass all four checks; the table is in `notes/accuracy.md`.

- **Loads: 96 of 96.** Every mod is taken with nothing refused. Two collision meshes do not load (FS05's
  `Anaheim_pole.col`, Nat10's `nat10_Mailbox_Closed.col`), and both are in their mods' `todo`.
- **Heights: 96 of 96.** Equal to the original's query at 8,000 points on every map (2,000 in each of the
  four modes), on the ground the shell builds. The ride's own `World::ground` equals the harness at every
  point but 2 to 14 a map in the last sample row and column (asked, low).
- **Start: 72 of 96.** 12 of the 14 maps whose scene start is under the ground wreck or end under the
  terrain on a prop (BT_A and BT_B 291 ft under it): openmx does not stand the rider on the ground as
  the original does (M12/M13's ask, now with the ride). 11 arenas throw the rider off the map at once.
  Untamed's Nat_PA drops 129 ft and wrecks (the original drops too; whether it wrecks was not seen).
- **Edge: 85 of 96.** openmx's edge is the scene's `256 × w × scale` on all 96 (MD3). On the 11 arenas
  above the start is within its 150 ft margin. The arenas' scenes give `TerrainBorderWidth`, and the game
  holds `edge + 2 × border` as a float pair, once in each of the 46 bordered maps' RAM images; openmx
  leaves it out. Asked, with what reads it still to be decompiled.
- **Look.** MD4 draws every map's ground from its own pages: no flat ground on any of the 96 (overview
  shots). MD5's windmill: the class-1 `.col` parses, but MC01's windmill blades, Nat10's mill wheel and
  Nat04's Ferris wheel are not drawn. Animated toys are openmx's own V43 (open); nothing new to file.
  Grey slabs on Untamed's AM_GFUC, Nat_IN, Nat_MA and Nat_NC look like MD13's race props.

## What failed on the way

- The first run counted a start under the scene's ground as a failure. The original stands the rider on
  the ground over such a start (M12, M13), so the scene's `y` is not openmx's fault; the verdict became
  "the bike ends the ride on the ground, not wrecked", and the start's depth is a note.
- The first heights verdict failed nat04, sx07 and sm_hs on `object_checks`' box. The box is M12's
  known ask (`world::Ground` sizes from the scene); the heights agree over it, so it is a note now.
- `World::ground` and the harness differed at 2 to 14 points a map, all at x or z ≥ (samples − 1) ×
  scale (5755.5 on the big maps). `world::Ground::at` refuses `sx + 1 >= samples`. Counted apart as "the
  last strip", since it is inside the 150 ft shoot-back margin, and asked.
- Running openmx from `openmx/` with `../extracted/...`: `openmx` is a symlink, so `..` is the real
  parent. Absolute paths (the memory note said so).
- `montage` with no shots: the first batch of end-of-ride frames had that relative path.

## Not verified

- **What reads `edge + 2 × border` in the original**, and so whether the original's map edge on an arena
  is the terrain grown by the border. The pair was found by its value in RAM (46 of 46 bordered maps, one
  copy each). Neither SLUS_211.04 nor SLUS_217.01 was decompiled for it. That the original does not throw
  the rider at those starts rests on M12/M13 having ridden every map from its start in PCSX2.
- **Whether the original wrecks** on Nat_PA's 129 ft drop, or anywhere on a start under the ground: M12
  and M13 saw where the camera was, not whether the rider fell.
- **The look against the original.** The shots were read by eye for the ground's textures and the
  animated toys only, on 360 × 270 contact sheets. They were not paired with the original's (T7's pairs
  are Lago's alone), and plants, sky, fog and props were not compared here.
- **Each jumbotron.** The look column says "white" on the 44 stadium maps from T11's audit; I saw white
  screens on the sheets but did not check every one of the 44 shots for a screen in view.
- **The ride** is six seconds with nobody on the pad. Nothing was ridden across the map, into a prop or to
  the edge, and the windmill blades' collision (MD5 in the simulation) was not ridden into: openmx does
  not build the animated toys at all (V43).
- **The shell's load** is copied from `crates/openmx/src/track_mods.rs` and `main.rs` (a binary this repo
  cannot link). `start` and `edge` read the binary's own log, so a drift in the copy would show there as
  the dirt or the edge not matching, but the heights check uses the copy. openmx's printed dirt matched
  the copy's ground on all 96 starts.
- **The RAM images for MX vs. ATV Unleashed** are slot s2's (M12's), read in place, not copied. The mods
  are this slot's `out/` from T11's rebuild; a change to the importer since then would not show.
- **The world mods and the elevation mods** (`world_*`, `elev_*`) were not ridden: the brief names the 96
  MX vs. ATV and Untamed mods, and T11 rode the others.
