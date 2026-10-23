# The accuracy harness (T7)

`tools/accuracy` holds a track mod against its source game running in PCSX2:

- **heights**: the original's own terrain queries, run on a RAM image of the game in a ride on the map,
  against openmx's port of the query on the mod's `.trn`, at a thousand points and more, bit for bit;
- **the look**: screenshot pairs from the same camera poses, the original's (`./kit shot`, its camera
  put at the pose over PINE) and openmx's (`--shot-pose`, openmx MD10), side by side in
  `out/accuracy/<map>/`.

Proved on one map by T7: MX vs. ATV Unleashed's **Lago** (`Nat05`, `mxvatv_nat05`); M12 ran it on all 56 (*Every map*, below), M13 on the 40 Untamed maps (*Untamed (M13)*). The heights are equal at
every point. The original's half of the pairs is taken from 8 poses. openmx's half is waiting on openmx
MD2 (a track from a mod folder) and MD10 (`--shot-pose`). Neither is in openmx yet.

## Running it

From the repo root, with the source disc in `iso/` (or `$DISCX_ISO`) and `../ps2-decomp-kit` beside the
repo (here: `~/repos/ps2-decomp-kit`). One emulator copy of its own: `KIT_PCSX2=9`.

```sh
accuracy setup --game mxvatv                       # extracted/mxvatv/kit.toml (disc by boot ELF)
export KIT_CONFIG=$PWD/extracted/mxvatv/kit.toml KIT_PCSX2=9 K=../ps2-decomp-kit/kit
$K pcsx2 copy 9                                    # once: copy 9's config, PINE, pad, screenshot hotkey
$K pcsx2 start
# to the ride: Start x3 at the logos, then Single Player > Free Ride > the map > Start Event
$K pad send "press start 100"          # (and so on: see "Getting to a Free Ride" below)
accuracy capture --out extracted/mxvatv/ram/nat05.bin
accuracy heights --mod out/mods/tracks/mxvatv_nat05 --ram extracted/mxvatv/ram/nat05.bin
accuracy poses   --mod out/mods/tracks/mxvatv_nat05          # out/accuracy/nat05/poses.txt
accuracy shoot   --mod out/mods/tracks/mxvatv_nat05          # every pose, in the running game
accuracy shoot   --mod out/mods/tracks/mxvatv_nat05 --live spawn   # the game's own view, as a pose
accuracy pair    --mod out/mods/tracks/mxvatv_nat05 --openmx openmx/target/release/openmx
$K pcsx2 stop
```

`cargo test -p accuracy --test lago` runs the heights on Lago when `extracted/mxvatv/ram/nat05.bin` and
`out/mods/tracks/mxvatv_nat05` are there, and says it skipped when they are not.

**Getting to a Free Ride** (fresh profile, US 1.02): at the main menu, Cross (Single Player), Down ×2
(Free Ride), Cross. The track list starts at St. Paul FMX; Lago is 18 Downs on. Presses are dropped while
a track's preview loads, so send them one call at a time with ~0.5–0.9 s between them, and look before
pressing Cross. Then Cross, then Cross (Start Event). Loading takes about 30 s. On a new profile only St.
Paul FMX, Desert Oasis, Lago and Juniper Park are open (M8's ✓ column). Save a state once there
(`$K pine save 4`): reloading it is quicker than the menus. PCSX2 writes a state a moment after `pine
save` returns, and a `state ram` straight after it finds nothing ("no save state in slot 4"), so
`accuracy capture` waits for the slot's file to change and settle before it extracts the RAM.

## Heights

`heights` finds the original's terrain object in the image by what it points at. A word at the object's
tree field (`+0x94`) holds a node whose four leading floats are the `.trn` root node's, and the scale
field beside it (`+0x90`) is the file's scale. Then it calls the original's query through openmx's R5900
interpreter (`ps2-oracle`, `Ee::with_ram`), in all four ways a terrain object can be set: 1 or 2 samples
a cell, split one way or by parity. The map's own setting goes first (Lago: 1 sample, one way). Each mode
gets 2000 points (default), drawn over the object's box inside its border: three in four anywhere, one in
eight on a sample column, one in eight on a sample corner. Both of the original's queries are compared,
every field bit for bit:

| the original | its port in openmx |
|---|---|
| `0x4838f0 (terrain, buf)`: point/y, normal, the cell's four surfaces, the corner weights | `contact_search::ground` |
| `0x4833c0 (terrain, point, normal, -, surface)`: y, normal, the nearest corner's surface (the wheels' query, 47 callers) | `ground` + `contact_search::nearest_surface` |

The port side is openmx's `Terrain` on the mod's file (through `modcheck::terrain::md1_view`, MD1),
`world::Ground::new(&terrain, &scene.terrain).field()`.

### Lago (Nat05), two captures

```
mxvatv_nat05: heights equal at all 8000 points (4 modes); openmx as it is: normal at 7114, ride normal
at 7114 differ; with MX vs. ATV Unleashed's normal step: every field equal
  same samples per unit: original 3e638e39, port 3e638e39
  same height scale: original 4.5, port 4.5
  same box x, z: original [0.0, 0.0]..[5760.0, 5760.0], port [0.0, 0.0]..[5760.0, 5760.0]
```

| field | openmx as it is | with the normal step |
|---|---|---|
| height (both queries) | equal at 8000/8000 | equal |
| four surfaces, nearest surface | equal | equal |
| corner weights | equal | equal |
| normal (both queries) | 7114 of 8000 differ, by 1–3 ulp a lane | equal |

The same holds for a second image captured later in the same ride (rider elsewhere, cache different).
The **terrain is the original's**: every height, surface and weight. The normals differ because of one
step in MX vs. ATV's cell reader (`0x47b300`), not because of the data. It hands each corner's normal
back over 127: `k = 1.0/127.0` (a `div.s`), then x and z are multiplied by `vertical_scale × k` and y by
`k`. MX Unleashed's reader, which openmx ports, keeps the raw bytes (x and z times the vertical scale).
The blend and the normalisation are the same code. Normalising from a vector 127 times shorter rounds
differently in the last bits. `heights::Over127` is that one step over openmx's own field, and with it
every field of every point is equal. For a mod from MX vs. ATV to ride bit-exact, openmx's query needs
that variant (notes/openmx-asks.md).

The other differences from MX Unleashed's query, which the points avoid:

- **The edge.** MX vs. ATV clamps the point into `[min + border, max − border]` (`+0x50`, 0.01 on Lago)
  before it scales it. Off the map, the original answers with the edge cell, and openmx's query (MX
  Unleashed's, with no clamp) misses. The points are drawn inside the border. Past it the two disagree
  by design, which is not compared.
- **The packed normal table.** When the word at `0x541518` is non-zero, the reader takes normals from a
  table (`0x39e320`, 12 bytes an entry, indexed by a byte at block +4) instead of the block's three bytes.
  It is 0 on Lago. `heights` says so and skips the step comparison if it is not.

### How the query was found

The boot ELF has no terrain code: neither MXU's function shapes nor the block offsets match anywhere in
it. The terrain code is in **`GAME_REL.BIN`**, an "MWo3" module (854,016 bytes) that the boot ELF copies
whole, header included, to the address in its header (`+8`: `0x45c580`). That is inside the ELF's BSS, so
it is only in RAM. `extracted/mxvatv/kit.toml` declares it as overlay `game`, and Ghidra has it.

1. In a RAM image of the ride: the `.trn`'s root node's first 16 bytes are at `0xbbe17c`. The node in RAM
   is `+0` children, `+8` block, `+0xc..+0x1c` the file's four floats, `+0x24` the level count (MXU's
   reader layout). The blocks are re-laid 0x8e0 apart (heights at `+0x27c`, normals `+0x550`), and
   duplicate blocks are shared.
2. The object (`0xbbd720` on Lago) points at the root from `+0x94`, with the box at `+0x30/+0x40`, the
   scale at `+0x90` and `1/4.5` at `+0x788`. MXU's fields from `+0x80` on, `+0x10` further.
3. Loads at `+0x788`, `+0x8b8`, `+0x9e0` exist only in RAM code past the ELF: `0x4833c0` and `0x4838f0`,
   both calling `0x47b300` (which reads `+0x27c` heights and `+0x550` normals). The ten-cell cache is at
   `0x541550` (count at `0x5251f0`). The harness empties it whenever it changes the mode.

| object field | MXU | MX vs. ATV |
|---|---|---|
| min, max | +0x30, +0x40 | +0x30, +0x40 |
| border (clamp) | none | +0x50 |
| height scale, tree | +0x80, +0x84 | +0x90, +0x94 |
| split (== 1 one way) | +0x760 | +0x770 |
| samples per unit | +0x778 | +0x788 |
| coarse | +0x8a8 | +0x8b8 |
| cache ready | +0x9d0 | +0x9e0 |

## The look

### The original's camera

- The follow camera's class has its preset loader at boot `0x1c5e00` (it reads `IsLookBackCam`,
  `LookSwing`, …). That leads to its vtable, `0x3cc6f0` (the loader sits at `+0x12c`, one slot past MXU's
  `+0x128`). The per-frame update is game `0x48b490`, `(dt in f12, cam)`, at vtable `+0x40`. It ends by
  calling boot `0x111c30 (viewport, &eye, &forward, &up, -, &fov)`, the viewport's setter (it normalises
  forward and up), then the viewport's vtable `+0xcc` (`0x111d20`, a look-at), then the base update.
- **The viewport**, on the heap (Lago: `0x12a5620` in one session, `0x106cc50` in the next): `+0x140`
  eye, `+0x150` forward, `+0x160` up, `+0x170` fov in degrees (85 on the ride camera), `+0x17c` aspect
  (4/3), `+0x1c0/+0x1d0` copies of eye and forward, and the view matrices before them (`+0x80..`). The
  eye is also at `0x3ea930`, the translation row of a global camera matrix. `ram::find_viewports` finds
  viewports by the doubled eye. The one whose eye is over the terrain is the ride's (the other is a menu
  viewport at `0x84da10`, fov 45).
- **Holding it.** Pointing the vtable slot (`0x3cc730`) at boot `0x1288a0` (`jr ra; move v0, a0`) stops
  the update. The viewport then keeps what is written into it, and the game draws from there: eye,
  forward, up and fov written over PINE move the picture (checked: 30 ft up and turned round). `shoot`
  holds the camera, writes each pose, waits 800 ms, shoots, reads the viewport back (the pose must still
  be there to a thousandth), and puts the slot back at the end. The same pose shot in two sessions
  differs by an RMSE of 0.06 %.
- Without the hold, the camera does not keep still: it swings round an idle rider (2–5 ft and a few
  degrees over a 4 s grab), and a copy's `./kit shot` presses L3+R3 (the screenshot hotkey).

### Poses

A pose is `eye, forward, up, fov`. openmx's `--shot-pose x,y,z,yaw,pitch,fov` takes yaw =
`atan2(forward.x, forward.z)` in degrees (0 along +z, 90 along +x), pitch = `asin(forward.y)` (up
positive), no roll, and the viewport's fov. The fov spans the width (T9): openmx's MD10 takes it as the
horizontal angle, as its camera presets do (85 at 4:3 is 69 vertically), and the Lago pairs frame alike
(below). A vertical 85 would make the original's view 33 % wider than openmx's at 4:3, which it is not.
`+0x178` holds 349.74, close to 320/tan(42.5°) = 349.2, which agrees. The
standard set (`accuracy poses`) is: the scene's start from behind, the centre looking four ways, an
overview at 150 ft (higher, the fog hides everything: 400 ft up showed only fog), and the highest ground
of a 32×32 scan looking across to the centre. `--live NAME` keeps the game's own view instead.

### Lago, the original's side

`out/accuracy/nat05/` (not in git): `spawn, start, centre-n, centre-e, centre-s, centre-w, overview,
ridge` `.original.png` and `poses.txt`; `index.md` lists them with their `--shot-pose`. The shots carry
the game's HUD (radar, speedometer), which openmx's `--overlay none` would not have. Read the pairs with
that in mind, or find the HUD's switch first.

## Findings for the importer and the port

- **Where the Free Ride starts** (M2's `todo` asked T7). In a fresh Free Ride on Lago, ~10 s after the
  load, the rider was at (2217, 404, 2980), heading (0.90, −0.27, −0.35). That is 110 ft from the scene's
  `DefaultStartPosition` (2113.45, 447.32, 3014.67), along its `DefaultStartDirection` (0.94, 0, −0.34),
  rolling downhill. A state saved later had moved on along the same line. This fits a start at the
  scene's position and direction. Frame 0 itself was not captured, nor was whether the game gives the
  bike a starting speed.
- **The heights are copied exactly**, so the importer needs no change for the ground. For bit-exact
  wheels, openmx's query needs MX vs. ATV's normal step and its clamp (openmx-asks).

## Every map (M12)

M12 ran the harness over all 56 MX vs. ATV Unleashed maps, one Free Ride each, in kit copy 9. The
per-map table is in `notes/mxvatv.md`, *Against the game (M12)*; this is how it was run.

**Getting to each map.** A fresh profile has six maps open. The cheat `PITPASS` (Options > Cheat
Codes; the game's list is `UI\CheatCodes.txt`, `TOOLAZY` opens everything) opens all tracks. State 5 of
copy 9 is the Free Ride track list after it, cursor on the first track. Presses still drop while a
preview loads, so the driver reads the list's place instead of counting presses: in that state the
cursor's row in the window is the word at `0x124cea4` and the window's first track the word at
`0x124cee4` (heap addresses, good for state 5 only); their sum + 1 is the map's place in
`UI\Tracks.dat` (M8's *UI* column). Down (or Up) until it is the map's, then Cross, Cross (Start Event),
about 35 s of loading, then `accuracy capture` until `accuracy pose` finds the ride camera. Per map
(`one.sh` in the session's scratch, not kept): `capture`, `heights --points 2000`, `poses --landmarks
3`, `shoot --live live`, `shoot`, `pair`; then `scene` and `files` on the image. About 2.5 minutes a map.

**New commands.**

- `accuracy scene --mod DIR --ram IMAGE`: `[SceneInfo]`, `[Fog]`, `[Environment]` and the lights as the
  game holds them, against the `.scn` read the way openmx reads its numbers (`strtod`, then the
  truncating double-to-float), bit for bit. The readers and offsets are in `tools/accuracy/src/sceneram.rs`.
  A key the scene leaves out is compared with the reader's default; every MX vs. ATV scene gives all of
  them except `EcosystemFile` (27 maps), and the five eco keys are not read without one.
- `accuracy files --mod DIR --ram IMAGE`: the names the game opened, from its resource name pool
  (names two NULs apart, `env\<map>\<map>.scn`, `geo\…`, `col\…`, `tex\…psm`, `eco\…`, the sky, then
  the rider's and machine's), against the mod's files (`tools/accuracy/src/loaded.rs`). Names of the
  rider, machines, UI and language are counted apart. With the disc in `$DISCX_ISO` it also says which
  missing names are on no part of the disc (the game asked and fell back). The pool is a reused buffer:
  on 7 maps (MC01–05, MC07, MC08) a later file's text overwrote its head, and the names from the scene's to
  the first left are not known; `files` says so.
- `accuracy poses --landmarks N` (default 3): a pose looking at each of the map's N biggest landmarks, a
  `[Model<n>]` placed once with no activity keys, ranked by its `.psg`'s size, 300 ft apart at least
  (`tools/accuracy/src/landmarks.rs`), from 90 ft out on the side of the map's centre.
- The poses' box is the file's leaves (`look::leaf_extent`), not openmx's `Ground::extent`, which is 56 ft
  across on the arenas (notes/openmx-asks.md).
- `shoot` checks each PNG ends in `IEND`. `./kit shot` copies PCSX2's screenshot once its size holds for
  0.3 s and caught it half written on 27 of the first 386 shots; then `shoot` waits for the newest file in
  the copy's `snaps` folder to be whole and copies that.
- `tools/accuracy/tests/every_map.rs` runs heights (1,000 points, the map's own mode, with the normal
  step), `scene` and `files` on every mod with an image; 56 maps in 110 s in a debug build, skipped where
  there are none.

**Findings, in short.** On all 56: every height, surface and weight equal at 8,000 points; every normal
equal with MX vs. ATV's step; the game's terrain box is the file's leaves; the scene's start, fog, terrain
keys, eco distances and scales, and both lights' colours, the sun's position and direction equal (the
direction to 1–2 ulp of `Target - Position` made unit here); every file the game opened for the map in
the re-imported mod. The Free Ride starts at the scene's `DefaultStartPosition` (MC01, FS05: the
`.atyList`'s), on the ground where that is under it.

## Untamed (M13)

M13 ran the harness over the 40 imported MX vs. ATV Untamed maps (SLUS_217.01), one Free Ride each, in
kit copy 9. The per-map table is in `notes/mxvatv-later.md`, *Against the game (M13)*; this is how it was
run and what differs from Unleashed's setup.

**The source row** (`source::UNTAMED`). `accuracy setup --game untamed` writes `extracted/untamed/kit.toml`
with `GAME_REL.BIN` as overlay `game` at `0x48ea80` (its header's `+8`; `UI_REL.BIN` loads there too, in
the menus). Ghidra has both under `extracted/untamed/ghidra` (`kit ghidra analyse`, about 2 minutes). The
terrain object was found as on Lago: the `.trn` root node's 16 bytes in RAM, the word pointing at it with
the file's scale beside it. Every field sits at Unleashed's offset; the loads of `+0x788` and `+0x9e0` led
to the two queries (`0x4b0e30`, `0x4b0910`) and the cell reader (`0x4a8ea0`: `0.007874016`, the step over
127, unless the word at `0x5440e8` turns on the packed table at `0x3bd6f0`). The ten-cell cache is at
`0x544120`.

**The scene's readers** (`sceneram::UNTAMED`): the key strings (`MinimumFogDistance`, `DefaultStartZone`,
`TerrainBreadthScale`, …) in the boot ELF lead to their readers, and their vtables were read off the RAM
image. What differs from Unleashed is in `sceneram.rs`; `sceneram::readers(boot)` picks the table, and
`scene` compares the start's direction made unit, as the reader holds it. On SX_C an unread `[SceneInfo]`
block (all zeros) came first in the image; the scene's is the first with a unit direction.

**The camera.** The viewport has Unleashed's layout (the object 0x20 before the offsets, vtable
`0x3d6b50` at its `+4`); the ride camera's fov is 75 (`look::ride_fov`), so the generated poses use 75.
No vtable slot holds the ride camera still: the follow camera's update path (`0x3ef174` → `0x4b6670` →
`0x1126c0` → `0x3ab0a0`) also builds the view matrices, so stopping it freezes the picture whatever the
viewport holds. The hold patches code instead: the first word of the viewport's setter (`0x111700`) and
of its look-at (`0x1117d0`) becomes `jr ra`, one word each (`Camera::also`). A first version that also
put a `nop` in the look-at's second word let the game run it half-written (PCSX2's log: `TLB Miss,
pc=0x1117d0`), and PCSX2's recompiler failed a few maps later ("Impossible block clearing failure",
the copy hung); one word a function ran the last nine maps clean, apart from 126 of those log lines and
no hang. A code write is still the riskier hold: restart the copy if `capture` stops finding new states.

**Getting to each map.** A fresh profile has a few tracks open; the game's cheat `YOUGOTIT` (Options >
Cheat Codes, from `UI\CheatCodes.txt.bpb`) opens everything. State 5 of copy 9 (serial SLUS-21701) is
Custom Event > Free Ride's track list after it, cursor on the first track. The list is `UI\Tracks.dat`'s
order, and all 40 imported maps are in it. In that state the window's first track is the word at
`0x125de14` and the cursor's row the word at `0x125ddd4` (found by diffing captures a press apart); their
sum is the map's place, from 0. Down until it is the map's, then Cross, Cross, about 35 s, then `capture`
until `pose` finds the ride camera, then `heights --points 2000`, `scene`, `files`, `poses`, `shoot --live
live`, `shoot`, `pair`. About 2.5 minutes a map.

**Also changed for it.** `capture` takes the newest state file of the slot (copy 9 had one per game's
serial and took the other game's); `files` knows the scene as `<map>.scn.bpb` and counts `Aud\Rider`
as the game's; `look::terrain_box` multiplies as the EE does (Nat_OR); `tests/every_map.rs` runs on
`untamed_*` with `extracted/untamed/ram/<map>.bin` too (40 maps in about 85 s in a debug build).

## For M12 and M13

- Every map: `capture` in its Free Ride, then `heights`. Then `poses` (add landmarks with `--add`),
  `shoot`, and `pair` once openmx has MD2 and MD10. Check `pair`'s `--openmx-args` template (`--mod
  {mod} --shot {shot} --shot-pose {pose}`) against what MD2 and MD10 name their flags.

## openmx's half (T9)

openmx's MD2 rides a mod by `--mods DIR --track <id>` (DIR holds `tracks/<id>/`), and MD10 takes
`--shot-pose`. `accuracy pair` runs `look::OPENMX_ARGS`: `--mods {mods} --track {id} --headless
--no-sound --frames 2 --size 640x480 --overlay none --graphics original --shot {shot} --shot-pose {pose}`,
from `--openmx-dir` (default `openmx`, where its `kit.toml` names the disc). `--frames 2` ends the run
after the shot (without it a headless ride runs on). `--graphics original` gives the disc's fog
(141..768 ft) and far clip, where openmx's own `full` sees past 8,000 ft. `--overlay none` drops the HUD,
which the original's shots carry (radar bottom left, speedometer bottom right): read the pairs past it.
A shot takes 1.5 s, all eight 12 s.

### Lago, the eight pairs

`out/accuracy/nat05/` (not in git), the original's shots of T7 (`spawn, start, centre-n/e/s/w, overview,
ridge`) against openmx built from openmx `3b11fd6`. What differs, by eye:

- **Framing: the same.** Every building, the water tower (centre-w), the fence gate (centre-n), the
  cactus (spawn) and the skyline sit at the same place and size in both. In centre-w the tower spans about
  x 172 to 232 of 468 in the original and 170 to 232 in openmx. So the eye, yaw, pitch and the fov's axis
  agree, and so do the props' placements and the terrain's shape (the hills' outlines match).
- **The ground's look: not drawn.** openmx draws the terrain as one flat tan, shaded only by its normals.
  The original's is a dark, textured dirt: ridden lines and trails (overview, centre-s, centre-w), darker
  grassy mounds (centre-n), and shadows. This is openmx MD4 (the ground from the terrain's own texture
  pages and the scene's `DetailMapTilingMultiplier`), still open. It is the largest difference in every
  pair.
- **Grey slabs that the original does not draw.** Tall untextured upright boxes stand in centre-n (two),
  centre-s, ridge and start (one each). They are the waypoint-race props: the scene's `[Model<n>]` with
  `Activity="Lago Waypoint Race"`, `Hide=`, `Invisible=` and `IsTrigger=`, which the port draws
  regardless (the mod's `todo`, and `notes/openmx-asks.md`, *MX vs. ATV scene keys the port does not
  read*). Which key hides them in the original was not looked up.
- **Plants: the same places.** Where both show a plant it stands in the same spot at the same size (the
  cactus and bushes in spawn, the shrubs in centre-e and ridge, the grass in start). The plants' colour is
  close. Counts and the far plants' fade were not compared.
- **Sky and fog: close.** The same cloud layer at the same place in every pair. The fog's colour and
  distance look alike in overview and ridge.
- **The rider.** openmx's rider stands at the scene's start, in front of the start pose. The original's
  was wherever it had been ridden to when T7 took the shots (in the middle distance of spawn). This is
  not the map's difference.

## Textures in openmx (T11)

Every track mod in `out/mods/tracks/` and the 27 MXU tiles `world_mxu` uses, 127 in all, ridden in
openmx `e7b66dd` (MD2, MD3, MD10, MD12 in) headless with `look::OPENMX_ARGS`: one `--shot-pose` at the
scene's start and one at the biggest landmark (`accuracy poses --landmarks 1`; the elevation maps have
none and take the overview). The five MXU freeride tiles need `--camera "Flight Cam 1"` (openmx-asks,
*what it draws wrong*). The shots were read by eye on contact sheets, and every material was resolved
by `tools/import-mxvatv/tests/textures.rs` (`cargo test -p import-mxvatv --test textures -- --ignored`)
through `Res::folder` and `world::Assets::resolve`, as the shell does. After the fix: 224 misses on 127
mods, every one named in its mod's `todo`.

| map | what was missing | cause | fixed or asked |
|---|---|---|---|
| all 40 `untamed_*` | every prop, plant, start gate and billboard drawn white (3,918 materials) | the materials are BPB only (`.psm.bpb`); the port reads text | fixed: the importer decodes `.psm`, `.toy`, `.prt` BPB beside itself (§2a), each named in `todo` |
| all 56 `mxvatv_*`, all 40 `untamed_*` | the ground one flat colour, shaded only by its normals | the ground is the `.trn`'s texture pages and `<MAP>_d`; the port has no reader for them yet | asked: openmx MD4 (filed) |
| 44 stadium maps (`mxvatv_fs*`, `sx*`, `mc06`–`mc08`; `untamed_fs_*`, `sx_*`, `mt_a`) | the jumbotrons white | `screen.psm` names `RenderTarget`, which the game draws at run time and does not read as a file | asked: *`RenderTarget`* |
| 45 maps (all `sx*` of both games, `fs01`, `fs02`, `fs05`, `nat02`, `untamed_fs_b`, `sm_mb`, MXU `nat01`, `nat03`, `nat04`, `nat06`, `nat13`, `nat14`, `nat19`, `nat20`, `nat22`) | a prop drawn with another prop's picture or model (`thq.psm` as `BaleTHQ`) | `Res::find_suffix` had no path boundary, and a folder is in path order | fixed in openmx c66d022 (M22: no map has it now); the stand-in case remains on 3 maps, asked: MD2, a missing name falls back to a suffix |
| `mxvatv_fs05`, `mxvatv_sx09` | `seating_blue.pst`: the stands drawn with `Anaheim_seating_blue.pst` | the map has no such file; the suffix finds a longer name | asked, with the above; the game's choice not looked up |
| `untamed_sx_g` (14), `untamed_sx_d` (1) | the stadium shell `FP03B1`–`8` and `skyline01` white | a material with no `.psm`; the game uses its own name's texture, which the mod holds | asked: *a material with no `.psm`* |
| `mxvatv_nat12` | `Nat12 _genericsandstone`, `Nat12 _pillarA` untextured | `TextureMap_1` with a space | asked: M9 *material texture names* |
| `mxvatv_nat05`, `mxvatv_mc05` | `Nat05_liverystable_detail` untextured | `TextureMap_1` with a `.dxm` tail, which the game swaps for `.pst` | asked: M9 *material texture names* |
| `mxvatv_nat12` | five plant species not drawn | `psg::Model::parse`: V2-8 texture coordinates | asked: M9 *`.psg` streams* |
| `mxvatv_nat16` | `NorthernLights` not drawn | `psg::Model::parse`: stream mask `0x1f` | asked: M9 *`.psg` streams* |
| MXU `nat03`, `nat22` tiles | `nat03_bridgeDirt01/02`, `nat03_start_slab` not drawn | `psg::Model::parse`: the file ends before the size it reads | in `todo` (the disc's own tracks too); not asked again here |
| `mxvatv_mc02`, `sm01`–`sm03` | no plants | `EcosystemFile` without `.esb` | asked: M9 *`EcosystemFile` without its extension* |
| `untamed_am_cs` | `AM_CS_Concretein`, `AM_CS_Concretein01` | the disc has only their `.col`: invisible collision, not a missing model | nothing to fix |
| four `elev_*` | (looked flat from the rider) | the NAIP picture at 2 m a pixel, no detail map: soft, not missing | nothing to fix; openmx-asks' elevation entry answered |
| 27 MXU tiles, 56 + 40 skies | nothing | the MXU ground (splat), every sky and every other material resolve | — |

Not a cause anywhere: the lower-case `tex\` beside `Tex\` (85 of the 127 here; the brief counted 86). `Res` folds case in every lookup, and
`Res::folder` refuses only one path spelt two ways, which no mod has (all 127 open). `.pst` against
`.PST` is the same fold.

## Every map in openmx (T10)

`accuracy ride --mod DIR --ram IMAGE --openmx BIN` (`tools/accuracy/src/ride.rs`) runs openmx once on the
mod: `--mods <mods> --track <id> --headless --no-sound --frames 180 --size 640x480 --overlay none
--graphics original`, with `--shot-pose` from the map's overview pose on the first frame (MD10), so one
run is the shot and six seconds of the start ridden with nobody on the pad (about 7 s). Where the `.toy`
has an `[AnimatedToy<n>]` with `HasCollision=TRUE`, a second two-frame run shoots the first of them. It
writes `openmx.txt` (the verdicts and openmx's log) and the shots in `--out` (default
`out/accuracy/<map>/`; this run's are in the slot's `out/accuracy/<id>/`) and prints the table's row.

- **loads**: openmx exits cleanly, prints `mods: <id>`, no `left out` line and nothing on stderr. A
  collision mesh `solid:` could not load must be in the mod's `todo` (FS05's `Anaheim_pole.col` and
  Nat10's `nat10_Mailbox_Closed.col` are).
- **heights**: the ground as the shell builds it (`ride::Shell::load`: the folder by `Res::folder`, the one
  `.scn`, the terrain by `track_mods::terrain_bytes`' suffix lookup, `mod.json`'s `spawn`), against the
  original's own query on the RAM image at `accuracy heights`' 2,000 points in each of the four modes
  (8,000), and the ride's own call (`World::ground`) against the harness's `look::height_at` at the same
  2,000. The box (`object_checks`) is a note, not a height (M12's ask).
- **start**: the dirt under the start that openmx prints is the shell ground's; at the end of the ride the
  bike is within 3 ft of the ground under it, and nothing wrecked it (`wreck: reason N`). The original
  stands the rider on the ground over a start under it (M12, M13), so the scene's own `y` is only noted.
- **edge**: openmx's `map: W x D` is the scene's `256 × w × scale` (MD3), and the rider is not thrown off
  the map (wreck 13). The file's box and `TerrainBorderWidth` are noted.
- **look**: the overview shots of all 96 read by eye on six contact sheets, and the toy shots of MC01,
  Nat04, Nat08 and Nat10.

openmx `2e369b3`, built into the slot's `extracted/openmx-target`. RAM images: M12's (slot s2) and M13's.
All 96 in 3 minutes, four at a time.

**In short.** All 96 load with nothing refused. Every height equals the original's at 8,000 points on
all 96, and the shell's ride query equals the harness's everywhere but the last sample row and column (†:
`world::Ground::at` stops a sample short; inside the 150 ft shoot-back margin; asked). MD4 draws every
map's ground from its own pages: no flat ground on any of the 96. The start fails on 24 maps: on 12 of
the 14 whose scene start is under the ground openmx does not stand the rider on it (asked again, with
the ride's evidence), on 11 arenas the rider is thrown off the map at once (openmx's edge leaves out
`TerrainBorderWidth`, which the game holds; asked), and on Nat_PA the rider drops 129 ft and wrecks.
MD5's windmill: the tower is drawn and its blades are not, and the same for Nat10's mill wheel and Nat04's
Ferris wheel. Animated toys are openmx's V43, open.

| map | loads | heights | start | edge | look |
|---|---|---|---|---|---|
| `mxvatv_fs01` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 3000, game holds 6896×6896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_fs02` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | **no**: start 20.9 ft under the dirt; wreck 11 (landed on the rear) | ok: 896×896; border 3000, game holds 6896×6896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_fs03` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 3000, game holds 6896×6896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_fs04` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 3000, game holds 6896×6896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_fs05` | ok: taken; Anaheim_pole.col not loaded (todo) | ok: 8,000 of 8,000 equal; 14 in the last strip † | **no**: start 23.5 ft under the dirt; bike ends -21.5 ft off the ground; wreck 13 (off the map) | **no**: 896×896; border 3000, game holds 6896×6896; thrown at the start | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_fs06` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | **no**: start 20.0 ft under the dirt; bike ends -12.1 ft off the ground; wreck 11 (landed on the rear), 11 (landed on the rear) | ok: 896×896; border 3000, game holds 6896×6896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_fs07` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 3000, game holds 6896×6896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_fs08` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.3 ft) | ok: 896×896; border 3000, game holds 6896×6896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_mc01` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured; windmill tower drawn, its blades not (V43) |
| `mxvatv_mc02` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_mc03` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.4 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_mc04` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_mc05` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_mc06` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 1250, game holds 3396×3396 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_mc07` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 2500, game holds 5896×5896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_mc08` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | **no**: wreck 13 (off the map) | **no**: 768×768; file's box 896×896; border 3000, game holds 6768×6768; thrown at the start | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_mc09` | ok: taken; Ring_Trigger.col not loaded (todo) | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_mc10` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_nat01` | ok: taken; recycling_can.col, trash_can.col not loaded (todo) | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_nat02` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_nat03` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_nat04` | ok: taken | ok: 8,000 of 8,000 equal; box 5760×5760, game's 4032×5760 | ok: on the ground (-0.5 ft) | ok: 5760×5760; file's box 4032×5760 | ground textured; Ferris wheel's stand drawn, the wheel not (V43) |
| `mxvatv_nat05` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_nat06` | ok: taken; Dino_Egg_A.col not loaded (todo) | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_nat07` | ok: taken; Nat07_churchsnow.col not loaded (todo) | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_nat08` | ok: taken; NatDropGate.col, NatStartGate.col not loaded (todo) | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_nat09` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_nat10` | ok: taken; nat10_Mailbox_Closed.col not loaded (todo) | ok: 8,000 of 8,000 equal; 2 in the last strip † | **no**: start 311.8 ft under the dirt; wreck 11 (landed on the rear) | ok: 5760×5760 | ground textured; mill house drawn, its wheel not (V43) |
| `mxvatv_nat11` | ok: taken; NatDropGate.col, NatStartGate.col, SingleBale01.col not loaded (todo) | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_nat12` | ok: taken; Nat12_Sphinx.col not loaded (todo) | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_nat13` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_nat14` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | **no**: start 6.4 ft under the dirt; wreck 11 (landed on the rear) | ok: 5760×5760 | ground textured |
| `mxvatv_nat15` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_nat16` | ok: taken; NatDropGate.col, NatStartGate.col not loaded (todo) | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_nat17` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.4 ft) | ok: 5760×5760 | ground textured |
| `mxvatv_sm01` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | **no**: start 201.4 ft under the dirt; bike ends -189.9 ft off the ground; wreck 11 (landed on the rear), 11 (landed on the rear) | ok: 5760×5760 | ground textured |
| `mxvatv_sm02` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | **no**: start 172.6 ft under the dirt; wreck 11 (landed on the rear) | ok: 5760×5760 | ground textured |
| `mxvatv_sm03` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | **no**: start 165.0 ft under the dirt; wreck 11 (landed on the rear) | ok: 5760×5760 | ground textured |
| `mxvatv_sx01` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 1280×1280; border 3000, game holds 7280×7280 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx02` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 1280×1280; border 2000, game holds 5280×5280 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx03` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 1000, game holds 2896×2896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx04` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 1000, game holds 2896×2896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx05` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 1250, game holds 3396×3396 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx06` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 1250, game holds 3396×3396 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx07` | ok: taken | ok: 8,000 of 8,000 equal; 8 in the last strip †; box 896×1792, game's 896×896 | ok: on the ground (-0.4 ft) | ok: 896×1792; file's box 896×896; border 1000, game holds 2896×3792 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx08` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | **no**: wreck 13 (off the map) | **no**: 896×896; border 1250, game holds 3396×3396; thrown at the start | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx09` | ok: taken; Anaheim_pole.col not loaded (todo) | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 3000, game holds 6896×6896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx10` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 2500, game holds 5896×5896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx11` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 1250, game holds 3396×3396 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx12` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 1250, game holds 3396×3396 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx13` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.4 ft) | ok: 896×896; border 3500, game holds 7896×7896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx14` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 1000, game holds 2896×2896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx15` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 1000, game holds 2896×2896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx16` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 250, game holds 1396×1396 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx17` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 8000, game holds 16896×16896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `mxvatv_sx18` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | **no**: wreck 13 (off the map) | **no**: 768×768; file's box 896×896; border 3000, game holds 6768×6768; thrown at the start | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_am_cs` | ok: taken; AM_CS_Metal_Bracket.col not loaded (todo) | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `untamed_am_gfuc` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured; a grey slab (race prop, MD13) |
| `untamed_am_jy` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 4480×4480 | ground textured |
| `untamed_bt_a` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | **no**: start 360.0 ft under the dirt; bike ends -290.6 ft off the ground | ok: 5760×5760 | ground textured |
| `untamed_bt_b` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | **no**: start 360.0 ft under the dirt; bike ends -290.6 ft off the ground | ok: 5760×5760 | ground textured |
| `untamed_fs_a` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | **no**: bike ends +12.4 ft off the ground; wreck 13 (off the map) | **no**: 896×896; border 1000, game holds 2896×2896; thrown at the start | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_fs_b` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | **no**: bike ends +6.1 ft off the ground; wreck 13 (off the map) | **no**: 896×896; border 1000, game holds 2896×2896; thrown at the start | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_fs_c` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | **no**: bike ends +3.6 ft off the ground; wreck 13 (off the map) | **no**: 896×896; border 5000, game holds 10896×10896; thrown at the start | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_fs_d` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | **no**: bike ends +3.2 ft off the ground; wreck 13 (off the map) | **no**: 896×896; border 1000, game holds 2896×2896; thrown at the start | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_mt_a` | ok: taken; FP06B6.col not loaded (todo) | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 2500, game holds 5896×5896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_nat_az` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `untamed_nat_co` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | **no**: start 7.8 ft under the dirt; wreck 11 (landed on the rear) | ok: 4480×4480 | ground textured |
| `untamed_nat_dvca` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: start 0.6 ft under the dirt | ok: 4480×4480 | ground textured |
| `untamed_nat_in` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured; a grey slab (race prop, MD13) |
| `untamed_nat_ma` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: start 0.8 ft under the dirt | ok: 4480×4480 | ground textured; a grey slab (race prop, MD13) |
| `untamed_nat_md` | ok: taken; NAT_MD_Drum2.col not loaded (todo) | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `untamed_nat_nc` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured; a grey slab (race prop, MD13) |
| `untamed_nat_ny` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 4480×4480 | ground textured |
| `untamed_nat_or` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `untamed_nat_pa` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | **no**: wreck 5 (landed too hard) | ok: 5760×5760 | ground textured |
| `untamed_nat_sdca` | ok: taken; SDCA_Powerlines.col not loaded (todo) | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `untamed_nat_ut` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `untamed_sm_as` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `untamed_sm_cf` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 4480×4480 | ground textured |
| `untamed_sm_hs` | ok: taken | ok: 8,000 of 8,000 equal; box 5760×5760, game's 4032×5760 | ok: on the ground (-0.5 ft) | ok: 5760×5760; file's box 4032×5760 | ground textured |
| `untamed_sm_mb` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | **no**: start 334.1 ft under the dirt; bike ends -287.9 ft off the ground; wreck 1 (over the bars) | ok: 5760×5760 | ground textured |
| `untamed_sm_of` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `untamed_sm_s1` | ok: taken | ok: 8,000 of 8,000 equal; 2 in the last strip † | ok: on the ground (-0.5 ft) | ok: 5760×5760 | ground textured |
| `untamed_sx_a` | ok: taken; FP01A_E.col not loaded (todo) | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.4 ft) | ok: 896×896; border 1000, game holds 2896×2896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_sx_b` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | **no**: wreck 13 (off the map) | **no**: 896×896; border 1000, game holds 2896×2896; thrown at the start | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_sx_c` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 1000, game holds 2896×2896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_sx_d` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 5000, game holds 10896×10896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_sx_e` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: start 3.2 ft under the dirt | ok: 896×896; border 1000, game holds 2896×2896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_sx_f` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | **no**: wreck 13 (off the map) | **no**: 896×896; border 1000, game holds 2896×2896; thrown at the start | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_sx_g` | ok: taken; FP03B4.col not loaded (todo) | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 1000, game holds 2896×2896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_sx_h` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: start 2.4 ft under the dirt | ok: 896×896; border 5000, game holds 10896×10896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_sx_i` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | **no**: start 9.6 ft under the dirt; wreck 11 (landed on the rear) | ok: 896×896; border 1000, game holds 2896×2896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_sx_j` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 1000, game holds 2896×2896 | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_sx_k` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | **no**: wreck 13 (off the map) | **no**: 896×896; border 1000, game holds 2896×2896; thrown at the start | ground textured; jumbotrons white (T11's `RenderTarget`) |
| `untamed_sx_l` | ok: taken | ok: 8,000 of 8,000 equal; 14 in the last strip † | ok: on the ground (-0.5 ft) | ok: 896×896; border 1000, game holds 2896×2896 | ground textured; jumbotrons white (T11's `RenderTarget`) |

† the last sample row and column: `World::ground` gives no ground at x or z ≥ (samples − 1) × spacing, where the original's query and the harness answer (notes/openmx-asks.md).
