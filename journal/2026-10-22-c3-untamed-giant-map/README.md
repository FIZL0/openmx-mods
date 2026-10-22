# C3: MX vs. ATV Untamed's giant map

`world_untamed` joins 23 open maps of the 40 that M4 imports into one world, using stitch: AM ×3, BT_A/B,
Nat ×12 and SM ×6. Each map's place, the build and check numbers and the dropped placements are in
`notes/mxvatv-later.md` *The giant map (C3)*.

## What was done

- `tools/stitch/worlds/world_untamed.json` (new): the 23 `untamed_` tiles in `Env\Master.atyList.bpb`'s
  order, everything at stitch's defaults (`max_slope` 0.5), so the sky and spawn are the first tile's,
  SM_OF. build.sh's stitch step (`stitch --mods out/mods --out <stage>`) picks the spec up and builds it
  from the tracks that the M4 import step made. No stitch code changed.
- `tools/stitch/tests/world_untamed.rs` (new), as C2's `world_mxvatv.rs`:
  - one plain test: the spec names exactly the 23 maps, once each, with the sky among them and no
    raised parameters;
  - one ignored test: it imports M4's chosen maps off the real disc (`untamed::chosen`, `load_map`)
    and derives the open set from the terrains (side over 1,280 ft). It asserts the spec equals that
    set, builds the world from the spec's values and runs `stitch check` with the disc.
- `notes/mxvatv-later.md`: new section *The giant map (C3)*, and a line under *Not verified*.

### The choice of maps

The brief says "non-racing maps (no SX, FS, MT or other stadium maps)", the same as C2. In the master
list every map has a race activity, so "non-racing" cannot be read literally. I took C2's rule: the
open outdoor maps, not the 896 ft arenas. Of M4's 40 that leaves 23. The 17 left out are SX_A–L,
FS_A–D and MT_A, all 16×16 leaves and 896 ft. SM_S1 "Motoville Speedway" is named like a stadium, but
it is a 5,760 ft 80×80 map like the other SM maps. C2 kept supermoto, so I kept it. BT_A and BT_B
(Corrido Armenta, day and night) are two maps on the disc with different `.trn` bytes, so both are in.
Their edges are identical in stitch's profile, so the ground is probably the same twice.

Maps M4 does not import (BT_C–F, HC_A, SJ_A/B, TR_A: no Free Ride) cannot be tiles, because stitch
builds from imported tracks. I did not add them.

The order: Untamed has no `UI\Tracks.dat`, and I did not look for a menu order in the code. I used
the master list's `[<MAP> General]` order instead, the disc's own. That makes SM_OF the first tile, and
so the sky. Nothing in the brief picks a sky, and C1's default is the first tile's.

## How it was proved

All runs were release builds on 32 threads, using the USA disc from the main checkout's `iso/`.

- `import-mxvatv --disc ".../MX vs. ATV Untamed (USA).iso" --out target/c3/mods/tracks --no-check`: 40
  maps imported in 12 s.
- `stitch --mods target/c3/mods --out target/c3/stage` is build.sh's own path. world_mxvatv was skipped
  (no `mxvatv_` tracks in that folder), and so were the `mxu_` worlds (no DISCX_ISO set). world_untamed
  built in 25 s: widen 2.441, 32,764 × 33,844 ft, 674 join pieces, 6 collars for the 3.5 ft maps, and 54
  placements dropped. A second build into `stage2` gave the same layout.
- `stitch check target/c3/stage/world_untamed --mods target/c3/mods/tracks --iso …/iso`: **ok** in
  123 s. It covered 213,608 seam points (worst height step 0.00439 ft, worst slope step 0.00236). All 23
  tiles came back byte for byte their imported maps', and modcheck compared each against the disc.
  This is the brief's test.
- `stitch map` picture (scratchpad only, looked at): the 5 × 5 grid as tabled in the notes, Nat_OR and
  Nat_DVCA darker, and the joins fading with no visible step.
- Dropped placements: I looked up each one's `SLT`/`COL` in its tile's `.scn` (the list is in the notes).
- `cargo test --release -p stitch --test world_untamed -- --include-ignored` with `DISCX_ISO` set:
  both tests passed in 101 s. The open set the disc gives equals the spec's 23 maps, and the world
  built from the spec checks ok.
- `./check.sh` with `DISCX_ISO` set: ok, 114 passed.

## What I tried that failed

- The first `./check.sh` (without `DISCX_ISO`) failed in M13's `accuracy` test `every_map`:
  `untamed_sm_as: the game opened files the mod lacks: [("part\\mxaiparticles.", No)]`. Neither the
  slot's `out/mods/tracks/untamed_sm_as` nor a fresh import has that name. The game asked for it and
  fell back, and it is on no part of the disc. The test can only remove such names when it can read
  the disc's PSR names, and this worktree has no `iso/` but does have `extracted/untamed/ram/`. With
  `DISCX_ISO` pointing at the main checkout's `iso/`, the test passes (91 s), and so does the whole
  gate. Nothing in git is involved, and C3 changes no code that this test reads. A worktree without
  the RAM images skips the test.

- The world itself: nothing failed. It built and checked on the first run at stitch's defaults. C8
  (holes) and C9 (per-edge gates) cover SM_HS's missing cells and the steep edges. The six 3.5 ft maps and Nat_OR's
  4.499999 ft go through C1's collars, and the check reports their meeting points without failing them.

## Not verified

- **No seam was ridden in openmx.** openmx has neither MD2 (load from `mods/`) nor MD8 (worlds of tiles),
  so it cannot load this world. The seams are proved only by `stitch check`'s numbers and the picture.
- **Nothing was compared in PCSX2 (T7).** The world is not a map the game has. Each tile is byte for
  byte its imported map, and M13 checked those against the game.
- **Where the 3.5 ft collars meet the 4.5 ft lattice**, the slope step reaches 0.2005 (51,696 points,
  height 0.00824 ft). The check reports it but does not fail it (C1's rule). I did not look at where
  the 0.2 is.
- **Inside the C9 seam gates** (8 of the 23 tiles, up to 360 ft out on SM_HS), the join ground is
  steeper than 0.5, up to 3.149 near a seam. The world's todo says so for each tile.
- **BT_A's and BT_B's heights** were not compared sample by sample. Only their cut edges match.
- **Untamed's menu order** was not looked up, so the tile order (and with it SM_OF as the sky) is the
  master list's.
- **The full build.sh was not run**, only its stitch step, through the same `cmd_specs` path.
- **The places in the notes depend on the tile order and stitch's layout.** The tests check the seams,
  not the places.
