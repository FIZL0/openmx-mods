# MX Unleashed's terrain limits

The question (T3): can MX Unleashed's own loader and terrain query take what the other games ship,
which is 80×80 leaves at 2 levels (6400 against free06's 2304), 4.5 ft spacing, `TerrainWidth=5`, and a
non-empty texture table between the blocks and the tree?

**Yes. MX Unleashed already ships that exact shape.** Its 22 national tracks (`tracks\nat01` … `nat22`)
are 2-level PLN v7 terrains with 6400 leaves over root cells 0..4 (`TerrainWidth=5`,
`TerrainBreadth=5`), 4.5 ft a sample (4.0 on nat06, nat08 and nat15), 538 blocks (794 on nat09 and nat18),
and 32 or 48 texture pages between the last block and the tree. Its 24 supercross tracks carry 16 pages
the same way. The tables below are from MXU's own disc, read with openmx's PSR and `.res` readers and
walked node by node from the header's tree offset (a scratch tool that was not kept; see the journal).

| tracks | levels | leaves | root cells | spacing (file and scene) | blocks | pages in the gap | fog end |
|---|---|---|---|---|---|---|---|
| free06, 07, 09, 10, 11 | 2 | 2304 | 3×3 | 9.0 | 842 | 0 | 768–1000 |
| nat01–22 (not 06, 08, 09, 15, 18) | 2 | 6400 | 5×5 | 4.5 | 538 | 32 | 768 |
| nat09, nat18 | 2 | 6400 | 5×5 | 4.5 | 794 | 48 | 768 |
| nat06, nat15 | 2 | 6400 | 5×5 | 4.0 | 538 | 32 | 768 |
| nat08 | 2 | 6400 | 5×5 | 4.0 | 538 | 32 | 450 (Visibility 0.5) |
| sx01–24 | 1 | 256 | 1×1 | 3.0 | 257 | 16 | 1700–2000 |

Gap = tree offset − end of blocks: exactly 16, 32 or 48 × 65,560 on every one. Each page record starts
`0x27, 1, 256, 256, 65536`: format 0x27, one mip, 256×256, 65,536 bytes of pixels.

So the MX vs. ATV big maps (`notes/mxvatv.md`: 80×80 leaves at 4.5, 794–1050 blocks, 16 pages) are
inside what MXU's code already runs, except that they have more blocks (1050 against 794). Block count has
no fixed cap in the code (below). The code-side reading follows. Every address is in `SLUS_207.67` and can
be read with `./kit fn` / `./kit dis` from `openmx/`.

## The `.trn` loader, `0x001d9930`

Called from `0x00205410` and `0x001a5070` as `(terrain, path, 1, 8, "<name>_d.tga", 0, 0, 0,
DetailMapTilingMultiplier, …, players, …)`. **It is never handed TerrainWidth, TerrainBreadth or either
scale.** The terrain's extent comes from its own tree: the bounds at terrain +0x30/+0x40 are grown by the
node loader and then multiplied by the file's scale (`0x001da8c8` onwards).

It reads the file **sequentially** and never seeks to the header's tree offset:

| order | what | where it goes | limit |
|---|---|---|---|
| tag, version `'7'`, tag, name (32), palette flag (+1024) | | +0x70 palette | |
| scale | f32 | +0x80 | none |
| levels | i32 | stack, handed to the root | see the tree |
| tree offset | i32 | stack, **never used** | |
| block count A | i32 | +0x770 | heap array `A*4` (`0x001da660`), no cap |
| block count B | i32 | stack | B blocks are loaded into the A-slot array: **B ≤ A**, or the heap is overrun |
| A words | skipped | | |
| texture count C | i32 | +0x55c | |
| texture count D | i32 | stack | D pages are loaded |
| C words (page offsets) | skipped | | |
| B blocks | 2129 bytes each | 0x7b0-byte objects (`0x001d0f30`) | none |
| D pages | see below | `sw $20, 0x560($18)` at `0x001da828`: terrain +0x560 + 4i | **128 pages**: +0x560..+0x75f is the whole array. Page 128 would overwrite +0x760, the PLN/LOD switch. No check. |
| the root node | 97 bytes | `0x001d1550` at `0x001da8c8` | must start where the last page ends |

A page is read by the texture object's load (`0x00187f80`, vtable 0x0044fad0 +0x28), after the terrain
loader has read its format word: a mip count, then for each mip a 16-byte header (`0x00188160`) and its
pixels. Mips go into slots at texture +0xcc, 0x3c apart (`addiu $19, $19, -60` at `0x001880a0`).
Slot 6 would be at +0x234, which is where the palette pointer is kept (`0x0018804c`), so **at most 6
mips**. Every shipped page has 1. One one-mip 256×256 8-bit page is 4 + 4 + 16 + 65,536 = **65,560
bytes**, which is the 65,560 spacing seen on every disc.

A PLN block (`0x001d0f30`): base word (4), heights 680 → +0x15c, normals 867 → +0x430, then **289 bytes
skipped** (`addiu $6, $zero, 289` / `jal 0x00124580` at `0x001d0fb0`), then surfaces 289 → +4. On the
PLN path the game never reads the 289-byte array that game-sim calls `Block::unread`. A writer can
fill it with anything.

The loader also ends with `+0x78c = min(DetailMapTilingMultiplier × 64, 360)` (`slti $1, $2, 360` at
`0x001daa88`). That is a detail-texture setting and not a grid limit.

## The tree, `0x001d1550` / `0x001d1cb0`

A node stores its level (+0x24), its shift (+0x25, 4 × level) and its corner as two **u16** (`sh` at
`0x001d1d94` / `0x001d1d98`), in samples: a child at slot i is at `(parent + (i & 15)) × 16`. Its block
index is a **u16** (+0x1e). Children are reached by absolute seeks to the 256-entry offset table and to
each child. There is no global node count: each node is a 0x80 heap object.

- Root cells per side: the descent takes `index = coord >> shift` and refuses `index > 15` (`slti $1,
  $4, 16` / `slti $1, $3, 16` at `0x001ddb24` / `0x001ddb30`). So **16 root cells a side**, which is 4096
  samples at 2 levels. TerrainWidth 5 is 1280.
- Corner coordinates and the cell cache key are 16-bit, so at most 65,535 samples a side.
- Blocks: at most 65,536 (u16 index).

The 16-word table of a node goes into its own 0x90-byte heap block (`0x001d1cb0`), again with no cap.
`0x00105990(0x14, 0x500)` (the pool at 0x0049d070) looks like a cap but belongs to the LOD format only.
What uses it (`0x001d1b90`, the LOD callbacks at `0x001d2440`/`0x001d25f0`, and `0x001d2900` →
`0x001d2fd0`) is reached only when terrain +0x760 == 0 (`0x001d1cb0`, `0x001d3a84`), and PLN is 1.

## The query: `terrain_point_query` `0x001dce80`, `terrain_cell_read` `0x001ddaf0`

- World to sample: `ix = (int)(x × terrain+0x778)`, clamped at 0 below. The loader sets +0x778 to
  `1 / (2^(shift − 4·levels) × scale)` = `1 / scale`, because the root's shift is always 4 × levels. No
  grid size enters.
- The cell cache: `DAT_00420870` (10) entries of 0x70 at 0x0049f540, keyed `ix << 16 | iz` (`sll $3,
  $19, 16` / `or` at `0x001dcf68`). This is 16 bits each way, as above.
- Off the tree: an index over 15 at any level, or an empty child slot, is a miss. The query then hands
  back `(0, 1, 0)` and surface 0.

Nothing in either function is sized from TerrainWidth or the leaf count.

## The height band

y_world = **scale (terrain +0x80) × the leaf's `height_scale` (node +0x10)** × raw word, or ×
`(base + vertical_scale × (raw − base))` when `vertical_scale ≠ 1`. `terrain_cell_read` applies
`height_scale` and `vertical_scale` (node +0x0c), and `terrain_point_query` multiplies the four heights by
+0x80 (`vmulbc` after the read). This is game-sim's `TerrainNode::height_scale` with
`Terrain::scale`, which D6 proved on free06.

On every MXU terrain above, every node has the same `height_scale`, so the band is one number a track.
As `1 / (scale × height_scale)` raw units per foot: free06 170.6, free07/09/11 243.9, free10 235.5, nat
52.2 (nat17) to 380.3 (nat21), sx01 485.4, sx02 631.1, sx03–24 556.6–643.4.

game-render's `terrain.rs` says the band is "not known" and fits it from the props: 1/540 on "the first
supercross track" and 1/627 on "the second". sx02's own number is 631.1, within 0.7% of its fit. sx01's
is 485.4, 11% away from 540. Either the fit or my guess at which track was "first" is off (not checked).
The band is in the file: it does not need a fit.

game-render's header map is also off. What it reads at 1073/1077 is levels and the tree offset (its "u32
1, u32 size"), and at 1081/1085 the two block counts (its "columns, rows"). These happen to be 257 on a
supercross track, but they are 538 on a national, where it would read a 538×538 field. Only
`first-draw` uses that reader. The shell draws from game-sim's `Terrain`.

## The scene's numbers, and what reads them

`0x001a7b30` reads `[Environment]` into the level's scene record (level +0x3b4), as floats. Scene +0x530
TerrainWidth, +0x534 TerrainBreadth, +0x538 TerrainWidthScale, +0x53c TerrainBreadthScale, +0x540
TerrainBorderWidth, +0x544 DetailMapTilingMultiplier (int, default 8), +0x548..+0x554 Xmin, Xmax, Zmin,
Zmax (default `−border` and `border + width × scale × 256`), +0x558 MinDivisionSize (96). No clamp on
any of them. I scanned the whole ELF for loads of +0x530..+0x568 within 12 instructions of loading the pointer
at +0x3b4. It finds four readers. A reader that keeps the pointer longer would be missed:

| reader | reads | does |
|---|---|---|
| `0x001a6e50` (edge query; the biplane, bike and others call it) | width, breadth, both scales | W = width × 256 × widthScale, D likewise; outside `[m, W−m]` is off the map. Floats, no cap: 5 × 256 × 4.5 = 5760. |
| `radar_init` `0x0035ce10` (at `0x0035d000`..`0x0035d03c`) | **the two scales only** | radar +0x80 = max(192 × widthScale, 192 × breadthScale). On a 4.5 ft map the radar covers 864 ft, against 1728 on free06. It zooms with the spacing and ignores the map's size. |
| `0x002ad180` (debug readout "x= %d, y= %d", gated on a debug mode < 3) | the two scales | fmod(pos, 256 × scale) × 960 / (256 × scale). Debug only. |
| `0x001a2c30` (level load) | Xmin..Zmax | the default bounds of the scene's 0x80-byte records at +0x350 whose own box is empty |

The oracle's `radar_update 0x0035d3e0 … max(terrain+0x538, +0x53c) * 192` is the same number, computed in
`radar_init` and kept at radar +0x80. The "terrain" there is the scene record (`+0x3b4` of the level).

## Drawing: one real cap

Each frame the frustum walk (`0x001d4cc0`, reset of the count in `0x001d4c10`) hands PLN **leaves**
to `0x001d51d0`. A leaf nearer than the fog end (display +0x4c, which `renderer_set_fog` sets) is
appended to a list at terrain +0x794 (16,000 bytes allocated at `0x001da98c`), count +0x790:

```
001d5270  lw    $4, 0x790($6)
001d5274  slti  $1, $4, 4000      ; full: the leaf is silently not drawn
```

(again at `0x001d5374`). **At most 4000 leaves are drawn a frame.** free06 has 2304 in all, so this
cannot bind there. An 80×80 map has 6400, so it binds only if more than 4000 of them are both in the
frustum and nearer than the fog end. At 4.5 ft a leaf is 72 ft. A square bound puts that at a fog end of
at least about 2200 ft, a disc estimate at about 2570 ft. Every national's fog ends at 768 or less, which
is at most (2·768/72 + 2)² ≈ 545 leaves even before the frustum. It matters only for a mod whose
scene sets the fog past about 2200 ft. The port draws from its own mesh and has no such list.

## What a writer (T2) has to respect

- 2 levels, at most 16 root cells a side (TerrainWidth/Breadth ≤ 16). 5 is fine.
- Block count A ≥ B. Write them equal, as every disc does. At most 65,536 blocks.
- At most 128 texture pages, each at most 6 mips. Pages are format word, mip count, then per mip a
  16-byte header and pixels. **Write the root node immediately after the last page.** The game ignores
  the header's tree offset, so a file whose offset and actual position disagree loads wrongly in the
  game while openmx (which seeks to the offset) reads it fine. Write the offset anyway, equal to the
  stream position.
- The 289-byte array per block is never read by the PLN loader.
- The scene's TerrainWidth × 256 × TerrainWidthScale is the map edge (`0x001a6e50`) and should equal the
  tree's extent × the file's scale. TerrainWidthScale is also the radar's zoom, so it should equal the
  file's scale.
