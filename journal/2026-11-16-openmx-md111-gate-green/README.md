# openmx MD111: ./check.sh green against today's openmx

Two tests had gone red because openmx's readers got better, not because of anything here (full write-up
in openmx research/journal/2026-11-16-md111-import-music-end-flag).

- `tools/import-music`: openmx B1 (3f380b287) stops stereo `.vig` reads at the end-flagged pair, so the
  filler pair after it on MX vs. ATV Unleashed's songs (28 frames) is no longer played (MD7 done).
  `ReadBack` gains `pairs`; `problems()` drops the filler line and reports a read that stops anywhere but
  the end flag instead. `built.rs` and `disc.rs` expect reads to the flag and the T7 `todo` line alone.
- `tools/accuracy` `the_shell_loads_a_generated_mod_as_openmx_does`: openmx S320 (cdca4ffff) answers the
  field's last sample row and column, so the 2 strip points now equal the harness. The test expects 0.

## How it was proved

- `./check.sh` with `openmx` linked to openmx task/MD111 (main a143a26a5): 153 passed, 0 failed.
- `DISCX_ISO=~/repos/openmx-mods/iso cargo test -p import-music --test disc -- --ignored`: passes on
  both discs; 20 MX vs. ATV songs each 1 pair past the flag, 22 Untamed songs 0, every read at the flag.

## Not verified / not 1:1

- That the PS2 never plays the filler pair: B1's finding, not re-checked here.
- notes/accuracy.md's table (its † strip counts) and the built packs' mod.json `todo` under out/ were not
  regenerated; both still say what openmx did before B1/S320.
