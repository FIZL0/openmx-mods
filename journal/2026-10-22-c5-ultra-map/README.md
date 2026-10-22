# C5: the ultra map

Every game's giant map (C2's world_mxvatv, C3's world_untamed, C4's world_mxu) joined into one world,
`world_ultra`, with each world kept whole. The method, the layout, the numbers and how to re-run it for a
new game are in `notes/ultra-map.md`.

## What was done

- **The decision: join worlds, not tiles.** One option was to pour all 77 tiles into one `stitch
  build`. That would re-lay them all, so no game's world would stay whole, and every join would change.
  Instead each built world is one source of C1's blend. Its interior is its whole rectangle, its ring
  its outer edge as its own joins have it. The standard already says an ultra map "copies the tiles and
  joins it keeps" (§3). So each world's tiles and joins are copied byte for byte and moved by one
  offset, and new ground is made only outside every world. The seam where new ground meets a world is
  then the same kind as a tile's seam (pinned ring: the world's height on the edge, its last cell's slope
  in the first new cell). That is how "as smooth as inside one" is met.
- `tools/stitch/src/ultra.rs` (new):
  - `World::load` reads a world's outer band with `Terrain::cell`, plus its extent and the offsets it
    may take (each tile's leaf).
  - `plan` lays the worlds out: a chain by edge height, the grid with the least area, offsets rounded up
    to each world's quantum, gaps by `build::gap_for`. A world with an off-ratio tile goes at the origin.
  - `UltraPlan::at` is `Plan::blend` with worlds.
  - `pieces`, `coarse_slope`, `piece`, and `build`, which also writes mod.json and hard-links or copies
    the files.
- `tools/stitch/src/lib.rs`: `Ring::new(edges)`, taken out of `Tile::ring` (which now calls it) so a
  world's edge can be a ring. `pub mod ultra`.
- `tools/stitch/src/build.rs`: `gap_for` made `pub` (one word).
- `tools/stitch/src/check.rs`: `Report.world_seam_*`, and `world_seams` (height and slope across every
  world edge sample that has new ground outside). New pieces are held past the worlds' gates. With
  `--mods`, copied joins are compared byte for byte, and tiles also against their world's copy (MXU's
  tiles are not tracks).
- `tools/stitch/src/main.rs`: `stitch ultra …`. In build.sh mode, specs with `worlds` are built after the
  tile specs, from the stage or `--mods`/tracks/.
- `tools/stitch/worlds/world_ultra.json`: world_mxu, world_mxvatv, world_untamed, sky mxu_free06. MX
  Unleashed is first, so its spawn is the ultra map's. The order of the spec does not set the layout.
- `notes/ultra-map.md` (new); a section in `notes/stitch.md`; an entry in `notes/openmx-asks.md` (MD8
  at 116,068 ft and 2,212 terrains: query tiles in their own coordinates, load what is near).

## How it was proved

- `tests/world_ultra.rs`:
  - `two_worlds_join_whole_with_continuous_height_and_slope`: generated worlds 200 ft apart, one holding
    a 4.499999 ft tile.
    - That world is at the origin, and the other is past it on a whole leaf.
    - Every tile is moved by its world's offset, with `keep` unchanged, and every join is there renamed.
    - `stitch check` passes, with world seam steps under 0.05 ft and 0.01 slope.
    - A copied join replaced with other bytes fails the byte comparison.
    - **Mutation check**: with the pinned ring turned off in `UltraPlan::at`, this test fails (the world
      seam slopes).
  - `a_step_at_a_worlds_edge_fails_the_check`: the new piece on a world's x1 edge rewritten 2 ft higher
    fails `world_one edge 2`.
  - `the_spec_joins_every_games_world`: the spec's worlds are every spec of tiles that is not a subset of
    another. MXU is first, sky mxu_free06, nothing raised.
  - Ignored `the_ultra_map_of_every_game_builds_and_checks`: imports both MX vs. ATV discs, runs the
    `stitch` binary as build.sh does, then `check` with the discs. **Passed** in 507 s (release).
- On the real discs (DISCX_ISO=/home/rycs/repos/openmx-mods/iso), from M2/M4 imports in `target/c5/`:
  - `stitch ultra`: 8 s, 116,068.5 × 33,844.5 ft, widen 4.768, 281 new pieces, new ground 0.436 away
    from the edges.
  - `stitch check --mods <worlds> --iso`: **ok**, 5 min 28 s.
    - 728,216 tile seam points, worst 0.01056 ft.
    - **46,316 world seam points, worst 0.00372 ft and slope step 0.00162.**
    - All 1,854 copied joins byte for byte; all 77 tiles byte for byte; modcheck against the discs.
- build.sh's path, `stitch --mods target/c5/mods --out target/c5/stage2`: world_mxu, world_mxu_free,
  world_mxvatv, world_untamed, then world_ultra, in 83 s. The three worlds and the ultra map are
  `diff -rq` identical to the first build. The first worlds were built before `Ring::new` was taken out
  of `Tile::ring`, so that refactor changed no byte.
- `stitch map` picture at 72 ft a pixel, looked at: three whole worlds in a row, smooth bands between.
- `./check.sh` (with DISCX_ISO=/home/rycs/repos/openmx-mods/iso): ok, 117 passed, 0 failed.

## What did not work, or surprised

- The first synthetic world with a 4.5 and a 4.499999 ft tile took 4.499999 as its lattice (a tie goes
  to the finer one). The ultra build then refused it as "two lattices", which is the intended
  refusal. The test world has three tiles.
- With two square worlds the layout chose a column, not a row (equal area, fewer columns). The test's
  assertion was about x and is now about either axis.
- The generated 4.499999 ft scene prints its scale as 4.4999986, which modcheck wants a todo for. It is a
  test-tile artefact, and the tile now carries the todo.
- My first copy-tampering assertion was hidden by the 50-failure truncation (seam and blend failures came
  first). The byte comparison now runs before the seams, as the tiles' does.
- Hard links: a tamper test that writes into a linked file in place changes the world's own copy too,
  so the comparison cannot see it. The test removes the file and writes a new one. build.sh never
  writes in place (`rm -rf`, `mv`). The note says so.
- `-0` offsets printed for the z of worlds rounded up from 0 (`ceil(-1e-9)` is −0). Fixed with `+ 0.0`.

## Not verified

- **Not ridden or loaded in openmx.** openmx has neither MD2 (mods folder) nor MD8 (worlds), so no
  world loads at all. Whether a 116,000 ft world with 2,212 terrains works there is an open question,
  written as an ask.
- **No T7 comparison.** The new ground is generated, with no original. The tiles are the worlds'
  tiles, which C2/C3/C4 and M12/M13 compared, and they are unchanged here.
- **The look was not seen drawn.** The `.blend` bytes were checked (sums of 255, steps of at most 64,
  modcheck). Nothing renders a blend yet (MD9).
- **Per-gap widening** was not tried. All gaps widen together (build's rule). The MX vs. ATV / MX
  Unleashed gap is wider than it needs to be. That is noted as a limit, not filed as a task.
- **A fourth game's world** (M3, M11) does not exist yet. The two-by-two layout and a lattice other than
  4.5 ft were exercised only by the synthetic test (two worlds, a column) and the refusal message, not
  on real data.
- **Hard links across file systems:** the copy fallback was not exercised. Everything here was on one
  file system.
