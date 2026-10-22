# C2: MX vs. ATV Unleashed's giant map

`world_mxvatv`: the 27 open maps of M8's table (Nat01–17, SM01–03, MC01–05, MC09–10) joined by stitch
into one world. Each map's place, the build and check numbers and the dropped placements are in
`notes/mxvatv.md` *The giant map (C2)*.

## What was done

- `tools/stitch/worlds/world_mxvatv.json` (new): the 27 `mxvatv_` tiles in `UI\Tracks.dat`'s order,
  `sky` mxvatv_mc02, everything else at stitch's defaults (`max_slope` 0.5). build.sh's stitch step
  (`stitch --mods out/mods --out <stage>`) builds it from the tracks the import-mxvatv step made. No
  stitch code changed.
- `tools/stitch/tests/world_mxvatv.rs` (new): one plain test that the spec names exactly the 27 open
  maps, once each, with the sky among them and no raised parameters; one ignored test that imports all
  56 maps off the real disc, derives the open set from the terrains (side over an arena's 1,280 ft),
  asserts the spec equals it, builds the world from the spec's values and runs `stitch check` with the
  disc.
- `notes/mxvatv.md`: new section *The giant map (C2)*, and a line under *Not verified*. M8's "the 33 big
  ones" corrected to 27 (its own list adds up to 27).

### The choice of maps

The brief excludes stadiums with crowd seats and takes "MC and the open national-style maps if they
qualify by M8". M8 calls the 80×80-leaf maps the open outdoor worlds and the 896–1,280 ft ones arenas.
So the world takes all 27 big maps and leaves out SX01–18, FS01–08 and MC06–08. The nationals and
supermoto maps do have trackside crowd: crowd sounds in every `.aud`, crowd toys, and SM's
`bleacher_big_512`. They are still 5,760 ft outdoor maps, not stadiums, so I kept them. Taking out every
map with a spectator would leave only the MC maps (seven), which is not what the brief asks for.

The sky is mc02 Desert Oasis's. It is the first tile in the menu's order and one of the six maps
unlocked at the start. Nothing in the brief picks one, and C1's default is the first tile's.

## How it was proved

All runs were release builds on 32 threads, using the USA disc from the main checkout's `iso/`.

- `import-mxvatv --no-check` imported all 56 maps in 13 s into `target/c2/mods/tracks`.
- `stitch --mods target/c2/mods --out target/c2/stage` is build.sh's own path. The two `mxu_` worlds were
  skipped because the worktree has no `iso/`. world_mxvatv built in 15 s at 0.5. The seam gates took
  2.6 s. The layout widened three times (0.870, 0.685, 0.561, beside nat09 and nat12) and passed at
  1.953: 37,732 × 31,900 ft, 554 join pieces, join slope 0.470 away from the seams. 36 placements were
  dropped.
- `stitch check target/c2/stage/world_mxvatv --mods target/c2/mods/tracks --iso …/iso`: **ok**, 76 s.
  It covered 258,412 seam points: worst height step 0.00406 ft, worst slope step 0.00177. All 27 tiles
  came back byte for byte their imported maps', and modcheck compared each against the disc. This is
  the brief's test: every seam continuous, every interior identical.
- `stitch map` picture: I looked at it (scratchpad only). It shows the 6 × 5 grid as tabled in the
  notes, with joins fading between tiles and no visible step.
- `cargo test --release -p stitch --test world_mxvatv -- --include-ignored` with `DISCX_ISO` set: both
  tests passed in 113 s. The disc-derived open set equals the spec's 27, and the world built from the
  spec checks ok.
- I looked up nat08's 28 dropped placements in its scene. They are 14 redwoods (`Nat08_redwood_B`/`_E`,
  model and collision each), one per root cell along three edges, inside its cut.
- `./check.sh`: ok, 109 passed.

## What I tried that failed

- The first `./check.sh` failed in M12's `accuracy` test `every_map`. That test reads the worktree's
  ignored `out/mods/tracks/`. This slot's copy of mxvatv_nat05 there was imported at 02:03, before M12
  taught the importer to pull the `.pco` sets and shadow textures. I re-imported nat05 with the current
  importer (`--map nat05`) and moved it in place. The gate then passed. The fix is local game data only:
  nothing in git changed, and a fresh checkout skips that test.
- The world itself: nothing failed. C8 (nat04's hole) and C9 (per-edge gates) had already removed the
  two things that stopped the 30-map world at 0.5. Without the three arenas, it built and checked on the
  first run.

## Not verified

- **No seam was ridden in openmx.** The brief asks for it, but openmx has neither MD2 (load from `mods/`)
  nor MD8 (worlds of tiles), so it cannot load this world or any mod. The seams are proved only by
  `stitch check`'s numbers and the map picture.
- **Nothing was compared in PCSX2 (T7).** The world is not a map the game has. Each tile is byte for
  byte its imported map, and M12 checked those against the game.
- **Inside the C9 seam gates** (20 of the 27 tiles, up to 333 ft out on mc09), the join ground is steeper
  than 0.5, at up to 3.95 near a seam. The world's todo says so per tile. Where the 3.95 is was not
  looked up.
- **28 redwoods on nat08's edges are dropped** because they stand in its cut band. The world's `drop`
  lists them. Whether the game's view shows them was not checked.
- **The full build.sh was not run.** Only its stitch step was, through the same `cmd_specs` path. The
  `mxu_` worlds were skipped in that run because the worktree has no `iso/`.
- **The places in the notes depend on the tile order and on stitch's layout.** A later change to either
  moves the maps. The tests check the seams, not the places.
