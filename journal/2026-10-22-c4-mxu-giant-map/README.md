# C4: MX Unleashed's giant map

MX Unleashed's 27 freeride and national maps, imported off its disc as tiles and joined by stitch into
`world_mxu`, plus `world_mxu_free` (the five freerides, P71's scope). Findings, layout and the P71
comparison are in `notes/mxu.md`.

## What was done

- `tools/import-mxvatv/src/mxu.rs` (new): the MXU origin (`mxu_`, SLUS_207.67, `Tracks`). `tracks()` lists
  every `Tracks\<track>.res` that is not a `_2` or `_ss` half. `map_folders()` keeps `freeNN` and `natNN`.
  `load_map()` builds M2's `Map` from `<track>.res` and `<track>_2.res`, with no `.atyList`, no loose
  members and no pulls, and with a `FreeRide` whose todo says why the name is the PSR's and the spawn the
  scene's. Everything after that is M2's `import` unchanged.
- `tools/import-mxvatv/src/lib.rs`: `pub mod mxu`, and the MX vs. ATV todo "the .atyList's [Free Ride]
  gives no StartPosition" is not written for an MXU map (two lines).
- `tools/import-mxvatv/src/main.rs`: takes the SLUS_207.67 disc. `--map` there takes any track
  (sx included). The read-back is `modcheck::check_tile`, since an `mxu_` mod is only a tile (§1).
- `tools/modcheck/src/lib.rs`: `pub fn check_tile`, which is `check` with `as_tile` set (additive).
- `tools/stitch/src/main.rs` + `Cargo.toml`: in build.sh mode, a spec's `mxu_` tile that is not in
  `--mods/tracks/` is imported off the MXU disc in `$DISCX_ISO` (or `iso/`) into `<stage>.mxu/` and read
  back as a tile. This is what build.sh's comment already said would happen ("stitch copies from the disc
  itself"). The stage's sibling is under build.sh's `.stage/`, which it removes. A missing disc skips the
  world with a line, as a missing tile does.
- `tools/stitch/worlds/world_mxu.json` (27 tiles, `max_slope` 8 with `max_slope_why`) and
  `world_mxu_free.json` (5 freerides, defaults).
- `notes/mxu.md` (new), `notes/openmx-asks.md` (two entries: `COL=` with a space; P71 could load
  `world_mxu_free`).
- Follow-up filed: **C10**, world_mxu back to 0.5 after C9. The runner first numbered it C8, which main
  already uses (C7 filed C8 and C9 there), so I renamed it C10 and moved it under Combined maps, away from
  where main's C8/C9 sit, so the plan merges cleanly.

## How it was proved

- `tests/disc.rs` `an_mxu_freeride_imports_from_both_halves_and_reads_back_only_as_a_tile`: a generated
  ISO with `MX.PSR` holding free01 (both halves, a shared file with the same bytes in each), its `_ss`
  pair, sx01 and nat02. It checks that only free01 and nat02 are chosen, that both archives are read, that
  the second half's own file and the shared file (once) are copied, `source.game`/`boot`, that the MXU
  todo is there and the MX vs. ATV spawn todo is not, that `check` refuses it alone, and that `check_tile`
  passes it. Mutation-checked: dropping the `_ss` filter fails it (`free01_ss` listed), and removing
  the MXU guard on the spawn todo fails it.
- `every_mxu_freeride_and_national_imports_and_reads_back_as_a_tile` (ignored, real disc): 27 tracks, all
  import and pass `check_tile`, in 13 s. Passed with `DISCX_ISO=/home/rycs/repos/openmx-mods/iso`.
- The CLI on the real disc: 27 maps imported, 0 failed, 8.5 s, 21–54 todo each.
- `stitch build` of all 27 (`--max-slope 8 --sky mxu_free06`): 556 pieces (3 collars: nat06, nat08,
  nat15 at 4 ft), 17 s, 689 MB. `stitch check --mods --iso`: **ok**. 256,232 seam points, worst height
  step 0.0106 ft, slope step 0.0043; 27,904 collar/lattice points, worst 0.0034 ft; every tile's files
  byte for byte its source's, and modcheck against the disc.
- `world_mxu_free` through build.sh's own path (`stitch --mods <empty> --out <stage>`, `DISCX_ISO` set):
  27 tiles imported off the disc, both worlds built (37 s in all), `world_mxu` byte-identical to the
  hand-built one (`diff -rq`: nothing). `stitch check` on `world_mxu_free`: **ok**, 29,400 seam points,
  worst 0.0020 ft, join slope 0.000.
- `stitch map` pictures of both worlds looked at (scratchpad only). The freerides' berms are visible,
  and so is nat17's mountain streaking into the empty grid cell next to it.
- `./check.sh`: ok, 93 passed.

## What did not work

- **`world_mxu` at the default `max_slope` 0.5 does not build.** nat17's corner root cell has a mountain
  up to 744 ft, and the 150 ft cut meets its flank at an edge slope of 22.5. The join carries 7.77 out to
  171 ft at every widening. I took nat17 out to see whether it was the only one: nat21 then holds 0.63 at
  180 ft through every widening (stopped after 4). That is C9, filed on main by C7 ("the rideable gate
  never passes on the real MX vs. ATV world"). C9 is not on this branch, and the rules say not to merge
  main. So `world_mxu` ships at 8, just above nat17's 7.77, with the reason in the spec, and C10 puts it
  back.
- First `pkill -f "stitch build …"` killed my own shell, because the pattern matched its command line. I
  used `pgrep -fa "[s]titch build"` after that. The worlds were rebuilt from scratch afterwards.

## Findings worth keeping (also in notes/mxu.md)

- All five freerides share one outer-ring template: a 189.9 ft plain 630 ft wide, then a ~50 ft berm.
  C1's cut stops at the shootoff band, so the berms stay as the dividers P71 wants. In
  `world_mxu_free`, stitch's layout puts the freerides exactly 6912 ft apart, which is edge to edge, so
  the joins replace just the two shootoff bands, flat. That is P71's layout.
- MXU's own tracks place 95 `COL=` shapes their archives lack. 30 of them (nat20's 29, free09's Large
  Egg) are there under the name without its space, which the port does not try (`Props::load` reads
  `COL=` as written). Written in `notes/openmx-asks.md`.

## How P71 compares (the brief's question)

P71 (openmx, open) splices the five freerides inside the port. `world_mxu_free` is the same five maps in
the same places, each bit-exact to the disc, with a shootoff only round the whole and the hills as the
dividers. The differences are one global sky against each map's own, each tile keeping its own scene
against merged objects and emitters, and flat generated ground where the shootoff bands were. **P71 can
load this world instead, once openmx has MD8 (worlds) and MD2 (the mods folder).** Its own splicing would
be a second copy of MD8. Details: `notes/mxu.md` *P71 against this world*.

## Not verified

- **Nothing was ridden in openmx or compared in PCSX2 (T7).** openmx has neither MD2 nor MD8, so it
  cannot load a world, or a mod at all. No seam has been ridden, and no tile has been checked against the
  game running.
- **This branch's stitch is C1 + C6 without C7's pruning.** After the merge, build.sh builds the worlds
  with main's stitch (per-tile radius, C7), so the join bytes and timings will differ from the numbers
  here. The world was not rebuilt or checked with C7's code.
- **`world_mxu` is not rideable everywhere at 0.5**: built at 8, with a join wall next to nat17 (C9, C10).
  The world's own `mod.json` todo does not mention the raised `max_slope`. Only the spec and
  `stitch.max_slope` (8) record it, because putting it in the todo means changing stitch's builder,
  which C9 is about to change. C10 covers it.
- **The MXU todo text was not tailored.** The tiles carry M2's generic todo lines. Some are worded from
  MX vs. ATV's side (e.g. "DetailMapTilingMultiplier=24 where game-render's terrain assumes 8, MX
  Unleashed's" on MXU's own free06, which is a true statement about game-render but odd on MXU's own
  map). The kinds the port reads outside game-sim/game-render (`.phy`, `.ini`, `.dat` …) are listed as
  "no reader".
- **MXU's menu names were not found.** Tiles are named `free06` and so on. UI strings were not looked
  for in the ELF or the `UI\` files.
- **Whether MXU's resource manager opens loose PSR members** as SLUS_211.04's does was not looked up in
  the ELF. No map file names one of the PSR's few loose members, so it would change nothing today.
- **The other 65 missing `COL=` shapes** (nat07 17, nat08 16, nat03 11, …): what the original does with
  them was not looked up.
- **The `stitch --mods` MXU import path has no automated test.** That would need a generated MXU disc in
  stitch's tests. It was proved once end to end on the real disc (above).
- **`world_mxu_free`'s build time alone** was not measured (only inside the 37 s run).
