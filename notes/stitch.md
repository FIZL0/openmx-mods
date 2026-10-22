# The map joiner (tools/stitch, C1)

How `tools/stitch` turns track mods into one world mod (standard §3), what it found on real maps, and
what C2–C5 need to know before they use it.

```
stitch build OUT --id world_x --name NAME [--sky TILE] [--max-slope 0.5] [--min-gap 300] [--chunk 256]
             [--fade 48] [--threads N] [--no-prune] TILE_DIR...
stitch check WORLD_DIR [--mods DIR] [--iso DIR]     seams, slopes, look, tiles against their sources
stitch profile TILE_DIR [SAMPLES]                   a map's edge profile and the cut find_keep makes
stitch map WORLD_DIR OUT.pgm [FEET]                 the world's ground as a shaded greyscale picture
stitch edges [--max-slope X] [--fade FT] TILE_DIR... each edge's height range, steepest cell, seam gate (C9)
stitch --mods OUT/mods --out STAGE                  build.sh: every tools/stitch/worlds/*.json
```

A world spec for build.sh (C2 and later add theirs to `tools/stitch/worlds/`):
`{"id": "world_mxvatv", "name": "...", "tiles": ["mxvatv_nat05", ...]}`, optionally `sky`,
`max_slope`, `min_gap`, `fade`, `chunk`. The tiles are looked up in `--mods`/tracks/. A spec whose
tiles are not all built is skipped with a line.

## The method

1. **The cut** (`Tile::find_keep`). Each side loses the shootoff band first: 150 ft (game-sim
   `off_map::MARGIN`, MXU `0x002522c0`), or the scene's `TerrainBorderWidth` if that is wider. Then it
   loses more for as long as the side's profile keeps falling inward faster than 2%. The profile is the
   median height at each distance from the edge. This loss is the berm's inner slope. It is capped at
   400 ft past the band and at a quarter of the map.
   The sides are those of the tile's **ground** (C8, `Tile::ground`, `whole_cells`): the largest
   rectangle of whole 16-sample leaves whose cells all exist. That is the whole field on every map but
   nat04. Where the tree has a hole at an edge, the side is the hole's edge. Only what is left of the
   shootoff band past the hole is cut there, since the game has no shootoff at a hole, only no ground.
   Then the berm test runs as on any side. So no cell outside the ground is ever read, and the world's
   `todo` names the tile and its ground.
2. **The layout** (`build::plan`). Tiles are sorted by the mean height of their interior's edge and
   laid in a near-square grid, filled snake-wise, so neighbours are close in height. Each gap is
   `max(min_gap, 1.6 |Δh| / max_slope + room_a + room_b)`, where Δh is between the two facing edges and
   an edge's room is `lattice + fade` plus however far its seam gate (step 6) is past `2 lattice + 3 fade`.
   Each tile is centred in its cell. Its `at` is a whole leaf (16 samples). Where its spacing is a small
   fraction p/q of the lattice's, its interior's edges are moved onto lattice lines, by cutting at most
   q − 1 more samples per side.
3. **The ground** (`build::Plan::at`), on one lattice for the whole world, at the commonest tile spacing:
   - Inside an interior (closed), the tile's own ground, from the kept cell.
   - Within one lattice step of an interior (the pinned ring), that tile's edge height plus its edge
     slope times the distance. So the first join cell has the last tile cell's slope.
   - Elsewhere, modified Shepard over every tile within its radius. The weight is
     `((R − d′)₊ / (R d′))²`, where `d′` is the distance past the pinned ring. Each tile has its own `R`
     (C7, `build::radii`): twice the farthest any coarse point of its neighbourhood is from its nearest
     interior. Its neighbourhood is the points it is nearest to, or within a coarse cell's diagonal of
     the nearest. So both tiles on each side of a gap reach at least twice as far as the gap's middle is
     from either. A point only extends the tiles whose weight there is not 0 (`build::Bins`, below). Each tile is extended
     from its edge. The edge is blurred along its length by `σ = d′² / (d′ + fade)`, and the edge's
     slope is carried outward with a taper that levels off over `fade` feet. At a corner, the two edges
     are mixed by the angle.

   A weight has zero gradient where it reaches 1, and so does the blur. That is why the ground meets
   each interior with the interior's height and slope.
3a. **Collars** (C6, `build::Collar`). A tile whose spacing is p/q of the lattice's with q > 1 (3.5 ft
   against 4.5: 7/9) gets joins at its own spacing, `c<tile>_<x>_<z>`, over its interior grown by
   `width` every way. They come first in `joins`, so they own that ground (§3 precedence), and a
   lattice chunk wholly inside a collar is not written. The interior's edges are on both lattices
   already (step 2), so the collar's edges are too, and every `period` = q × spacing feet along them
   (31.5 ft: 9 × 3.5 = 7 × 4.5) is a sample of both. Within `band` = max(fade, period) of a collar edge
   the field is pulled onto the straight line between those shared samples: by
   `fall(across / band) × fall(past the ends / band) × (line − field at the foot)`, where `fall` is 1 at 0
   and drops to 0 with zero slope at both ends. On the edge the ground is that line, which both lattices
   draw exactly, so collar and lattice are the same height at every sample of either. Off it the pull
   fades out smoothly. At a collar corner the other edge's pull is 0, because the corner is a shared
   sample. `width` is the least multiple of `period` past `band + 2 × lattice`, so the pull never
   reaches the tile's pinned ring. A collar is only made where its band stays `2 × lattice` clear of
   every other interior and other collars' bands, and inside the world. Otherwise the tile keeps C1's
   behaviour, with a todo saying why. A tile whose spacing is a whole multiple of the lattice's (9 ft
   on 4.5) needs no collar.
4. **The look** (`<join>.blend`). The same weights, as bytes summing to 255 (largest remainder), for the
   tiles that have a share anywhere in the piece. A piece that only one tile reaches lists the nearest
   other tile with 0 everywhere, because modcheck wants two or more. The surface byte is the nearest
   tile's edge sample's id, with the top bit 0.
5. **Pieces.** The lattice is cut into chunks of `--chunk` cells. A chunk wholly inside one interior is
   skipped. Each piece is written with trnwrite's `build`/`write`, with normals taken from the field
   across the piece's edges.
6. **Rideable.** A coarse pass (every 4th sample) and then the full pass measure the steepest join cell
   past every tile's seam gate. Past `max_slope`, every gap is widened by 1.25 and the build runs again.
   A seam gate is per edge (C9, `Source::edge_gates`): `2 lattice + 3 fade` (C1's), or farther where the
   edge's own extension stays steeper than 0.75 `max_slope` (`build::GATE_SHARE`) farther out. Past a
   corner the larger of the two edges' gates counts. See "Steep edges" below.

mod.json gets two fields of the joiner's own (the loader ignores them, §1): `joins[].samples` (the
piece's samples a side, which `check` and `map` read), and `stitch` (the lattice, `max_slope`, `fade`,
the radius, the widening used, the collars, the seam gates of every tile that has one past C1's, and the
steepest join slope away from and near the seams). Each tile with a wider gate also gets a todo line.

`stitch check` holds join cells to `max_slope` past the same gates (read from `stitch.gates`). It takes a tile's seam points from the join that covers each stretch of the edge (the
collar, round a collared tile). Where joins of two spacings meet, it also compares the two at every
sample of either along the line (`join_seam_points`). Heights must agree to the join's quantum. The
slope across is only reported, because each side's cell is at its own spacing.

## What the MX vs. ATV Unleashed maps showed

Measured with `stitch profile` on nat05, nat02, mc02 and mc06, imported by M2 into the scratchpad:

- **There is no berm in the terrain.** Each 256-sample root cell's relief fades to one flat base height
  at the cell's border: 400 ft on nat05 and nat02, 135 on mc02, 90 on mc06. The map's outer edge is
  that same flat base. The cut is therefore just the 150 ft shootoff band (34 samples at 4.5 ft) on
  the nationals and MC maps. On mc02 one side is 45 samples, because of a slope near the edge.
- Much of each national is the same relief tile repeated per root cell, with the map's own track
  carved into a few cells (visible in `stitch map`).
- The relief that the cut runs through has edge cells as steep as 1.0 (nat05) and 1.49 (nat02). The
  first version extended each edge sample straight out, and those hills became ridges as steep as 2.5
  that never flattened. Widening the gaps did nothing. Blurring the edge with distance (step 3) fixed
  it.
- No national had a placement within 150 ft of its edge. mc06 (a stadium) lost 1,755 placements: 1,209
  stand off its terrain altogether (the stands) and 546 in its cut band.
- A world of nat05, nat02, mc02 and mc06: built in 2 s (32 threads), 96 pieces, 60 MB of joins (37 MB
  of tiles). `stitch check`: 29,492 seam points, worst height step 0.0034 ft, worst slope step 0.0015,
  join slope 0.492 away from the seams. modcheck passed with the disc comparison. With C1, mc06's 3.5 ft samples
  between 4.5 ft lattice points stepped by up to 0.51 ft. Since C6 they meet a collar (below).

## Limits (each also in the world's `todo` where it applies)

- **Mixed spacings** (C6). A collared tile (step 3a) meets its joins exactly at every one of its
  samples. C1 measured 0.51 ft steps between lattice points on mc06. A world of nat05, nat02 and mc06
  now has none: 20,336 seam points, worst step 0.0035 ft, and 1,096 points where mc06's collar meets
  the 4.5 ft lattice, worst 0.0017 ft. Along a collar edge the two sides' cells have slopes up to 0.08
  apart on that world. That is one-sided differences at 3.5 and 4.5 ft of a curved field, about
  `h'' × 4 ft`, the same size as the slope change between two neighbouring cells of one lattice
  there. It is not a crease the field has, and not failed. Still not exact: a tile whose spacing has
  no small ratio to the lattice's, or whose collar does not fit (a todo says which), meets the lattice
  only at shared points, as before.
- **The outer edge** of the world is flat-ish join ground with no berm. The shootoff is the world's
  (MD8).
- **The radius** is per tile (C7), but a tile beside a wide gap still reaches far: 8,163 ft on the
  30-map world below, where most tiles reach 650–1,000. `--chunk` trades file count against wasted
  samples inside interiors.
- **`max_slope` 0.5 is a choice, not the game's.** MX Unleashed has no climbing limit (nothing in
  openmx's oracle). Join ground near a seam takes the tile's own edge slope (up to 1.77 near seams in
  the smoke world) and flattens over about 3 × fade.

## At C2's scale (C7)

**The pruning.** Each tile has its own radius (method, step 3). A spatial grid (`build::Bins`, squares of
32 lattice cells) lists, for each square, the tiles whose weight is above 0 somewhere within one lattice
step of it. A piece takes the union of the squares under it, and the coarse pass takes the point's
square. Then each point works out every candidate's distance (by `Source::extend_distance`, `extend`'s
own arithmetic) and extends only the nearest tile and the ones inside their radius. A tile left out has
weight 0 exactly. The sums see the same terms in the same order, so the ground is the same to the bit.
`--no-prune` (`Params::prune`) makes every tile a candidate everywhere. `tests/prune.rs` builds a nine-map
world both ways and compares every file byte for byte. The coarse pass runs on threads and keeps only
height and distance per point (C1 kept every weight: 5 GB on 30 maps).

**The 30-map world**: every national (17), every MC (10, mc06–08 at 3.5 ft) and every SM (3) map of MX
vs. ATV Unleashed, imported by `import-mxvatv` into the scratchpad. 32 threads, release build.

| run | wall | peak memory | pieces | joins |
|---|---|---|---|---|
| C1, `--max-slope 1.7` | 56.2 s | 1.6 GB | 583 (one more widening) | 443 MB |
| C7 pruned, `--max-slope 1.7` | 22.0 s | 0.44 GB | 505 | 367 MB |
| C7 `--no-prune`, `--max-slope 1.7` | 32.0 s | 0.81 GB | 505 | 367 MB, identical to pruned (`diff -r`) |
| C1, default 0.5 (fails, 14 coarse passes) | 182.9 s | 5.2 GB | – | – |
| C7, default 0.5 (fails, 14 coarse passes) | 33.7 s | 0.47 GB | – | – |

- Per pass on the 1.7 world: planning (the radii and the grid) about 1.2 s, coarse pass 0.3 s (C1:
  about 13 s, single-threaded), solve and write 1.7 s pruned and 6.1 s unpruned. The rest of the 22 s
  is loading 30 tiles and copying them (277 MB of tiles).
- At the default 0.5, the first layout is 36,508 × 30,100 ft and 563 pieces, so about 420 MB of joins
  (750 KB a 256-cell piece). Tile radii run from 705 to 8,163 ft.
- `stitch check` on the 1.7 world (with the disc): 257,270 seam points, worst height step 0.0041 ft,
  worst slope step 0.0018; 1,264 points between lattice points (mc06–08) step up to 0.72 ft (C6). It
  still FAILS, on the two problems below. 39 s.

**Why the world did not build at the default `max_slope`**: two things, neither fixed by wider gaps.

- **nat04 has missing cells** (C8). Its terrain is 1024 × 1280 samples, but there are no cells past x ≈ 896
  on its rows (`stitch profile`: NaN). `find_keep` keeps x up to 990, and the ring reads the missing cells
  as 0 ft. The join beside them drops 400 ft: near-seam slope 63, coarse slope 1.37 at 171 ft at every
  widening, and 2,800 `no cell` failures in `stitch check`. **Fixed in C8**: nat04's ground is
  `[0, 0, 896, 1280]`, so its keep is `[34, 34, 896, 1246]`. The hole's edge is a flat at 291 ft at the
  foot of a slope down from the 400 ft base, with no berm and nothing cut. With that keep, the 30 maps
  build at 1.7 on the first pass (405 pieces, where C7 needed 505), and `stitch check` passes: 261,352
  seam points, worst height step 0.0041 ft. Before C9, at 0.5 every widening stopped at mc09. Without mc09 they
  stopped at nat11 (0.619 at 171 ft, steepest edge cell 2.76). A world of nat04, nat05, nat06 and mc05
  stopped at 0.516 beside nat04's z = 1246 edge, not the hole's. These are all C9's kind.
- **Steep edges** (C9, fixed: next section). Without nat04, the 29 maps failed: 0.726 at 171 ft from
  mc09 (steepest edge cell 3.1) at every widening, and 0.608 at 315 ft with `--fade 96`. The edge's own
  slope carried just past the gate's `2 lattice + 3 fade`.

## Steep edges (C9)

Where a tile's cut runs through a hill, its edge has relief: heights that climb and fall along it, at
up to 3.1 (mc09) or 22.5 (mxu_nat17's corner mountain) a cell. The extension blurs the edge along its
length by `σ ≈ d′` (step 3), so the relief's slope across and along the join is about `relief / σ`
at distance `d`, and the blur grows no faster than the distance. Ground beside such an edge cannot be
held to `max_slope` within about `relief / max_slope` of it, however wide the gap, and C1's fixed gate
(`2 lattice + 3 fade` = 153 ft) asked for it there. A wider fade only trades taper for blur (0.608 at
315 ft for mc09).

So the gate is per edge. `Source::edge_gates` evaluates the tile's own extension alone on lattice cells
straight out from every lattice point of each edge, and on the corner squares, out to `2 lattice +
3 fade` plus twice the edge's relief over the limit (its height range plus its steepest slope across
carried over `lattice + fade`). The gate is the farthest steep cell plus one lattice step. The limit is
0.75 `max_slope`: past every gate each extension is at most that, and the rest is left for the blend
between tiles, which the widening brings down. It depends on the edge alone, so it is measured once per
build on each tile's cut (a collared tile's interior can lose up to `q − 1` samples more in the layout;
its gate is the cut's). That takes 3–5 s on 30 maps (eight threads per tile). The layout gives the
wider gate room (step 2), so the ground past it exists in every gap.

What the ground is did not change: only where it is held to `max_slope`. Inside a wider gate the join
ground carries the edge's relief down at the slope the blend gives it, and the world's todo says so,
per tile, with its gates.

Measured on the real maps (`stitch edges`, at 0.5): MX vs. ATV's mc09 297–333 ft, nat01 207–270,
nat13 306, nat17 189–315, the rest at most 306 and mostly 153; MX Unleashed's mxu_nat17 392, 315,
1,206 and 2,808 ft (its x1 and z1 edges run up the mountain), mxu_nat21 230–473, mxu_nat10 441.

- **The 29 MX vs. ATV maps** (C7's 30 without nat04) build at the default 0.5: widened to 2.44, 614
  pieces, join slope 0.452 away from the seams, 3.95 near them; 26.6 s with the copying. `stitch check`
  with the disc: ok, 253,012 seam points, worst height step 0.0041 ft, slope step 0.0018, join slope
  0.451 away from the seams.
- **All 27 MX Unleashed tiles** (world_mxu's) build at 0.5: widened to 1.56, 626 pieces, 0.472 away
  from the seams; `stitch check` with the disc: ok, 256,196 seam points, worst 0.0106 ft. C10 puts
  world_mxu.json back to 0.5.
- **The 30 maps with nat04** now pass the build's gate at 0.5 too, but only because nat04's missing
  cells (read as 0 ft, C8) count as relief: its z0 and z1 edges get gates of 657 and 662 ft, which take
  in the 400 ft drop (near-seam slope 63). `stitch check` still fails there on the missing cells. C8 is
  still needed, and a wide gate on a tile whose edge reads 0 ft is a sign of it. (Measured on C9's
  branch, before C8 merged; with C8's keep nat04's edges no longer read missing cells. Not re-measured.)
