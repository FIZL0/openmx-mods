# T6: tools/survey

`survey DISC` reads every map on a disc through openmx's readers and prints one markdown row per map:
levels, leaves, spacing, side, blocks, texture pages, scene name, the scene's terrain keys, models,
collision placements (with the `.col` they name: found in the map, found loose, or missing), and the
`.col` read and refused. Then it prints every refused `.col` with the reader's reason, and every `.col` loose
on the PSR by folder. `survey res FILE.res` does one archive on disk. The tables are in
`notes/mxvatv.md` (56 maps) and `notes/mxvatv-later.md` (49 maps), pasted unedited. Each note has a
summary on top.

## What it reads, and from where

- MX vs. ATV Unleashed: `Env\<MAP>\<map>.res`. MX Unleashed: `Tracks\*.res`, which also works (56 tracks)
  but goes into no note. Untamed: `res\env\<MAP>\<MAP>.res`, plus SM_IP, whose terrain is only loose.
  `_mat`, `_mst` and `_2` archives are skipped, and so is an archive with no `.trn`.
- Loose `.trn` already covered by an archive are not surveyed twice. On Untamed the loose and archived
  terrains are identical (an ignored test compares all 48).
- MX.PSR lists `Tracks\free06_ss.res` twice. It is surveyed once, and the output says so.

## The two views

The brief asked for this: until openmx MD1 lands, read the terrain's texture table here and pass
`Terrain::parse` a view without it. Because the tree's offsets are absolute, the tree cannot move. So the
view moves the blocks up against the tree, grows the skipped `declared` word array over the space they
leave, and zeroes the texture count and table. The survey walks the pages first, as the game's loader
does (format, mips, per mip a 16-byte header and its pixels). Each table entry must name the page the walk
is at, and the last page must end at the tree, so the view is only made when the gap is fully accounted
for. A file that `Terrain::parse` accepts as it is is read without the view, and the row's † drops.

Not in the brief: **`Res::parse` refuses all 48 Untamed map archives.** Their directory is zero-padded
past the last entry. `res_view` cuts the trailing zeros (longest cut first, at most 4 tries, since the
last entry's size can end in zeros) and keeps the cut that `Res::parse` then accepts in full. It reads no
directory entries itself. Filed in `notes/openmx-asks.md`, with more MD5 evidence: class-1 `.col` are
the animated objects, and MXU has 8 distinct ones, not 1.

## Findings that change the notes

- The page count is 16/32/48/52/64, not always 16 (the first survey looked at 1-level maps).
- Nat04, BT_E and SM_HS are 56×80 leaves. The "4480 leaves" in the old notes is that.
- 16 `.col` named by MX vs. ATV Unleashed placements are on no part of the disc.
- Scene keys that disagree with the file: MC08 and SX18 have `TerrainWidthScale=3` on a 3.5 file, SX07
  has `TerrainBreadth=2`, and Nat04 says 5×5.
- Untamed's `Col\FS_Panel_Blue01.col` is 0 bytes. Nat_OR's scale is 4.499999.

## How it was proved

- `tools/survey/tests/row.rs` builds a `.res` holding a PLN v7 terrain with 2 texture pages before the
  tree (3×2 leaves, 2 blocks), an INI scene with a model and four placements (two of one file in the
  archive, one loose, one missing), a mesh `.col` and a class-1 `.col`. The row must equal a written-down
  list of cells. The same archive with 260 bytes of directory padding gives the same row through the
  `.res` view. A unit-level test checks the view: the tree is unmoved, and every block reads its own
  heights. Another checks that a page table that disagrees with the walk is refused.
- Mutation checks: shifting the view's block copy by one byte fails the view test, and dropping one leaf
  fails the row tests. Both were restored.
- Ignored tests on the real discs (`DISCX_ISO=… cargo test -p survey --release -- --ignored`), read-only
  off the main checkout's `iso/`: every block of all 56 + 49 terrains reads through the view as the file's
  own bytes, with the tree unmoved, and Untamed's 48 loose terrains equal their archives'.
- Cross-checks with the first survey (2026-10-21): the same levels, spacing, sides and block ranges.
  `Nat07_WM_Blade.col` is refused for class 1 in both. The `.col` totals match `discx ls` (14,301 in
  archives; 1,643 loose on MX vs. ATV; 1,604 loose on Untamed).

## Not verified

- Nothing was checked against the running game (T7). The survey says what the readers accept and count,
  not that heights, placements or shapes are right.
- Untamed's scenes are BPB (M10), so its scene, model and placement columns are empty, and which shared
  `Col\` shapes a map uses is not known.
- The texture pages are walked, not decoded. Format `0x27` is taken as 8-bit indexed from T3, not
  re-checked.
- Why Untamed pads its `.res` directories, and what the game does with a placement whose `.col` is
  missing or with a scene key that disagrees with its terrain, was not looked up in either ELF.
- Whether MXU's nat01 scene places its class-1 `watertower`/`mainhouse` was not checked.
- The `game-sim` warning (`noise.rs:43`, an unneeded `mut`) is openmx's and was left alone.
