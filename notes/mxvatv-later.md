# MX vs. ATV Untamed (PS2, SLUS-21701) — and later: Reflex, Alive

## Untamed (PS2)

First survey 2026-10-21, USA disc. Same container family as MX Unleashed and MX vs. ATV Unleashed:

```
MX08PS2.PSR   1.44 GB   PSR version 2, 20,552 members; openmx's Psr::parse reads it unchanged
MX08AUD.PSR   223 MB    audio
BLANKS.PAD    2.7 GB    padding, ignore
SLUS_217.01, GAME_REL.BIN, UI_REL.BIN
```

### Terrain: the same PLN v7

Surveyed 2026-10-22 by `tools/survey` (T6, `survey untamed`; the tables are under *Survey tables*). Each
map is in `res\env\<MAP>\<MAP>.res` (with `_mat.res` and `_mst.res` beside it): the `.ps2.trn`, the
`.scn.bpb`, `col\*.col`, `geo\*.psg`, the sky, particles. The 49 loose `Env\<MAP>\<MAP>.ps2.trn` are the
same bytes as their archive's (an ignored test in `tools/survey/tests/row.rs` compares all 48 pairs),
except SM_IP, which has no archive. Its scene is a 1,648-byte stub, as is TR_A's (1,616 bytes), and
neither map has a `.col`.

**`Res::parse` refuses all 48 map archives:** `the header claims a 1730-byte directory, 1470 bytes were
read` (FS_A). The directory is followed by 56 to 2,003 zero bytes that the header's directory length
counts. The survey reads them through a view without the padding, cut until `Res::parse` accepts the
directory and every block (`notes/openmx-asks.md`).

All 49 terrains read, through the same view as MX vs. ATV Unleashed's (`notes/mxvatv.md`, *Embedded
textures*; openmx MD1). Every block was checked against the file.

| maps | levels | leaves | spacing (ft) | side (ft) | blocks | texture pages |
|---|---|---|---|---|---|---|
| AM_GFUC, BT_C/D/F, Nat_AZ/IN/NC/PA/SDCA/UT, SJ_A/B, SM_AS/IP/MB/OF/S1, TR_A | 2 | 80×80 | 4.5 | 5760 | 794 | 48 |
| AM_JY, Nat_CO, Nat_DVCA, Nat_MA, Nat_NY, SM_CF | 2 | 80×80 | 3.5 | 4480 | 794 | 48 |
| AM_CS, Nat_MD | 2 | 80×80 | 4.5 | 5760 | 538 | 32 |
| Nat_OR | 2 | 80×80 | 4.499999 | 5759.999 | 538 | 32 |
| BT_A, BT_B | 2 | 80×80 | 4.5 | 5760 | 1050 | 64 |
| BT_E, SM_HS | 2 | 56×80 | 4.5 | 4032 × 5760 | 853 | 52 |
| HC_A | 2 | 80×80 | 4.5 | 5760 | 282 | 16 |
| FS_A–D, MT_A, SX_A–L | 1 | 16×16 | 3.5 | 896 | 257 | 16 |

Every page is format `0x27`, 1 mip, 256×256, 65,560 bytes, as on MX vs. ATV Unleashed. Nat_OR's file
scale is the single that prints as 4.499999, not 4.5. The first survey's "BT_E, SM_HS: 4480"
leaves is 56×80.

### Scenes: binary (BPB), decoded by `tools/bpb` (M10)

There is no text `.scn`. Each map has `<MAP>.scn.bpb` (and `.toy.bpb`, `.rpy.bpb`); 7,697 `.bpb` members
are loose on the PSR (6,288 `.psm.bpb`, 337 `.lyt.bpb`, 245 `.lyr.bpb`, ... 50 `.scn.bpb`), and the 48
map archives hold their scene again, the same bytes; `MX08AUD.PSR` has 16 more (`Aud\*.aud.bpb`,
`Aud\mix.pb.bpb`), which decode too. BPB is every text INI of the earlier games compiled
to a table. `bpb FILE.bpb` prints the INI; `bpb -r DIR OUT` does a tree.

Read off SLUS_217.01 (Ghidra project under `extracted/untamed/ghidra`, ../ps2-decomp-kit with
`KIT_CONFIG` at its `kit.toml`; source file string `Parameterblocks`):

- The loader (0x1425c0) opens `%s.bpb`, unwraps a `ZLIBME` file (u32 length at 12, zlib from 16;
  none is on the disc), and hands it to the header reader (0x142a50). With no `.bpb` it reads the text
  INI instead and builds the same table in memory (0x143680, 0x143160), every value as a string.
- **Header**, 0x20 bytes: `BPB\0`; u32 section-record offset (0x20); u32 key-record offset; u16
  sections, u16 keys; u32 name-table offset and length; u32 value-string offset and length.
- **Records are 16 bytes, not 12.** Section: u32 hash, u32 name offset, u16 index (0xffff none), u16
  key count, u32 first key. Its INI title is `%s%d` of name and index (0x142c30): `Model` 17 is
  `[Model17]`, `Sticker ` 61 is `[Sticker 61]`; index 0 occurs (636 sections). Key: u32 hash, u32 name
  offset, u16 type, u16 0, then by type **1** u32 offset of a string in the value strings, **2** a string
  of up to 5 bytes in the record's last 6 (`T`, `False`, `SX_C`), **4** i32, **8** f32. No other type
  is on the disc.
- **Hash**: djb2 (`h*33 + c` from 5381) over the name lowered through the ctype table (0x138510,
  0x347778). Lookups (0x142ca0, 0x140df0) compare hash and, for sections, index, then fall back to a
  case-folded compare. `HasCOL` and `HasCol` are both on the disc and hash alike.
- **Getters**: as text (0x141310) a float is `%.06f`, an int `%d`; as an int (0x141420) a float is
  truncated and a short string `T`/`ON`/`Y` is 1, `F`/`OFF`/`N` 0 (prefix, case-folded), else `strtol`;
  as a float (0x141690) an int converts and a string goes through `strtod`.
- Tables: each distinct name (and string) once, first-use order, NUL-terminated, padded with zeros to
  16. `tools/bpb`'s encoder rebuilds every one of the 7,697 files byte for byte from the decode.

**The INI** is CRLF, `[Title]` / `Key=value`, a blank line between sections. A float is `%.06f` when that
reads back to its bits both through a truncating `strtod`-to-single (the game's and openmx's text reader)
and round to nearest; otherwise (10,392 of 47,530 on the disc, e.g. 0.4) the shortest decimal of the
float as a double, which is exact. Three values end in a CR their source text had (`StoreText=Twin
Air\r` in `UI\Store.dat.bpb`, `NetChat.lyt.bpb`, `MXStuntMapping_16x9.lyt.bpb`) and are written as held.

Proof (`cargo test -p bpb --release -- --ignored`): all 7,697 parse, every section and key hash
recomputes from its name, every file re-encodes byte-exact, and all 50 scenes (48 archive copies too)
are read by openmx's `Scene::parse` with the terrain file, `TerrainWidth`, the float keys (bit for bit)
and the model and collision placement counts the BPB holds: 77,542 placements.

### Collision

1,472 `.col` in the 48 map archives, all read by `Collision::parse`. Loose on the PSR: `Col\` 1,536,
`Veh\Col\` 67, `Env\Nat_DVCA\Anim\` 1. Two are refused. `Col\FS_Panel_Blue01.col` is a 0-byte file.
`Env\Nat_DVCA\Anim\Nat05_Gallows_Door.col` is class 1, the animated-object class (MX vs. ATV
Unleashed's Nat05 gallows door, carried over). Which shared `Col\` shapes a map places is in its
scene: the survey reads it (M14), and every shape an Untamed scene places is in the map's own
archive or in `Col\`, except seven that are nowhere on the disc (*Survey tables*).

### The importer (M4)

`tools/import-mxvatv` reads Untamed too (`src/untamed.rs`): `import-mxvatv --disc <Untamed image>
--out STAGE` writes `untamed_<map>/` for each chosen map. It uses M2's copy and checks with three
additions: the archives opened through `discx::ResFile::unpadded`, `<MAP>.scn` decoded beside the
`.scn.bpb` (§2a), and BPB materials and toys decoded in memory so they can be checked. Journal:
`journal/2026-10-22-m4-import-untamed`.

**Which maps.** `Env\Master.atyList.bpb` is the one activity list, with `[<MAP> General]`
(`DisplayName`, `Activity` lines) for every map, and the activities' own sections (`Type`,
`StartPosition`, `ATYFile`, …). The 40 archives whose activities include `free ride` or `national` are
imported. Left out: BT_C–BT_F (opencross), HC_A (hill climb), SJ_A, SJ_B (ski jump), TR_A (a2b race).
The list also has TestAG, MT_B, AM_AF and SM_IP with a Free Ride, but none of them has a map archive.
Only Nat_MA's Free Ride gives a `StartPosition`. FS_A's gives `DefaultStartPosition`, a key the
activity reader (`0x002cb110`) does not read: it is the scene's (`0x0030e490`).

**Loose PSR members.** SLUS_217.01's resource manager is SLUS_211.04's (*Loose PSR members (M15)* in
`notes/mxvatv.md`), at other addresses. `0x0011e950` and `0x0011eff0` look a name up among the
loaded archives (`0x0011ec20`). On a miss `0x0011c7f0` opens the PSR member (`0x001bc760`, which
hashes the name through `0x00256c50` into the table that `0x001bd090` fills from each PSR named in
`MX08PS2.PSR,MX08AUD.PSR`). Flag `0x1000` turns the fallback off. Only `0x0011f250` sets it, from the
`.res` loader `0x00222050` when its fifth argument is non-zero, and all 43 call sites pass
`move $8, $zero`. What a map asks for, as the code builds it:
- placed models, `Geo\<slt>.psg`, lowercased (`0x00186bd0`; `0x001c27d0` sets the model folder
  `0x003fe640` to `Geo\`);
- collision, `Col\<name>.col` (`0x00114f20`, the extension swapped from the last dot);
- the sky, `Skies\<name>.ps2.cub` (`0x0030c4d0`: `%s.ps2.cub`, then `0x0018f230`: `Skies\%s`);
- the map's material table, `Env\<MAP>\<MAP>.mst` (`0x001c2d20`, `%s%s\%s.mst`). Its `_mst.res` is
  `res\env\<MAP>\<MAP>_mst.res`, and the archive holds it as `env\<map>\<MAP>.mst`;
- each material, through the BPB loader: `0x00168ff0` → `0x001425c0` opens `<line>.psm.bpb`, then
  `<line>.psm` as text, both through `0x0011e950`;
- each material's textures: `%s%s` of the folder and `TextureMap_<n>` (`0x00163930`, skipping
  `RenderTarget`, `RenderTarget2`, `FrameBuffer`, `DisplayBuffer`), tried as `.pst`, then `.atm`
  (`0x0016cba0`). Unlike MX vs. ATV Unleashed's, there is no `.pss` or `.tga` step.
- plants: the eco code is in GAME_REL.BIN (strings `EcoSystem.cpp`, `.esb`), which was not loaded.
  `Eco\<model>.psg` is M15's rule, and on the disc it pulls nothing.

Over the 40 maps the importer pulls **2,481 members (36 MB)**: 1,230 `.psm.bpb`, 1,202 `.pst`, 27
`.col`, 18 `.psg`, 2 `.cub` and 2 `.atm`. These include each map's `Env\<MAP>\TerrainLOD.psm.bpb`,
Nat_OR's 22 `Col\` shapes and 13 `Geo\` props (its archive has neither), AM_JY's and BT_B's shared
pieces, `Geo\nat04_lightflare.psg` (AM_GFUC, SM_AS, SM_HS), and the skies of SM_CF
(`Skies\sky04.ps2.cub`) and SX_B (`Skies\sky25.ps2.cub`).

**What the port cannot read yet** (each mod's `todo`; `notes/openmx-asks.md`). The archives' padding.
Every BPB but the decoded scene: 24 to 130 `.psm.bpb` a mod, the `.toy.bpb`, `.prt.bpb` and
`.aud.bpb`. Through tools/bpb's decode, game-render's `world::Assets::resolve` finds a texture for
every placed model's material but `screen.psm` (`RenderTarget`, the SX/FS/MT_A jumbotrons), SX_D's
`skyline01` and SX_G's 14 `alt-02_*`. Those last have no `.psm` on the disc at all. `Blimp.mot`
(Nat_CO, SM_S1) is refused by `skin::Clip::parse`. `DetailMapTilingMultiplier` is 6 on BT_B, SM_MB,
SM_OF and SM_S1, and 16 on FS_C. SM_HS's scene says `TerrainWidth=5` over a 4-root-cell terrain, and
Nat_OR's says 4.5 ft over a 4.499999-ft one. Both are kept (§2a). AM_CS places two
`AM_CS_Concretein*.psg` that are nowhere on the disc.

### Survey tables

`cargo run --release -p survey -- untamed`, 2026-10-22. † = the terrain read through the view. The
scene columns are from each map's `<MAP>.scn.bpb`, decoded by `tools/bpb` and read by `Scene::parse`
(M14); "(BPB, n B)" is the file's size.

Scene, read 2026-10-22 (M14):

- All 49 scenes read: 69,331 models (`[ModelN]` with an `SLT`) and 8,211 collision placements
  (`[ObjectN]` with a `COL`), 77,542 together, M10's count. The ignored test
  `on_untamed_every_scene_is_read_from_its_bpb` in `tools/survey/tests/row.rs` checks each map's two
  counts against the sections its BPB holds.
- Every scene's `TerrainWidth` and `TerrainBreadth` are 5 (1 on the 16×16 FS, MT and SX maps), and the
  scale is the terrain header's spacing on every map but Nat_OR, whose scene says 4.5 where its
  terrain file says 4.499999.
- `TerrainFile` is `<MAP>.trn` on 43 maps, where the archive holds `<MAP>.ps2.trn` (the table shows
  it); SX_F to SX_K name `<MAP>.ps2.trn`. How the game gets from one to the other was not looked up.
- SM_IP and TR_A, the two 1.6 KB stubs, place nothing: 0 models, 0 collision.
- Seven `.col` that a scene places are on the disc nowhere, not in the map's archive, not in
  `Col\`, not in any other map archive (only a `Geo\<name>.psg` of each is on the PSR): AM_CS
  `AM_CS_Metal_Bracket.col` ×4 and `AM_CS_Metal_Bracket2.col` ×2, MT_A `FP06B6.col` ×1, Nat_MD
  `NAT_MD_Drum2.col` ×1, Nat_SDCA `SDCA_Powerlines.col` ×2, SX_A `FP01A_E.col` ×1, SX_G
  `FP03B4.col` ×1: 12 placements of 8,211. What the game does with them (skip the object, or collide
  with nothing) was not looked up. Every other placed `.col` is in the map's own archive, or for
  AM_JY (3 files), BT_B (2) and Nat_OR (22) in the PSR's shared `Col\` (the "loose" in the table).

| map | levels | leaves | spacing (ft) | side (ft) | blocks | texture pages | scene | scene terrain keys | models | collision placements | `.col` read | `.col` refused |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| AM_CS | 2 | 80×80 | 4.5 | 5760 | 538 | 32 × 256×256 fmt 0x27, 1 mip, 65560 B † | AM_CS (BPB, 88736 B) | 5×5 @ 4.5, TerrainFile=AM_CS.trn | 391 | 378 (60 files: 58 here, missing am_cs_metal_bracket.col am_cs_metal_bracket2.col) | 58 of 58 | 0 |
| AM_GFUC | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | AM_GFUC (BPB, 62160 B) | 5×5 @ 4.5, TerrainFile=AM_GFUC.trn | 243 | 218 (30 files: 30 here) | 30 of 30 | 0 |
| AM_JY | 2 | 80×80 | 3.5 | 4480 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | AM_JY (BPB, 102528 B) | 5×5 @ 3.5, TerrainFile=AM_JY.trn | 445 | 443 (33 files: 30 here, 3 loose) | 30 of 30 | 0 |
| BT_A | 2 | 80×80 | 4.5 | 5760 | 1050 | 64 × 256×256 fmt 0x27, 1 mip, 65560 B † | BT_A (BPB, 37024 B) | 5×5 @ 4.5, TerrainFile=BT_A.trn | 128 | 110 (21 files: 21 here) | 21 of 21 | 0 |
| BT_B | 2 | 80×80 | 4.5 | 5760 | 1050 | 64 × 256×256 fmt 0x27, 1 mip, 65560 B † | BT_B (BPB, 25328 B) | 5×5 @ 4.5, TerrainFile=BT_B.trn | 112 | 84 (15 files: 13 here, 2 loose) | 13 of 13 | 0 |
| BT_C | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BT_C (BPB, 23392 B) | 5×5 @ 4.5, TerrainFile=BT_C.trn | 113 | 109 (25 files: 25 here) | 25 of 25 | 0 |
| BT_D | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BT_D (BPB, 38240 B) | 5×5 @ 4.5, TerrainFile=BT_D.trn | 187 | 163 (25 files: 25 here) | 25 of 25 | 0 |
| BT_E | 2 | 56×80 | 4.5 | 4032 | 853 | 52 × 256×256 fmt 0x27, 1 mip, 65560 B † | BT_E (BPB, 113488 B) | 5×5 @ 4.5, TerrainFile=BT_E.trn | 649 | 538 (82 files: 82 here) | 82 of 82 | 0 |
| BT_F | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BT_F (BPB, 25936 B) | 5×5 @ 4.5, TerrainFile=BT_F.trn | 156 | 152 (25 files: 25 here) | 25 of 25 | 0 |
| FS_A | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | FS_A (BPB, 423072 B) | 1×1 @ 3.5, TerrainFile=FS_A.trn | 3996 | 21 (5 files: 5 here) | 5 of 5 | 0 |
| FS_B | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | FS_B (BPB, 330144 B) | 1×1 @ 3.5, TerrainFile=FS_B.trn | 3098 | 17 (3 files: 3 here) | 3 of 3 | 0 |
| FS_C | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | FS_C (BPB, 186752 B) | 1×1 @ 3.5, TerrainFile=FS_C.trn | 1597 | 53 (16 files: 16 here) | 16 of 16 | 0 |
| FS_D | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | FS_D (BPB, 308912 B) | 1×1 @ 3.5, TerrainFile=FS_D.trn | 2979 | 9 (3 files: 3 here) | 3 of 3 | 0 |
| HC_A | 2 | 80×80 | 4.5 | 5760 | 282 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | HC_A (BPB, 69392 B) | 5×5 @ 4.5, TerrainFile=HC_A.trn | 316 | 12 (1 files: 1 here) | 1 of 1 | 0 |
| MT_A | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | MT_A (BPB, 217408 B) | 1×1 @ 3.5, TerrainFile=MT_A.trn | 1935 | 176 (19 files: 18 here, missing fp06b6.col) | 28 of 28 | 0 |
| Nat_AZ | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat_AZ (BPB, 61328 B) | 5×5 @ 4.5, TerrainFile=Nat_AZ.trn | 300 | 296 (46 files: 46 here) | 46 of 46 | 0 |
| Nat_CO | 2 | 80×80 | 3.5 | 4480 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat_CO (BPB, 46144 B) | 5×5 @ 3.5, TerrainFile=Nat_CO.trn | 198 | 190 (44 files: 44 here) | 44 of 44 | 0 |
| Nat_DVCA | 2 | 80×80 | 3.5 | 4480 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat_DVCA (BPB, 67664 B) | 5×5 @ 3.5, TerrainFile=Nat_DVCA.trn | 282 | 266 (68 files: 68 here) | 68 of 68 | 0 |
| Nat_IN | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat_IN (BPB, 58544 B) | 5×5 @ 4.5, TerrainFile=Nat_IN.trn | 247 | 235 (41 files: 41 here) | 41 of 41 | 0 |
| Nat_MA | 2 | 80×80 | 3.5 | 4480 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat_MA (BPB, 38256 B) | 5×5 @ 3.5, TerrainFile=Nat_MA.trn | 170 | 150 (36 files: 36 here) | 36 of 36 | 0 |
| Nat_MD | 2 | 80×80 | 4.5 | 5760 | 538 | 32 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat_MD (BPB, 54688 B) | 5×5 @ 4.5, TerrainFile=Nat_MD.trn | 244 | 243 (49 files: 48 here, missing nat_md_drum2.col) | 48 of 48 | 0 |
| Nat_NC | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat_NC (BPB, 66496 B) | 5×5 @ 4.5, TerrainFile=Nat_NC.trn | 291 | 249 (59 files: 59 here) | 59 of 59 | 0 |
| Nat_NY | 2 | 80×80 | 3.5 | 4480 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat_NY (BPB, 63344 B) | 5×5 @ 3.5, TerrainFile=Nat_NY.trn | 329 | 238 (53 files: 53 here) | 53 of 53 | 0 |
| Nat_OR | 2 | 80×80 | 4.499999 | 5759.999 | 538 | 32 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat_OR (BPB, 53232 B) | 5×5 @ 4.5, TerrainFile=Nat_OR.trn | 256 | 246 (51 files: 29 here, 22 loose) | 29 of 29 | 0 |
| Nat_PA | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat_PA (BPB, 52208 B) | 5×5 @ 4.5, TerrainFile=Nat_PA.trn | 268 | 267 (43 files: 43 here) | 43 of 43 | 0 |
| Nat_SDCA | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat_SDCA (BPB, 41104 B) | 5×5 @ 4.5, TerrainFile=Nat_SDCA.trn | 215 | 205 (38 files: 37 here, missing sdca_powerlines.col) | 37 of 37 | 0 |
| Nat_UT | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | Nat_UT (BPB, 111984 B) | 5×5 @ 4.5, TerrainFile=Nat_UT.trn | 524 | 509 (32 files: 32 here) | 32 of 32 | 0 |
| SJ_A | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | SJ_A (BPB, 70384 B) | 5×5 @ 4.5, TerrainFile=SJ_A.trn | 531 | 55 (29 files: 29 here) | 29 of 29 | 0 |
| SJ_B | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | SJ_B (BPB, 64384 B) | 5×5 @ 4.5, TerrainFile=SJ_B.trn | 531 | 55 (29 files: 29 here) | 29 of 29 | 0 |
| SM_AS | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | SM_AS (BPB, 83824 B) | 5×5 @ 4.5, TerrainFile=SM_AS.trn | 581 | 343 (40 files: 40 here) | 40 of 40 | 0 |
| SM_CF | 2 | 80×80 | 3.5 | 4480 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | SM_CF (BPB, 75024 B) | 5×5 @ 3.5, TerrainFile=SM_CF.trn | 490 | 321 (38 files: 38 here) | 39 of 39 | 0 |
| SM_HS | 2 | 56×80 | 4.5 | 4032 | 853 | 52 × 256×256 fmt 0x27, 1 mip, 65560 B † | SM_HS (BPB, 124080 B) | 5×5 @ 4.5, TerrainFile=SM_HS.trn | 700 | 570 (70 files: 70 here) | 70 of 70 | 0 |
| SM_IP | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | SM_IP (BPB, 1648 B) | 5×5 @ 4.5, TerrainFile=SM_IP.trn | 0 | 0 (0 files: 0 here) | 0 of 0 | 0 |
| SM_MB | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | SM_MB (BPB, 105584 B) | 5×5 @ 4.5, TerrainFile=SM_MB.trn | 500 | 452 (61 files: 61 here) | 61 of 61 | 0 |
| SM_OF | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | SM_OF (BPB, 74944 B) | 5×5 @ 4.5, TerrainFile=SM_OF.trn | 331 | 319 (54 files: 54 here) | 54 of 54 | 0 |
| SM_S1 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | SM_S1 (BPB, 88752 B) | 5×5 @ 4.5, TerrainFile=SM_S1.trn | 542 | 238 (37 files: 37 here) | 44 of 44 | 0 |
| SX_A | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | SX_A (BPB, 419552 B) | 1×1 @ 3.5, TerrainFile=SX_A.trn | 4042 | 25 (13 files: 12 here, missing fp01a_e.col) | 16 of 16 | 0 |
| SX_B | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | SX_B (BPB, 375408 B) | 1×1 @ 3.5, TerrainFile=SX_B.trn | 3375 | 23 (13 files: 13 here) | 17 of 17 | 0 |
| SX_C | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | SX_C (BPB, 451280 B) | 1×1 @ 3.5, TerrainFile=SX_C.trn | 4377 | 24 (12 files: 12 here) | 16 of 16 | 0 |
| SX_D | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | SX_D (BPB, 244160 B) | 1×1 @ 3.5, TerrainFile=SX_D.trn | 2192 | 24 (13 files: 13 here) | 17 of 17 | 0 |
| SX_E | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | SX_E (BPB, 599216 B) | 1×1 @ 3.5, TerrainFile=SX_E.trn | 5713 | 24 (13 files: 13 here) | 17 of 17 | 0 |
| SX_F | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | SX_F (BPB, 477744 B) | 1×1 @ 3.5 | 4479 | 27 (12 files: 12 here) | 16 of 16 | 0 |
| SX_G | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | SX_G (BPB, 553872 B) | 1×1 @ 3.5 | 5347 | 23 (14 files: 13 here, missing fp03b4.col) | 15 of 15 | 0 |
| SX_H | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | SX_H (BPB, 244608 B) | 1×1 @ 3.5 | 2191 | 19 (11 files: 11 here) | 15 of 15 | 0 |
| SX_I | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | SX_I (BPB, 336176 B) | 1×1 @ 3.5 | 3204 | 23 (14 files: 14 here) | 19 of 19 | 0 |
| SX_J | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | SX_J (BPB, 336960 B) | 1×1 @ 3.5 | 3116 | 23 (13 files: 13 here) | 17 of 17 | 0 |
| SX_K | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | SX_K (BPB, 468368 B) | 1×1 @ 3.5 | 4348 | 22 (16 files: 16 here) | 20 of 20 | 0 |
| SX_L | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | SX_L (BPB, 322992 B) | 1×1 @ 3.5, TerrainFile=SX_L.trn | 3072 | 24 (16 files: 16 here) | 20 of 20 | 0 |
| TR_A | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | TR_A (BPB, 1616 B) | 5×5 @ 4.5, TerrainFile=TR_A.trn | 0 | 0 (0 files: 0 here) | 0 of 0 | 0 |

49 maps. † 49 terrains are refused by `Terrain::parse` as they are (openmx MD1); the survey read their texture table itself and passed the reader a view without it, the blocks moved up against the tree.
`.col` in the map archives: 1472, of which 0 refused by `Collision::parse`.
48 loose `.trn` on the PSR are the same maps as an archive's and are not surveyed again.

- AM_CS, AM_GFUC, AM_JY, BT_A, BT_B, BT_C, BT_D, BT_E, BT_F, FS_A, FS_B, FS_C, FS_D, HC_A, MT_A, Nat_AZ, Nat_CO, Nat_DVCA, Nat_IN, Nat_MA, Nat_MD, Nat_NC, Nat_NY, Nat_OR, Nat_PA, Nat_SDCA, Nat_UT, SJ_A, SJ_B, SM_AS, SM_CF, SM_HS, SM_MB, SM_OF, SM_S1, SX_A, SX_B, SX_C, SX_D, SX_E, SX_F, SX_G, SX_H, SX_I, SX_J, SX_K, SX_L, TR_A: `Res::parse` refuses the `.res`: its directory is padded with zero bytes past the last entry, so fewer bytes are read than the header claims; read through a view without them
- BT_A, BT_B: `Terrain::parse` on the file: 1050 blocks of 2129 from 5553 end at 2241003, but the tree starts at 6436843
- FS_A, FS_B, FS_C, FS_D, MT_A, SX_A, SX_B, SX_C, SX_D, SX_E, SX_F, SX_G, SX_H, SX_I, SX_J, SX_K, SX_L: `Terrain::parse` on the file: 257 blocks of 2129 from 2189 end at 549342, but the tree starts at 1598302
- HC_A: `Terrain::parse` on the file: 282 blocks of 2129 from 2289 end at 602667, but the tree starts at 1651627
- AM_CS, Nat_MD, Nat_OR: `Terrain::parse` on the file: 538 blocks of 2129 from 3377 end at 1148779, but the tree starts at 3246699
- AM_GFUC, AM_JY, BT_C, BT_D, BT_F, Nat_AZ, Nat_CO, Nat_DVCA, Nat_IN, Nat_MA, Nat_NC, Nat_NY, Nat_PA, Nat_SDCA, Nat_UT, SJ_A, SJ_B, SM_AS, SM_CF, SM_IP, SM_MB, SM_OF, SM_S1, TR_A: `Terrain::parse` on the file: 794 blocks of 2129 from 4465 end at 1694891, but the tree starts at 4841771
- BT_E, SM_HS: `Terrain::parse` on the file: 853 blocks of 2129 from 4717 end at 1820754, but the tree starts at 5229874
- SM_IP: loose on the PSR, no map `.res` holds it
- The `.res` directory padding runs from 56 to 2003 zero bytes.

#### Every `.col` loose on the PSR

| folder | `.col` | read | refused |
|---|---|---|---|
| Col | 1536 | 1535 | fs_panel_blue01.col |
| Env\Nat_DVCA\Anim | 1 | 0 | nat05_gallows_door.col |
| Veh\Col | 67 | 67 | 0 |

| loose `.col` refused | why |
|---|---|
| Col\FS_Panel_Blue01.col | a collision file: wanted 4 bytes at 0 of a 0-byte file |
| Env\Nat_DVCA\Anim\Nat05_Gallows_Door.col | collision class 1 is not understood |

## Reflex, Alive (PS3 / 360)

No discs yet. Reflex has deformable terrain, so likely a heightfield again.

## Not verified

- The scene placements are counted as `Scene::parse` reads them; that the running game places the same
  objects (and what it does with the seven missing `.col`) is not checked (T7).
- The `.res` view's directory is the reader's own, but why the game pads it (alignment? a reserved
  table?) was not looked up in the ELF.
- No `.psg` or texture parsed. The texture pages were walked, not decoded.
- Heights are read, not checked against the running game (T7). The 3.5 vs 4.5 spacing comes from the
  terrain header, not from a scene.
