# C1: tools/stitch, the map joiner

Track mods in, one world mod (standard §3) out. Every tile is copied whole and placed at a whole-leaf
offset. Its edge band is cut. The gaps become new ground on one lattice, a blend that meets every
interior with the interior's own height and slope, and the same blend gives a texture weight per tile
(`<join>.blend`, MD9). The world names one sky tile (default: the first given). Method, findings and
limits are in `notes/stitch.md`. This file is the account of the work.

## What was done

- `tools/stitch/` (new crate, depends on game-sim, ps2-float, trnwrite and modcheck by path):
  - `lib.rs`: `Tile` loads a track mod folder (one `.scn` by `Scene::parse`, the terrain the scene
    names, found as the shell finds it and read by `Terrain::parse` through trnwrite's MD1 view).
    `find_keep` makes the berm cut. `ring` reads the edge ground from the kept cells. `placements`
    gives section names with positions, by `Scene::parse`'s own rule, cross-checked against its
    counts. `Source::extend` is a tile's ground carried past its edge, and `shepard` is the weight.
  - `build.rs`: the layout (`plan`), the field (`Plan::at`), the pieces (`solve`, threaded) and the
    world folder with its mod.json (`build`). The mod.json has tiles, joins, `drop`, `sky`, `spawn`,
    a `todo` of every known gap, and the joiner's own `stitch` and `joins[].samples` fields.
  - `check.rs`: `stitch check`, the part of §5 step 3 that is C1's, read back with openmx's
    `Terrain::cell` (below). Also `heightmap` for `stitch map`.
  - `main.rs`: `build`, `check`, `profile`, `map`, and build.sh's `--mods/--out` form, which builds
    `tools/stitch/worlds/*.json`. There are no specs yet: C2 and later write theirs. build.sh's
    existing `after - stitch tracks C1` line now finds the crate.
  - `tests/join.rs`, 6 tests, in `./check.sh`.
- `standard/OPENMX-TRACK-STANDARD.md` §3: one sentence. The joins' scale is the world lattice's (the
  commonest tile spacing), where it was "any one of its neighbours' spacings". The new `samples`
  field is named.
- `notes/stitch.md`: new.
- Follow-ups filed: C6 (joins at a tile's own spacing beside mixed-spacing tiles) and C7 (candidate
  pruning and timing before C2's scale).

## How it was proved

`cargo test -p stitch` (6 tests, about 6 s in debug). Each test generates track mods with trnwrite:
257×257 samples, a 45 ft berm rising over the outer 240 ft, gentle hills, two surface ids, and a
scene with one prop in the middle and two in the berm band. The tests then build a world and run
`stitch check`, which does all of the following:

- runs modcheck's world check (§1, every tile as a track, every join terrain read by `Terrain::parse`
  with no MD1 view, since joins have no pages, every blend sums to 255, every drop names a section);
- requires every tile's `at` to be a whole leaf and its `keep` to be inside its terrain, with no two
  interiors overlapping, and every tile file byte for byte its source mod's;
- checks **every seam point**: each tile sample and each join lattice point on each interior edge.
  The tile side is the kept cell from `Terrain::cell`, and the join side is the cell outside it. Both
  give a height and a slope across the seam, and they must agree within the join's height quantum
  (`scale × height_scale`, about 0.003–0.01 ft here). The touching tile's blend weight there must be
  255;
- requires neighbouring join pieces to agree on their shared samples, join ground more than
  2·lattice + 3·fade from every interior to be no steeper than `max_slope`, and no blend weight to
  step by more than 64 between neighbouring samples.

The tests:

1. `two_bermed_maps_join_with_continuous_height_and_slope`: the brief's test. Every seam point is
   counted (as a tile sample and as a lattice point, both on one lattice), with worst steps under
   0.01 ft and 0.005. The berm band found is 34–60 samples on every side. Each interior is the
   source's terrain bytes, and every kept cell is equal by `Terrain::cell`.
2. `props_in_the_cut_band_are_dropped_and_listed`: Model2 and Object1 are dropped from both tiles,
   Model1 is kept, and sky and spawn default to the first tile.
3. `a_tall_neighbour_gets_a_wide_enough_gap`: 80 ft against 380 ft. The gap is at least 600 ft and
   the far slope stays within 0.5.
4. `a_coarser_map_meets_the_lattice_at_its_samples`: a 9 ft map between 4.5 ft ones. There are no
   between-lattice points and the check passes.
5. `the_blend_fades_from_one_tile_to_the_other`: more than 1000 samples share the look between two
   tiles, and every sample sums to 255.
6. `a_tampered_world_fails_the_check`: a join raised by 1% fails at its seams, and a tile whose scene
   gained a section fails the byte comparison.

Mutations, each reverted afterwards. Dropping the pinned ring failed test 1, with slopes of −23 at
the seam: without the ring the Shepard weight divides by zero there. Dropping the slope from the
extension failed test 1, with join slope 0 against tile slope 0.018.

**On real maps.** I imported MX vs. ATV Unleashed's nat05, nat02, mc02 and mc06 with `import-mxvatv
--no-check` into the session scratchpad, from the main checkout's `iso/`, read-only. I built a world
of the four with the release binary. `stitch check --mods … --iso …` passed: 29,492 seam points,
worst height step 0.0034 ft, worst slope step 0.0015, join slope 0.492 away from the seams. It
included modcheck's byte comparison of every tile file with the disc. `stitch map` drew the world.
Looked at by eye, the joins are smooth and the cut edges' relief fades out into the gap. mod.json
listed the 1,755 mc06 drops, and the nationals had none. Numbers are in `notes/stitch.md`.

`./check.sh` passes (run by `agent-runner done`).

## What failed on the way

- The first field extended each edge sample straight outward, unblurred. On the real maps the
  relief cut by the edge (MX vs. ATV's repeated noise cells, edge cells up to 1.49 steep) turned into
  ridges as steep as 2.5 across the whole gap. Widening the gaps nine times over changed nothing, so
  the build gave up. The fix blurs the edge along its length by `σ = d′² / (d′ + fade)`, with zero
  blur and zero derivative on the pinned ring so the seam stays exact, and mixes the two edges at a
  corner by the angle. The synthetic tests had passed before this, because their berms are smooth
  along the edge. Without the real maps this would not have been found.
- The berm search found nothing on MX vs. ATV. Their terrains have no berm: the edge is the flat base
  height every root cell fades to (400 ft on nat05 and nat02). So the cut is the 150 ft shootoff band,
  as it should be. I looked with the new `stitch profile`, which prints the profiles and a coarse
  overview of the map.
- The tamper test's first version blanked a tile's scene. `Tile::load` then refused the tile before
  the byte comparison ran, so I changed it to append a section instead.

## Decisions

- **One lattice for every join** at the commonest tile spacing (the finer on a tie). Joins then agree
  where they meet, and each interior's edges can be put on lattice lines (`at` a whole leaf, the
  interior cut a few samples more to align when the spacings are p/q). This changes the standard's
  "any one of its neighbours' spacings", and the standard now says so. Mixed spacings are C6.
- **A closed-form blend, not an iterative solve.** Shepard over Taylor extensions is local, needs no
  global system, is exact on the seam by construction, and computes each piece independently. That
  made threading trivial. A biharmonic solve over a 9000² lattice is what it replaces.
- **`max_slope` 0.5** (26.6°) by default. The game has no climbing limit to take it from, so it is
  configurable and stored in mod.json.
- **All four sides are cut**, the outer ones too. The world's outer edge is flat join ground 300 ft
  past the outermost interiors (150 ft shootoff + half the minimum gap). It has no berm (todo).
- **Layout**: a near-square grid, sorted by edge height, snake-filled. Gaps are sized per pair from
  the height difference and widened by 1.25 until the far slope passes. This is simple and keeps
  rows of similar height together. It does not pack tiles of different sizes tightly (mc06 sat alone
  in a 5760 ft cell), and C2 can feed it a better order.
- **Every blend lists at least two tiles** (with a zero column when only one reaches), because
  modcheck requires two.
- Nothing was added to `notes/openmx-asks.md`. The world format needs MD8 and MD9, which are already
  filed, and no openmx reader refused a file.

## Not verified

- **Nothing was loaded in openmx or the game.** MD8 (worlds) and MD9 (blend, world sky) are not in
  openmx yet. That the port draws and rides these joins as `stitch check` reads them is unproven.
  That includes the precedence rule (keep, then the first join covering a point) and the floats far
  from the origin (§3 leaves that to MD8). T7-style riding of the seams is C2's.
- **Seam continuity is checked with game-sim's bilinear blend** (`Ground::at`'s), not the game's
  triangle split. On a seam line the two agree, because both are linear along an edge. Inside the
  first join cell they differ, so "slope" here is the cell's slope across the seam as the bilinear
  blend gives it.
- **The berm search was only seen working on generated berms.** No real map tried has a terrain berm
  (MX vs. ATV's edges are flat). MX Unleashed's maps (C4) and Untamed's (C3) were not profiled, and
  the 2% / 400 ft constants are not tuned on any real berm.
- **mc06's seams** (3.5 ft against 4.5 ft) step up to 0.51 ft at its samples between lattice points.
  This is reported as a note and a todo, not failed (C6).
- **Surface ids in the joins** are the nearest tile's edge id, and the top bit is written 0. What the
  bit means, and whether a surface id from one game's `.phy` table means the same in another's, was
  not checked.
- **The interior identity** is shown by the tile files being byte for byte their source's and by
  `Terrain::cell` being equal over every kept cell (test 1). That the port, given a world, queries a
  tile's terrain and not a join inside `keep` is MD8's to do and is not tested here.
- **Scale.** Only a 4-map world was built (2 s, 60 MB of joins). C2's tens of maps were not tried, so
  time and size there are estimates (C7).
- `stitch --mods/--out` (build.sh's form) was run by hand, not through build.sh. With no spec it
  prints a line and exits 0. With a temporary spec (nat05 + mc02, removed afterwards) it built the
  world into the stage folder (54 pieces, far slope 0.448). build.sh's packaging of that output was
  not run.
