# M13: MX vs. ATV Untamed accuracy pass (every imported map through T7)

All 40 imported MX vs. ATV Untamed maps were ridden in PCSX2 (kit copy 9, US disc SLUS_217.01 1.01) and
held against their mods, as M12 did for Unleashed. Each map got the ground, point by point; the scene's
numbers as the game holds them; every file the game opened for it; and the original's half of the
screenshot pairs, 9–11 poses a map. The per-map table is in `notes/mxvatv-later.md`, *Against the game
(M13)*. How it was run is in `notes/accuracy.md`, *Untamed (M13)*. What only openmx can fix is in
`notes/openmx-asks.md`, in the M13 entries above MD10.

## What was done

- **A kit config and a Ghidra project for Untamed** (`extracted/untamed/`, not in git). `GAME_REL.BIN`
  is the gameplay module. Its header puts it at `0x48ea80`, where `UI_REL.BIN` sits in the menus.
  `accuracy setup --game untamed` writes the same config.
- **`source::UNTAMED`**. I found the terrain object in a RAM image of SX_A as on Lago: the `.trn` root
  node's 16 bytes, and a word pointing at them with the scale beside it. Every field is at Unleashed's
  offset. The loads of `+0x788`/`+0x9e0` led to the queries `0x4b0e30` and `0x4b0910`. They lead on to
  the cell reader `0x4a8ea0`, which has the same step over 127; its packed-table flag is `0x5440e8`.
- **The scene's readers** (`sceneram::Readers`, `UNTAMED`; Unleashed's constants kept as `MXVATV`).
  I found them through their key strings in the boot ELF and took their vtables off the image.
  - `[SceneInfo]` `0x30e490` makes the direction unit.
  - `[Fog]` `0x310180` adds `FogStart` and `FogEnd`.
  - `[Environment]` `0x30f0b0` adds `DetailMapLODBias` and `EcosystemZone`, so every word from the
    detail multiplier on moves.
  - The lights use Unleashed's members, with vtable `0x3d647c`.
  - `scene` compares the start's direction made unit, as a computed field.
- **The camera**. The viewport has Unleashed's layout. The ride camera's fov is 75 (`look::ride_fov`),
  so the generated poses use 75. Holding the camera took three tries (below). What works is one word
  of code each: the viewport setter `0x111700` and its look-at `0x1117d0` become `jr ra`
  (`Camera::also`).
- **Driving.** The game's cheat `YOUGOTIT` (`UI\CheatCodes.txt.bpb`) opens every track. State 5 of
  copy 9 is the Free Ride list. In that state the place is word `0x125de14` + word `0x125ddd4`, found
  by diffing captures a press apart; the order is `UI\Tracks.dat`'s. Per map I ran `capture`,
  `heights --points 2000`, `scene`, `files`, `poses`, `shoot --live live`, `shoot` and `pair`, about
  2.5 minutes a map. Then I re-ran `heights`, `scene` and `files` on all 40 images with the final
  binary.
- **tools/import-mxvatv**:
  - Each map's archives now include `res\Shadows.res`: the six shadow sets' 18 `.psm.bpb`
    (`0x30cb50`), which every ride opened and no mod had (the files check found them first on SX_A).
  - M12's spawn rule now applies to Untamed: no "not looked up" entry, and a todo where the start is
    under the ground (BT_A, BT_B, SM_MB, Nat_CO, SX_E, SX_H, SX_I).
  - All 40 were re-imported into `out/mods/tracks/`.
- **tools/accuracy**, beyond the above:
  - `capture` takes the newest state file of the slot. Copy 9 has one per game's serial, and the first
    capture took Unleashed's.
  - `files` knows the scene as `.scn.bpb` and counts `Aud\Rider` as the game's.
  - `look::terrain_box` multiplies the way the EE does (Nat_OR).
  - `tests/every_map.rs` runs on `untamed_*` too.
  - New tests: Untamed's reader layout on a synthetic image, the kit config, the fov and the hold.

## How it was proved

- `heights`, on all 40: every height, surface and corner weight of both queries equals openmx's at
  8,000 points. Normals are 1–3 ulp off at 67–92 % of the points, and every field is equal with the
  step. No map turns on the packed table. The 18 exits of 1 are the box: openmx's `Ground` against the
  game's, on the one-level maps and SM_HS.
- The game's terrain box is the file's leaves on all 40 (`tests/every_map.rs`).
- `scene`: 32 or 35 fields a map, none differing. The sun's direction and the start's direction, both
  made unit, are within 1–2 ulp.
- `files`: every map name the game opened is in the re-imported mod, on all 40.
- `cargo test -p accuracy --test every_map` with the images: 40 maps, ok, 87 s (debug).
- `cargo test --release -p import-mxvatv --test untamed --test disc -- --ignored`: the 40 Untamed and
  56 Unleashed real-disc imports, ok. The new in-gate tests are `every_map_takes_the_shadow_archive`
  and `without_a_list_start_the_scene_spawns_as_the_game_does`.
- 423 original shots, every one a whole PNG (`shoot` checks), each one's pose read back from the
  viewport.
- `./check.sh` passes: 112 tests.

## What was tried and did not work

- **The first hold**: patching the look-at's first two words (`nop` its `addiu sp`, then `jr ra`). The
  game ran it between the two writes (PCSX2's log: `TLB Miss, pc=0x1117d0`). Some maps later the
  recompiler logged 23 million "Impossible block clearing failure" lines, stopped saving states, and
  the copy hung. That happened twice: after 29 maps, then 5 maps after a restart. The second word was
  never needed: with `jr ra` in the first word, the delay slot is a harmless `addiu $3`. With one word
  a function the last nine maps ran clean. The log had 126 of those lines, and nothing hung.
- **A data-only hold** (vtable slots, as M12's): stubbing the viewport's own slots left the eye
  written. Stubbing the follow camera's `0x3ef174` (`0x4b6670`) kept the viewport's numbers, but the
  picture did not move, because that path also builds the view matrices.
- **Restarting the copy every five maps**: the pad server did not come back after one restart, and
  every later map failed at its first press. Once the cause was found, the restarts were not needed.
- **`[SceneInfo]`, the first block with the vtable**: on SX_C that was an unread block (all zeros). The
  scene's block is the first with a unit direction (falling back to the first, for M12's synthetic test).

## The gate, moved under the task

openmx's live checkout took its S98 during the session (`c42f4b3`): `Terrain::parse` now reads
texture pages, which is MD1 done on openmx's side. That broke tests that asserted it refuses them:
`import-mxvatv` `disc.rs`, `modcheck` `disc.rs`, `survey` `row.rs` and `trnwrite` `grids.rs`. Main had no
fix. Each test now checks what openmx does with the file: the old assertion while it refuses, and the
file read directly once it accepts. The tools themselves were not changed; they already fall back to
the view only on a refusal. Main's changes since the branch (C2) touch none of these files. MD1 is
still under *Filed* in `notes/openmx-asks.md`: whether it is done is for the human who files asks to
say.

## Not verified

- **openmx's half of the pairs**: no MD2 or MD10 in openmx, as for M12. No difference of look was found
  by comparing pictures; the look was checked through the data the game holds.
- **The pool's head on 36 maps** (all but FS_A–D) was overwritten before the capture. The names lost
  are the first members of the map's own archive (on Nat_IN, its `.scn.bpb` and 11 props), which the
  mod holds whole, so they are in the mod. That they are the only names lost was not proved. A capture
  earlier in the load would show it.
- **Ecosystem density**, the lights' shadow keys and the `[DistanceFade]` defaults were not compared,
  as on M12.
- **The start under the ground** is inferred from the ride camera's height in the image (342–368 ft
  over the start on BT_A, BT_B and SM_MB; 11–16 ft, against 8 elsewhere, on the four small ones). The
  rider's own position was not read.
- **Where the shadow sets' count (6) comes from**: `0x30cb50` was called with 6 on every map, judging
  by the pool (`shadowCaster1`–`6`, no 7th). The caller was not found.
- **Only a bike was ridden**, so names opened only for other machines were not seen.
- **Which activity props a Free Ride draws** (`Activity`, `Hide`) was not settled.
- **The 28 images taken with the two-word hold**: their checks are offline and their shots verified
  the pose. Whether the TLB miss changed anything the shots show was not checked; it was the look-at's
  own stack and register, in the frame before the hold.
- **The disc's `dynamicShadowReceiver%d`** (`0x30cb50`, when its fifth argument is set) was not asked
  for in any ride, so it is not pulled.
