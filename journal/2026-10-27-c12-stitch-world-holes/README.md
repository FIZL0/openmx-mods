# C12: no holes in a stitched world's ground

The user (2026-10-27): openmx's `world holes:` counts 2189 on world_mxu without wrapping and ~5000 with
`"wrap": "xz"`, gaps about every 1152 ft at the world's edges, and a rider drops in or is thrown near one.

## Log

- **What the holes were.** Probed the installed worlds (`openmx/mods/tracks/world_*`, built 2026-10-23
  10:54, before C11's fix) with game-sim's own `Tiled::holes()` through `stitch::openmx::load`, and
  with openmx's `close_seams` repeated (game-sim's `generate::seam_joins`, same spacing and edge
  probing): world_mxu 2189 open / 5052 wrapped, exactly the user's numbers. Every one is 4.5 ft wide
  at a multiple of 1152 ft (256 cells × 4.5): C11's strip, the last cell of each 256-cell join that
  openmx drops. Stitch has not made them since C11 (`build::cuts`, a leaf of overlap); the installed
  worlds were never rebuilt. Wrapping only cuts the same strips into more rectangles (the seam joins'
  edges add cut lines to `holes()`), so 2189 becomes 5052. This slot's out/mods worlds (built in M23,
  after C11): 0 holes open and 0 wrapped.
- **What wrapping really broke.** With the holes gone, the wrapped world still had no join across the
  wrap: stitch's field at the far edge is the last row's extension, at the near edge the first row's,
  and openmx's seam strip ramps straight between them over 15 cells of its spacing (1 ft here: no
  whole spacing divides a 4.5 ft world's side, so 15 ft). On this slot's M23 world_mxu that is
  130 ft (x) and 193 ft (z) of difference over 15 ft; on world_mxu_free 45 ft over 135 ft. The
  snake layout makes it worst on z: rows go up in height, so the wrap joins the highest row to the
  lowest. On the installed world_mxu: a 12.6 ft step in half a foot.
- **The fix: the field is a torus on a wrapped axis** (`build::Plan::period`, `build::images`). Every
  source is also seen a period (`size × spacing`) either way, its Shepard weight the sum over its
  images; `radii`, `Bins`, `candidates`, `gated` and `clear` measure to the nearest image. So the
  ground at x = W is the ground at x = 0, bit for bit up to summation order, and the last column of
  tiles meets the first across the wrap through a blend like any other gap. A world with no wrap has
  one image (shift 0) and is the same to the bit (prune test, every C1–C11 test unchanged).
- **The layout sizes the wrap gap.** In `plan`, after the last line on a wrapped axis, each row's
  (column's) last tile and first tile are a facing pair with `gap_for`'s gap, as between two lines;
  the world's far edge goes out until the band past the last tile plus the band before the first
  is that wide (at least the old `pad`). Widening (C9) widens it too.
- **The ultra map** (`ultra::UltraPlan::period`) the same: its `at` blends over the worlds' images,
  `inside` finds a world's image (a far-edge sample of new ground is the first world's edge), a
  world's look takes its nearest image's foot, and the map runs on past the last world until each
  pair across the wrap has its gap.
- **openmx's extent is `size × step − step`** (it drops the last cell), so its strip starts at the
  whole foot under that, 5 ft short of stitch's period on a 4.5 ft lattice. The strip then runs
  from h(W − 5) to h(0) = h(W): a sub-foot kink, no step.
- `stitch check` now holds a wrapped world's near-edge join samples to the far edge's at the same
  place, to both quanta (`check::wrap_seams`, `Report::wrap_points`).
- `stitch::openmx::World::closed` repeats openmx's `close_seams`; `stitch::openmx::across_seams`
  walks every seam half a foot a step; `stitch holes WORLD_DIR...` prints both (openmx's
  `world holes:` count, and the steepest step across the seams).
- Tests, `tools/stitch/tests/wrap.rs` (in the gate): a generated 2×2 world wrapped xz, heights 30 ft
  apart — the field equal a period on at every edge sample (< 1e-6 ft), `stitch check` ok with its
  wrap points, openmx's closed world 0 holes, 69,918 points across both seams all with ground and
  the steepest step 0.25 ft in half a foot; the same tiles built with edges and then marked wrapped
  (as the installed worlds were) have a 3.7 ft step, which the test asserts so it can tell. And a
  wrapped ultra map of two wrapped worlds: check ok, 0 holes, steepest step 0.07 ft.
- Turned the throwaway probe into `stitch holes WORLD_DIR...`; on the installed world_mxu before the
  rebuild it gives the user's 5052 holes and a 12.6 ft step in half a foot across the seam.
- **Rebuilt.** Imported MX vs. ATV Unleashed (56) and Untamed (40) fresh into `target/c12/mods/tracks`,
  then `stitch --mods target/c12/mods --out target/c12/stage` (7 min 11 s, MX Unleashed's tiles off the
  disc by stitch itself): world_mxu widen 2.441 (1.5625 before: the images reach into its edge bands),
  41,733 × 38,898 ft, 1,059 lattice joins; world_mxu_free 1.0; world_mxvatv 1.953; world_untamed 2.441;
  world_ultra 3.815, 121,140 × 40,050 ft, 799 new pieces. Replaced the five `world_*` in this slot's
  `out/mods/tracks/` and in openmx's `mods/tracks/` (ignored by openmx's git) with them. The removal
  with variable paths was refused by the harness; done again with literal paths.
- Built openmx 09da0164 (clean tree) into `extracted/openmx-target` for the openmx checks below.
- The ignored disc tests (`world_mxvatv`, `world_untamed`, `openmx_worlds`' `params`) now take the
  spec's `wrap`, as build.sh's run does.

## How it was proved

- `./check.sh` (includes `tests/wrap.rs`'s two tests and every earlier stitch test, unchanged results;
  `prune.rs` still finds the world the same to the bit with and without pruning).
- `stitch check --mods target/c12/mods --iso <discs>` on the five rebuilt worlds: all ok. Wrap samples
  near edge against far: world_mxu 17,920 (worst 0.0040 ft), world_mxu_free 3,235 (0.0044),
  world_mxvatv 15,914 (0.0066), world_untamed 15,674 (0.0070), world_ultra 35,822 (0.0034). Join slope
  away from the seams 0.443 / 0.346 / 0.459 / 0.492 / 0.492.
- `stitch holes` on the five: 0 holes each; across the seams 546,607 / 375,585 / 485,632 / 478,315 /
  1,092,401 points, 0 with no ground, steepest step 0.172 / 0.148 / 0.135 / 0.105 / 0.141 ft in half a
  foot.
- **openmx itself** (09da0164, `--headless --track <world> --trace 1 --trace-feel`), on the installed
  `openmx/mods`: `world holes: none, so only the outer edge throws` on all five, each `world wraps: x
  every … and z every …`, extents equal to `stitch holes`'.
- 30 openmx rides on the gas (`--trace-gas --trace-at`, 500 ticks), from 300 ft short of each far edge
  on three lines per axis per world: all 30 cross the wrap, none thrown. 7 have one wreck, each 543–1,078
  ft past the wrap on the first tile's ground (the mxu_free ones about 1,050 ft in, coming down that
  map's ridge as in M23), none at the seam.
- Ignored disc tests run: `world_mxvatv` (256 s) and `world_untamed` (370 s), both ok wrapped.

## Not verified

- `openmx_worlds`' ignored `every_world_of_the_three_discs_in_openmx`, `rides_in_openmx`,
  `shots_at_every_gate`, `world_mxu_free_meets_ridge_to_ridge`, and `world_ultra`'s ignored test were not
  run (about 25 minutes together); the same worlds were built, checked and ridden above by other means,
  but those tests' own assertions (e.g. the ultra map's per-world ride positions, C11's 76,032 ft offset
  for world_mxu, now 77,184) were not rerun and may need their numbers updated.
- No picture was taken across a seam: the look (`.blend`) across the wrap is held only by `stitch
  check`'s blend-step rule, not by eye.
- The rides cross on the gas in a straight line; slow, turning or airborne crossings and the camera at
  the wrap were not tried.
- The main checkout's `out/` still holds whatever it held (never touched); `./build.sh` there rebuilds
  it.
- Not checked against PCSX2 (T7): the worlds are combined maps, and the wrap is the port's own.
