# T3: MX Unleashed's terrain limits

Result: `notes/mxu-terrain-limits.md`. Port asks: `notes/openmx-asks.md` (under *Open*: MD1 evidence and
four MD3 items).

## What was done

Everything was read from openmx's Ghidra project with `./kit fn`, `./kit dis`, `./kit xref` and
`./kit data`, run read-only from `openmx/`.

1. **The loader, `0x001d9930`.** I read it whole as a decompile (`./kit fn`) and followed its callers'
   stack arguments in the disassembly (`0x002055a8..0x002055dc`, `0x001a5194..0x001a51cc`). It is
   handed DetailMapTilingMultiplier and the player count, never TerrainWidth. It reads the file
   sequentially, so the root has to follow the last texture page. The page array is fixed at
   +0x560..+0x75f (128 slots). The draw list is 16,000 bytes.
2. **Where TerrainWidth lands.** I found the `.scn` `[Environment]` reader (`0x001a7b30`) through the
   `TerrainWidth` string and mapped scene +0x530..+0x568. A scan of a whole-ELF disassembly (scratch,
   not kept) for loads at those offsets right after loading the +0x3b4 pointer gave four readers: edge,
   radar_init, a debug readout and the level load.
3. **The radar.** The oracle names `radar_update`, but the ×192 is not in it. The same scan found it in
   `radar_init` at `0x0035d000`: max of the two *scales* × 192, stored at radar +0x80.
4. **The tree** (`0x001d1550`, `0x001d1cb0`), **the query** (`0x001dce80`, `0x001ddaf0`) and **the draw
   walk** (`0x001d4c10`, `0x001d4cc0`, `0x001d51d0`). The only cap is the 4000-leaf list at
   `0x001d5274`/`0x001d5374`, cut at the fog end (display +0x4c). The descent refuses an index over 15
   per level. The pool at 0x0049d070 and its users are LOD-format only.
5. **Texture pages** (`0x00187f80`, `0x00188160`): a mip count, then per mip a 16-byte header and the
   pixels. Mip slots at +0xcc stride 0x3c run into the palette pointer at +0x234 after 6. One page is
   4 + 4 + 16 + 65,536 = 65,560 bytes.
6. **Checked on the disc.** I wrote a throwaway crate in the session scratchpad (not committed; it
   depended on `openmx/crates/game-render` (`disc::Disc`) and `game-sim`). It opened `openmx/MX Unleashed
   (USA).iso` and walked every `tracks\*.res` terrain from the header's tree offset by hand. It counted
   leaves and root cells, measured the gap between blocks and tree, decoded the first page record, and
   collected every node's `height_scale`. A second binary printed each `.scn`'s TerrainWidth/Scale and
   fog. That is where the table in the note comes from. The headline: **MXU's own nat01–22 are
   80×80 leaves at 4.5 ft (4.0 on three) with 32–48 pages in the gap**, and sx01–24 have 16. So the
   question "can MXU take this shape" is answered by MXU shipping it. It also shows that MD1's
   `Terrain::parse` refusal hits 46 of MXU's own tracks, not only the other games'.

## What did not work, or was corrected

- `./kit xref 0x49d070` found no references (the global is reached through `lui`/offset pairs). I
  grepped the disassembly for `-12176(` instead.
- The decompile of `radar_update` has no 0x538 in it. The oracle's line describes a value computed in
  `radar_init`. I wrote the note against the instruction that computes it.
- The first draft of the note said the 0x90 node tables were LOD-only. The 16-word table's 0x90 block
  in `0x001d1cb0` is allocated on any format (just with no cap), so I corrected it.
- The brief lists `research/oracle.txt` as a file this task creates. That file is openmx's
  (`openmx/research/oracle.txt`), and AGENTS.md forbids editing openmx from here, so I did not create
  or change it. Names the oracle lacks (`0x001d9930` the `.trn` loader, `0x001d1550`/`0x001d1cb0` the
  node loader, `0x001d51d0` the leaf draw list, `0x001a6e50` the edge query, `0x001a7b30` the
  `[Environment]` reader, `0x00187f80` the texture load) are in the note for a human to carry over.

## Not verified

- **Nothing was run.** No limit was exercised in the emulator or with `./kit oracle`. Every limit is
  read from the decompile and disassembly. In particular, the 4000-leaf drop and the 128-page/6-mip
  overruns were not provoked.
- That `0x00124580(file, n, 1)` is a relative skip and `(file, off, 0)` an absolute seek is inferred
  from how the loader and the node loader use it, and from game-sim's reader agreeing with that layout.
  I did not read the function.
- The 289-byte array being unread holds for the PLN loader `0x001d0f30` only. I did not check whether
  anything else re-reads the file.
- The MX vs. ATV discs were not opened. Their fog distances, and whether any page there has more than
  one mip, are unchecked, so whether the 4000-leaf cap could bind on one of those maps in the original
  is open. The images live in the main checkout's `iso/`, which this worktree must not touch.
- The heap cost of a 1050-block map in the original (≈ 1050 × 0x7b0 bytes of blocks against nat09's
  794) is an estimate. The PS2's free heap on a national track was not measured.
- Which track game-render's "first" and "second supercross track" fits (1/540, 1/627) were taken on is
  assumed to be sx01 and sx02. sx02 agrees with the file's band to 0.7%. sx01 is 11% away, which
  was not explained.
- The ELF scan for scene-field readers only catches a load within 12 instructions of loading the +0x3b4
  pointer. A reader holding the pointer longer would be missed.
- The asks' line numbers (`main.rs:250`, `:2768`, `web.rs:71`) are from openmx's checkout as of this
  session.
