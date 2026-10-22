# M16: surface ids for the elevation mods

Until now every sample of the four elevation maps carried surface id 0, because nobody knew which of the
bike's `.phy` surfaces is sand. This task found out what the ids mean and what MX Unleashed's own
freeride ground uses. The elevation maps now take their ids from the aerial picture: sand 4, dirt 0,
grass 3. The full account and the tables are in `notes/elevation.md`, *Surfaces (M16)*.

## Findings

- **The id is the `.phy` friction zone less one.** The bikes' `.phy` comments name the 16 zones
  (1 dirt, 2 edge-track sludge, 3 off-track sludge, 4 "standard off track zone, presumably grassy",
  5 sandy dirt, 6 ice, 7 and 8 water or mud). Zones 9–16 equal zone 1 key for key in all 224 bike `.phy`
  files on the MX Unleashed disc. The four-wheelers in `machine.res` use one row for all 16. MX vs. ATV
  Unleashed's comments name 9–16 by paint colour, and all of them as standard dirt.
- **The freeride maps (free06, 07, 09, 10, 11) use ids 0, 9, 12, 13, 14 and 15, all of which ride as
  zone 1.** The ids follow the paint in places (free09's sand is 0 at 96%, free11's sand is 9 at 99%,
  free10's rock is 13 at 92%), but not as one code. id 15 is the ring outside the middle 512 × 512. So in
  the original, freeride sand, rock and grass all ride as dirt. Zone 5 (sand) is used on one map in the
  game (nat07, 0.4%), and zone 4 (grass) on none.
- Many freeride samples have the surface byte's top bit set (185,000–245,000 per map). That bit is not
  part of the id (game-sim masks it) and was not looked into.

## What changed

- `tools/elevation/src/surfaces.rs` (new):
  - `layers`, `survey`, `Survey::table` and `ids`: the freeride survey.
  - `seen`: NAIP's four bands averaged over each sample's square.
  - `ndvi`, `Ground` (dunes / desert / forest) with `classify` (NDVI averaged over 5 × 5 samples
    against a threshold, through a summed-area table), and `Cover` with the ids.
- `lib.rs`:
  - `Request.ground: Option<(Ground, f32)>`. With a picture, `build_mod` writes the classified ids into
    the grid. `prove_terrain` already checked every sample's surface through openmx's `Terrain`, so the
    ids are proved in the build.
  - `Facts.surfaces` holds the counts per id, and mod.json has `elevation.surfaces`.
  - The surfaces `todo` line now says what each id is and how it was chosen, and that the original rides
    all of its freeride paint as dirt.
  - `version` is now 1.2.0.
  - `window_of(req)` gives a window's place without reading it.
- `sites.rs`: each site has `ground` and `green` (Glamis dunes 0.03, Little Sahara dunes 0.0, Ocotillo
  desert, Hatfield forest −0.2).
- `main.rs`:
  - `elevation surfaces [MAP...]` reads the survey off the MX Unleashed disc; other maps get their ids
    counted.
  - `elevation cover OUT.png SITE [--green T] [--picture RGB.png]` writes the classes and the picture.
  - `build` takes `--ground KIND --green T`.
- Tests `tools/elevation/tests/surfaces.rs` (3):
  - A generated DEM with a four-band picture, green east and bare west. It is built as dunes, forest
    and desert, with a fixed id, and with no picture, and every sample is read back with `Terrain`.
  - Averaging: a lone green sample is lost, a patch is kept, and a sample with no picture counts for
    nothing.
  - The survey on a generated terrain and mask.
  - `generated.rs` and `sky.rs` only gained `ground: None`.

## How it was proved

- `cargo test -p elevation`: 12 tests pass (7 + 2 + 3).
- Mutation checks: making dunes' bare ground dirt, and mirroring the picture's columns in `seen`, each
  fail the build test.
- `elevation surfaces` on the disc gave the table in the notes. `elevation surfaces nat01 … sx24` counted
  every national and supercross track.
- The `.phy` comparisons were made with throwaway scripts in the session scratchpad. They are not in
  git. Re-running means extracting `Bike\*.res:**.phy` with discx and comparing the zone keys.
- All four sites were built with `DISCX_ISO=… elevation --out STAGE` (sky on, modcheck ok, no FAIL):
  - Glamis: sand 99.9%.
  - Little Sahara: sand 64.8%, dirt 35.2%.
  - Ocotillo: dirt 100%.
  - Hatfield: grass 96.5%, dirt 3.5%.
- `elevation cover` pictures were compared by eye against the NAIP picture for each site (thresholds in
  the notes).

## Tried and dropped

- **One NDVI threshold for all sites (0.2), per sample.** On Hatfield the shaded side of every hill came
  out bare (47%). On Little Sahara only the junipers came out green. Raw NAIP NDVI is not calibrated and
  falls in shadow.
- **One dunes threshold.** Glamis's sand is at about 0 everywhere. At 0.0 its slip faces became dirt
  (16%), and at −0.01 the whole map did, while Little Sahara needs 0.0 to find its sagebrush. So the
  threshold belongs to the site.
- **Per-sample classes.** They were speckled. Averaging over 5 × 5 samples made patches that follow the
  picture.

## Follow-up filed

- M18: telling Ocotillo's sandy washes from its badlands, and giving Hatfield's river a water zone (7): the
  NDVI test cannot do either.

## Not verified

- **How the ids ride in the game.** Nothing was run in PCSX2 or in the port (T7). Whether sand 4 and
  grass 3 feel right on these maps, or are even noticeable, was not checked.
- **Going past the original.** MX Unleashed's own freeride maps ride every paint as zone 1. Giving the
  elevation maps zones 5 and 4 is a choice beyond the original, made because the brief asks for it. It is
  said in every mod's `todo`.
- **What else reads the id.** I looked only at the bikes' and four-wheelers' `.phy` tables and the
  `.aud` files (no surface keys). The ELF was not decompiled for other readers of the id (sound, dust,
  AI). openmx's particles note that the disc's `mxparticlezones.dat` names no zones.
- **The thresholds** were set by eye on one picture per site, not against ground truth (no land-cover
  set). Sand and dirt come from the site's kind, not from the picture.
- **The top bit** of the freeride surface bytes was not looked into.
- **`build.sh --check`** was not run in full. The elevation step was run directly, and it is the only
  step this task changes.
