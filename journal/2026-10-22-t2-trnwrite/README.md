# T2: tools/trnwrite

The shared writer: a height grid (plus optional texture pages) → a PLN v7 `<stem>.ps2.trn` and a
`<stem>.scn`. It also decodes and rewrites any disc terrain.

## What it does

```
trnwrite build STEM OUTDIR --size W,D (--feet FILE | --raw FILE --height-scale H) --scale FEET
               [--surfaces FILE] [--normals FILE] [--palette FILE] [--page FILE]...
               [--page-map FILE --tiles-x N] [--start X,Y,Z] [--direction X,Y,Z]
trnwrite check FILE.trn
trnwrite rewrite IN.trn OUT.trn
```

The library (`tools/trnwrite/src/lib.rs`):

- **`Trn`/`Node`/`Block`/`Page`** hold the file decoded. Each node holds its own block's contents
  (not an index), so two terrains compare with `==`.
- **`write`** lays out the header, the A block offsets, the C page offsets, the blocks, the pages, then
  the tree in preorder (node, its 256-word child table, its children). This is the order of every disc
  file. Blocks are shared when byte-identical, numbered in the order the preorder walk first meets them.
  The header's tree offset is the root's real position (T3: the game reads in sequence, openmx seeks).
  `check` enforces T3's limits: 1–2 levels, ≤ 128 pages, 1–6 mips, ≤ 65,536 blocks, page indices in
  range, a name under 32 bytes.
- **`read`** walks the tree with openmx's `Terrain` (`root`, `child`, `node`). It takes the palette,
  the pages and the raw block bytes (openmx's `Block` masks the surface's top bit and has no raw
  accessor) from the offsets `header` finds.
- **`openmx_view`** is the MD1 workaround. Today `Terrain::parse` refuses any file with pages
  (`notes/openmx-asks.md` MD1). The view turns the page offsets and the pages into extra block-offset
  words in front of the blocks, which the reader skips. The blocks then end at the tree again, and the
  tree and every offset in it stay where they were. A file with no pages is passed through unchanged.
  Delete the view once MD1 lands.
- **`build`** cuts a grid into 17×17 leaves 16 samples apart. It uses 1 level for ≤ 257 samples a side,
  else 2 (at most 16 root cells, 4097 samples). Samples past the grid repeat the edge.
- **`scene::SceneText`** writes the scene's *must* keys (standard §2a) in MXU's spelling and number
  format, CRLF. TerrainWidth/Breadth come from the tree's occupied root cells. Both scales equal the
  file's. Border 0 and MinDivisionSize 96 come from free06, nat01 and sx01. DetailMapTilingMultiplier=8
  is the game's default.

## What the discs showed (survey of all 162 terrains, scratch Python, not kept)

These findings are why the model looks the way it does. The scripts lived in the session scratchpad.

- Every file is `PLN\0` `7` `ps2\0`, named `UnCompressedPSXPal8`, with a palette (flag 1). Levels are 1
  (sx, FS, MC06–08, SX) or 2. A = B. The A words are the block offsets, the C words the page offsets,
  and C = D.
- **Nodes above the leaves have blocks too.** free06's 842 = 1 root + 9 level-1 + 832 leaf blocks. An
  upper block samples the leaves' effective heights every 16 (or 256) samples, rounded, with base 0.
  Normals and the unread byte subsample the same way (44,601 of 45,184 normals match). Surface bytes
  do not (about half differ). `build` subsamples all four. The PLN query and draw that T3 read only use
  leaves.
- **The disc does not share all identical blocks.** It shares by *flattened reuse*: a leaf points at
  another leaf's block with `vertical_scale` < 1. That is lossy, and the original tool chose it. free06
  has 842 blocks of which only 527 are byte-distinct. The writer shares exact copies only (the
  accuracy rule), so rewriting gives 527 on free06. Over the three discs: 27,778 → 23,816, 29,974 →
  27,659, 29,127 → 26,947.
- **Leaves do not always agree on their shared edge.** On free06, 2,723 of 76,704 edge samples differ
  in effective height, 3,822 in normal, 2,724 in surface. That is why the model is per node and not one
  global grid: a single grid could not reproduce a disc map's cells.
- **The node's 16-word table is texture pages** (decompile of `0x001d1cb0`: each word `& 0xfffe`,
  0xfffe → −1, else `>> 1`; a node with any page gets a 0x90 block at node +0x3c). On MXU's nationals
  the level-1 node of each root cell holds 16 consecutive pages. On sx01 the root holds 0..15. Untamed
  writes 0 instead of "none" on its other nodes. So each entry is the page under one 64×64-sample
  quarter of a quarter of a 256-sample node. `build` puts the page map on the node over each root cell,
  as MXU does.
- The other node words: the half-word before the block index is read to node +0x1c, and its only use
  in `0x001d0000..0x001de800` is the LOD branch (`0x001d52c4`, after `+0x760 != PLN`). The word after
  the block index is masked and dropped (stack local). The word after the table is read and dropped.
  The flag byte is 0x27 on every disc node. `write` keeps the half-word and the flag. It writes 0xfffe
  and 0 for the dropped words, which are the values on every disc node (low half).
- Page mips: `(256, 256, 65536, X)` with X either 0 or what looks like a leftover pointer
  (0x40815c, …). `Page::psmt8` writes 0.
- Bounds: a full-relief leaf's min_y/max_y is exactly the range of `TerrainNode::height` over its
  block (36,928 of 36,928 in the sample). An upper node's is the range of its children (153 of 195
  exactly). `build` computes both that way.
- Normals: the original tool's formula is unknown. Central differences scaled to 127 and truncated
  match the disc's three bytes at 67% of free06's interior full-relief samples (51% rounded, the same
  for averaged face normals). The mean worst-axis error is about 1. `normal()` uses that, and only for
  grids that come without normals.

## How it was proved

- `cargo test -p trnwrite` (`tests/grids.rs`, 8 tests, in `./check.sh`). Generated grids are built,
  written, and parsed by openmx's `Terrain` (through the view when there are pages). Every cell at
  steps 1 and 2 over every leaf matches the grid's height (`hs × raw`, bit for bit), its normal and its
  surface id. A cell where `cell` must refuse is checked as None. Also covered: one and two levels with
  partial leaves (70×45, 300×290), decoding back gives an equal `Trn`, and writing again gives the same
  bytes. A flat 5×5-leaf grid makes 2 blocks and a bumpy one 26. Pages sit between the blocks and the
  tree, the offset table names each record, and raw `Terrain::parse` refuses the file (MD1). The page
  map lands on the root of a one-level map. The tests also cover the refusals (129 pages, a page index
  out of range, 4098 samples, a 32-byte name), 4097 samples being accepted, feet quantisation, and the
  scene read back by game-sim's `Scene::parse` with matching layout and scales.
- Mutation checks, each reverted afterwards: transposing heights in both `to_bytes` and `from_bytes`
  failed 3 tests, swapping x/z in the child order failed 4, and shifting child offsets by 4 failed 5.
- **The disc round trip** (`tests/disc.rs`, `--ignored`) ran with
  `DISCX_ISO=<main checkout>/iso cargo test -p trnwrite --release --test disc -- --ignored --nocapture`
  in 250 s. Every `.trn` on the three discs (MXU 57 in `Tracks\*.res`, MX vs. ATV 56 in `Env\*\*.res`,
  Untamed 49 loose in the PSR, 162 in all) went read → write → read. For each one:
  1. The decoded `Trn` is equal.
  2. Every cell at steps 1 and 2 over the occupied root cells, plus one sample past them, is equal.
     That is 1,108,224 cells on free06 and 3,078,400 on each national.
  3. Every leaf's block bytes are equal.
  4. Pages and palette are equal.
  5. The header's tree offset is where the tree is.
- CLI by hand: `build` on a 300×260 f32 grid wrote 2 levels, 323 leaves and a CRLF scene. `check` read
  it. `rewrite` of MXU nat01 gave 524 blocks, 32 pages, and openmx read it through the view.
- `./check.sh` passes (16 tests).

## What failed on the way

- The first survey script read the node's table 4 bytes late (a misread of the 97-byte layout), so
  every child-table offset looked wrong. Fixed against game-sim's `Terrain::node`.
- The first disc run iterated 4096×4096 samples per two-level terrain (the whole 16×16 root), and its
  "skip an empty root cell" test skipped only one sample. It was too slow, so I stopped it and limited
  the walk to the occupied root cells plus one sample past them.
- `pkill -f` on the test's command line also matched my own shell, so one edit had to be redone.

## Decisions

- **Per-node model, not one grid.** Reason: the disc's leaves disagree at edges and flatten shared
  blocks, so "same cells" for disc maps is only possible per leaf. `build` is the grid path.
- **Exact sharing only.** The disc's flattened reuse is lossy. The brief's "share identical leaf blocks"
  is what the writer does, so it writes fewer blocks than the disc has.
- **A palette is always written** (a grey ramp when none is given). Every disc terrain has one, and
  whether the loader takes flag 0 was not checked.
- **`DISC_NAME`** (`UnCompressedPSXPal8`) is the default name for the same reason.
- Nothing was added to `notes/openmx-asks.md`. The one refusal met is MD1, which is filed and has
  T3's evidence; the view is this task's stand-in for it.

## Not verified

- **Nothing was loaded in the game or PCSX2.** Whether MXU's loader and draw take a `build` terrain is
  unproven, in particular the upper-node blocks (subsampled; the disc's surfaces there differ), the
  bounds of upper nodes (153 of 195 match the disc rule), `lod` = 0, and the grey palette. That is T7's
  job.
- **The page table's quadrant order** (`i = qx + 4 qz`) and the pixel orientation within a page are
  assumed by analogy with the child table. The disc only shows consecutive runs, which fit either order.
  Nothing that draws pages (game or port, MD4) was read.
- A palette flag of 0 (no palette) is written by `write` when a `Trn` has none. No disc file has that,
  and the game was not checked with it. `build` never produces it.
- The normal formula for generated grids is an approximation (67% exact on free06). The tool's real
  formula was not found.
- The disc round trip compares openmx's readings of two files. It cannot catch a field that openmx and
  this crate both ignore. That covers the words the game drops, which `write` sets to the disc's values
  (not the disc's leftover high halves), and the three unused height columns per row (0 on every disc
  block, checked by the survey's `pads` count on three files only).
- The disc round trip ran from this worktree against the main checkout's `iso/` (read-only, via
  `DISCX_ISO`). The test's default `iso/` path inside a checkout was not exercised.
- The page survey covers every page header on the three discs (4,700 records). The upper-block and
  bounds statistics come from a sample of 16–20 files (every 8th or 10th), not all 162.
