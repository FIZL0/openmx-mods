# The map joiner (tools/stitch, C1)

How `tools/stitch` turns track mods into one world mod (standard §3), what it found on real maps, and
what C2–C5 need to know before they use it.

```
stitch build OUT --id world_x --name NAME [--sky TILE] [--max-slope 0.5] [--min-gap 300] [--chunk 256]
             [--fade 48] [--threads N] TILE_DIR...
stitch check WORLD_DIR [--mods DIR] [--iso DIR]     seams, slopes, look, tiles against their sources
stitch profile TILE_DIR [SAMPLES]                   a map's edge profile and the cut find_keep makes
stitch map WORLD_DIR OUT.pgm [FEET]                 the world's ground as a shaded greyscale picture
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
2. **The layout** (`build::plan`). Tiles are sorted by the mean height of their interior's edge and
   laid in a near-square grid, filled snake-wise, so neighbours are close in height. Each gap is
   `max(min_gap, 1.6 |Δh| / max_slope + 2 (lattice + fade))`, where Δh is between the two facing edges.
   Each tile is centred in its cell. Its `at` is a whole leaf (16 samples). Where its spacing is a small
   fraction p/q of the lattice's, its interior's edges are moved onto lattice lines, by cutting at most
   q − 1 more samples per side.
3. **The ground** (`build::Plan::at`), on one lattice for the whole world, at the commonest tile spacing:
   - Inside an interior (closed), the tile's own ground, from the kept cell.
   - Within one lattice step of an interior (the pinned ring), that tile's edge height plus its edge
     slope times the distance. So the first join cell has the last tile cell's slope.
   - Elsewhere, modified Shepard over every tile within the radius. The weight is
     `((R − d′)₊ / (R d′))²`, where `d′` is the distance past the pinned ring. Each tile is extended
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
   more than `2 lattice + 3 fade` from every interior. Past `max_slope`, every gap is widened by 1.25
   and the build runs again.

mod.json gets two fields of the joiner's own (the loader ignores them, §1): `joins[].samples` (the
piece's samples a side, which `check` and `map` read), and `stitch` (the lattice, `max_slope`, `fade`,
the radius, the widening used, the collars, and the steepest join slope away from and near the seams).

`stitch check` takes a tile's seam points from the join that covers each stretch of the edge (the
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
- **The radius** is twice the farthest any point is from an interior, so in a small world every tile
  is a candidate everywhere. Time grows with tiles × lattice points. On C2's scale (tens of 5760 ft
  maps), expect minutes and gigabytes of joins. `--chunk` trades file count against wasted samples
  inside interiors.
- **`max_slope` 0.5 is a choice, not the game's.** MX Unleashed has no climbing limit (nothing in
  openmx's oracle). Join ground near a seam takes the tile's own edge slope (up to 1.77 near seams in
  the smoke world) and flattens over about 3 × fade.
