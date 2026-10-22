# T8: TerrainWidth in the standard

Standard wording only; no code changed.

## What I did

- §2a's table: `TerrainWidth`/`TerrainBreadth` now say "256-sample units a side", that this is the
  root-cell count only on a two-level tree (MX vs. ATV's SX and FS maps: one level, 16 root cells of 16
  samples, `TerrainWidth=1`), and the rule is "equal to the terrain's samples a side / 256".
- §2a: a new paragraph on a disc scene that disagrees with its own terrain (Nat04, SX07, MC08, SX18,
  numbers from journal/2026-10-22-t4-build and notes/mxvatv.md). The scene is copied unedited; the mod
  names each key in `todo` with the scene file and the key; the check fails without it and notes it
  with it. Generated maps (T2) get no such allowance.
- §5 step 2: the extent check reworded to samples and scale, with the `todo` exception.

This is what `tools/modcheck`'s `layout` (`tools/modcheck/src/lib.rs`) already does; I read it to make
the words match: samples = root cells × 16^levels, scale by equality, a `todo` entry matched by
case-folded scene stem and key.

Nat04's example: notes/mxvatv.md gives 56×80 leaves on 2 levels, so 4×5 root cells; `TerrainBreadth=5`
agrees (1280 samples), `TerrainWidth=5` does not (4 root cells, 1024 samples by modcheck's count).

## Not verified

- What the game does with a disagreeing scene (edge and radar from the scene, ground from the tree) is
  stated as T7's to see; nobody has run those maps in PCSX2 or looked it up in the ELF.
- I did not rerun modcheck on the real disc: no code changed, and T4 did that run.
