# C8: tools/stitch, a tile with missing terrain cells

MX vs. ATV Unleashed's nat04 is 1024 × 1280 samples by its root cells (4 × 5). But leaf columns 56–63
(x = 896–1024) are missing on every row. The scene says `TerrainWidth=5`, which is wrong as well.
`find_keep` measured the cut from x = 1024 and kept x up to 990. The ring and `ground_in` then read the
missing cells as 0 ft (`unwrap_or`), so the join beside them dropped 400 ft. Now the interior is cut
inside the cells that exist.

## What was done

- `lib.rs`:
  - `whole_cells(t, n)`: the largest rectangle of whole leaves (16 samples) whose cells all exist, found
    by the histogram method over the leaf grid. A leaf counts as there when the cells at its first and
    last corner are (one leaf's cells come from one block).
  - `Tile::ground` holds that rectangle. `load` sets it, and fails if no leaf is there.
  - `find_keep`, `profile` and `sample` work on the ground's sides, not on `0..n`. A side at a hole cuts
    only what is left of the shootoff band past the hole: `m0 − hole`, which is 0 for nat04, because the
    game has no shootoff at a hole, only no ground. Then it runs the berm test as on any side. On a map
    with every cell, the ground is `[0, 0, n.0, n.1]`, so its keep does not change. All the other tests
    and worlds are untouched.
- `build.rs`:
  - The world's `todo` gets a line for each tile whose ground is not the whole field: "mxvatv_nat04:
    its terrain has no cells outside samples 0..896 × 0..1280 of its 1024×1280 (as on its disc): its
    interior is cut inside the cells that are there, so the ground past the hole is not in the world".
  - A dropped placement over the missing cells is now named that way, not "in the cut berm band".
- `check.rs`: fails a tile whose keep runs outside its ground.
- `main.rs`: `stitch profile` prints the ground, and its columns are the ground's sides.
- `tests/holes.rs` (new): a generated 3 × 2 root-cell tile. Its first root cell column is missing whole,
  and its last has no leaves past its half (nat04's shape), made by zeroing the child words in the
  written `.trn`. The test asserts:
  - the ground is `[256, 0, 640, 512]`;
  - the keep stops at both holes, within 2 samples (the test's hills fall faster than the 2 % berm test
    allows there), and has 34 samples of shootoff on z;
  - every keep cell exists;
  - a world of it and a whole 1 × 1 tile builds at the default max_slope 0.5;
  - the todo names the holed tile and not the whole one;
  - `stitch check` passes.
- `notes/stitch.md`: step 1 of the method, and the C8 result in the C7 section.

## How it was proved

- **The test fails without the fix.** With `ground` forced back to `[0, 0, n.0, n.1]`, which is the old
  behaviour, and the test's tile assertions removed, the build fails: "the join ground stays too steep
  however wide the gaps". That is nat04's failure. With the fix it passes (9 s in debug).
- **nat04 off the disc** (`import-mxvatv`, all 56 maps into `target/c8/`):
  - `stitch profile` gives ground `[0, 0, 896, 1280]` and keep `[34, 34, 896, 1246]`.
  - The x = 896 edge is a flat at 291.23 ft, at the foot of a slope from the 400 ft base. It has no
    berm, so nothing more is cut there.
- **The 30-map world** (17 nat, 10 mc, 3 sm, as in C7) at `--max-slope 1.7`:
  - It builds on the first pass: 405 pieces, where C7 needed 505 after widening; 12.6 s.
  - `stitch check --mods --iso` is **ok**: 261,352 seam points, worst height step 0.00409 ft, slope step
    0.0018. 3,384 points where joins of two spacings meet, worst 0.0016 ft. No `no cell` failures (C7
    had 2,800). Every tile is byte for byte its source.
- A world of nat04, nat05, nat06 and mc05 at 0.52 builds, and `stitch check` is ok: 37,304 seam points,
  worst 0.0035 ft.
- `./check.sh`: ok, 98 passed, 0 failed.

## What failed on the way, and what it showed

- **The 30 maps at the default 0.5 still do not build.** Every widening stops at mc09, 0.726 at 171 ft.
  That is C9. Without mc09 they stop at nat11 (0.619 at 171 ft, steepest edge cell 2.756), the same
  kind. nat04 is never the steepest point there.
- **The four-map world at 0.5** stops at 0.516 next to nat04, the same at every widening. The piece
  (`j002_005`, x 2304–3456, z 5760–6912) is in the gap past nat04's z = 1246 edge, toward mc05. It is
  not at the hole (x = 4968 in that layout). nat04's steepest edge cell is 1.073, so this is a C9 steep
  edge too. It builds at 0.52.
- The first test run asserted `levels == 2` for the whole 1 × 1 tile too, which is 1 level. The second
  run expected the keep to end exactly at 640 and got 639, because the test's hills (fall up to 5/190)
  pass the 2 % berm test there. The assertion is now a range.
- Rebuilding the world with a single tile, to read the log, deleted the built world: build clears its
  output folder before it plans. A failed build also leaves a folder with no mod.json, and the next
  build will not replace it ("not a world mod"). I did not change either behaviour, since neither is
  this task's. I deleted the folder by hand.

## Decisions

- **Use the largest whole rectangle, not the holes' bounding box.** A keep is one rectangle, and every
  cell in it has to exist for the ring and the joins. For an edge hole like nat04's, the largest
  rectangle is simply the field up to the hole. A hole in the middle of a map would cost a side of the
  map. Then the todo says how much the ground lost. No real map has such a hole.
- **No shootoff band at a hole.** The rider is thrown back 150 ft inside the terrain's own edge, not
  inside a hole. Cutting 150 ft more of nat04 would throw away ground the game lets you ride (accuracy
  rule). The band left past the hole still applies, so a hole narrower than 150 ft cannot move the keep
  past the real shootoff.
- `ground_in` and `ring` keep their `unwrap_or`. With the keep inside the ground they are never reached.
  `check` now fails if a keep is not inside the ground.

## Not verified

- **Not ridden in PCSX2 or openmx.** Nobody rode nat04's hole edge, or what the original game does past
  x = 896. Its `TerrainWidth=5` suggests it treats the missing cells as no ground.
- **A hole inside a map** (not at an edge) is only handled by the largest-rectangle rule. No real map
  has one, and no test makes one.
- **The default max_slope 0.5 does not build** on the 30-map world, or on nat04's four-map world. Both
  stop on C9's steep edges. nat04's world builds at 0.52.
- **No world was rebuilt but the two above.** Of the 83 imported maps (56 MX vs. ATV and 27 MXU,
  `stitch profile`), nat04 is the only one whose ground is not its whole field. So every other tile's
  keep is unchanged, and `world_mxu`/`world_mxu_free` were not rebuilt.
