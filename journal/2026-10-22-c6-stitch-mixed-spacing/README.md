# C6: joins at a tile's own spacing beside mixed-spacing tiles

C1 put every join on one lattice at the commonest tile spacing. A tile with another spacing (mc06, 3.5
ft, in a 4.5 ft world) met the join exactly only where the two lattices share a sample. At its own
samples in between, the two piecewise-linear edges differed by the tile's curvature: 0.51 ft on mc06.
Now such a tile gets a **collar** of joins at its own spacing, so every one of its seam samples is a join
sample. The collar meets the lattice's joins along lines where both lattices have samples, and it meets
them exactly there. Method: `notes/stitch.md` step 3a.

## What was done

- `tools/stitch/src/build.rs`:
  - `Collar` (tile, spacing, rect, width, period, band) and `collars()`, which plans one round every tile
    whose spacing is p/q of the lattice's with q > 1. The rect is the interior grown by `width`, the least
    multiple of `period` = q × spacing past `band + 2 × lattice`. Its edges are on both lattices, because
    C1's `place_axis` already puts the interior's edges there. A collar is only made if its band stays
    `2 × lattice` clear of every other interior and every other collar's band, and inside the world.
    Otherwise the tile is in `Plan::uncollared` with the reason, and keeps C1's behaviour and todo.
  - `Plan::at` is the old blend (now `Plan::blend`) plus `Plan::pull`. Within `band` of a collar edge,
    `pull` moves the field onto the straight line between the shared samples either side of the point's
    foot on the edge. It is weighted by a fall-off that is 1 on the edge and drops to 0 with zero slope.
    On the edge the ground is piecewise linear with knots at shared samples, so both lattices draw it
    exactly. The knots are evaluated with every source, so the collar's pieces and the lattice's pieces
    compute the same line.
  - `Spec` (name, origin, spacing, cells) replaces the `(name, x0, z0, w, d)` tuple. `pieces()` lists
    the collar's chunks first (`c<tile>_<x>_<z>`, at its spacing; first in `joins` = precedence, §3),
    then the lattice's. It leaves out a lattice chunk wholly inside a collar. `piece()` samples at the
    spec's own origin and spacing.
  - mod.json: `stitch.collars` (the joiner's own field). The todo for a collared tile now says what its
    joins are, and the old "between lattice points" todo is kept only for an uncollared tile.
- `tools/stitch/src/check.rs`:
  - Tile seams: the points are the tile's samples, plus the samples of every join that reaches the
    edge's line, each counted only where that join is the one covering the seam. A tile sample is
    strict if it is on the covering join's lattice. Beside a collar that is all of them, so
    `between_points` is 0.
  - New `join_seams`. Along every join's edges, at the samples of every lattice the joins use: where the
    ground just inside is this join and just outside is a join of another spacing, the two heights at
    the point must agree to the height quantum (`join_seam_points`, `join_seam_height`). The slope
    across is reported, not failed (`join_seam_slope`).
  - The "neighbouring joins agree" check keys samples by world position, not by lattice index, because
    the joins now have two spacings. The far-slope distance uses `stitch.spacing`, not `joins[0]`'s
    spacing (which is now a collar piece).
- `tools/stitch/src/main.rs`: `stitch check`'s summary line gives the join-seam count and worst step.
- `tools/stitch/tests/join.rs`: `a_finer_map_gets_joins_at_its_own_spacing` (the brief's test), and
  `tile_rippled`/`world_rippled` (ripples of 3 ft, about 25 ft across, for curvature at the edges).
- `standard/OPENMX-TRACK-STANDARD.md` §3: one sentence saying joins round an off-lattice tile are at its
  spacing and listed first.
- `notes/stitch.md`: step 3a, the check's new parts, and an updated Limits entry with the real-map numbers.

## How it was proved

`cargo test -p stitch` (7 tests). The new test has two worlds:

1. 4.5 / 3.5 / 4.5 ft tiles with 3 ft ripples. The lattice is 4.5 ft and one collar is made, on the
   3.5 ft tile, with period 31.5. `stitch check` passes. `between_points` is 0. More than 100 points
   are where the collar meets the lattice (1,160 measured, worst 0.0021 ft). Seam steps are under
   0.01 ft and 0.005. `seam_points` is exactly twice every tile's edge-sample count, so each 3.5 ft
   seam sample is counted as both a tile sample and a collar sample.
2. 3.5 and 4.5 ft. On a tie the lattice is 3.5 ft, so the 4.5 ft tile is the collared one (ratio 9/7).
   Same assertions.

Mutations, each reverted afterwards:

- No pull (`pt.h += 0.0 * pull`): the test failed at the collar edge, collar against lattice off by
  0.02–0.05 ft. Without ripples it did not fail, because the synthetic ground was too smooth 63 ft out to
  show it. That is why the test uses ripples.
- No collars (`collars()` replaced by empty lists): the test failed with 448 between-lattice points,
  worst 0.74 ft (C1's behaviour).

**On real maps.** I imported MX vs. ATV Unleashed's nat05, nat02 and mc06 with `import-mxvatv --no-check`
into the session scratchpad, from the main checkout's `iso/`, read-only. I built a world of the three
with the release binary (1.3 s, 65 pieces, one collar `c02_000_000` round mc06, 63 ft wide). `stitch
check --mods … --iso …` passed:

- 20,336 seam points, worst height step 0.0035 ft, slope step 0.0015, with no between-lattice note;
- 1,096 points where mc06's collar meets the lattice, worst 0.0017 ft;
- join slope 0.484 away from the seams;
- modcheck's disc comparison of the tile files.

`stitch map` at 1.5 ft, cropped round mc06, shows no visible line at the collar's edges.

`./check.sh`: ok, 92 passed.

## What failed on the way

- The first version of the test used the smooth generated tiles. It passed even with the pull turned
  off, so it proved nothing about the collar edge. Adding ripples made the mutation fail.

## Decisions

- **A collar, not a per-tile lattice for the whole gap.** A band round the tile keeps the rest of the
  world on the one lattice (C1's reason for one lattice still holds). The new boundary is
  join-to-join, which nothing in the game sees as a seam. It is made exact by the straight-line pull.
- **Pull, not overwrite.** Setting only the edge samples to the line would leave a crease of up to the
  field's curvature over 31.5 ft. The pull fades with zero slope, so the field stays smooth across the
  edge, and only the knots between shared samples remain as tiny kinks along it.
- **Band = max(fade, period)**, with width the least period-multiple past `band + 2 × lattice`. That gives
  63 ft for mc06. This is the smallest collar whose pull stays off the tile's pinned ring.
- **Fit or fall back.** If a collar would touch another interior's ring or another collar, the tile gets
  C1's treatment and a todo with the reason, rather than a broken seam elsewhere. In the tests and the
  real world every collar fit (gaps are at least 300 ft, and a collar plus its band is about 111 ft).
- **The slope across a collar edge is reported, not failed.** Each side's cell is at its own spacing, so
  their slopes differ by one-sided differences of a curved field, about `h'' × 4 ft`. That is 0.08 on the
  real world and 0.012 in the test, the same size as the slope change between two neighbouring cells of
  one lattice there. The field's own slope is continuous across the edge (the fall-off has zero slope).
- Nothing was added to `notes/openmx-asks.md`. The standard already allowed a join's scale to be "one of
  the world's tile spacings", and no reader refused a file.

## Not verified

- **Nothing was loaded in openmx or the game.** MD8/MD9 are still not in openmx. That the port honours
  `joins` order as precedence where a collar piece and a lattice piece overlap (the collar listed first)
  is the standard's rule, but no loader exercises it yet.
- **That the slope step across a collar edge is no worse than the lattice's own cell-to-cell change**
  is an estimate (`h'' × 4 ft`). I did not measure the lattice's neighbouring-cell slope change at the
  same place to compare.
- **The fall-back path** (a collar that does not fit, or a ratio with no small fraction) was not hit by
  any test or by the real world. Its todo text was read, not produced.
- **A collar on a finer-than-lattice tile whose period is large** (for example q = 64) was not tried.
  The band would grow with the period, and such a collar would usually not fit, so it would fall back.
- **Untamed's and MX Unleashed's maps** were not used. Their spacings were not checked against each
  other here.
- **The picture check** was by eye on one crop of `stitch map`'s greyscale shading.
