# The accuracy harness (T7)

`tools/accuracy` holds a track mod against its source game running in PCSX2:

- **heights**: the original's own terrain queries, run on a RAM image of the game in a ride on the map,
  against openmx's port of the query on the mod's `.trn`, at a thousand points and more, bit for bit;
- **the look**: screenshot pairs from the same camera poses, the original's (`./kit shot`, its camera
  put at the pose over PINE) and openmx's (`--shot-pose`, openmx MD10), side by side in
  `out/accuracy/<map>/`.

Proved on one map by T7: MX vs. ATV Unleashed's **Lago** (`Nat05`, `mxvatv_nat05`); M12 ran it on all 56 (*Every map*, below). The heights are equal at
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
positive), no roll, and the viewport's fov. Whether 85 spans the width or the height is not
established. `+0x178` holds 349.74, close to 320/tan(42.5°) = 349.2, which would make it the width. The
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

## For M12 and M13

- Every map: `capture` in its Free Ride, then `heights`. Then `poses` (add landmarks with `--add`),
  `shoot`, and `pair` once openmx has MD2 and MD10. Check `pair`'s `--openmx-args` template (`--mod
  {mod} --shot {shot} --shot-pose {pose}`) against what MD2 and MD10 name their flags.

## openmx's half (T9, waiting)

On 2026-10-22 openmx's main (`b63486a`) has neither MD2 nor MD10: both are open in its `plan/TODO.md`,
no branch or worktree carries them, and its `--help` has `--shot` and `--shot-frame` but no
`--shot-pose` and no flag for a mod folder. `accuracy pair` on Lago says so in `index.md` and takes
nothing. Ready for when they land:

- The template is `look::OPENMX_ARGS`, the command line `look::openmx_args` (tested: a path with a
  space stays one word, the `--shot-pose` word reads back as the pose). MD10's flag and its
  `x,y,z,yaw,pitch,fov` are fixed by openmx's plan. MD2's flag is not named there: `--mod` is a
  stand-in. When MD2 lands, change `OPENMX_ARGS` to its flag (if it is a track id rather than a
  folder, `--track {id}` needs a `{id}` placeholder, from mod.json).
- `pair` checks the binary's `--help` for every flag in the template as a whole word. The old check
  looked for the string `--mod`, which `--modifiers` (in openmx's argument parser) would have passed.
- Still to do then: settle the fov's axis (Poses, above) by shooting one pose in openmx with 85 and
  comparing where the horizon and a landmark fall at the frame's edges, take the eight pairs, and write
  what differs here. The original's shots carry the HUD; give openmx `--overlay original` or `none`
  according to what the pairs are for.
- Untamed (M13): no `source` row yet. It needs a kit config with its own module (look for a `*_REL.BIN`
  on its disc), its object layout, query, cell reader and camera found the way above, and a check of
  which normal path its reader takes.
