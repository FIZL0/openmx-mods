# C7: tools/stitch at C2/C5 scale

C1's joiner gave every tile a say at every join point, with one radius: twice the farthest any point was
from an interior. This task gives each tile its own radius and gives each point only the tiles that can
have a say there. It also measures a 30-map world before C2 builds one. The method and the numbers are
in `notes/stitch.md` ("At C2's scale"). This file is the account of the work.

## What was done

- `build.rs`:
  - `radii`: each source's radius is twice the farthest coarse point of its own neighbourhood. That is
    the points it is nearest to, or within a coarse cell's diagonal of the nearest, plus C1's coarse
    margin. In a world of two tiles this comes to C1's radius, or close to it.
  - `Bins`: a spatial grid of 32-cell squares. Each square lists the sources whose weight is above 0
    within one lattice step of it (`rect gap ≤ R + 2 step`).
  - `Plan` has `radii`, `bins` and `prune` added. `radius` is still there, as the largest; mod.json's
    `stitch` adds `radius_min`. `candidates(rect)` goes through the grid. `at` works out every
    candidate's distance first and calls `extend` only for the nearest and for those inside their radius.
  - `coarse_slope` runs on threads, keeps only `(h, near)` per point, and takes each point's candidates
    from its square.
  - `build` logs every pass: world size, pieces, radius range and times for planning, the coarse pass and
    the solve. When the fine gate fails, the log names the steepest piece and the tile nearest it.
  - `Params::prune` (default on); `--no-prune` on the command line.
- `lib.rs`: `Source::extend_distance` gives `extend`'s `d` by the same arithmetic. A debug assert in
  `at` checks the two agree to the bit.
- `tests/prune.rs` (new): nine 256-sample maps in a 3 × 3 grid, one of them 220 ft above the rest, so the
  radii differ. The world is built pruned and unpruned. Every file of the two (joins' `.trn` and `.blend`,
  mod.json, the tiles) must match byte for byte. The radii must differ by more than 1.2×. Some piece
  must have fewer than nine candidates, and `stitch check` must pass.
- `notes/stitch.md`: the method's step 3, the limit on the radius, and the new scale section.
- Follow-ups C8 (nat04's missing cells) and C9 (steep edges the gate cannot pass), both before C2.

## How it was proved

- **Same output both ways.** `tests/prune.rs` passes (about 30 s in debug). I checked that it catches a
  wrong pruning: with the cut-off at half the radius, the test fails. The build itself fails ("the join
  ground stays too steep however wide the gaps"), because the weights cut off early leave steps in the
  ground.
- **On real maps.** I imported all 56 MX vs. ATV Unleashed maps (`import-mxvatv`, 16 s) into the
  scratchpad and built the 30-map world (17 nat + 10 mc + 3 sm) with `--max-slope 1.7`, pruned and
  `--no-prune`. `diff -r` of the two world folders finds no difference (505 pieces, 366,947,321 bytes of
  joins).
- **Times** (32 threads, release, measured by a wrapper with `getrusage` for peak memory): C1 56.2 s and
  1.6 GB, C7 22.0 s and 0.44 GB, C7 unpruned 32.0 s and 0.81 GB. The full table is in the notes.
- `stitch check` on the pruned 1.7 world, with the disc: 257,270 seam points, worst height step
  0.0041 ft, worst slope step 0.0018. Every tile is byte-identical to its source. It FAILS on nat04's
  missing cells (C8, below).
- `./check.sh`: ok, 90 passed, 0 failed (52 s). This includes C1's six join tests, which now run with
  the per-tile radii.

## What failed on the way

- **The 30-map world does not build at the default `max_slope` 0.5, with C1 or with C7.** The gates
  widen 14 times and give up. Two causes:
  1. nat04 has no cells past x ≈ 896, but its kept interior runs to 990. The joiner reads the missing
     cells as 0 ft (`ground_in(..).unwrap_or((0.0, 0))`, and the ring's samples), so its edge drops
     400 ft. → C8.
  2. Without nat04, mc09's steep edge (steepest edge cell 3.1) leaves 0.726 just past the gate at every
     widening, and 0.608 with `--fade 96`. → C9.
  So I took the time and size measurements at `--max-slope 1.7`, the smallest round value that builds.
  Its gaps are narrower than 0.5's would be. The 0.5 layout's size comes from the per-pass log of the
  failing runs: 563 pieces at widen 1, so about 420 MB of joins at 750 KB a piece.
- `--max-slope 1.5` also fails (fine pass 1.649 at every widening, the same nat04 spot). Those runs
  still gave the first pruned/unpruned/C1 timings, at the widest layout: 93 s, 264 s and 202 s.
- Tests: generated ids must start with `elev_` (modcheck's Real elevation prefix), and the terrain
  must be whole 256-sample root cells to match the scene's `TerrainWidth=1`. The first two runs of the
  new test failed on those, in the check only (the byte comparison passed already).
- No `/usr/bin/time` and no `bc` here, so I measured with a small Python `getrusage` wrapper in the
  scratchpad.

## Decisions

- **"Without the pruning" means the same radii with every tile a candidate.** It does not mean C1's one
  global radius, which is a different blend. The brief's test, "identical within the height quantum",
  then holds exactly: a pruned tile's weight is exactly 0, and adding 0 to a float sum is exact, so
  the output matches to the byte, which is stronger than within the quantum. The per-tile radius does
  change the ground relative to C1. C1's two-tile test worlds still pass all their own tests.
- **A radius per tile**, Franke–Nielson style, rather than a radius that varies over space. Each weight
  stays a function of distance alone, so the zero-gradient meeting at every interior (C1, step 3) is
  untouched.
- **The neighbourhood slack** is one coarse cell's diagonal, so the boundary between two neighbourhoods
  is not missed between coarse points. That puts both tiles beside a gap at twice the gap's half-width
  or more.
- **The bin size is 32 lattice cells (144 ft at 4.5 ft).** Lists stay short for the coarse pass's
  per-point lookups. Building the grid takes about a second at 30 maps, and C2 re-plans every widening.
- Parallelising the coarse pass and dropping its weights was not in the brief. At 30 maps it took about
  13 s and 5 GB a pass under C1, and the gate re-runs it on every widening, so I did it here.

## Not verified

- **Not checked in PCSX2 or openmx**: no seam was ridden. Nothing this task changes is read by the
  port differently.
- **Not measured at the default `max_slope` 0.5**: the 30-map world does not build there (C8, C9). Time
  and size at 0.5 are estimates from the planned layout's piece count.
- **Not tried at C5 scale** (several games' giant maps together): no other game's giant map exists yet.
- **Not timed on other machines**: all numbers come from this one, with 32 threads and 124 GB.
- **`stitch check` does not pass** on the 30-map world because of nat04 (C8). Its seam numbers are
  still the real ones.
- **The bit-identity guarantee is not proven for every possible world.** It rests on reasoning: a dropped
  tile's weight is exactly 0, and the nearest tile is always within its radius by the coarse margin.
  The nine-map test and the 30-map `diff -r` confirm it.
