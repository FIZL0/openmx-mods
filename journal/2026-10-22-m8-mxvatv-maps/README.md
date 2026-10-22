# M8: which MX vs. ATV Unleashed maps are freeride, and what each is

The table and the first-mods pick are in `notes/mxvatv.md`, *Free Ride: what each map is*.

## What was done

- Pulled into the session scratchpad (not the repo) with `discx x mxvatv`: `Env\*.aty`, `Env\*\*.aty`,
  `Env\*\*.atylist`, `Env\*\*.scn`, `UI\*.dat`, `UI\*.txt`, `Loc\*`, `TrackList.res`; `TrackList.res`'s
  members with `discx res`. Pulled `SLUS_211.04`, `GAME_REL.BIN` and `UI_REL.BIN` off the ISO with
  `bsdtar` for `strings` only.
- Read the 56 `.atyList`s with a throwaway INI reader (comments after `;` dropped): `DisplayName`, the
  `[General]` `Activity` list, each section's `Type`, and the `[Free Ride]` section's `ATYFile`,
  `StartPosition` and `BarVehicleType`. The table rows were printed by a script from those files,
  `UI\Tracks.dat` (order, `Locked = FALSE`), `UI\Store.dat` (names) and the survey table already in the
  note (leaves, side), then pasted, so no row was typed by hand.
- Looked for map names in the `Loc\` string tables: none. They hold activity names (`STR_MODE_FREERIDE`).
  The names come only from `DisplayName` (and, upper-cased, from `Store.dat`).
- Compared `TrackList.res`'s 57 members with the loose files: all 57 byte-identical.
- In the ELF's strings: the activity type list, the same list without spaces (`freeride`), and the
  formats `%s%s.aty`, `%s%s\%s.atyList`, `%sTracks.dat`. That is the evidence for type → `Env\<Type>.aty`.

## Findings

- All 56 maps offer Free Ride; no `Activity` line is commented out. What differs is the `[Free Ride]`
  section: Nationals and Supermoto name no `ATYFile` (only `Env\Freeride.aty`), MC and FS name their own
  `.aty`, SX name a per-map `FreeRide.aty` (crowd and flash groups). Only MC01 and FS05 give a start.
- 33 maps are big outdoor worlds (80×80, Nat04 56×80); the rest are 896–1280 ft arenas.
- Store names differ from `DisplayName` on the FS maps ("FMX" dropped) and MC04 ("Glamis" for "Glamis
  South").
- First mods: nat03 Glamis, nat09 Deadman Wash, nat02 Tipperary. They are the clean National maps (every
  placed `.col` present, none refused), their terrain shape matches MXU's own nat09/nat18, and their Free
  Ride needs only `Env\Freeride.aty`. The reasons are in the note.

## Tried and dropped

- `discx ls mxvatv '*.aty*'` matched nothing (`*` stays within one path segment); listing everything and
  grepping worked.
- A first draft of the note said `Machine Challenge ab.aty` was "the one type name that keeps its
  spaces". The ELF's space-free list has no entry for `machine challenge ab`, so that was reworded to say
  only what the files show.

## How it was proved

No code changed, so `./check.sh` is the only gate. Each table cell comes from a file named in the note;
the scripts that printed them stay in the scratchpad (game-data readers, not tools to keep).

## Not verified

- That the game's Free Ride menu lists all 56 maps, with these names, was not checked in PCSX2 (T7). The
  table says what the `.atyList`s ask for.
- The type → `Env\<Type>.aty` link, and that a `[Free Ride]` with no `ATYFile` loads only
  `Env\Freeride.aty`, are read from file names and ELF strings, not decompiled. There is no Ghidra project
  for `SLUS_211.04` yet, and setting one up for this was out of proportion.
- Where the game puts the rider in a Free Ride with no `StartPosition` (54 of 56) was not looked up.
- "Clean" in the first-mods list means only what the T6 survey checked (readers accept every file, every
  placed `.col` exists). Textures, models, skies and the look of the maps were not checked (M9, T7). That
  Glamis, Deadman Wash and Tipperary look different from each other is a guess from their names.
- `GAME_REL.BIN` and `UI_REL.BIN` were only grepped for strings.
