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
scene (now decodable with `tools/bpb`; the survey does not read it yet).

### Survey tables

`cargo run --release -p survey -- untamed`, 2026-10-22. † = the terrain read through the view. The
scene columns are empty: the survey predates `tools/bpb` (M10) and does not decode BPB yet.

| map | levels | leaves | spacing (ft) | side (ft) | blocks | texture pages | scene | scene terrain keys | models | collision placements | `.col` read | `.col` refused |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| AM_CS | 2 | 80×80 | 4.5 | 5760 | 538 | 32 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 88736 B, not read (M10) | – | – | – | 58 of 58 | 0 |
| AM_GFUC | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 62160 B, not read (M10) | – | – | – | 30 of 30 | 0 |
| AM_JY | 2 | 80×80 | 3.5 | 4480 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 102528 B, not read (M10) | – | – | – | 30 of 30 | 0 |
| BT_A | 2 | 80×80 | 4.5 | 5760 | 1050 | 64 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 37024 B, not read (M10) | – | – | – | 21 of 21 | 0 |
| BT_B | 2 | 80×80 | 4.5 | 5760 | 1050 | 64 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 25328 B, not read (M10) | – | – | – | 13 of 13 | 0 |
| BT_C | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 23392 B, not read (M10) | – | – | – | 25 of 25 | 0 |
| BT_D | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 38240 B, not read (M10) | – | – | – | 25 of 25 | 0 |
| BT_E | 2 | 56×80 | 4.5 | 4032 | 853 | 52 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 113488 B, not read (M10) | – | – | – | 82 of 82 | 0 |
| BT_F | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 25936 B, not read (M10) | – | – | – | 25 of 25 | 0 |
| FS_A | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 423072 B, not read (M10) | – | – | – | 5 of 5 | 0 |
| FS_B | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 330144 B, not read (M10) | – | – | – | 3 of 3 | 0 |
| FS_C | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 186752 B, not read (M10) | – | – | – | 16 of 16 | 0 |
| FS_D | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 308912 B, not read (M10) | – | – | – | 3 of 3 | 0 |
| HC_A | 2 | 80×80 | 4.5 | 5760 | 282 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 69392 B, not read (M10) | – | – | – | 1 of 1 | 0 |
| MT_A | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 217408 B, not read (M10) | – | – | – | 28 of 28 | 0 |
| Nat_AZ | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 61328 B, not read (M10) | – | – | – | 46 of 46 | 0 |
| Nat_CO | 2 | 80×80 | 3.5 | 4480 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 46144 B, not read (M10) | – | – | – | 44 of 44 | 0 |
| Nat_DVCA | 2 | 80×80 | 3.5 | 4480 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 67664 B, not read (M10) | – | – | – | 68 of 68 | 0 |
| Nat_IN | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 58544 B, not read (M10) | – | – | – | 41 of 41 | 0 |
| Nat_MA | 2 | 80×80 | 3.5 | 4480 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 38256 B, not read (M10) | – | – | – | 36 of 36 | 0 |
| Nat_MD | 2 | 80×80 | 4.5 | 5760 | 538 | 32 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 54688 B, not read (M10) | – | – | – | 48 of 48 | 0 |
| Nat_NC | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 66496 B, not read (M10) | – | – | – | 59 of 59 | 0 |
| Nat_NY | 2 | 80×80 | 3.5 | 4480 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 63344 B, not read (M10) | – | – | – | 53 of 53 | 0 |
| Nat_OR | 2 | 80×80 | 4.499999 | 5759.999 | 538 | 32 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 53232 B, not read (M10) | – | – | – | 29 of 29 | 0 |
| Nat_PA | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 52208 B, not read (M10) | – | – | – | 43 of 43 | 0 |
| Nat_SDCA | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 41104 B, not read (M10) | – | – | – | 37 of 37 | 0 |
| Nat_UT | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 111984 B, not read (M10) | – | – | – | 32 of 32 | 0 |
| SJ_A | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 70384 B, not read (M10) | – | – | – | 29 of 29 | 0 |
| SJ_B | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 64384 B, not read (M10) | – | – | – | 29 of 29 | 0 |
| SM_AS | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 83824 B, not read (M10) | – | – | – | 40 of 40 | 0 |
| SM_CF | 2 | 80×80 | 3.5 | 4480 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 75024 B, not read (M10) | – | – | – | 39 of 39 | 0 |
| SM_HS | 2 | 56×80 | 4.5 | 4032 | 853 | 52 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 124080 B, not read (M10) | – | – | – | 70 of 70 | 0 |
| SM_IP | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 1648 B, not read (M10) | – | – | – | 0 of 0 | 0 |
| SM_MB | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 105584 B, not read (M10) | – | – | – | 61 of 61 | 0 |
| SM_OF | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 74944 B, not read (M10) | – | – | – | 54 of 54 | 0 |
| SM_S1 | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 88752 B, not read (M10) | – | – | – | 44 of 44 | 0 |
| SX_A | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 419552 B, not read (M10) | – | – | – | 16 of 16 | 0 |
| SX_B | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 375408 B, not read (M10) | – | – | – | 17 of 17 | 0 |
| SX_C | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 451280 B, not read (M10) | – | – | – | 16 of 16 | 0 |
| SX_D | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 244160 B, not read (M10) | – | – | – | 17 of 17 | 0 |
| SX_E | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 599216 B, not read (M10) | – | – | – | 17 of 17 | 0 |
| SX_F | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 477744 B, not read (M10) | – | – | – | 16 of 16 | 0 |
| SX_G | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 553872 B, not read (M10) | – | – | – | 15 of 15 | 0 |
| SX_H | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 244608 B, not read (M10) | – | – | – | 15 of 15 | 0 |
| SX_I | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 336176 B, not read (M10) | – | – | – | 19 of 19 | 0 |
| SX_J | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 336960 B, not read (M10) | – | – | – | 17 of 17 | 0 |
| SX_K | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 468368 B, not read (M10) | – | – | – | 20 of 20 | 0 |
| SX_L | 1 | 16×16 | 3.5 | 896 | 257 | 16 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 322992 B, not read (M10) | – | – | – | 20 of 20 | 0 |
| TR_A | 2 | 80×80 | 4.5 | 5760 | 794 | 48 × 256×256 fmt 0x27, 1 mip, 65560 B † | BPB, 1616 B, not read (M10) | – | – | – | 0 of 0 | 0 |

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

- No `.bpb` decoded. The record layout above is read off one hexdump, so no Untamed scene key, model or
  placement is counted.
- The `.res` view's directory is the reader's own, but why the game pads it (alignment? a reserved
  table?) was not looked up in the ELF.
- No `.psg` or texture parsed. The texture pages were walked, not decoded.
- Heights are read, not checked against the running game (T7). The 3.5 vs 4.5 spacing comes from the
  terrain header, not from a scene.
