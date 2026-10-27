# M20: Untamed's 360 and backflip in PCSX2

The M19 follow-up. The findings are in `notes/tricks.md`: the 360 section's Untamed paragraph (which
button the modifier is) and a new section, "Untamed in PCSX2 (M20)". One new entry at the end of
`notes/openmx-asks.md`; follow-up filed (M24 now; M21 was taken on main) (Untamed's R1 gestures).

## Answers

- **MAPKEY_STUNT_MOD is R1.** `SLUS_217.01` loads `UI\PSXController_S%i_P%i.ini` (format string
  `%s%sController_S%i_P%i.ini`); the disc's first preset, `UI\PSXController_S1_P1.ini`, has
  `MAPKEY_STUNT_MOD = BUTTONID_R1` (and moves `MAPKEY_STUNT3` to Select). Live: R1 alone sets key-set
  byte +0x18 (`0x3a2e40` reads it) and nothing else; Circle sets +0x12, Triangle +0x14. The arm reads only
  that byte, so R1 + Circle arms and fires too (captured), unlike MX vs. ATV's "R1 alone". The
  motion-axis flag the step tests first (`0x252630`) is 0 on the PS2.
- **A capture of each on a lip, the attitude every other frame:** `clean360` and `backflip` (closed-loop
  driver), `spin2`, `mid120`, `step30`, `step60` (`kit record` + pad scripts).
- **A clean 360 landing:** `clean360`: X −1100 / Y +1100, 1.87 s of air, 355° about a diagonal axis
  (pitch −249°, yaw +253°), the clip stopped at frame 404 once the bike was back within 35°, landed at
  416 1.6° off the take-off heading, no wreck. The backflip landed clean too (−360°).
- **The mid-clip wreck reason: 0xb**, four times (`spin2`, `mid120`, `step30`, `step60`), read from bike
  +0xc70 where `0x4cbc60` (called by the wreck virtual `0x4fa390`) stores it. 0xf never came up.

## How it went

1. Ghidra (`extracted/untamed`, M13's project): `0x4fd630` and `0x4fd0d0` for the field offsets (they
   differ from MX vs. ATV's: ground +0x354, stick x +0x12bc, the 360's block at +0x11c0, clip states
   +0x1794/+0x1798), `0x3a2e40`/`0x3a2c60` for the key-set bytes, the wreck virtual via the live vtable.
2. `discx x untamed 'UI\PSXController*.ini'` into the scratchpad for the presets.
3. PCSX2 copy 9 (`WAYLAND_DISPLAY=wayland-1 DISPLAY=:0`), state 5 → Rocksbury, saved at the gate as
   state 8. Bike object found by its `[Three Sixty]` floats (six objects; the player's is the one whose
   speed moved). `kit probe vsync` found the integer counter 0x410318 (written into `kit.toml`).
4. `extracted/untamed/watch/air.toml` and pad scripts, as M19's. A gas run found the first big jump
   (~4.7 s, 2.6 s of air).
5. Pad-script tries (all in the notes' table): holding the stick left from before the climb steered the
   bike off line and a wheel touched right after take-off, which ends the move; the full flick then
   spun and wrecked 0xb; stepped flicks gave the minima and went over backwards (0xb); a pause at centre
   gave a middle torque (0xb).

## What failed

- **Wall-clock pad timing.** From the gate and even from a pre-lip save state (scratch state 9), the
  approach varied run to run (heading ±12°, the lip ±0.2 s), so the stick hit the first lip frame on
  some runs and not others; one run armed the wrong direction and never fired. Fixed with
  `extracted/untamed/watch/air360.py`, a closed-loop driver that reads the bike over PINE every frame
  and writes to the pad FIFO when the game's own flags change (lip seen → left; armed → right; fired →
  centre). Its first run landed the clean 360.
- **A shorter jump for the full flick** (gas for 2.7 s only, 1.7 s of air): the slower approach hits a
  second kicker on the lip right after take-off, which ended the move every time (three runs).
- **Countering in the air** (R1 up, stick against the spin) was never tested properly: both runs missed
  the lip (timing, above) and one crashed on the lip (reason 1). Not needed once the driver landed it.

## Second landing (2026-10-27, slot s2)

The first attempt (wip/M20, 2026-10-23) finished the work but was left for a human: the merge's gate
failed in `import-mxvatv --test disc`, a test this task's notes-only diff does not touch (stale imports
in the merging checkout's out/mods, or no DISCX_ISO). Its journal, `notes/tricks.md` and
`notes/openmx-asks.md` hunks were carried onto task/M20 as they were; the asks hunk conflicted only in
place with T12/C11's asks and now follows them. Its follow-up (R1 gestures) is M24: main had used M21
since. The captures were not re-run; the CSVs and the driver are in slot s1's `extracted/untamed/`.

## Not verified

- **0xf** (`0x500880`'s contact-surface test) was never produced; no capture shows which surfaces carry
  its flag.
- **Scoring**: whether the clean 360 and backflip scored (`+0x1548` went 0x36 → 0xff around the moves;
  not traced) was not checked; the HUD was not screenshotted.
- **Presets 2–6**: only the default (preset 1) was used live; that the game picks preset 1 by default is
  inferred from the live key byte (R1 → +0x18), which presets 1 and 2 both give.
- **The decay** (R1 up, stick against the spin) in Untamed was read from the ELF only; no capture
  exercised it.
- **The motion-axis path** (`MAPAXIS_STUNT_360`/`_BACKFLIP`, fixed torques) is the Wii's; on the PS2 its
  flag was 0 in every run, and it was not exercised.
- **Untamed's R1 gestures** (the Classic '1' key with R1 now the modifier): not looked at; filed as M24.
- The maps other than Rocksbury, and bikes other than the state's default, were not tried.
- The 2026-10-23 captures were not re-run for this landing; nothing they rest on changed (notes only).
