# MX vs. ATV Unleashed (PS2, SLUS-21104)

First survey 2026-10-21, off the USA disc, using nothing but openmx's own readers (`game-sim`'s
`archive::{Psr, Res}` and `track::{Terrain, Scene, Collision}`). Headline: **the track formats are MX
Unleashed's.** Same container, same `.res`, same PLN v7 terrain, same INI scene, same `.col`. Bringing a map
across is a repack plus the gaps below, not a new decompilation.

## Where things are

```
SLUS_211.04, GAME_REL.BIN, UI_REL.BIN    the boot ELF and two overlays (boot and GAME_REL in Ghidra since M15)
MXVSATV.PSR   1.64 GB   the one container: PSR version 2, 20,391 members, the same directory layout
                        as MX.PSR (openmx's Psr::parse reads it unchanged)
  Env\<MAP>\<map>.res        the map: <MAP>.ps2.trn, <MAP>.scn, <MAP>.toy, .esb, Col\*.col, Geo\*.psg,
                             sky, particles — openmx's Res::parse and read() take it unchanged
  Env\<MAP>\<map>_mat.res    the map's materials/textures
  Env\<MAP>\<map>_mst.res    small; not opened yet
  Env\<MAP>\<MAP>.scn/.toy   also loose beside the .res
  Env\*.aty                  activity types: Freeride.aty, OpenClass.aty, HillClimb.aty, National.aty …
```

MX Unleashed calls the folder `Tracks\`; this disc calls it `Env\`. Unlike MX.PSR, members here are
mostly loose (20k of them) with the per-map `.res` beside them.

Finding the PSR in the ISO: plain ISO 9660, root directory from the PVD at sector 16 (+156). openmx's
`game-render/src/disc.rs` does exactly this for `MX.PSR`; only the file name differs. `tools/discx` does it
for all three discs: `discx ls mxvatv 'Env\*\*.res'`, `discx x mxvatv 'Env\Nat01\*' extracted/mxvatv/`.

## The 56 maps

Surveyed 2026-10-22 by `tools/survey` (T6), which reads every `Env\<MAP>\<map>.res` and every `.col` on
the PSR with openmx's readers alone: `Terrain::parse`, `Scene::parse`, `Collision::parse`, `Res`.
`survey mxvatv` prints the tables below; they are pasted unedited under *Survey tables*.

- **Terrain.** All 56 read. 2 levels, 80×80 leaves at 4.5 ft (5760 ft) on MC01–05, MC09–10, Nat01–17 and
  SM01–03, except Nat04, which is 56×80 (4032 × 5760 ft; the first survey's "4480 leaves" is that). 1 level,
  16×16 at 3.5 ft (896 ft) on FS01–08, MC06–08 and SX03–18. SX01–02 are 16×16 at 5 ft (1280 ft). For
  comparison, MX Unleashed's `free06` is 2 levels, 48×48 leaves at 9 ft (6912 ft), 842 blocks.
- **Scene terrain keys** agree with the file (`TerrainWidth` × 16 = leaves across, `TerrainWidthScale` =
  the file's scale) everywhere except: MC08 and SX18 give `TerrainWidthScale=3` against the file's 3.5,
  SX07 gives `TerrainBreadth=2` on a 16×16 terrain, and Nat04 says 5×5 for its 56×80. Many scenes name
  `<MAP>.trn` where the archive holds `<MAP>.ps2.trn`. MC03's `SceneName` is `MachineChallenge04`.
- **Collision placements** name 16 `.col` that are on no part of the disc: not in the map's archive,
  not in `Col\`, not in any other archive (`anaheim_pole`, `dino_egg_a/b/c`, `natdropgate`,
  `natstartgate`, `ring_trigger`, `trash_can`, `recycling_can`, …; the tables list them per map). Some
  have a `.pco` beside the map instead (`Env\Nat01\recycling_can.pco`). What the game does with such a
  placement is not known yet.
- **`.col`:** 14,301 in the map archives, 7 refused. Of the 1,643 loose ones (`Col\`, `Env\*\Anim\`,
  `Veh\Col\`), 15 are refused. Every refusal is `collision class 1 is not understood`, and every one sits in
  an `Anim\` folder: windmill blades, a Ferris wheel, swinging signs and doors, a tyre swing, animals.
  So class 1 is the animated-object class (openmx MD5).

`MC` is the scenes' `SceneName=MachineChallenge0N`. Which maps the game offers as Free Ride, and what each
is called, is under *Free Ride: what each map is* (M8).

## Free Ride: what each map is

Read 2026-10-22 (M8) off the USA disc, every file named here pulled with `discx x mxvatv`:

- **`Env\<MAP>\<MAP>.atyList`** (Nat02's is `NAT02.atylist`), an INI. `[General]` holds `DisplayName` and
  one `Activity = "<section>"` per activity offered; each such section gives `Type`, `DisplayMenu`,
  `DisplayTrack`, the `.tdf`/`.cs`/`.rpy` it loads, an optional `ATYFile`, `StartPosition` /
  `StartDirection`, and `BarVehicleType` lines. No `Activity` line on the disc is commented out, and no
  section is left unlisted.
- **`Env\<Type>.aty`**, one per activity type, with the spaces dropped: `Freeride.aty`, `National.aty`,
  `HillClimb.aty`, `MachineChallenge.aty`, … (`machine challenge ab` has `Machine Challenge ab.aty`, with
  its spaces, and no space-free name in the ELF's list below). `Freeride.aty` turns everything off:
  `OffTrackReset`, `UseTriggers`, `KeepLapRecords`, `ShowOffTrack`, `ShowWrongWay` all `FALSE`, no lap
  count, no duration, no start delay, and no race audio (3-2-1, drop
  gate, fireworks, waypoint). The type → file link is read from the boot ELF's strings, not its code: `SLUS_211.04` holds the type names (`free ride` at file offset `0x2a398b`) followed by the
  same list without spaces (`freeride` at `0x2a3a50`), and the formats `%s%s.aty` (`0x2a81a6`) and
  `%s%s\%s.atyList` (`0x2a8254`).
- **`UI\Tracks.dat`**: the order the UI lists the maps in, and the six unlocked from the start (`[sx03]`,
  `[sx09]`, `[nat05]`, `[nat06]`, `[mc02]`, `[FS04]` with `Locked = FALSE`). **`TrackList.res`** (named
  in the ELF at `0x29b1d6`) is `UI\tracks.dat` plus the 56 `.atyList`s, each byte-identical to the loose
  file.
- **`UI\Store.dat`**: one store item per map (`AssetName = "NAT01"`, `StoreText = "GOBLIN GULCH"`), plus
  the hill-climb and short-track unlocks. Its names agree with `DisplayName` except that it drops "FMX" on
  the FS maps and calls MC04 "Glamis".
- **`Loc\*.loc`, `Loc\L_*_ENG.bin`** have activity names (`STR_MODE_FREERIDE`), not map names. The map
  names are the `.atyList`s' `DisplayName`s and nothing else.
- **SceneName**: the survey table's *scene* column (`Env\<MAP>\<MAP>.scn`); it is a working title
  (`National02:Ireland`, `Nat07[Quadzilla]`, MC03's is `MachineChallenge04`) and never what the UI shows.

**All 56 maps offer Free Ride.** Every `.atyList` lists `Activity = "Free Ride"` with `Type = "free ride"`.
Its section differs by series:

- **Nationals and Supermoto** (Nat01–17, SM01–03): no `ATYFile`, so only `Env\Freeride.aty`. The map's own
  `Nat0N.aty` (the drop gate `NatDropGate`, leader indicators) belongs to the National and is not named.
  `StartPosition` is commented out on all of them (`;StartPosition = 300.0f, 90.0f, 500.0f`).
- **Open class / machine maps** (MC01–10): `ATYFile = MC0N.aty`, the same file the race uses. Only MC01 sets
  a start (its race's: 2465.5, 401.1, 1255.1).
- **Freestyle** (FS01–08): `ATYFile = FS0N.aty`, `AllowRadar = FALSE`. Only FS05 sets a start.
- **Supercross** (SX01–18): `ATYFile = FreeRide.aty` from the map's own folder (262 bytes: three camera-flash
  groups, three crowd groups), `AllowRadar = FALSE`.

Where no start is given, the game puts the rider at the scene's `DefaultStartPosition` (M12, seen in
PCSX2 on every map; *Against the game*).

Columns: **map**; **name**, the `.atyList`'s `DisplayName` (the store's text where it differs); **UI**,
the map's place in `UI\Tracks.dat`, ✓ = unlocked at the start; **activities**, from `[General]`, by `Type`
(*Waypoint* is `open class waypoint`, *Machine A-B* is `machine challenge ab`); **Free Ride ATYFile**,
**start** (an uncommented `StartPosition`) and **barred** (`BarVehicleType`; *air* = BiPlane and
Helicopter) from the `[Free Ride]` section; **leaves, side (ft)** from the survey table above.

| map | name | UI | activities | Free Ride ATYFile | start | barred | leaves, side (ft) |
|---|---|---|---|---|---|---|---|
| fs04 | St. Paul FMX (store: St. Paul) | 1 ✓ | High Score, Combo, Follow the Leader, Free Ride | FS04.aty | no | air, buggy, baja, sand rail | 16×16, 896 |
| fs05 | New Orleans FMX (store: New Orleans) | 2 | High Score, Combo, Follow the Leader, Free Ride | FS05.aty | yes | air, buggy, baja, sand rail | 16×16, 896 |
| fs07 | Portland FMX (store: Portland) | 3 | High Score, Combo, Follow the Leader, Free Ride | FS07.aty | no | air, buggy, baja, sand rail | 16×16, 896 |
| fs02 | Austin FMX (store: Austin) | 4 | High Score, Combo, Follow the Leader, Free Ride | FS02.aty | no | air, buggy, baja, sand rail | 16×16, 896 |
| fs03 | Boston FMX (store: Boston) | 5 | High Score, Combo, Follow the Leader, Free Ride | FS03.aty | no | air, buggy, baja, sand rail | 16×16, 896 |
| fs06 | Las Vegas FMX (store: Las Vegas) | 6 | High Score, Combo, Follow the Leader, Free Ride | FS06.aty | no | air, buggy, baja, sand rail | 16×16, 896 |
| fs08 | Orlando FMX (store: Orlando) | 7 | High Score, Combo, Follow the Leader, Free Ride | FS08.aty | no | air, buggy, baja, sand rail | 16×16, 896 |
| fs01 | Little Rock FMX (store: Little Rock) | 8 | Follow the Leader, High Score, Combo, Free Ride | FS01.aty | no | air, buggy, baja, sand rail | 16×16, 896 |
| mc02 | Desert Oasis | 9 ✓ | Machine, Free Ride | MC02.aty | no | — | 80×80, 5760 |
| mc01 | Killington | 10 | Machine A-B ×2, Free Ride | MC01.aty | yes | — | 80×80, 5760 |
| mc03 | Red Country | 11 | Machine A-B, Free Ride | MC03.aty | no | — | 80×80, 5760 |
| mc04 | Glamis South (store: Glamis) | 12 | Machine, Free Ride | MC04.aty | no | — | 80×80, 5760 |
| mc05 | Woodstock | 13 | Machine, Free Ride | MC05.aty | no | — | 80×80, 5760 |
| mc06 | Monster Stadium | 14 | Machine, Free Ride | MC06.aty | no | air | 16×16, 896 |
| mc07 | Trophy Stadium | 15 | Machine, Free Ride | MC07.aty | no | air | 16×16, 896 |
| mc08 | 19th Hole | 16 | Machine, Free Ride | MC08.aty | no | air | 16×16, 896 |
| mc09 | Quechee Gulch | 17 | Machine Waypoint, Free Ride | MC09.aty | no | — | 80×80, 5760 |
| mc10 | Land and Air | 18 | Machine Waypoint, Free Ride | MC10.aty | no | — | 80×80, 5760 |
| nat05 | Lago | 19 ✓ | National, Short Track, Waypoint, Free Ride | — | no | — | 80×80, 5760 |
| nat06 | Juniper Park | 20 ✓ | National, Hill Climb ×10, Free Ride | — | no | — | 80×80, 5760 |
| nat03 | Glamis | 21 | National, Short Track, Waypoint, Free Ride | — | no | — | 80×80, 5760 |
| nat10 | Maple Hill | 22 | National, Short Track, Waypoint, Free Ride | — | no | — | 80×80, 5760 |
| nat02 | Tipperary | 23 | National, Short Track, Waypoint, Free Ride | — | no | — | 80×80, 5760 |
| nat01 | Goblin Gulch | 24 | National, Hill Climb ×10, Free Ride | — | no | — | 80×80, 5760 |
| nat04 | Oceanside | 25 | National, Hill Climb ×6, Free Ride | — | no | air | 56×80, 4032 |
| nat14 | Santiago | 26 | National, Hill Climb ×10, Free Ride | — | no | — | 80×80, 5760 |
| nat09 | Deadman Wash | 27 | National, Hill Climb ×10, Free Ride | — | no | — | 80×80, 5760 |
| nat16 | Northern Lights | 28 | National, Short Track, Waypoint, Free Ride | — | no | — | 80×80, 5760 |
| nat13 | Sienna Ridge | 29 | National, Hill Climb ×10, Free Ride | — | no | — | 80×80, 5760 |
| nat07 | Valhalla | 30 | National, Hill Climb ×10, Free Ride | — | no | — | 80×80, 5760 |
| nat11 | Sutter Raceway | 31 | National, Short Track, Waypoint, Free Ride | — | no | — | 80×80, 5760 |
| nat08 | Redwood Valley | 32 | National, Short Track, Waypoint, Free Ride | — | no | — | 80×80, 5760 |
| nat17 | Keweni Falls | 33 | National, Hill Climb ×10, Free Ride | — | no | — | 80×80, 5760 |
| nat12 | Temple of Apophis | 34 | National, Short Track, Waypoint, Free Ride | — | no | air | 80×80, 5760 |
| nat15 | Smuggler's Notch | 35 | National, Hill Climb ×10, Free Ride | — | no | — | 80×80, 5760 |
| sm01 | Mesa | 36 | Supermoto, Free Ride | — | no | — | 80×80, 5760 |
| sm02 | Highlands | 37 | Supermoto, Free Ride | — | no | — | 80×80, 5760 |
| sm03 | The Glades | 38 | Supermoto, Free Ride | — | no | — | 80×80, 5760 |
| sx03 | Savannah | 39 ✓ | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx09 | New Orleans | 40 ✓ | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx01 | Los Angeles | 41 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 1280 |
| sx18 | Boulder | 42 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx10 | Toronto | 43 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx07 | St. Paul | 44 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx16 | Tucson | 45 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx08 | Boston | 46 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx02 | Denver | 47 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 1280 |
| sx04 | San Diego | 48 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx13 | Las Vegas | 49 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx14 | Mexico City | 50 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx11 | Orlando | 51 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx15 | Portland | 52 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx06 | Little Rock | 53 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx12 | Philadelphia | 54 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx17 | Charleston | 55 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |
| sx05 | Austin | 56 | Supercross, Free Ride | FreeRide.aty | no | air | 16×16, 896 |

So by `.atyList` there are 56 freeride maps. What a freeride mod for OpenMX wants, an open outdoor world
like MX Unleashed's `free06`, is the 33 big ones: Nat01–17, SM01–03, MC01–05, MC09–10 (80×80 leaves,
5760 ft; Nat04 56×80). The FS, SX and MC06–08 maps are 896–1280 ft arenas.

### The first mods

The big maps sorted by what the survey above found against them:

| group | maps | against them |
|---|---|---|
| clean | nat02 Tipperary, nat03 Glamis, nat09 Deadman Wash, nat13 Sienna Ridge, nat14 Santiago, nat15 Smuggler's Notch, nat17 Keweni Falls, mc02 Desert Oasis, mc03 Red Country, mc04 Glamis South, mc05 Woodstock, mc10 Land and Air, sm01 Mesa, sm02 Highlands, sm03 The Glades | nothing the survey found: every `.col` a placement names is on the disc and reads |
| a placement's `.col` is on no part of the disc | nat01 (trash and recycling cans), nat06 (dino eggs), nat07 (`nat07_churchsnow`), nat08, nat11, nat16 (drop and start gates; nat11 also `singlebale01`), nat12 (`nat12_sphinx`), mc09 (`ring_trigger`) | what the game does with such a placement is open |
| class-1 (animated) `.col` refused | mc01 (windmill blade), nat04 (Ferris wheel), nat05 (signs, gallows door), nat10 (mill wheel, tyre swing; also four missing `.col`) | needs openmx MD5 |

**First: nat03 Glamis, nat09 Deadman Wash, nat02 Tipperary**, in that order, once M2's importer exists.
Why these three:

- All three are clean. Each is a National with 794 blocks and 48 pages, the shape MXU's own nat09 and
  nat18 already have (`notes/mxu-terrain-limits.md`), so nothing about the terrain is new to the port
  beyond MD1.
- Their Free Ride is `Env\Freeride.aty` alone, so a mod needs nothing from the race's `.aty`. All three
  have the same todo: the start position, since none is in the `.atyList`.
- They differ from each other: Glamis (`SceneName=National03:Glamis`) and Tipperary
  (`National02:Ireland`) are named for different places, and Deadman Wash has ten hill climbs in its
  `.atyList` where the other two have a short track and a waypoint race. That they look different is a
  guess from these names, not checked.
- None is unlocked at the start; that does not matter to a mod.

Next would be the rest of the clean list: nat14, nat15, nat17, nat13 (1050 blocks and 48 pages, the
one big map whose page count breaks the one-page-per-16-blocks rule above), then sm01–03 and mc02–05,
mc10. The other two groups wait on a decision about missing `.col` and on MD5.

## The importer (M2): what the 56 mods hold and what is in their todo

`tools/import-mxvatv` (2026-10-22, the USA disc) writes one track mod per map, `mxvatv_<map>`: the
members of `Env\<MAP>\<map>.res`, `<map>_mat.res` and `<map>_mst.res` at their own paths (25,011 files
over the 56 maps, 449 MB), and a `mod.json`. Nothing is edited. No two archives of a map hold one path
with different bytes, and none spells one path two ways (the import would stop on either). Every mod
reads back through `modcheck` against the disc. The loose members beside the archives are not copied
(race, AI, camera, replay, the activities' `.aty`, `Anim\`), except those the game opens for the map
because its archives lack them (M15, *Loose PSR members* below). Every loose member at the path of an archive member
(the scene, `.toy`, `.esb`, `.aud`, …) has the same bytes as that member. Every gap is a `todo` entry (1,112 over the 56). By kind:

- **Name and start**: the `.atyList`'s `DisplayName`, and its `[Free Ride]` `StartPosition` /
  `StartDirection` as `spawn` on MC01 and FS05. The other 54 spawn at the scene's
  `DefaultStartPosition`, which M12 saw is the game's (*Against the game*); their todo now says so only
  where that is under the ground.
- **Scene keys MX Unleashed never has**, against MXU's 57 scenes (`src/mxu_keys.rs`):
  `EcoAmbientScale` and `EcoDiffuseScale` in every `[Environment]` (1,1,1 on all but two maps).
  On the 27 big maps (Nat01–17, SM01–03, MC01–05, MC09–10), `[Model<n>]` and `[Object<n>]` also carry
  `Activity` ("Free Ride", "Climb01".., "<name> Waypoint Race", "Airplane Race", "Killington Forwards
  Race"), `Hide`, `HasCol`, `MinPitch`, `MaxPitch` and `PlayerToFace`. 20 of them also carry
  `AlphaFade`, and 10 carry `Invisible`, `IsTrigger` and `Waypoint`. These are start zones, waypoint gates,
  hill-climb markers and air-race rings. The port reads none of these keys, so it draws and collides every one
  of them in a Free Ride. Which of them the game shows in a Free Ride was not looked up.
- **Collision**: as the survey found (16 `.col` placed and on no part of the disc outside a `.res`; class 1 on
  mc01, nat04, nat05, nat10).
- **Scene against terrain**: Nat04, SX07, MC08, SX18, as §2a has them.
- **Ecosystem**: MC02, SM01, SM02 and SM03 give `EcosystemFile=<map>` with no extension, and the
  archive holds `<map>.esb`. game-render reads the name as written, so it finds no plants there.
  Nat02's ecosystem names ten models (`nat02_busha.psg` … `nat02_treee.psg`) that its archives lack,
  and they sit loose in the PSR's `Eco\`. MC03, MC04, MC05, MC09, MC10, Nat13, SM01 and SM03 (maps that reuse
  another map's plants) lack that map's `<map>eco.psm` and `<map>bb.psm`, which are loose in `Tex\`.
  The game reads them loose, and since M15 the importer copies them (*Loose PSR members* below).
- **Models**: 2,772 `.psg`, all read by `psg::Model::parse` except Nat12's five `Eco\` plants
  (`unexpected UNPACK of 12 x 2 x 8 bits at 489`) and Nat16's `Geo\northernlights.psg` (`the file ends
  at 23513 but 3933048 bytes were wanted at 373`).
- **Textures**: all 2,889 `.pst`, 129 `.atm` and 56 skies decode. A material's texture is missing in
  three ways. (1) `screen.psm` on the 32 stadium maps wants `RenderTarget`, a texture drawn at run time.
  (2) Nat12's `Nat12 _genericsandstone` and `Nat12 _pillarA` carry a space, and the archive has the
  name without it. (3) Nat05's and MC05's `Nat05_liverystable_detail.dxm` has a `.dxm` tail, and the
  archive has the name without it.
- **Animated objects**: the 44 `.mot` under `Env\<map>\anim\`. The 24 of mc01, mc07, mc08, nat04, nat05 and
  nat10 are refused by `skin::Clip::parse` (`not a .mot: a name of … bytes`), so the port has no clip
  for those windmills, signs and score-tower people.
- **Kinds no reader here takes**: `.mst`, `.pco`, `.prt`, `.mcf`, `.vut`, `.emt`, kept as they are.
  So are `<MAP>.aud` (the port reads only `tracks\<stem>\<stem>.aud`, MD2), `DetailMapTilingMultiplier`
  where it is not 8 (12 maps, MD4), the texture pages (MD1, MD4) and the 80×80-leaf size (27 maps, MD3).

## Loose PSR members (M15)

Read 2026-10-22 from SLUS_211.04 and its GAME_REL.BIN overlay, in Ghidra (`extracted/mxvatv/ghidra`,
set up as M10 did Untamed's: `kit.toml` there, `KIT_CONFIG=… ../ps2-decomp-kit/kit fn <addr>`).
GAME_REL.BIN starts `MWo3`, version 2, and loads whole, header included, at the address in its third
word, `0x0045c580` (`[[overlay]] load = "0x45c580"`). **The game opens a loose PSR member when the
archives it has loaded lack the name.** The importer now copies every such member a map needs.

**The loader.** At boot `0x00155300` opens `cdrom0:\MXVSATV.PSR;1`, and `0x001c72d0` reads its
directory (version 1 or 2, a count, then a length-prefixed full path, a size and a sector per member)
into a table of every member (`0x0040c798`, count `0x0040c7a8`). `0x001c6a70` opens a member by full
path from that table, compared without case (`0x00357158` is `stricmp`). Every file a loader wants goes
through the resource manager at `0x003ea990`: `0x00120750`, `0x0011ff70` and `0x00120490` look the name
up, lowercased and whole, in a hash of the members of the `.res` files loaded so far (`0x00122e70`).
When none holds it, they call `0x0011dcf0`, which opens the PSR member at the same path through
`0x001c6a70`. That is how a map reaches past its archives. Two switches could stop it, and neither is
set on the disc:
- flag `0x1000` on the manager turns the fallback off. Only `0x001209e0` sets it, called from the
  `.res` loader `0x0022cda0` when its fifth argument is non-zero. None of its 43 callers passes one
  (the map's `.res` and `_mat.res` load at `0x002c8310` and `0x002c871c` with 0).
- flag `0x800` makes a loose file win even over an archive's. It comes from an INI's
  `ResourceFileOverride` (`0x00127640`), default 0.

**What a map asks for**, each as the code builds the name:
- **Plants.** GAME_REL `0x0046f070` swaps `EcosystemFile`'s extension for `.esb`, or adds it when there
  is none. So `EcosystemFile=mc02` finds `MC02.esb` in the game (M2's todo said this was not looked up).
  Each species is loaded by `0x0046d040`. It sets the model folder (`0x003ddb80`, normally `Geo\`) to
  `Eco\` and the material folder (`0x003ddc90`) to `Tex`, then puts both back. The model loader
  `0x0018b690` adds a `\` when the folder lacks one and swaps the extension for `.psg`. A species is
  therefore `Eco\<model>.psg`.
- **Materials.** Materials are not loaded by name on demand. `0x001339b0` looks a name up in the
  material table (`MaterialStringTable`, `0x003f3528`). On a miss it returns the first material and
  sets a "missing" flag. The table is the map's `.mst`, its *Material String Table*
  (`MST\Env\<MAP>.mst` in `<map>_mst.res`, read by `0x0019db30`/`0x0019dd90`): one material per line,
  `;` comments, and `#include "MST\Global.mst"` for the shared ones. A listed material is parsed the
  first time something draws with it (`0x00133630`, when its count is above 0), from `<line>.psm`
  through the resource manager (`0x001a0e90` → `0x00143c30` → `0x0011ff70`). Every map's table lists
  the borrowed eco materials ("NOTE: include only needed eco materials here!"): MC03 lists `Tex\Nat11BB`
  and `Tex\nat11Eco`.
- **Textures.** For each `[ShaderPass_<p>]`, `p` ≤ `[General] TotalShaderPasses` (default 0),
  `0x00167720` takes `TextureMap_1` … `TextureMap_<totalTextureMaps>` (default 1). It skips
  `RenderTarget`, `RenderTarget2`, `FrameBuffer` and `DisplayBuffer`, and puts the `.psm`'s own folder
  in front: `Tex\` + `nat11eco`. A material with no `.psm` gets its own name as its one texture
  (`0x00167f10`). `0x00170500` swaps the extension (from the last dot) and tries `.pst`, `.pss`, `.tga`,
  then `.atm`, each through `0x00120750`. That makes it archive `.pst`, loose `.pst`, archive `.pss`,
  and so on. One side effect: `Nat05_liverystable_detail.dxm` becomes `…_detail.pst`, which the
  archive has (M2's todo, M9).
- **Collision.** `0x001bb770` asks for `Col\<name>.col` through `0x00120750`. When nothing answers, it
  returns 0 and builds no shape. The 16 `.col` the scenes place but the disc lacks are on no part of
  the PSR, so the fallback finds nothing for them either. What the scene loader does with the
  placement then was not followed.

**What the importer copies** (`tools/import-mxvatv` `pull_loose`): it follows those three chains on
each map's own files. That is the `.esb` its scene names, the lines of its own `MST\Env\<MAP>.mst`
(not the `#include`s), and the textures of those materials. At each step a path an archive holds is
the archive's. A path only the PSR holds is copied at its member path, byte for byte, listed in
`source.members`, and given a reason in mod.json's `loose` (`"Tex\\Nat11BB.psm": "material
Tex\\Nat11BB in mst/env/mc03.mst"`). Over the 56 maps that is **3,490 members, 44 MB**: 1,735 `.psm`,
1,679 `.pst`, 66 `.atm` and 10 `.psg`, from 5 (MC06) to 159 (FS08) a map. They include Nat02's ten
plants, the borrowed eco and billboard materials and textures of MC03/04/05/09/10, Nat13, SM01 and
SM03, every map's `Env\<MAP>\TerrainLOD.psm`, and the `<MAP>_d.pst` detail map (named by the archive's
`TerrainPlain.psm`, `TextureMap_2 = NAT01_d`). Each pulled path is either in no `.res` at all (1,075)
or only in other maps' archives (2,415). None is in an archive outside `Env\`, so no shared archive the
game might have loaded answers first. The todo entries that named these files as loose are gone,
and every map still reads back through modcheck.

- **The table is taken whole.** The game parses a listed material only when something draws with it.
  1,676 of the 1,735 pulled `.psm` are named, as text, by a file in the mod (a model, the scene, the
  `.esb`, another `.psm`). The other 59, such as `TerrainLOD.psm`, are named by the code or by
  nothing. Which of them the game actually opens on a Free Ride was not checked (T7).
- **Folder spelling.** A pulled member keeps the PSR's spelling, so a mod can hold both `Env/mc03/…`
  (the archive's) and `Env/MC03/TerrainLOD.psm` (the PSR's). The files differ, so the standard's lookup
  is not ambiguous. modcheck passes them, and on a case-insensitive file system the two folders are one.

## Embedded textures

The header's texture table (`textures`, `textures_again`, then that many words) is empty on MXU's
`free06` (0, 0). Here every terrain has one. The first survey's "16 offsets" holds only on the 1-level
maps: the count is **16, 32, 48, 52 or 64**, roughly one page per 16 blocks (16 at 257 blocks, 32 at 538,
48 at 794, 52 at 853, 64 at 1050, but 48 on Nat13's 1050). The survey walks them as the game's loader does
(`notes/mxu-terrain-limits.md`): every page is format `0x27`, 1 mip, 256×256, 65,560 bytes. Every table
entry names exactly the page the walk reaches, and the last page ends exactly where the tree starts.
openmx's reader assumes the tree starts where the blocks end and rejects all 56 (MD1).

**Read through a view (until MD1 lands).** The survey passes `Terrain::parse` a copy in which the
blocks are moved up against the tree, the `declared` word array grows over the space they leave, and the
texture count and table are zeroed. The tree stays at its own offsets, because its offsets are
absolute. A test (`tools/survey/tests/row.rs`, ignored by default, run against the discs) checks every
block of all 56 terrains: through the view they are the file's own, byte for byte. A terrain that
`Terrain::parse` takes as it is is read without the view, so the tables will show it once MD1 lands
(no † on the page count).

## Render-side formats (M9)

Checked 2026-10-22 with `tools/renderprobe`. It runs a map's `<map>.res` and `<map>_mat.res` through
openmx game-render's readers and loaders, and nothing of its own. It runs `Texture::parse`, `Material::parse`, `Model::parse`, `Scene::parse`,
`Sky::parse`/`Sky::load`, `Ecosystem::parse`/`Grove::load`, `Splat::load`, `toy::cars`,
`Assets::load_placed`, and `terrain::Terrain::parse`. All 56 maps were run
(`renderprobe mxvatv <MAP>`), and the loose members by glob (`renderprobe mxvatv psg '**.psg'`,
`pst '**.pst'`, `pst '**.atm'`, `skies`). MX Unleashed's `free06` and `nat01` were run as the baseline.
game-render can only make a `Res` out of an ISO whose root holds `MX.PSR` (see openmx-asks, *game-render
opens only MX.PSR*). So the probe puts each `.res`, byte for byte, as the one member of a PSR in a small
ISO in memory, and opens that with game-render's own `Disc::read_from`.

**Headline: the render side is MX Unleashed's too.** On all 56 maps:

- every texture in the map archives decodes (`Texture::parse`, `.atm` by `parse_animation`)
- every material parses
- every sky decodes
- every scene, and every ecosystem the scene names by its full file name, parses

Every model in the map archives reads, except 6 on two maps (below). Every model a scene places loads with its textures from
the two archives, except one on Nat16. The gaps are in names, a few model streams, and the terrain's own paint.

### What reads unchanged

| file | checked on | result |
|---|---|---|
| `<map>_mat.res`: `Tex\*.pst`, `*.atm`, `*.psm` | all 56; e.g. Nat01: 39 `.pst`, 41 `.psm` | 0 refused. Loose: 4,642 of 4,645 `.pst` and 30 of 30 `.atm` read |
| `Geo\*.psg`, `Eco\*.psg` in `<map>.res` | all 56; e.g. Nat04: 104 | all version 5; refused only on Nat12 and Nat16 (below) |
| `.scn` (game-render's reader) | all 56 | Same sections as MXU (`SceneInfo`, `Environment`, `Fog`, `Light`N, `Model`N, `Object`N, `StaticModels`, `CollisionObjects`). `[Environment]` adds `EcoAmbientScale` and `EcoDiffuseScale` (Nat01, SX01, every map), which MXU free06/nat01 do not have and game-render does not read |
| placed models, `Assets::load_placed` | all 56; e.g. SX01: 50 of 50, Nat04: 92 of 92 | All but Nat16's `NorthernLights` load from the map's own two archives. No placed model needs a loose file |
| `Skies\*.ps2.cub` | all 112 loose; the one in each map `.res` | 96 packets each, all read; Nat01's `sky28` face 0 looked at: clouds above, fog colour below, as on MXU |
| `.esb` | the 23 that the scene names with `.esb` (Nat01–17, MC01, MC03–05, MC09–10) | version 2, 9–16 species, 2,245 (Nat04) to 14,738 (Nat05) plants. The farthest plant is within the map's side (Nat01: 5,746 of 5,760 ft), so the 65536ths-of-the-larger-side rule holds at 5,760 ft |

### What differs

- **Material texture names with an extension or a space.** `Material::parse` keeps `TextureMap_1` as
  written, and `world::texture_files` appends `.pst`:
  - `Tex\SunFlare<N>.psm` names `sunflare<N>.pst`, so it looks for `sunflare33.pst.pst`. That is all 56
    maps; `MoonFlare01/02` on four. `SunFlare33.pst` is in Nat01's `_mat.res`. MXU has the same thing
    (`free06`: `sunflare03.pst.pst`).
  - Nat05 `Tex\Nat05_liverystable_detail.psm` names `….dxm`, and the `.pst` is beside it.
  - Nat12 `Nat12_genericsandstone.psm` and `Nat12_pillarA.psm` name `Nat12 _genericsandstone` and
    `Nat12 _pillarA` (with a space). The `.pst` is `Nat12_genericsandstone.pst`. That leaves 10 of
    Nat12's placed meshes untextured (`Nat12_Temple`, `nat12_column`, …). The scene reader already drops spaces
    (`scene::slt_name`); the material reader does not.
- **`EcosystemFile` without `.esb`.** SM01–03 and MC02 say `EcosystemFile=sm01` (`mc02`). The archive holds
  `Env\sm01\sm01.esb`. `Grove::load` matches the name as a suffix, so these four maps get no trees.
- **Eco billboards and species that are not in the map's archives.** On MC03, MC04, MC05, MC09, MC10
  and Nat13 every species names the billboard `Nat11BB.tga`, and no `nat11bb.psm` is in those archives
  (MC03's holds `nat08econew2.psm` instead). `Tex\Nat11BB.psm` and `Tex\Nat11bb.pst` are loose on the PSR,
  and in Nat11's `_mat.res`. On Nat02 all 10 species' models are missing from the archive but loose
  under `Eco\` (`Eco\nat02_treea.psg`, …). So the game also reads loose PSR members, and an importer
  (M2) has to take these from there. M15 found the loader that does it and the importer now does
  (*Loose PSR members*).
- **Models game-render refuses.**
  - Nat12: 5 eco models (`Eco\nat12_busha/bushb/bushc/weedsb/waitaminute.psg`, 6 of the 10 species)
    fail with `unexpected UNPACK of 12 x 2 x 8 bits`. That is 8-bit texture coordinates (V2-8), where
    the reader knows only V2-16. Loose `Geo\nat12_jumpgatea.psg` fails the same way.
  - Nat16: `Geo\northernlights.psg`, which is placed, fails with `the file ends at 23513 but 3933048 bytes
    were wanted at 373`. Its one mesh has stream mask `0x1f`; the reader knows bits 1, 2, 4 and 8.
  - Loose only, on no map's archive: `Tex\dallas_stadium.psg` (`UNPACK of 16 x 3 x 16 bits`, 16-bit
    normals). `Env\Nat05\Anim\nat05_bank_sign2/3.psg` and `nat05_courthouse_sign2.psg` are 8-byte stubs
    (`psg\0`, version 5, nothing else).
- **Three loose `.pst` with another header**: `Tex\Box01.pst`, `Crushedcar.pst`, `SM2Bridge.pst` start
  `1, 1, 4` and then what looks like a 16-entry palette. `Texture::parse` refuses them
  (`4278190080 mips`). They are on no map's archive.

### The terrain's texture pages, and the ground's paint

- **The pages decode as `.pst`.** A page is `0x27`, a mip count of 1, and a 16-byte mip header
  (`256, 256, 65536, 0`), then 65,536 one-byte pixels. That is a `.pst` without its palette. Put the
  terrain's own header palette in front (`renderprobe::page_pst`: the `.pst` magic, the 1,024 palette
  bytes, then the page from its mip count) and `Texture::parse` decodes every page of all 56 maps:
  1,812 pages. The palette is in the same GS order as a `.pst`'s (`pst::unswizzle`, as game-render's
  terrain reader already assumes), with alpha `0x80` everywhere, and the pages use all 256 entries.
- **What the pages show** (looked at, Nat01 and SX01, written as PNGs to the scratchpad only): one aerial
  painting of the whole map, cut into 256×256 tiles. On SX01 the 16 tiles are the supercross
  track and the grass around it; neighbouring tiles continue one another. Which tile lies where, and
  how the blocks pick theirs, was not worked out (MD4).
- **MXU has them too.** MXU `nat01` has 32 pages that decode the same way. `free06` has none: it paints
  its ground with the screen-space splat. So this is not new with MX vs. ATV; openmx draws neither
  (MD4).
- **game-render's terrain reader** (`terrain::Terrain::parse`, used only by `first-draw`) refuses the 27
  2-level maps (`794 x 794 is not a whole number of patches`). On the 29 1-level maps it "reads" 257 × 257,
  but only because the block counts sit where it expects a size (MD3). Its 1 MiB "ground texture" is
  then page 0 from its 24-byte header on.
- **No screen-space splat.** `Splat::load` finds no `terrainscreenspacemesh.psm` on any of the 56 maps. In each `_mat.res` the ground's material is
  `Env\<map>\TerrainPlain.psm`: `Vu1TerrainRenderer.vuo`, two texture maps, the first unnamed (the pages)
  and `TextureMap_2 = <MAP>_d`, alpha-blended. `Material::parse` reads only `TextureMap_1`.
  `<MAP>_d.pst` is a 512×512 detail map. It is loose beside the map (`Env\Nat01\Nat01_d.PST`, all 56
  read) and not in either archive. Loose `TerrainLOD.psm`, `TerrainUseScreenSpace.psm` and the screen-space
  prologue/epilogue sit beside 31 of the maps, but no archive holds them. No `<map>_shadow_NN.pst` exists
  anywhere.

### Not read by game-render

- **`.toy`.** `toy::cars` finds 0 cars on all 56 maps: no `.toy` has a `[Cars]` section. They hold
  `AnimatedToy`N (`Name, Position, Box, LookVector, UpVector, MCFName, InitialMotion, HasCollision`: the
  crowd, the windmills, …), `PhysicsCollection`N, `TerrainZone`N (16 on every map), `Trigger`N with
  actions, `SunFlare`, `ParticleEmitter`N (SX/FS), `Balloons` (MC01). MXU's nationals have
  `AnimatedToy` too (nat01: 4), so this is not new; game-render has no reader for it.
- **Particles.** `Part\<map>particles.prt` and `_gpu.prt` are in every map `.res`. openmx reads `.prt` in
  the shell crate (`crates/openmx/src/particles.rs`), which this repo cannot call (as with `.vig`).
- **No reader anywhere in openmx**, in the map archives:
  - `.aud` (all 56; the shell's audio reads MXU's by name)
  - `.pco` (38)
  - `.mcf` and `.mot` beside the animated toys (24)
  - `.vut` (20)
  - `.emt` (5)
  - `<map>_mst.res`, which holds `MST\*.mst` (Nat01: 20 of them: `Global.mst`, `TerrainShadow.mst`,
    `Vehicle.mst`, …), not looked into
- **Microprograms** named by the maps' materials, all known to MXU's set or near it:
  - `Vu1StripDiffuse`, `…Scaled`, `…GlossySpecularMap`, `…TexTransform`
  - `Vu1StripPrelit`, `…Scaled`, `…TexTransform`
  - `Vu1StripConstantColor`, `Vu1StripSkyUntextured`, `Vu1TerrainRenderer`, `Vu1Ecosystem2D`
  - `Vu1Particles`, `Vu1ParticlePhysics`, `Vu1NopLoop`

  Some are spelled in lower case (`Vu1Stripdiffuse`, `Vu1Stripprelit`). game-render's `Material::shading`
  lower-cases them, so that is harmless. `Tex\Screen.psm` (the stadiums' jumbotrons) names
  `RenderTarget`, as on MXU.

## Against the game (M12)

Every map, 2026-10-22, in PCSX2 (kit copy 9, the USA disc): a Free Ride on it with a bike, a RAM image of
the ride, and `tools/accuracy` (T7) on the image and the mod (`notes/accuracy.md`, *Every map*). Four
things are checked, and the mods in `out/mods/tracks/` are the importer's after the fixes below:

- **Heights** (`accuracy heights`): the game's two terrain queries (`0x4838f0`, the wheels' `0x4833c0`)
  run on the image, against openmx's on the mod's `.trn`, at 2,000 points in each of the four ways a
  terrain object can be set (8,000), every field bit for bit.
- **Scene** (`accuracy scene`): `[SceneInfo]`, `[Fog]`, `[Environment]` and the two lights as the game's
  readers left them, against the `.scn` read as openmx reads it. 24–31 fields a map.
- **Files** (`accuracy files`): every name in the game's resource name pool that is the map's (props,
  shapes, materials, species, sky, terrain, toys), against the mod's files.
- **Shots** (`accuracy shoot`): the original's half of the pairs, from 9–11 poses a map (611 in all) (the scene's
  start, the centre looking four ways, an overview, the highest ground looking across, two or three
  landmarks, and the game's own camera as it was), in `out/accuracy/<map>/` (not in git). openmx's half
  waits on MD2 and MD10 (T9).

**What matches, on all 56.** Every height, surface and corner weight (8,000 points a map); every normal,
once MX vs. ATV's cell-reader step is applied (`notes/openmx-asks.md`); the game's terrain box, which is
the file's leaves; every scene field (the start, the fog's colour, visibility, haziness and distances,
the terrain keys, `MinDivisionSize`, the eco distances, `ShrubberyDistLimit`, `DetailMapTilingMultiplier`,
the eco scales, both lights' colours, the sun's position, and its direction to 1–2 ulp of `Target -
Position` made unit); and every file the game opened for the map. No map turns on the packed normal
table. Every scene gives every `[Fog]` and `[Environment]` key the readers default, except
`EcosystemFile` (27 maps).

**What the importer was missing, and now copies** (`pull_loose`; each pulled member is in mod.json's
`loose` with the reason):

- the shadow materials, `Tex\shadowCaster<n>.psm`, `terrainShadowReceiver<n>.psm` and
  `shadowReceiver<n>.psm` for n = 1–6 (`0x0031bc90`, six sets on every map), 18 on each;
- the toys' physics collections: the `.pco` of the first `NumberOfPhysicsCollections` sections
  `[PhysicsCollection<n>]` of `<map>.toy` (`0x002171b0`, in the map's folder), and each part's
  `Col\<Name>m.col` and `f.col` (GAME_REL.BIN `0x004922a0`). Lago's saloon doors; Nat01, Nat02, Nat05,
  Nat06 and Nat07 have them;
- the machines' particle sheets in the map's folder (`0x004a6810`): `<machine>particles.dat`,
  `<machine>aiparticles.dat` and `mx`/`atvparticlezones.dat`, which the game takes over `Part\`'s.
  On a bike MC10, Nat02 and SM01–03 open their own `mxparticlezones.dat` (seen in the pool); 15 maps have
  some sheet for some machine.

**Where the Free Ride starts.** At the scene's `DefaultStartPosition` facing `DefaultStartDirection`, or
the `.atyList`'s `StartPosition` on MC01 and FS05 (MC01's camera was 15 ft from it, 1,203 ft from the
scene's). About a minute in, the ride camera was 14 ft behind the start on 50 maps; on six the rider
had rolled downhill (FS08, MC05, Nat05, Nat14, Nat17, SX06, 39–263 ft). On FS02, FS06, Nat10, Nat14 and
SM01–03 the start is under the ground (6–312 ft), and the game stands the rider on the ground over it.
The importer's todo now says so on those seven, and no longer says the start is unknown on the rest.

**What differs, and why** (the port's side; `notes/openmx-asks.md`):

- **Normals**, every map: openmx's query keeps the raw normal bytes, MX vs. ATV's divides by 127. 1–3
  ulp a lane at 60–91 % of the points.
- **The box**, the 28 arenas and Nat04: openmx's `world::Ground` sizes the terrain from the scene
  (`TerrainWidth × 16^levels`), the game from the file. The heights still agree (the query does not
  clamp to the box).
- **Camera-following props**, MC01, Nat04, Nat16: `Type=CameraFollowing` (the rain sheet `CameraRain`
  on MC01) follows the camera in the game, and openmx would draw it once at the map's centre.
- **Physics objects**, Nat01, Nat02, Nat05, Nat06, Nat07: the `.pco` are in the mods now, but openmx has
  no reader for them.
- **The start under the ground**, FS02, FS06, Nat10, Nat14, SM01–03 (above).

M2's todo items stand as they were (class-1 `.col`, the missing `.col`, the `.mot`, the activity keys, …):
this pass found nothing that changes them. *Pool* below: on MC01–05, MC07 and MC08 a later file had
overwritten the pool's head by the time of the capture, so the names from the scene's to the first left
were not seen; *files* counts the rest. *Shots*: on MC09, MC10 and Nat16 only two landmarks are 300 ft
apart (MC06: one), so ten poses (nine).

| map | name | heights equal at, normals with the step | scene fields equal | map's files opened, in the mod | shots | the importer now copies | differs (the port's side) |
|---|---|---|---|---|---|---|---|
| fs01 | Little Rock FMX | 8000 ✓ | 28/28 | 381/381 | 11 | 18 shadow materials | normals 60 % 1–3 ulp (the step); Ground's box |
| fs02 | Austin FMX | 8000 ✓ | 28/28 | 425/425 | 11 | 18 shadow materials | normals 60 % 1–3 ulp (the step); Ground's box; start under the ground |
| fs03 | Boston FMX | 8000 ✓ | 28/28 | 384/384 | 11 | 18 shadow materials | normals 70 % 1–3 ulp (the step); Ground's box |
| fs04 | St. Paul FMX | 8000 ✓ | 28/28 | 406/406 | 11 | 18 shadow materials | normals 68 % 1–3 ulp (the step); Ground's box |
| fs05 | New Orleans FMX | 8000 ✓ | 28/28 | 412/412 | 11 | 18 shadow materials | normals 71 % 1–3 ulp (the step); Ground's box |
| fs06 | Las Vegas FMX | 8000 ✓ | 28/28 | 393/393 | 11 | 18 shadow materials | normals 66 % 1–3 ulp (the step); Ground's box; start under the ground |
| fs07 | Portland FMX | 8000 ✓ | 28/28 | 415/415 | 11 | 18 shadow materials | normals 61 % 1–3 ulp (the step); Ground's box |
| fs08 | Orlando FMX | 8000 ✓ | 28/28 | 421/421 | 11 | 18 shadow materials | normals 70 % 1–3 ulp (the step); Ground's box |
| mc01 | Killington | 8000 ✓ | 31/31 | 148/148 (head lost) | 11 | 18 shadow materials | normals 85 % 1–3 ulp (the step); camera-following prop |
| mc02 | Desert Oasis | 8000 ✓ | 31/31 | 172/172 (head lost) | 11 | 18 shadow materials, 2 particle sheets | normals 90 % 1–3 ulp (the step) |
| mc03 | Red Country | 8000 ✓ | 31/31 | 107/107 (head lost) | 11 | 18 shadow materials, 2 particle sheets | normals 90 % 1–3 ulp (the step) |
| mc04 | Glamis South | 8000 ✓ | 31/31 | 136/136 (head lost) | 11 | 18 shadow materials, 2 particle sheets | normals 88 % 1–3 ulp (the step) |
| mc05 | Woodstock | 8000 ✓ | 31/31 | 99/99 (head lost) | 11 | 18 shadow materials, 2 particle sheets | normals 88 % 1–3 ulp (the step) |
| mc06 | Monster Stadium | 8000 ✓ | 28/28 | 149/149 | 9 | 18 shadow materials, 2 particle sheets | normals 70 % 1–3 ulp (the step); Ground's box |
| mc07 | Trophy Stadium | 8000 ✓ | 28/28 | 391/391 (head lost) | 11 | 18 shadow materials | normals 76 % 1–3 ulp (the step); Ground's box |
| mc08 | 19th Hole | 8000 ✓ | 28/28 | 690/690 (head lost) | 11 | 18 shadow materials | normals 74 % 1–3 ulp (the step); Ground's box |
| mc09 | Quechee Gulch | 8000 ✓ | 31/31 | 131/131 | 10 | 18 shadow materials | normals 88 % 1–3 ulp (the step) |
| mc10 | Land and Air | 8000 ✓ | 31/31 | 146/146 | 10 | 18 shadow materials, 2 particle sheets | normals 87 % 1–3 ulp (the step) |
| nat01 | Goblin Gulch | 8000 ✓ | 31/31 | 203/203 | 11 | 18 shadow materials, 6 physics files | normals 90 % 1–3 ulp (the step); physics objects (.pco) |
| nat02 | Tipperary | 8000 ✓ | 31/31 | 462/462 | 11 | 18 shadow materials, 2 particle sheets | normals 90 % 1–3 ulp (the step); physics objects (.pco) |
| nat03 | Glamis | 8000 ✓ | 31/31 | 454/454 | 11 | 18 shadow materials, 2 particle sheets | normals 87 % 1–3 ulp (the step) |
| nat04 | Oceanside | 8000 ✓ | 31/31 | 351/351 | 11 | 18 shadow materials | normals 80 % 1–3 ulp (the step); Ground's box; camera-following prop |
| nat05 | Lago | 8000 ✓ | 31/31 | 326/326 | 11 | 18 shadow materials, 6 physics files, 2 particle sheets | normals 88 % 1–3 ulp (the step); physics objects (.pco) |
| nat06 | Juniper Park | 8000 ✓ | 31/31 | 291/291 | 11 | 18 shadow materials, 30 physics files | normals 88 % 1–3 ulp (the step); physics objects (.pco) |
| nat07 | Valhalla | 8000 ✓ | 31/31 | 218/218 | 11 | 18 shadow materials, 3 physics files, 2 particle sheets | normals 86 % 1–3 ulp (the step); physics objects (.pco) |
| nat08 | Redwood Valley | 8000 ✓ | 31/31 | 238/238 | 11 | 18 shadow materials | normals 90 % 1–3 ulp (the step) |
| nat09 | Deadman Wash | 8000 ✓ | 31/31 | 465/465 | 11 | 18 shadow materials, 2 particle sheets | normals 88 % 1–3 ulp (the step) |
| nat10 | Maple Hill | 8000 ✓ | 31/31 | 334/334 | 11 | 18 shadow materials | normals 89 % 1–3 ulp (the step); start under the ground |
| nat11 | Sutter Raceway | 8000 ✓ | 31/31 | 429/429 | 11 | 18 shadow materials | normals 90 % 1–3 ulp (the step) |
| nat12 | Temple of Apophis | 8000 ✓ | 31/31 | 204/204 | 11 | 18 shadow materials | normals 89 % 1–3 ulp (the step) |
| nat13 | Sienna Ridge | 8000 ✓ | 31/31 | 281/281 | 11 | 18 shadow materials | normals 86 % 1–3 ulp (the step) |
| nat14 | Santiago | 8000 ✓ | 31/31 | 446/446 | 11 | 18 shadow materials | normals 90 % 1–3 ulp (the step); start under the ground |
| nat15 | Smuggler's Notch | 8000 ✓ | 31/31 | 404/404 | 11 | 18 shadow materials | normals 90 % 1–3 ulp (the step) |
| nat16 | Northern Lights | 8000 ✓ | 31/31 | 151/151 | 10 | 18 shadow materials, 2 particle sheets | normals 86 % 1–3 ulp (the step); camera-following prop |
| nat17 | Keweni Falls | 8000 ✓ | 31/31 | 225/225 | 11 | 18 shadow materials | normals 87 % 1–3 ulp (the step) |
| sm01 | Mesa | 8000 ✓ | 31/31 | 384/384 | 11 | 18 shadow materials, 6 particle sheets | normals 83 % 1–3 ulp (the step); start under the ground |
| sm02 | Highlands | 8000 ✓ | 31/31 | 422/422 | 11 | 18 shadow materials, 6 particle sheets | normals 88 % 1–3 ulp (the step); start under the ground |
| sm03 | The Glades | 8000 ✓ | 31/31 | 400/400 | 11 | 18 shadow materials, 6 particle sheets | normals 88 % 1–3 ulp (the step); start under the ground |
| sx01 | Los Angeles | 8000 ✓ | 28/28 | 778/778 | 11 | 18 shadow materials | normals 68 % 1–3 ulp (the step); Ground's box |
| sx02 | Denver | 8000 ✓ | 28/28 | 768/768 | 11 | 18 shadow materials | normals 73 % 1–3 ulp (the step); Ground's box |
| sx03 | Savannah | 8000 ✓ | 28/28 | 721/721 | 11 | 18 shadow materials | normals 71 % 1–3 ulp (the step); Ground's box |
| sx04 | San Diego | 8000 ✓ | 28/28 | 698/698 | 11 | 18 shadow materials | normals 74 % 1–3 ulp (the step); Ground's box |
| sx05 | Austin | 8000 ✓ | 28/28 | 673/673 | 11 | 18 shadow materials | normals 79 % 1–3 ulp (the step); Ground's box |
| sx06 | Little Rock | 8000 ✓ | 28/28 | 673/673 | 11 | 18 shadow materials | normals 83 % 1–3 ulp (the step); Ground's box |
| sx07 | St. Paul | 8000 ✓ | 28/28 | 709/709 | 11 | 18 shadow materials | normals 77 % 1–3 ulp (the step); Ground's box |
| sx08 | Boston | 8000 ✓ | 28/28 | 687/687 | 11 | 18 shadow materials | normals 73 % 1–3 ulp (the step); Ground's box |
| sx09 | New Orleans | 8000 ✓ | 28/28 | 753/753 | 11 | 18 shadow materials | normals 78 % 1–3 ulp (the step); Ground's box |
| sx10 | Toronto | 8000 ✓ | 28/28 | 683/683 | 11 | 18 shadow materials | normals 80 % 1–3 ulp (the step); Ground's box |
| sx11 | Orlando | 8000 ✓ | 28/28 | 676/676 | 11 | 18 shadow materials | normals 78 % 1–3 ulp (the step); Ground's box |
| sx12 | Philadelphia | 8000 ✓ | 28/28 | 710/710 | 11 | 18 shadow materials | normals 76 % 1–3 ulp (the step); Ground's box |
| sx13 | Las Vegas | 8000 ✓ | 28/28 | 717/717 | 11 | 18 shadow materials | normals 80 % 1–3 ulp (the step); Ground's box |
| sx14 | Mexico City | 8000 ✓ | 28/28 | 670/670 | 11 | 18 shadow materials | normals 73 % 1–3 ulp (the step); Ground's box |
| sx15 | Portland | 8000 ✓ | 28/28 | 715/715 | 11 | 18 shadow materials | normals 75 % 1–3 ulp (the step); Ground's box |
| sx16 | Tucson | 8000 ✓ | 28/28 | 699/699 | 11 | 18 shadow materials | normals 81 % 1–3 ulp (the step); Ground's box |
| sx17 | Charleston | 8000 ✓ | 28/28 | 747/747 | 11 | 18 shadow materials | normals 77 % 1–3 ulp (the step); Ground's box |
| sx18 | Boulder | 8000 ✓ | 28/28 | 727/727 | 11 | 18 shadow materials | normals 75 % 1–3 ulp (the step); Ground's box |

## Survey tables

`cargo run --release -p survey -- mxvatv`, 2026-10-22. † = read through the view above.

| map | levels | leaves | spacing (ft) | side (ft) | blocks | texture pages | scene | scene terrain keys | models | collision placements | `.col` read | `.col` refused |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| fs01 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Freestyle01 | 1×1 @ 3.5 | 3582 | 228 (9 files: 9 here) | 233 of 233 | 0 |
| fs02 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Freestyle02 | 1×1 @ 3.5 | 3974 | 236 (12 files: 12 here) | 238 of 238 | 0 |
| fs03 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Freestyle03 | 1×1 @ 3.5 | 4771 | 198 (8 files: 8 here) | 232 of 232 | 0 |
| fs04 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Freestyle04 | 1×1 @ 3.5 | 4376 | 307 (8 files: 8 here) | 232 of 232 | 0 |
| fs05 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Freestyle05 | 1×1 @ 3.5 | 1883 | 248 (9 files: 8 here, missing anaheim_pole.col) | 232 of 232 | 0 |
| fs06 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Freestyle06 | 1×1 @ 3.5 | 1475 | 232 (7 files: 7 here) | 231 of 231 | 0 |
| fs07 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Freestyle07 | 1×1 @ 3.5 | 2585 | 179 (8 files: 8 here) | 232 of 232 | 0 |
| fs08 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Freestyle08 | 1×1 @ 3.5 | 3952 | 231 (7 files: 7 here) | 233 of 233 | 0 |
| mc01 | 2 | 80×80 | 4.5 | 5760 | 1050 | 64 × 256×256 fmt 0x27, 1 mip, 65560 B † | MachineChallenge01 | 5×5 @ 4.5 | 145 | 140 (24 files: 24 here) | 24 of 25 | Nat07_WM_Blade.col |
| mc02 | 2 | 80×80 | 4.5 | 5760 | 1050 | 64 × 256×256 fmt 0x27, 1 mip, 65560 B † | MachineChallenge02 | 5×5 @ 4.5 | 123 | 120 (27 files: 27 here) | 27 of 27 | 0 |
| mc03 | 2 | 80×80 | 4.5 | 5760 | 1050 | 64 × 256×256 fmt 0x27, 1 mip, 65560 B † | MachineChallenge04 | 5×5 @ 4.5 | 38 | 36 (17 files: 17 here) | 17 of 17 | 0 |
| mc04 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | MachineChallenge04 | 5×5 @ 4.5 | 133 | 131 (21 files: 21 here) | 21 of 21 | 0 |
| mc05 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | MC05 | 5×5 @ 4.5, TerrainFile=MC05.trn | 120 | 118 (8 files: 8 here) | 8 of 8 | 0 |
| mc06 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | MC06:MonsterJam | 1×1 @ 3.5 | 1687 | 194 (8 files: 8 here) | 11 of 11 | 0 |
| mc07 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | MC07: Trophy Truck Race | 1×1 @ 3.5 | 2090 | 379 (25 files: 25 here) | 260 of 260 | 0 |
| mc08 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | MC08: 19th Hole | 1×1 @ 3 | 1548 | 171 (23 files: 23 here) | 485 of 485 | 0 |
| mc09 | 2 | 80×80 | 4.5 | 5760 | 1050 | 64 × 256×256 fmt 0x27, 1 mip, 65560 B † | MC09 | 5×5 @ 4.5 | 66 | 52 (20 files: 19 here, missing ring_trigger.col) | 19 of 19 | 0 |
| mc10 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | MC10 | 5×5 @ 4.5, TerrainFile=MC10.trn | 124 | 110 (21 files: 21 here) | 21 of 21 | 0 |
| nat01 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | National01:GoblinValley2.0 | 5×5 @ 4.5, TerrainFile=Nat01.trn | 392 | 141 (39 files: 37 here, missing recycling_can.col trash_can.col) | 37 of 37 | 0 |
| nat02 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | National02:Ireland | 5×5 @ 4.5, TerrainFile=Nat02.trn | 276 | 209 (46 files: 46 here) | 270 of 270 | 0 |
| nat03 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | National03:Glamis | 5×5 @ 4.5 | 268 | 213 (36 files: 36 here) | 260 of 260 | 0 |
| nat04 | 2 | 56×80 | 4.5 | 4032 | 853 | 52 × 256×256 fmt 0x27, 1 mip, 65560 B † | National04 | 5×5 @ 4.5 | 478 | 251 (53 files: 53 here) | 53 of 54 | Nat04_FerrisWheel.col |
| nat05 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | National05:HighPlains | 5×5 @ 4.5 | 271 | 243 (48 files: 48 here) | 48 of 51 | Nat05_Courthouse_Sign.col Nat05_Gallows_Door.col Nat05_Bank_Sign.col |
| nat06 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | National06: | 5×5 @ 4.5 | 312 | 95 (54 files: 51 here, missing dino_egg_a.col dino_egg_b.col dino_egg_c.col) | 51 of 51 | 0 |
| nat07 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat07[Quadzilla] | 5×5 @ 4.5, TerrainFile=Nat07.trn | 394 | 171 (35 files: 34 here, missing nat07_churchsnow.col) | 34 of 34 | 0 |
| nat08 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | National08:Oregon | 5×5 @ 4.5 | 542 | 539 (47 files: 45 here, missing natdropgate.col natstartgate.col) | 45 of 45 | 0 |
| nat09 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat09[Quadzilla] | 5×5 @ 4.5, TerrainFile=Nat09.trn | 488 | 221 (40 files: 40 here) | 264 of 264 | 0 |
| nat10 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat10 | 5×5 @ 4.5, TerrainFile=Nat10.trn | 114 | 180 (54 files: 50 here, missing nat10_mailbox_closed.col nat10_mailboxopen.col nat10_tray_water.col nat_tray_runnoff.col) | 51 of 53 | nat10_MillWheel.col Nat10_TireSwing.col |
| nat11 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat11 | 5×5 @ 4.5, TerrainFile=Nat11.trn | 198 | 195 (25 files: 22 here, missing natdropgate.col natstartgate.col singlebale01.col) | 252 of 252 | 0 |
| nat12 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat12 | 5×5 @ 4.5, TerrainFile=Nat12.trn | 221 | 265 (42 files: 41 here, missing nat12_sphinx.col) | 41 of 41 | 0 |
| nat13 | 2 | 80×80 | 4.5 | 5760 | 1050 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat13 | 5×5 @ 4.5, TerrainFile=Nat13.trn | 449 | 156 (74 files: 74 here) | 74 of 74 | 0 |
| nat14 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat14 | 5×5 @ 4.5, TerrainFile=Nat14.trn | 385 | 58 (31 files: 31 here) | 255 of 255 | 0 |
| nat15 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat15 | 5×5 @ 4.5, TerrainFile=Nat15.trn | 306 | 60 (22 files: 22 here) | 246 of 246 | 0 |
| nat16 | 2 | 80×80 | 4.5 | 5760 | 1050 | 64 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat16 | 5×5 @ 4.5 | 113 | 109 (20 files: 18 here, missing natdropgate.col natstartgate.col) | 18 of 18 | 0 |
| nat17 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat17 | 5×5 @ 4.5, TerrainFile=Nat17.trn | 279 | 64 (32 files: 32 here) | 32 of 32 | 0 |
| sm01 | 2 | 80×80 | 4.5 | 5760 | 538 | 32 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supermoto01 | 5×5 @ 4.5, TerrainFile=SM01.trn | 269 | 84 (22 files: 22 here) | 251 of 251 | 0 |
| sm02 | 2 | 80×80 | 4.5 | 5760 | 538 | 32 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supermoto02 | 5×5 @ 4.5, TerrainFile=SM02.trn | 343 | 210 (35 files: 35 here) | 262 of 262 | 0 |
| sm03 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supermoto03 | 5×5 @ 4.5, TerrainFile=SM03.trn | 166 | 213 (18 files: 18 here) | 251 of 251 | 0 |
| sx01 | 1 | 16×16 | 5 | 1280 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross01 | 1×1 @ 5 | 3028 | 394 (49 files: 49 here) | 509 of 509 | 0 |
| sx02 | 1 | 16×16 | 5 | 1280 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross02 | 1×1 @ 5 | 2446 | 312 (61 files: 61 here) | 515 of 515 | 0 |
| sx03 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross03 | 1×1 @ 3.5 | 3864 | 150 (28 files: 28 here) | 482 of 482 | 0 |
| sx04 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross04 | 1×1 @ 3.5 | 4846 | 149 (26 files: 26 here) | 480 of 480 | 0 |
| sx05 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross05 | 1×1 @ 3.5 | 4148 | 177 (24 files: 24 here) | 478 of 478 | 0 |
| sx06 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross06 | 1×1 @ 3.5 | 3781 | 178 (28 files: 28 here) | 482 of 482 | 0 |
| sx07 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross07 | 1×2 @ 3.5 | 4156 | 181 (26 files: 26 here) | 480 of 480 | 0 |
| sx08 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross08 | 1×1 @ 3.5 | 4874 | 114 (34 files: 34 here) | 488 of 488 | 0 |
| sx09 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross09 | 1×1 @ 3.5 | 2009 | 165 (46 files: 45 here, missing anaheim_pole.col) | 499 of 499 | 0 |
| sx10 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross10 | 1×1 @ 3.5 | 2100 | 212 (27 files: 27 here) | 481 of 481 | 0 |
| sx11 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross11 | 1×1 @ 3.5 | 4104 | 184 (20 files: 20 here) | 474 of 474 | 0 |
| sx12 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross12 | 1×1 @ 3.5 | 3003 | 192 (20 files: 20 here) | 480 of 480 | 0 |
| sx13 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross13 | 1×1 @ 3.5 | 1740 | 208 (29 files: 29 here) | 483 of 483 | 0 |
| sx14 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross14 | 1×1 @ 3.5 | 1621 | 261 (26 files: 26 here) | 480 of 480 | 0 |
| sx15 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross15 | 1×1 @ 3.5 | 2877 | 115 (28 files: 28 here) | 482 of 482 | 0 |
| sx16 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross16 | 1×1 @ 3.5 | 2867 | 165 (32 files: 32 here) | 486 of 486 | 0 |
| sx17 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross17 | 1×1 @ 3.5 | 1567 | 196 (28 files: 28 here) | 482 of 482 | 0 |
| sx18 | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | Supercross18 | 1×1 @ 3 | 1572 | 189 (28 files: 28 here) | 482 of 482 | 0 |

56 maps. † 56 terrains are refused by `Terrain::parse` as they are (openmx MD1); the survey read their texture table itself and passed the reader a view without it, the blocks moved up against the tree.
`.col` in the map archives: 14301, of which 7 refused by `Collision::parse`.

| map | `.col` refused | why |
|---|---|---|
| mc01 | Env\mc01\anim\Nat07_WM_Blade.col | collision class 1 is not understood |
| nat04 | Env\nat04\anim\Nat04_FerrisWheel.col | collision class 1 is not understood |
| nat05 | Env\nat05\anim\Nat05_Courthouse_Sign.col | collision class 1 is not understood |
| nat05 | Env\nat05\anim\Nat05_Gallows_Door.col | collision class 1 is not understood |
| nat05 | Env\nat05\anim\Nat05_Bank_Sign.col | collision class 1 is not understood |
| nat10 | Env\nat10\anim\nat10_MillWheel.col | collision class 1 is not understood |
| nat10 | Env\nat10\anim\Nat10_TireSwing.col | collision class 1 is not understood |

- nat13: `Terrain::parse` on the file: 1050 blocks of 2129 from 5489 end at 2240939, but the tree starts at 5387819
- mc01, mc02, mc03, mc09, nat16: `Terrain::parse` on the file: 1050 blocks of 2129 from 5553 end at 2241003, but the tree starts at 6436843
- fs01, fs02, fs03, fs04, fs05, fs06, fs07, fs08, mc06, mc07, mc08, sx01, sx02, sx03, sx04, sx05, sx06, sx07, sx08, sx09, sx10, sx11, sx12, sx13, sx14, sx15, sx16, sx17, sx18: `Terrain::parse` on the file: 257 blocks of 2129 from 2189 end at 549342, but the tree starts at 1598302
- sm01, sm02: `Terrain::parse` on the file: 538 blocks of 2129 from 3377 end at 1148779, but the tree starts at 3246699
- mc04, mc05, mc10, nat01, nat02, nat03, nat05, nat06, nat07, nat08, nat09, nat10, nat11, nat12, nat14, nat15, nat17, sm03: `Terrain::parse` on the file: 794 blocks of 2129 from 4465 end at 1694891, but the tree starts at 4841771
- nat04: `Terrain::parse` on the file: 853 blocks of 2129 from 4717 end at 1820754, but the tree starts at 5229874

#### Every `.col` loose on the PSR

| folder | `.col` | read | refused |
|---|---|---|---|
| Col | 1493 | 1493 | 0 |
| Env\MC01\Anim | 1 | 0 | nat07_wm_blade.col |
| Env\MC07\Anim | 2 | 2 | 0 |
| Env\MC08\Anim | 2 | 2 | 0 |
| Env\Nat01\Anim | 7 | 7 | 0 |
| Env\Nat03\Anim | 18 | 18 | 0 |
| Env\Nat04\Anim | 7 | 6 | nat04_ferriswheel.col |
| Env\Nat05\Anim | 8 | 2 | nat05_bank_sign.col nat05_bank_sign1.col nat05_bank_sign2.col nat05_courthouse_sign.col nat05_courthouse_sign1.col nat05_gallows_door.col |
| Env\Nat08\Anim | 1 | 0 | bigfoot.col |
| Env\Nat10\Anim | 5 | 1 | milkcow.col nat10_house_swing.col nat10_millwheel.col nat10_tireswing.col |
| Env\Nat12\Anim | 2 | 0 | packcamel.col saddlecamel.col |
| Env\SX01\Anim | 2 | 2 | 0 |
| Env\SX02\Anim | 2 | 2 | 0 |
| Env\SX03\Anim | 2 | 2 | 0 |
| Env\SX04\Anim | 2 | 2 | 0 |
| Env\SX05\Anim | 2 | 2 | 0 |
| Env\SX06\Anim | 2 | 2 | 0 |
| Env\SX07\Anim | 2 | 2 | 0 |
| Env\SX08\Anim | 2 | 2 | 0 |
| Env\SX09\Anim | 2 | 2 | 0 |
| Env\SX10\Anim | 2 | 2 | 0 |
| Env\SX13\Anim | 2 | 2 | 0 |
| Env\SX15\Anim | 2 | 2 | 0 |
| Env\SX16\Anim | 2 | 2 | 0 |
| Env\SX17\Anim | 2 | 2 | 0 |
| Env\SX18\Anim | 2 | 2 | 0 |
| Veh\Col | 67 | 67 | 0 |

| loose `.col` refused | why |
|---|---|
| Env\MC01\Anim\Nat07_WM_Blade.col | collision class 1 is not understood |
| Env\Nat04\Anim\Nat04_FerrisWheel.col | collision class 1 is not understood |
| Env\Nat05\Anim\Nat05_Bank_Sign.col | collision class 1 is not understood |
| Env\Nat05\Anim\Nat05_Bank_Sign1.col | collision class 1 is not understood |
| Env\Nat05\Anim\Nat05_Bank_Sign2.col | collision class 1 is not understood |
| Env\Nat05\Anim\Nat05_Courthouse_Sign.col | collision class 1 is not understood |
| Env\Nat05\Anim\Nat05_Courthouse_Sign1.col | collision class 1 is not understood |
| Env\Nat05\Anim\Nat05_Gallows_Door.col | collision class 1 is not understood |
| Env\Nat08\Anim\bigfoot.col | collision class 1 is not understood |
| Env\Nat10\Anim\MilkCow.col | collision class 1 is not understood |
| Env\Nat10\Anim\Nat10_House_Swing.col | collision class 1 is not understood |
| Env\Nat10\Anim\nat10_MillWheel.col | collision class 1 is not understood |
| Env\Nat10\Anim\Nat10_TireSwing.col | collision class 1 is not understood |
| Env\Nat12\Anim\PackCamel.col | collision class 1 is not understood |
| Env\Nat12\Anim\SaddleCamel.col | collision class 1 is not understood |

## Not verified

- Render side (M9): readers accepting a file is all that was checked. Nothing was drawn and compared with
  the running game (T7). Of the decoded pictures, only Nat01's and SX01's pages and Nat01's sky face 0 were looked at.
  Particles, `.aud`, `.pco`, `.mst` and the animated toys were not read (no reader here).
- The survey checks that each reader accepts each file and counts what it returns. It does not check that
  heights, placements or collision shapes are right against the running game (T7).
- What the game does with a placement whose `.col` is not on the disc, and with a scene key that
  disagrees with its terrain file (MC08, SX18, SX07, Nat04), was not looked up in the ELF.
- Whether MX Unleashed's own loader takes an 80×80-leaf terrain (T3) is open.
- M8's Free Ride table is read from the `.atyList`s, `UI\Tracks.dat` and `UI\Store.dat` only. That the
  game offers every one of them in its Free Ride menu was not seen in PCSX2 (T7), and the type →
  `Env\<Type>.aty` link is read from the ELF's strings, not its code. (M12 rode every map's Free Ride
  and saw where each starts: *Against the game*.)
- M12 (*Against the game*): the look was checked through what the game holds (scene, files) and the
  original's shots; openmx's half of the pairs waits on MD2 and MD10 (T9), so no picture was compared.
  Ecosystem density, `[DistanceFade]` and the lights' shadow keys were not read from RAM.
