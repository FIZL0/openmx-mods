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

Every terrain parses with `Terrain::parse` once its "tree follows the blocks" check is relaxed (see
*Embedded textures* below). Leaves counted by walking `Terrain::leaf` over the whole sample range; y is
`node.height × file scale`, the range sampled one cell per leaf.

| maps | levels | leaves | spacing (ft) | side (ft) | blocks |
|---|---|---|---|---|---|
| MC01–05, MC09–10 | 2 | 80×80 | 4.5 | 5760 | 794 or 1050 |
| Nat01–17 | 2 | 80×80 (Nat04: 4480 leaves) | 4.5 | 5760 | 794–1050 |
| SM01–03 | 2 | 80×80 | 4.5 | 5760 | 538–794 |
| MC06–08, FS01–08, SX03–18 | 1 | 16×16 | 3.5 | 896 | 257 |
| SX01–02 | 1 | 16×16 | 5.0 | 1280 | 257 |

For comparison MX Unleashed's `free06` is 2 levels, 48×48 leaves at 9 ft (6912 ft), 842 blocks. So the big
maps here are *finer and slightly smaller* than MXU's freeride areas, and need more leaves (6400 vs 2304).
The scene says so too: `TerrainWidth=5` (5 × 16 leaves) against MXU's 3.

`MC` is the scenes' `SceneName=MachineChallenge0N`. Which maps the game offers as Freeride is in each
map's `.atyList` (not read yet) — that decides the first mods (M8).

## Embedded textures

The header's texture table (`textures`, `textures_again`, then that many words) is empty on MXU's
`free06` (0, 0). Here it holds 16 offsets, 65,560 bytes apart, pointing at texture data that sits
**between the last leaf block and the tree**. openmx's reader assumes the tree starts where the blocks
end and rejects the file; with `blocks_end <= tree_at` instead, everything else reads. 65,560 = 65,536 +
24: a 256×256 8-bit page with a 24-byte header, presumably on the header's PSX palette (`pal` = 1 here).
Not decoded yet (M9).

The same goes for every Untamed terrain (`notes/mxvatv-later.md`).

## Scenes and collision

`Scene::parse` reads FS01, MC01 and Nat07 unchanged (models 145–3582, collision placements 140–228).
`Collision::parse` reads 291 of the 292 `.col` in those three maps. The one it refuses is
`Nat07_WM_Blade.col`: **collision class 1**, a windmill blade — the animated-toy kind, which MXU's own
animated toys (openmx V43) will need too.

## Not verified

- `.psg` models, `.pst`/`.psm` textures, `_mat.res`, `.toy`, `.esb`, skies and particles were not run
  through openmx's game-render readers.
- Only three of the 56 scenes and their collision were parsed.
- The terrain heights were sampled, not checked against the running game.
- Whether MX Unleashed's own loader takes an 80×80-leaf terrain (T3) is open.
