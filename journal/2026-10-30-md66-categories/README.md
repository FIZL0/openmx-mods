# openmx MD66: a category on every map

openmx's MD66 filters the track pick and the store's Browse by a map's category. This side adds the
optional `category` field to standard §1 and writes it from every importer.

## What each tool writes

- `import-mxvatv` (both games and MXU tiles): `category(origin, id)` reads the event type from the id after
  the game's prefix. `fs` is Freestyle, `sx` Supercross, `nat` National, `sm` Supermoto, `mc` Machine
  Challenge, `bt` Opencross, and `am`, `mt` and MXU's `free` towns are Free Roam. Anything else is Other.
- `elevation`: Free Roam (real ground, no event).
- `stitch` (`build.rs`, `ultra.rs`): World, on every world.
- The 105 built mods in `out/mods/tracks` and openmx's installed copies got the field written in place
  after `"name"`, the same word the tools now write. I did not reimport them: no other byte would change,
  and a full reimport takes a long time.

The words come from the games' own activity types, read off the Untamed disc's `Master.atyList`:
`AM_*` misc/free ride, `BT_*` opencross, `MT_A` misc/free ride, `FS` high-score/combo challenge, `Nat`
national, `SM` supermoto, `SX` supercross. MX vs. ATV Unleashed's `mc` maps load `MachineChallenge.aty`.

## Not verified

- `mc` is "Machine Challenge", not the brief's "Motocross/National". The game's own type for those maps
  is the machine challenge, and openmx's brief asked for the game's own names.
- Untamed's `am`/`mt` are "misc" plus free ride in the game. Free Roam is my name for that, not a string
  from either game's UI.
- I did not check the words against the games' menus in PCSX2.
- Tiles inside the built worlds (`tiles/*/mod.json`) keep no category. The pick never lists a tile, and
  a rebuilt world copies the new track mod.json files.
- `version` was not bumped on the edited mod.json files. The tools' VERSION constants did not change
  either, so a reimport gives the same version.
