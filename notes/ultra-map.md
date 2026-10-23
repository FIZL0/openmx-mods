# The ultra map (C5)

Every game's giant map joined into one world mod, `world_ultra`. Each game's world stays whole: its tiles
and its joins are that world's own, byte for byte, moved by one offset. Between the worlds is new
ground, made the way stitch makes it between tiles. Code: `tools/stitch/src/ultra.rs`. Spec:
`tools/stitch/worlds/world_ultra.json`. Tests: `tools/stitch/tests/world_ultra.rs`.

```
stitch ultra OUT --id world_ultra --name NAME [--sky TILE] [--max-slope 0.5] [--min-gap 300] [--fade 48] WORLD_DIR...
stitch --mods OUT/mods --out STAGE       build.sh's step: a spec with "worlds" is built after every spec of tiles
stitch check WORLD_DIR [--mods DIR] [--iso DIR]
```

The spec is `{"id", "name", "worlds": [...], "sky"}`, and may also have `max_slope`, `min_gap`, `fade`,
`chunk` and `todo` (as a world spec). Each world is looked for in the stage first (built in the same run),
then in `--mods`/tracks/. If one is missing, the map is skipped with a line, as a world with a missing
tile is.

## The method

1. **A world is one source.** `World::load` reads a built world: its lattice (`stitch.spacing`), its
   extent (the lattice joins' far edges), and its outer band. The band is each sample on the outer edge
   and one step inside, read with openmx's `Terrain::cell` from the first join covering it. Stitch leaves
   300 ft of join ground round every world (its pad), so the band is always join ground. The load fails
   if a tile's interior comes within two steps of the edge. From the band comes a `Ring` like a tile's:
   per edge sample, the height, the slope of the last cell across and along, and the surface, plus the
   look there (the join's `.blend` bytes, by tile). The world's `Source` is its whole rectangle with that
   ring. Its seam gates are C9's `Source::edge_gates`, measured once on the world alone.
2. **The layout** (`ultra::plan`). The worlds are put in a chain. It starts with the spec's first world,
   or with the one world that can only be at the origin (below). Each next world is the one left whose
   mean edge height is nearest the last one's. The chain fills a grid snake-wise. The number of columns
   is the one whose bounding box, at the narrowest gaps, has the least area. Every world sits at its
   column's and row's start, rounded up to its own quantum. The gap to the next column or row is
   `build::gap_for` over the facing worlds: `1.6 |Δh| / max_slope` plus the edges' room past their gates,
   at least `min_gap`. Here Δh is between the mean heights of the two facing edges.
3. **Offsets.** A tile's origin must stay on a whole leaf (§3). So a world moves only by a multiple of
   every tile's leaf: 72 ft for 4.5 ft tiles, 576 ft for world_mxu (4.5, 9 and 4 ft), 504 ft for 4.5 and
   3.5 ft. A tile whose spacing has no small ratio to the lattice has no such multiple. Untamed's Nat_OR
   (4.499999 ft) is one. Its world can only stay where it is, so it goes first in the chain, at the
   origin. Two such worlds cannot both be at the origin, and the build refuses them.
4. **The new ground** (`UltraPlan::at`) is `Plan::blend` with worlds in place of tiles. Every world is
   a candidate. Each extends past its edge with its own `Source::extend` (the edge blurred with
   distance, the slope tapered over `fade`). The extensions are mixed by modified Shepard weights, with
   per-world radii from `build::radii`. Within one step of a world the ground is that world's extension
   alone (the pinned ring). So on the edge it is the world's own sample, and the first new cell has the
   world's last cell's slope. A point on a world's edge, or one step inside it (for the normals), is read
   from the band itself.
5. **Pieces.** The bounding rectangle is cut along every world's edges. Each cell of that cut that is
   not inside a world is chunked into pieces of at most 256 lattice cells: `u<column>_<row>_<x>_<z>`.
   Since C11 they are whole leaves that start a whole foot inside the cell before and run past their
   cell's end (`build::cuts`), so openmx's dropped last row and column are someone's; inside a world they
   hold the world's own ground, and the world, listed first, owns it (§3). Each world's joins are listed first, in the
   world's own order, then the new ones.
6. **Rideable.** The steps are build's: a coarse pass (every 4th sample) and then the full pieces measure
   the steepest new cell past every world's gate. Past `max_slope`, every gap is widened by 1.25.
7. **The look.** Each new sample's share for a tile is the sum, over the worlds, of the world's weight
   times that tile's byte in the world's own look at its nearest edge sample. So on a world's edge the look
   is that world's edge look, and across the gap it fades into the other world's. The surface byte is the
   nearest world's edge sample's.
8. **mod.json.** It holds every world's tiles (`at` moved, `keep` the same) and joins (renamed
   `<world less world_>_<join>`, `at` moved, `between` and `samples` the same), then the new pieces. Also
   the union of the worlds' `drop`, `source.games`, `stitch.gates` and `stitch.collars` (rects moved).
   The sky is the spec's tile, or the first world's sky. The spawn is the first world's (a tile's). The
   todo is the ultra map's own lines, then every world's lines once each. `stitch.worlds` records each
   world's `prefix`, `at`, `rect`, grid `cell`, mean edge heights and gates, for `stitch check`.
   Files are hard links where the file system allows it, so the worlds' bytes are on disk once. build.sh
   replaces a mod folder whole (`rm -rf` then `mv`), which never writes through a link. A hand edit in
   place to a world's join would change the ultra map's copy too.

`stitch check` on an ultra map (`stitch.worlds` present) adds three things to C1's checks:

- **World seams.** At every lattice sample of each world's edge that has new ground outside it, the
  world's join just inside and the new join just outside must agree. The height and the slope of the
  cell either side must match to the joins' height quantum, which is the tolerance a tile's seam gets.
  A sample with nothing outside it is on the ultra map's outer edge.
- **Slopes.** New pieces are also held to `max_slope` past every world's gates. A world's own joins are
  checked as before, against its tiles' gates.
- **Copies.** With `--mods`, every copied join is compared byte for byte with the world's own
  (`<mods>/<world>` or `<mods>/tracks/<world>`), and each tile with its world's copy when it is not a
  track there (MX Unleashed's).

## The map, built 2026-10-22

From the real discs (SLUS_207.67, SLUS_211.04, SLUS_217.01), at stitch's defaults (`max_slope` 0.5,
`min_gap` 300, `fade` 48). The worlds were built by the same run's specs (C2, C3, C4 + C10). Release,
32 threads.

| world | tiles | size, ft | quantum | edge height x0 / z0 / x1 / z1, ft | gates past 153 ft | at, ft | cell |
|---|---|---|---|---|---|---|---|
| world_untamed | 23 | 32,764.5 × 33,844.5 | origin only (Nat_OR) | 331.1 / 242.9 / 404.4 / 411.1 | x0 247.5 | 0, 0 | 0, 0 |
| world_mxvatv | 27 | 37,732.5 × 31,900.5 | 72 | 344.0 / 282.4 / 352.3 / 414.0 | none | 34,200, 0 | 1, 0 |
| world_mxu | 27 | 40,036.5 × 32,980.5 | 576 | 151.0 / 109.9 / 137.3 / 196.3 | z1 274.5 | 76,032, 0 | 2, 0 |

- **Layout.** Untamed must be at the origin, so it comes first. MX vs. ATV's edges (mean 348 ft) are
  nearer Untamed's (348) than MX Unleashed's (149) are, so it comes next. A row of three has less area
  than two by two with a cell empty. The ultra map is **116,068.5 × 33,844.5 ft** (22 × 6.4 miles).
  The gap from Untamed to MX vs. ATV is 1,435.5 ft. The gap from MX vs. ATV to MX Unleashed is
  4,099.5 ft (a 200 ft step down). The worlds are aligned at z = 0. So new ground also fills a strip
  under MX vs. ATV (1,944 ft) and under MX Unleashed (864 ft) as far as Untamed's 33,844.5 ft.
- **Build: 8 s** (the worlds read in 1.0 s, the gates in 4.3 s). It widened seven times. At 1.0 the
  steepest point was 0.549 at (71,406, 4,788), in the MX vs. ATV / MX Unleashed gap. From 1.25 to
  3.815 it was at (32,976–33,318, 1,944–3,006): the top of the Untamed / MX vs. ATV gap, where the
  facing edges differ in height near their corners far more than their means do. It passed at widen
  4.768. The result is 281 new pieces (622 MB: 466 MB of terrain, 157 MB of blend, 2 to 6 tiles a
  piece). New ground is 0.436 steep at most away from the worlds' edges and 1.588 near them. Then
  1,854 of the worlds' joins and 77 tiles were linked into place. Total 2.1 GB, of which 1.5 GB is the
  worlds' own bytes (693 MB of tiles, 822 MB of joins).
- **`stitch check`** with the discs and `--mods` the worlds: **ok**, in 5 min 28 s.
  - 728,216 tile seam points: worst height step 0.01056 ft, slope step 0.004551. These are the three
    worlds' own seams, moved.
  - **46,316 points where new ground meets a world's edge: worst height step 0.00372 ft, slope step
    0.001621.**
  - 121,844 points where joins of two spacings meet: worst 0.00824 ft, slope step 0.2115, reported and
    not failed (C6). That is more points than the worlds alone have, because one check now runs over
    every lattice alignment in the map.
  - Join slope at most 0.493 away from the seams (Untamed's own).
  - All 626 + 554 + 674 copied joins are byte for byte the worlds' own. Every tile is byte for byte its
    source, and modcheck compared each with its disc.
- **Through build.sh's path** (`stitch --mods <tracks> --out <stage>`, DISCX_ISO set): the four tile
  worlds and then the ultra map, 83 s. `diff -rq` against the hand-built map: nothing.
- `stitch map` at 72 ft a pixel (scratchpad only): three whole worlds in a row, joined by smooth bands.
  The MX vs. ATV / MX Unleashed band is the wide one.

## Limits (each a todo line where it applies)

- **New ground is no source game's.** It is a closed-form blend of the worlds' outer edges, which are
  themselves generated join ground. Each world's outer edge is still "flat join ground with no berm",
  and the shootoff is the ultra map's own edge (MD8). There is no original to compare it with (T7).
- **One widening for all gaps.** The rule is build's: every gap widens when any is too steep. Here the
  MX vs. ATV / MX Unleashed gap was sized at 4,099.5 ft, though its steepest point passed at widen 1.25.
  Widening only the gap that is steep would make a narrower map. It would change nothing about the
  smoothness.
- **The worlds keep their own layouts.** The new ground does not change anything inside a world. Where
  two worlds' facing edges differ in height, the difference is carried down across the gap, at most
  `max_slope` past the gates.
- **One lattice.** All three worlds are on 4.5 ft. A world on another lattice (a later game's, if
  its commonest spacing differs) is refused: "joins of two lattices between worlds are not made". The
  fix would be C6's collars at world scale.
- **The origin.** Only one world may hold a tile with no small ratio to the lattice. Today that is
  Untamed, with Nat_OR.
- **The look** on new ground uses each world's look at its nearest edge sample, not blurred along the
  edge the way the height is.
- **Size.** The world's far corner is at 116,068.5 ft. In single floats, world x there resolves to
  1/128 ft. MD8 has to hold positions relative to a tile or join (`notes/openmx-asks.md`).

## Re-running when a new game's giant map lands

Add its world id to `worlds/world_ultra.json`'s `worlds`. `tests/world_ultra.rs`
`the_spec_joins_every_games_world` fails until that is done. It holds the spec to every spec of tiles
that is not a subset of another (world_mxu_free is world_mxu's five freerides, so it is not joined).
Then build (build.sh, or `stitch --mods`) and run `stitch check`. Before you do, consider:

- **The layout will change.** A fourth world makes two by two the smallest grid. The chain still
  starts at Untamed.
- **The new world must be on a 4.5 ft lattice** (see Limits). It must not hold an off-ratio tile.
- **The ignored test** (`cargo test --release -p stitch --test world_ultra -- --ignored`, DISCX_ISO set)
  imports two of the discs, builds every spec, and checks the ultra map. It asserts "3 worlds, 77
  tiles", so update that assert as well.
