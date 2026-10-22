# C10: world_mxu back to max_slope 0.5

## What was done

- `tools/stitch/worlds/world_mxu.json`: dropped `max_slope` (8) and `max_slope_why`. The world now builds
  at the default 0.5.
- A spec can now add lines to the world's todo. `build::Params` gained `todo: Vec<String>`, and `build`
  appends those lines after stitch's own fixed lines and before the per-tile ones. `stitch --mods/--out`
  (build.sh's step, `cmd_specs`) reads the spec's optional `"todo": [...]` into it. This closes the gap
  the C4 journal described: a spec that raises `max_slope` can now say why in the world's mod.json, where
  before only the spec and `stitch.max_slope` recorded it. No spec uses `todo` today. world_mxu no longer
  needs a raised slope, so there was nothing to put in one. `stitch build` (the CLI) has no flag for it.
  Only specs need it, and I didn't add one I had no use for.
- notes/stitch.md documents the spec's `todo` key. notes/mxu.md replaces the "built at 8" paragraph with
  the 0.5 build and the numbers below.

## How it was proved

The release `stitch --mods target/c10/mods --out target/c10/stage` ran with
`DISCX_ISO=/home/rycs/repos/openmx-mods/iso` and an empty `mods/`. That is build.sh's path, so the 27
MXU tiles were imported off the disc into `stage.mxu/`. Both specs ran, in 26 s total:

- world_mxu: the seam gates were measured in 3.1 s (nat17: 392, 315, 1206, 2808 ft). The rideable gate
  failed at widen 1.0 (0.612, j017_004 next to mxu_nat15) and at 1.25 (0.587). It passed at 1.5625:
  40,036 × 32,980 ft, 626 pieces, join slope 0.472 away from the seams, 46.27 near them, and 3
  placements dropped.
- `stitch check stage/world_mxu --mods stage.mxu --iso …/iso`: **ok**. It covered 256,196 seam points
  (worst height step 0.01056 ft, slope step 0.0046) and 27,870 points where joins of two spacings meet
  (worst 0.00348 ft). Every tile came back byte for byte its source's. It took 41 s.
- The world's mod.json has `stitch.max_slope` 0.5 and 21 todo lines. Three are stitch's own, three are the
  4 ft collars (nat06, nat08, nat15), and fifteen are the C9 gate lines, one per national whose gate is
  past 153 ft.
- world_mxu_free still builds at 0.5 (0.000 away from the seams).

The test is `tools/stitch/tests/spec_todo.rs`. It builds two generated tiles with `p.todo` set to one
line and asserts that the line appears exactly once in the world's todo and that stitch's own lines come
first. It also runs `stitch check`, which passes. I mutation-checked it: with the `todo.extend` line
commented out, the test fails.

## What I tried that failed

Nothing failed. C9 had already done the work, and the world built and checked on the first run.

## Not verified

- The spec-to-Params path in `cmd_specs` (one line parsing `"todo"`) has no test. The test covers
  `Params.todo` going into mod.json. `cmd_specs` always reads `tools/stitch/worlds/`, so a test spec
  would have to be a real world spec. I checked by reading the code that the parse matches the other
  optional keys.
- Where the 46.27 near-seam slope sits was not looked up. It is inside a C9 gate, and the todo covers
  that ground.
- I did not ride the world or load it in openmx or PCSX2 (T7). No world mod can be loaded yet (openmx
  MD8/MD9).
- The full build.sh was not run. Only stitch's step of it ran, through the same `cmd_specs` path.
