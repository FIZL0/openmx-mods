# C9: tools/stitch, steep tile edges and the rideable gate

The rideable gate held join ground to `max_slope` from a fixed `2 lattice + 3 fade` (153 ft) off every
interior. Where a tile's cut runs through a hill, the ground there cannot be that flat. This task gives
every edge its own gate, measured from the edge's own extension, so C2's world builds at 0.5. The
method and the numbers are in `notes/stitch.md` ("Steep edges (C9)"). This file is the account.

## What was done

- **Why widening could not help.** The extension blurs the edge along its length by `σ ≈ d′`. An edge
  with relief (heights that climb and fall along it) keeps a slope of about `relief / σ` across and
  along the join at distance `d`, and that depends only on the distance from that edge, not on the
  gap. mc09's relief is 125 ft; mxu_nat17's corner mountain gives 1,113 ft. A wider fade only swaps
  taper for blur, which is C7's 0.608 at 315 ft. So I took the brief's "a gate distance per edge", and
  sized it by measuring.
- `lib.rs`:
  - `Source::gates` (per edge, feet).
  - `Source::edge_gates`: evaluates the tile's extension alone on lattice cells straight out from every
    lattice point of each edge, and on the four corner squares. It goes out to `base + 2 × relief /
    limit`. The gate is the farthest cell steeper than `limit`, plus one step, and never less than
    `base`. Eight threads, one per edge and one per corner.
  - `Source::gate_at`: past a corner, the larger of the two edges' gates.
- `build.rs`:
  - `gates()` uses limit = `GATE_SHARE` (0.75) × `max_slope` and base = C1's `2 lattice + 3 fade`.
  - `build` measures the gates once per tile, on its cut, and logs every tile whose gate is wider.
  - `plan` takes the gates. `gap_for` gives each edge room `lattice + fade` plus however far its gate
    is past C1's, so the default case is unchanged.
  - `Plan::clear` is the least, over the sources, of the distance less that source's gate.
    `Plan::gated` prefilters the sources whose gate can reach a rectangle.
  - The coarse pass and each piece's far/near split use `clear` where they used to compare the nearest
    distance with a constant. For an edge at C1's gate the two tests are the same.
  - mod.json: `stitch.gates` lists `{tile, edges}` for every tile with a wider gate, and a todo line
    per such tile gives its gates.
- `check.rs`: reads `stitch.gates`, so the slope check uses the same per-edge gates.
- `main.rs`: `stitch edges [--max-slope] [--fade] TILE_DIR...` prints each edge's height range,
  steepest cell and gate.
- `tests/steep.rs` (new): two 512-sample maps. One is flat. The other has a 300 ft mesa with 3:1
  flanks that its x0 cut runs through (steepest edge cell ≥ 3.0, asserted). The world builds at the
  default 0.5. The mesa edge's gate is past 203 ft (it is 554). The other seven edges keep 153.
  Ground inside the gate is steeper than 1 (so the gate is not slack). mod.json has the gates and the
  todo, and `stitch check` passes.

## How it was proved

- **The test catches the bug.** I made `gates()` return C1's 153 ft on every edge. The new test then
  fails as the real world did: 1.122 at 189 ft from the mesa at every widening, then "the join ground
  stays too steep however wide the gaps". With the fix it builds at widen 1 (far slope ≤ 0.5, coarse
  0.154).
- **First attempt at the test, too easy.** A 240 ft Gaussian bump 58 ft wide gave a 3.5 edge cell,
  but the old gate passed that world too: the bump blurs out within 153 ft. Steepness alone is not
  the problem; relief along the edge is. A 150 ft mesa also passed the old gate (only 162 ft of gate
  needed). The 300 ft one reproduces the bug.
- **Real maps.** I imported all 56 MX vs. ATV maps and MX Unleashed's 27 tiles into `target/c9/`
  with `import-mxvatv`. All builds and checks below are at the default max_slope 0.5, release, 32
  threads:
  - **29 MX vs. ATV maps** (C7's 30 without nat04): build ok, widened to 2.44, 614 pieces, far slope
    0.452. `stitch check --mods --iso`: **ok**, 253,012 seam points, worst step 0.0041 ft, far slope
    0.451.
  - **27 MXU tiles** (world_mxu's): build ok, widened to 1.56, 626 pieces, far slope 0.472.
    `stitch check --mods --iso`: **ok**, 256,196 seam points, worst step 0.0106 ft.
  - **30 MX vs. ATV maps with nat04**: the build passes 0.5 (widen 1.95), but `stitch check` FAILS
    with 2,811 `no cell` failures on nat04 (C8). nat04's missing cells read as 0 ft, so its z0 and z1
    edges look like 400 ft of relief and get gates of 657 and 662 ft, and that hides the drop from the
    build's gate (near slope 63). The C8 fix is still needed. I wrote this in the notes.
- `./check.sh`: ok, 101 passed, 0 failed (47 s).

## Decisions

- **Measure the gate rather than derive it.** Bounds such as `relief / σ` are loose by a factor of
  two or more either way (a narrow hill blurs out at once). Evaluating the extension costs 3–5 s per
  world and gives the actual distance.
- **0.75 × max_slope for the extension alone.** Past the gates the blend's slope is the weighted
  extensions' slopes (at most 0.75 × max_slope) plus a term from the weights' gradient times the
  height differences. Widening shrinks that term but never to zero, so the limit leaves it a quarter
  of the slope.
- **Gates on the cut, once per build.** They depend only on the edge, the lattice step, the fade and
  `max_slope`. A collared tile's layout can cut up to `q − 1` more samples. Its gate is still the
  cut's: a sample or two of edge does not move it.
- **The gate does not change the ground.** It only moves where `max_slope` is asked for. Inside a
  wider gate the join carries the edge's relief down as steeply as the blend makes it (3.95 near the
  seams on the 29-map world, 46 on MXU's next to mxu_nat17's 22.5 mountain edge). The world's todo
  says so, tile by tile.
- I did not edit `tools/stitch/worlds/world_mxu.json`. Putting it back to 0.5 is C10's task. The
  27-tile build above shows C10 can.

## Not verified

- **Not ridden in PCSX2 or openmx.** Nobody has driven the steep ground inside a wider gate, which is
  at most 2,808 ft out, on mxu_nat17. Nothing the port reads changed format. `stitch.gates` is a
  joiner field, and the loader ignores those (§1).
- **The 46 near-seam slope on the MXU world was not traced.** It is steeper than mxu_nat17's steepest
  edge cell (22.5), and I did not locate the cell. It lies inside a gate (the check passes), so it is
  not a gate failure. It may be the pinned ring carrying a 22.5 cell's slope round a corner. Not
  checked.
- **No guarantee that the gate's sampling catches every steep cell.** The extension is sampled on
  lattice cells aligned to the interior's edges, out to `base + 2 × relief / limit`. A steep cell past
  that reach, or between the sampled cells, is not caught by the measurement. The build's full pass and
  `stitch check` still look at every real cell past the gate, and both passed on the real worlds.
- **Not run on C2's actual world spec.** No `world_mxvatv.json` exists yet (C2). I used C7's set of
  maps.
- **C5-scale worlds (several games together)** were not built: they do not exist yet.
