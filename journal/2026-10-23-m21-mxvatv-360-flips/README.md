# M21: do MX vs. ATV's 360 and backflip run at once?

No, in either game. The findings are in `notes/tricks.md` "Both at once: no 360 backflip (M21)", at the
end of the file. One ask for openmx's MD20 is in `notes/openmx-asks.md`, after M19's backflip entry.

## How it went

1. **The two steps.** I decompiled SLUS_211.04's backflip (`0x4e4f20`) and 360 (`0x4e53d0`) with
   `kit fn` (`extracted/mxvatv/ghidra`, M19's project), and found their callers with `kit xref` and
   `kit dis`. Both callers (`0x4b9580`, `0x4d90f0`) run the backflip, then the 360, each substep. Each
   step's lip test refuses the lip when the other move is "applied" or "active", and the refusal clears
   its own arm. So both can arm together, but the first to fire locks the other out for the rest of the
   jump, and a same-frame tie goes to the backflip. Untamed's `0x4fd0d0`/`0x4fd630` are the same
   (`+0x1184/+0x118c` against `+0x11c0/+0x11c8`), and its callers `0x4dbbb0`, `0x4f10d0` have the same
   order. On the way: M19's note named +0x1194 as the backflip's "armed"; it is "applied". The note now
   says so in the new section; I left M19's line as it is.
2. **Wreck counter, MX vs. ATV.** I read the wreck call from `vtable+0x368` live (vtable 0x3d0450 →
   `0x4e2260`). It increments +0x6e4 and sets +0x16d4. The captures use +0x6e4 to tell a wreck from a
   clean landing.
3. **Capture driver.** `extracted/mxvatv/watch/both.py` (not in git; it holds the save states' RAM
   addresses) is M20's closed-loop pattern for both games: PINE plus the pad FIFO, gas and R1 held, the
   stick moved on the game's own flags (lip, armed, active). `both_sum.py` prints the frames where the
   flags change, the body-frame rotation in the air, path and net angles, up/forward extremes and the
   wrecks up to 1 s after touchdown. The moves are diag, diagx, spinflip, flipspin, spin, spinpull and
   flip, all 14 runs in PCSX2 kit copy 9: MX vs. ATV state 4, then Untamed state 8.
4. **What they show.** Diagonal → backflip; x first → 360. A second gesture during the first move's
   ground grace never arms. No frame in any run has both active or both applied. The 360 at the full
   flick is a cork about a roughly 45° axis: 366° and clean on Untamed's jump, 306° and a wreck on MX vs.
   ATV's shorter one. The stick held back in the air made MX vs. ATV's 360 complete 365° and land clean
   9° off its heading. On Untamed it over-rotated (436°) and wrecked.
5. **Strings.** The ELFs and both overlays name the 360 and the backflip only apart. The "combo" strings
   are the HUD's chain score.

## The gate

Without `DISCX_ISO`, `./check.sh` failed in accuracy's `every_map`, the known slot cause. With it, two
`import-mxvatv` tests failed (`disc.rs` `a_name_openmx_s_suffix_lookup…`, `textures.rs`
`each_material_is_resolved…`). The `openmx` checkout moved under the gate: its c66d022 (2026-10-23
01:51) makes `Res::find_suffix` prefer a whole-name match. The importer calls that function, so the
wrong-file todos the two tests expected (`shrubbb.psm → AShrubBB.psm`, `thq.psm → BaleTHQ.psm`) no longer
appear. No branch on origin had a fix. I changed only those two assertions to expect the new lookup
(the `treebb.psm` stand-in case still holds), and filed M22 for the importer's comment and the T11 ask.
Gate: `DISCX_ISO=/home/rycs/repos/openmx-mods/iso ./check.sh` → ok, 134 passed.

## What failed

- The first Untamed `spin` run, and one retry, ended the 360 on the frame after take-off without ever
  applying it. The input timing was identical to M20's clean run, so this is most likely the wheel
  touching once just after take-off (M20 saw the same). Retries gave the full turn. Of three single `spin`
  runs before the final loop, two were cut like that and one was full. The final loop kept, for each
  move, the first run with a path over 100° (up to four tries), and all seven came on the first try.
- The first wreck-field search compared bike objects at the end of the capture. By then the bike had
  ridden on (and in one run was in the air), so it found nothing. Reading the wreck call itself gave
  +0x6e4.
- The first summaries counted wrecks to the end of the capture, which picked up later crashes on the
  track after the landing. They now stop 1 s after touchdown.
- Node rows: MX vs. ATV's up/forward rows are at node +0x80/+0x90 (the backflip reads +0x84/+0x94), not
  Untamed's +0xb0/+0xc0. The first MX vs. ATV run read zeros there. Fixed before the kept runs.

## Not verified

- **A real DualShock 2 on a diagonal.** The kit's virtual pad sends full corners, and the game read
  (±1, ±1). A real stick's gate may stop short of both thresholds at once (|x| > 0.7 with |y| > 0.8 needs
  more than the unit circle). The result does not depend on it: arming them one after the other gives the
  same exclusion.
- **Untamed's wreck counter** (+0x708) is M20's identification, not re-traced here. Its clean/wreck calls
  match the clip state at touchdown in every run.
- **Scoring.** How a 360 or a backflip, or a chain of both on different jumps ("combo"), is scored was
  not traced. MD20's "scored as both" has no game behaviour to follow, since the game never runs both.
- **The air control's stick moment while a 360 turns.** It is shown by `spinpull` (+77° of pitch on MX
  vs. ATV) but not traced in the code. M19 traced the air control's damping branch, not why the stick
  moment still applies with R1 held.
- **The 360's axis.** The "about 45°" tilt is read from the summed body-frame rotations (x ≈ −y), not
  from the body's inertia tensor.
- **Other states and maps.** MX vs. ATV ran only on state 4's first lip (map not identified) and Untamed
  only on Rocksbury's first jump. ATV riders and other bikes were not tried; all 24 `.phy` share the
  same `[Three Sixty]` (M19).
