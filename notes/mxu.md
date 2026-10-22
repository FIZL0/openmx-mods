# MX Unleashed's giant map (C4)

MX Unleashed's freeride and national maps, imported off its disc (SLUS_207.67, `MX.PSR`) as they are and
joined by `tools/stitch` (C1, C6) into one world, `world_mxu`. That puts MXU in the same form as the other
games' giant maps for the ultra map (C5). A second world, `world_mxu_free`, is just the five freerides: the
scope of openmx's P71 (last section).

```
import-mxvatv --disc "iso/MX Unleashed (USA).iso" --out STAGE      27 tiles, mxu_<track>/
stitch build OUT --id world_mxu --name ... --sky mxu_free06 --max-slope 8 STAGE/mxu_*
stitch check OUT/world_mxu --mods STAGE --iso iso
build.sh: tools/stitch/worlds/world_mxu.json and world_mxu_free.json (the tiles come off the disc)
```

## The import

`import-mxvatv` takes the MX Unleashed disc too (`src/mxu.rs`). The rest is M2's importer unchanged: the
same copy, the same read-back with openmx's readers, the same todo.

- **A track is two archives**, `Tracks\<track>.res` and `Tracks\<track>_2.res`, which the shell loads
  (`track_res_load`, boot `0x00205c40` in openmx's oracle). Both are copied whole: the second half holds
  the camera presets, particle lists, `hit.dat` and the shared bike files. The paths they share
  (`skies\…`, `bike\…`) have the same bytes on every track, so §2's rule (identical bytes, one file) is
  enough.
- **Which tracks.** `free06`, `free07`, `free09`, `free10`, `free11` and `nat01`–`nat22`: 27. Left out are
  `sx01`–`sx24` (stadiums) and the `_ss` pairs. A `_ss` pair is the same freeride cut back for two views
  (`world_load`: no ruts, fewer props, a smaller scene), not a map of its own.
- **No `.atyList`, no loose members.** The name is the track's PSR name (`free06`): the menu names were
  not read (`UI\<track>.lsi` is only the loading picture). The spawn is the scene's default (§2b). The
  PSR's loose members are `UI\`, `AppStart\` and `Col\template.cmt`, and no map file names one, so
  nothing is pulled (M15's rule is SLUS_211.04's, and it was not looked for in MXU's ELF).
- **Only a tile.** The id is `mxu_<track>` (§1's table). modcheck refuses one alone and passes it as a
  tile (`modcheck::check_tile`, new). So no build.sh step packages them. Instead `stitch`'s build.sh mode
  imports any `mxu_` tile a spec names off the disc in `$DISCX_ISO`, into `<stage>.mxu/` beside its
  stage, which build.sh removes.
- **All 27 import and read back** in 8.5 s, with 21 to 54 todo entries each. Most are kinds of file
  that game-sim and game-render have no reader for (`.phy`, `.emt`, `.dat`, `.ini`, …: the port reads
  those elsewhere or not yet), `DetailMapTilingMultiplier` 24 (free06; game-render assumes 8), and
  `.aud` only at its disc path (MD2). Test: `tests/disc.rs`
  `an_mxu_freeride_imports_from_both_halves_and_reads_back_only_as_a_tile` (generated disc), and the
  ignored `every_mxu_freeride_and_national_imports_and_reads_back_as_a_tile` (the real disc: passes).

## What the maps are

| tracks | samples | spacing | side | edge |
|---|---|---|---|---|
| free06, 07, 09, 10, 11 | 768 × 768 | 9 ft | 6912 ft | a flat plain at 189.9 ft, 630 ft wide, then the berm |
| nat01–05, 07, 09–14, 16–22 | 1280 × 1280 | 4.5 ft | 5760 ft | the 135 ft base each root cell fades to |
| nat06, nat08, nat15 | 1280 × 1280 | 4 ft | 5120 ft | the same |

- **The freerides share one outer ring.** On all five, every side's median height is 189.9 ft (flat to
  0.001) out to sample 70 (630 ft). Then a berm rises about 50 ft over 180 ft, and falls 0.7 into the
  interior. There is one odd sample in the template, 135 ft at (0, 384) on all five, inside the shootoff
  band. C1's cut (`find_keep`) takes the 150 ft shootoff band (17 samples) and stops, because the profile
  is flat there, not falling inward. So **the berm stays**, and so does 480 ft of the plain: the hills
  are the dividers between the freerides in the world, as P71 wants them.
- **The nationals are built like MX vs. ATV's** (notes/stitch.md): each 256-sample root cell's relief
  fades to a 135 ft base at its border, and much of each map is one relief tile repeated. The cut is the
  shootoff band, 34 samples (38 at 4 ft), a few samples more where a slope runs in (nat10 59–62, nat21
  62–63).
- **Spacings.** The world's lattice is 4.5 ft (19 tiles). 9 ft is a whole multiple of it, so the
  freerides need no collar. The three 4 ft nationals (8/9 of the lattice) each get a C6 collar, every
  36 ft a shared sample.
- **Steep edges.** The cut runs through hills on most nationals: steepest edge cell 0.4–4.2 (nat10 3.7, nat20
  3.8, nat21 4.2), and **nat17 22.5**. nat17's corner root cell holds a mountain up to 744 ft (the rest of
  the map is under 240), and the cut meets its flank.

## The world

Both worlds are built by build.sh from `tools/stitch/worlds/` (the `mxu_` tiles imported off the disc
first). These measurements are on this branch's stitch (C1 + C6, before C7's pruning), 32 threads, release.

| world | tiles | lattice | pieces | build | size | `stitch check` |
|---|---|---|---|---|---|---|
| `world_mxu` | 27 | 4.5 ft | 556 (3 collars) | 17 s | 689 MB | 256,232 seam points, worst step 0.0106 ft; 27,904 collar points, worst 0.0034 ft: ok |
| `world_mxu_free` | 5 | 9 ft | 50 | not timed alone | 81 MB | 29,400 seam points, worst step 0.0020 ft, join slope 0.000: ok |

`stitch check` also ran modcheck over every tile against the disc (each tile's files byte for byte its
source's) and found every interior identical to its imported map. Through build.sh's path (`stitch
--mods` with no tiles built, `DISCX_ISO` set) the two worlds came out byte-identical to the ones built by
hand (`diff -r`), in 37 s with the import.

- **`world_mxu` builds at the default `max_slope` 0.5** (C10, since C9's per-edge seam gates). C4 built
  it at 8: nat17's corner mountain meets its cut at an edge slope of 22.5 and carried 7.77 out to 171 ft,
  and nat21 0.63 at 180 ft, past C1's fixed 153 ft gate at every widening. With C9 the gates are each
  edge's own (nat17's reach 392, 315, 1206 and 2808 ft; 15 of the 22 nationals have one past 153 ft, each
  named in the world's todo), and at widen 1.5625 the world is 40,036 × 32,980 ft in 626 join pieces,
  join slope 0.472 away from the gates. `stitch check` passes it: 256,196 seam points, worst height step
  0.0106 ft. Inside a gate the join is as steep as the edge it continues (46.3 at worst near a seam; where
  was not looked up): that is the ground C9 leaves unheld, and the todo says so.
- **`world_mxu_free` builds at the default 0.5** with dead-flat joins (steepest 0.0002), because every
  freeride's edge is the same 189.9 ft plain. The world sits on a 9 ft lattice, the freerides' own. The
  tiles' origins are 6912 ft apart, one whole tile: the 300 ft `min_gap` is just under the 306 ft of the
  two cut shootoff bands (17 samples each), so the layout rounds to the tile's own size. **Each freeride
  sits exactly where it would butted edge to edge**, and the joins are only where the two shootoff bands
  were, flat at the plain's height.
- **Dropped**: 3 placements, each a `Model1` at (0, 0, 0) in a cut band (free07, nat07; nat08's
  `Model250`).
- **The world's sky** defaults to `mxu_free06` (sky34).
- **Port gaps the tiles carry** (each in its tile's todo): 95 `COL=` names in no archive, 30 of them
  only because of a space the original drops (`notes/openmx-asks.md`); the nationals' terrains need MD1's
  view, as MX vs. ATV's do (32 texture pages between blocks and tree); `DetailMapTilingMultiplier` other
  than 8 on some maps.

## P71 against this world

openmx's P71 (open, not started on 2026-10-22) splices MXU's five freeride maps inside the port: side by
side, joined where they meet, no shootoff between them, the big edge hills kept as the dividers, a
shootoff only round the whole, each map's own ground and sky, terrain, objects and emitters all joined.
MD8 (worlds of tiles, also open) says P71 is the same problem and should share its code.

| | P71 (planned) | `world_mxu_free` / `world_mxu` |
|---|---|---|
| maps | the five freerides | the five freerides / those and the 22 nationals |
| each map | as the disc has it | as the disc has it, a tile queried by its own terrain (§3, MD8) |
| edge | maps abut, hills as dividers | the 150 ft shootoff band cut; hills and 480 ft of plain kept |
| between | nothing: edge meets edge | `world_mxu_free`: flat join ground in place of the two cut shootoff bands, the tiles where edge-to-edge puts them; `world_mxu`: at least 300 ft (`min_gap`) |
| shootoff | outer edge only | outer edge only (MD8) |
| objects, emitters | P71 joins them | each tile keeps its scene; 3 props in cut bands are dropped |
| sky | each map's own | one global sky (`sky`, MD9's Global setting), free06's by default |
| ground look | each map's own | each tile's own, joins blended (`.blend`, MD9) |

**Abutting works for the freerides, and only for them.** Their outer rings are the same flat 189.9 ft
template, so two freerides butted edge to edge meet at the same height with zero slope. Only the one
135 ft sample at (0, 384) breaks it, inside the shootoff band. That is why P71 needs no join ground and C1
does: C1's joins are for maps whose edges disagree (the nationals at 83–197 ft mean edge height, the other
games). On the freerides, C1's layout comes out as P71's anyway (above): the same positions, with the
shootoff bands replaced by flat ground at the plain's height. That one 135 ft dip is gone with them.

**Can P71 load this world instead?** Yes, once MD8 (load a world of tiles) and MD2 (load from `mods/`)
exist. `world_mxu_free` is P71's scope and P71's layout. Each tile under it is bit-exact to the disc's map
(§3: the port queries the tile's own terrain inside `keep`). What P71 would give up and gain:

- The ground between two maps is a generated join piece, where P71 meant to use each map's own shootoff
  band. Both are flat at 189.9 ft, and the join pieces are 9 ft PLN terrains with the nearer map's
  surface byte (`.blend` for the look, MD9).
- One sky for the world, where P71 keeps each map's own. A world mod names one sky (§3, the user's call
  for MD9). A sky per tile would be a change to the standard and to MD9's "Per map" setting.
- Objects and emitters: P71 meant to merge them. A world keeps each tile's scene, so merging them is
  MD8's job either way. One placement is dropped, free07's `Model1` at (0, 0, 0).
- It gains the 22 nationals (`world_mxu`) and the other games (C5), all through the same loader.

So P71's own splicing code would duplicate MD8. The cheaper road is P71 = MD8 + MD2 + this mod. That is
openmx's call. It is in `notes/openmx-asks.md`.
