# M24: Untamed's trick gestures on the PS2

M20 follow-up: does the Classic mapping's `1` (MX vs. ATV's R1 bit, SLUS_211.04 `0x267da0`) mean R1 on
Untamed? Yes, through the modifier. SLUS_217.01 `0x3a3220` decodes `1` into the row's STUNT_MOD flag
(+0x11), and the matcher `0x3a37d0` compares it with key-set byte +0x18, the byte R1 sets on the first
PS2 preset. So the R1 rows of the gesture grid in notes/tricks.md hold for Untamed. Captured in PCSX2:
R1 + Circle + up plays KOD and lands clean. Write-up in notes/tricks.md ("The gesture map", the Untamed
paragraph, and "Untamed's pose tricks in PCSX2 (M24)"); one ask in notes/openmx-asks.md.

## Log

- **Finding the decoder.** The string `DefaultKeyMappingClassic` (0x3c0dbc) is read only by the stunt
  loader `0x251e60`. It reads the key into a 16-byte buffer (default 0x3c0cbc, empty) and hands it to
  `0x3a3220`. The three Wii styles go to `0x3a3370`/`0x3a3530`/`0x3a3680`. `0x3a3220` sets +0xe for
  `C`, +0xf for `T` and +0x11 for `1`, and puts the index of `H U D L R` (table 0x3d45f8) in word +0.
  Anything else asserts. `0x3a3160`, the mapped-key decoder, names the same four bytes `1 2 3 M`, which
  makes +0x11 the M (modifier) flag.
- **The matcher.** `0x4fcad0` (the air step) calls `0x252590` with the key set at bike +0x14f4. That
  function tries each of the 54 rows (stride 0x58) with `0x3a37d0`. Classic is case 3: key-set word +4
  equals the row's direction, +0x12 == C, +0x14 == T, +0x18 == the `1` flag. `0x3a2ca0` first requires a
  direction (word +4 non-zero). STUNT3 (+0x16) is never compared.
- **The row table** is `import-tricks table untamed`: rows 13–24 carry `1CU` … `1CTR`, the same grid as
  MX vs. ATV.
- **Capture setup.** This slot had only a boot-ELF Ghidra project. I copied slot s1's whole
  `extracted/untamed/` set (overlay decomp, `watch/`, kit.toml) to `extracted/u24/`, rewriting the paths
  to s2, and left s1 alone. I wrote `extracted/u24/watch/gesture.py`, a closed-loop driver on M20's
  `air360.py` sampler: gas, then 8 frames into the air past the first big lip it presses the buttons
  and the stick, holds them 60 frames and lets go. It logs the key set, +0x1548 and the stunt clock.
- **What failed first:** `kit pcsx2 start` hung silently under the Bash sandbox (no PCSX2 process, no
  log). It worked with the sandbox off (`dangerouslyDisableSandbox`). The kit pad refuses `select`
  (PCSX2 hotkeys), so the run meant to show that STUNT3 (Select on preset 1) is ignored sent nothing
  extra. That bike wrecked off the lip (reason 1) and the run proves nothing.
- **Captures** (CSVs in `extracted/u24/capture/`): `kod` and `kod2` (R1+Circle+up gave +0x1548 = 12,
  KOD, a clean landing; the two runs were the same frame for frame), `shaolin` (R1+Triangle+left gave
  18, Shaolin), `taunt` (Circle+up gave 0, Taunt). The last two had a shorter jump (100 frames of air)
  and wrecked on landing with reason 0xc, with the stunt clock short of SafeTime and Duration.
- `./check.sh`: passed (run by `agent-runner done`).

## Not verified

- That STUNT3 / Select does not change a gesture: read only from `0x3a37d0`, not captured. The kit pad
  refuses Select on purpose, and writing the key-set byte over PINE would not have been the game's own
  input path.
- Presets 2–6 (L1 or Square as the modifier): read from the disc's .ini files in M20, not played. Only
  preset 1, the default, was driven.
- R1+Circle+Triangle rows and the down/right gestures: decoded, not captured (one gesture was asked
  for; I captured three rows).
- Whether the 0xc landing wreck is cut at SafeTime or Duration: both landings were short of both, so
  the two were not told apart. The clause that raises 0xc was not traced in the decompilation.
- The later reason-1 wrecks in `kod`/`kod2` (frames 662/672, ~3 s after landing, the bike hitting
  something on Rocksbury's track with nobody steering): not looked at, as in M20's `spin1`.
- Points or HUD text for the tricks: not read.
