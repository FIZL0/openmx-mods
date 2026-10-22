# MX vs. ATV Unleashed (PS2, SLUS-21104)

First survey 2026-10-21, off the USA disc, using nothing but openmx's own readers (`game-sim`'s
`archive::{Psr, Res}` and `track::{Terrain, Scene, Collision}`). Headline: **the track formats are MX
Unleashed's.** Same container, same `.res`, same PLN v7 terrain, same INI scene, same `.col`. Bringing a map
across is a repack plus the gaps below, not a new decompilation.

## Where things are

```
SLUS_211.04, GAME_REL.BIN, UI_REL.BIN    the boot ELF and two overlays (not looked at yet)
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

`MC` is the scenes' `SceneName=MachineChallenge0N`. Which maps the game offers as Freeride is in each
map's `.atyList` (not read yet), and that decides the first mods (M8).

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

- `.psg` models, `.pst`/`.psm` textures, `_mat.res`, `.toy`, `.esb`, skies and particles were not run
  through openmx's game-render readers. The texture pages were walked, not decoded (M9).
- The survey checks that each reader accepts each file and counts what it returns. It does not check that
  heights, placements or collision shapes are right against the running game (T7).
- What the game does with a placement whose `.col` is not on the disc, and with a scene key that
  disagrees with its terrain file (MC08, SX18, SX07, Nat04), was not looked up in the ELF.
- Whether MX Unleashed's own loader takes an 80×80-leaf terrain (T3) is open.
