# M19: MX vs. ATV's tricks for openmx

`tools/import-tricks` builds `out/mods/tricks/mxvatv_tricks/` (203 files) and `untamed_tricks/` (201):
each game's stunt table(s), every rider set's stunt `.mcf`/`.MLB` and the clips they name, the bike's
stunt clips, and the MX and ATV `.phy` that tune the rotations, all byte for byte, with a `mod.json`
(new standard §6, kind `tricks`). `build.sh` runs it for both discs; modcheck knows the kind. The
findings are `notes/tricks.md`; openmx's side is five new entries at the end of `notes/openmx-asks.md`.
Usage: `import-tricks mxvatv|untamed|all [OUT]`, `import-tricks --disc IMAGE --out STAGE` (build.sh),
`import-tricks table GAME`, `import-tricks check DIR`.

## How it went

1. **Tables.** `discx ls` found `Rider\Anim\{MX,MXF,ATV}_Rider_Stunts.ini` on both discs, plus
   `StuntInfo.res` (MX vs. ATV) / `res\StuntInfo.res` (Untamed). The ELF strings name the `.res` and the
   ini names, so the loader reads the tables out of the archive; the loose copies are identical
   (`tests/disc.rs` proves it). Untamed's archive holds only `MX_Rider_Stunts.ini.bpb`, and its ELF names
   no other table, so all of Untamed's rider sets use the MX table. Untamed's text `.ini` is the Wii's
   source with five mapping styles; the PS2 reads `DefaultKeyMappingClassic`.
2. **Ghidra.** SLUS_211.04 had a project in slot s2 (M15/T7); I copied its project and context into
   `extracted/mxvatv/` here and wrote `kit.toml` with `accuracy setup --game mxvatv`, rather than
   re-analysing. Untamed's was already in this slot (M13). Found by string xrefs and by the `.phy`
   key offsets: the stunt loaders (`0x2688e0`, Untamed `0x251e60`), the key decoder (`0x267da0`), the
   `.phy` loader (`0x4df740`), the backflip (`0x4e4f20`), the 360 (`0x4e53d0`), the air control's tail
   (`0x4e6720`), and Untamed's backflip and 360 (`0x4fd0d0`, `0x4fd630`). The first offset search
   for the `[Three Sixty]` fields hit three other objects' functions with clashing offsets (a steering
   step, an aero step, a model loader); the constants 1.25 / 0x3f19999a / 0x3f666666 found the right two.
3. **What is new.** By name against MXU's 57 (read with openmx's `Stunts::parse`): Varial, wheelie,
   stoppie, threesixtyL, threesixtyR. The backflip changed (clocks, points, R1, a clip). 16 MXU stunts
   are gone, the throw-away among them.
4. **Whip and front flip.** No string, `.phy` section or table row names a front flip, and the backflip
   step fires one way only. "Whip" appeared once I searched the text members properly: two loading tips
   describe whipping the bike in the air with the stick, which is the air control (MXU's step; MX vs.
   ATV lowers `UnrollGain`/`UnyawGain` and adds `GasGyroTorque`). My first text search extracted
   nothing (a per-member `discx x` loop that failed silently) and I had already written "no text
   mentions it" in the notes; the second search, by pattern, found the tips and the notes were
   corrected.
5. **Clips.** openmx's `skin::Clip::parse` refuses 123 of the 171 clips: they are `.mot` version 6 or
   7 (MXU's are 3). A quick probe (dropping the extra header word and reading the rest as v3) still ran
   off the end, so the body differs too; left as an ask, not decoded here.
6. **Clocks.** A test expecting 0.93 got 0.92999995: the loader truncates (`strtod` + the EE's
   conversion, openmx's `phy::number`), so MX vs. ATV's text clocks are one ulp under nearest, and
   Untamed's BPB (stored as floats) one ulp over. The disc test allows that ulp between the two games.
   I also assumed the MXF table equals MX's; it differs in the backflip's HoldTime (0.20 vs 0.60), and
   ATV's in saranwrap's SafeTime.
7. **PCSX2 (T7).** Kit copy 9, MX vs. ATV state 4 (a Free Ride in progress). PCSX2 would not start from
   this shell at first (Qt found no platform: `WAYLAND_DISPLAY`/`DISPLAY` were unset; set to `wayland-1`
   and `:0`). `kit probe vsync --save` picked a float that stalled (`0x1c41bc8`); a second probe gave
   `0x3fa6d8`, which runs. The bike object was found by its `[Three Sixty]` numbers in a RAM dump. The
   first 360 try never fired: I pushed the stick before the climb and the arm test needs the body rising
   faster than 5 (body +0x84), which I had read as "speed"; re-timed onto the lip, it fired. Seven
   captures: plain drive, fast 360, slow 360, backflip, reversed backflip (no front flip), whip, stick
   forward. The table is in `notes/tricks.md`; the CSVs, sidecars, watch spec and pad scripts are in
   `extracted/mxvatv/{capture,watch}/` (not in git).
8. **build.sh.** `./build.sh --check --no-install` with `DISCX_ISO` set: both packs built, `modcheck: 109
   ok, 0 failed` (203 and 201 files compared byte for byte with their discs). Not installed into openmx/
   (`--no-install`): the shell reads no trick packs yet, and the install target is the user's checkout.

## Tests

- `tools/import-tricks/tests/table.rs` (in the gate): the table reader on generated text and BPB
  (defaults, comments, quotes, truncation to the loader's float, name buffers, both title forms), the
  gesture decoder, the `.mcf` reader, openmx's 57-section rule, and a whole pack built from generated
  members, read back and refusing to replace another tool's folder.
- `tools/import-tricks/tests/disc.rs` (`--ignored`, needs the discs; run with
  `DISCX_ISO=/home/rycs/repos/openmx-mods/iso cargo test -p import-tricks --release -- --ignored`: 5
  passed): each game's tables read back off the disc (counts, names, the 24 gestures, the values of
  the new rows, MXF's and ATV's one difference each, Untamed's 41 shared rows to an ulp), the loose
  tables equal to the `StuntInfo.res` members, MXU's 57 and the five it lacks, and both packs built,
  every clip's openmx read matching its `.mot` version, every `.phy` holding `[Three Sixty]`, read
  back by `check` and proved by modcheck byte for byte.
- `tools/modcheck`: a `tricks` arm (every file `mod.json` names is in the pack, plus the byte check).

## Not verified

- **Untamed in PCSX2.** Its 360 and backflip are read from the ELF only; no capture. Which PS2 button is
  its `MAPKEY_STUNT_MOD` was not traced (R1 expected, since Classic "mirrors the PS2"). Follow-up: M20.
- **A clean landing** of a 360 or a backflip was not captured: state 4's jump is too short for a full
  360, and the backflip landed nose-down mid-clip. The wreck reason those landings gave was not read.
  The landing rule (clip state, 35°) is from the decompilation only.
- **Which map state 4 is** was not identified (a Free Ride near (2300, 400, 2940)).
- **The pad's timing** is wall-clock: each capture's frames shift by a few rows between runs; the
  numbers in the notes are from the runs kept.
- **Scoring**: how `PointValue`, `HoldPointValue` and `[Depreciation]` become points was not traced, nor
  what wheelie and stoppie score per second.
- **The air control's two gyro conditions** (+0x1290/+0x1294 and +0x318): which inputs they are was not
  traced; "gas" is the key's name, not a confirmed input. `+0xbe4` in the damping test is unidentified.
- **`.mot` v6/v7**: not decoded; only the header's first difference is known.
- **The rider's clips playing in openmx**: nothing in openmx reads a trick pack, so no pack was ridden.
- **`ThreeSixtyTorqueRateIncrease`** is read by the `.phy` loader and not used by the 360 step; I did not
  look for another reader of +0x11fc.
- **MXF_RiderStunts.MLB**: the ELF names it and the disc lacks it; what the game does then was not
  looked at (todo in the pack).
