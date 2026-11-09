# openmx P315e: the fliers pack names its skins, and modcheck checks skins[]

`import-atv fliers` now writes `skins[].name` and `skins[].about` for each of the eight skins, which
openmx's machine picker (P315) shows. Without them it named each skin "Plane 3" and so on.

Where they come from. All three tables are read off the disc and are not carried in the pack:

- **name**: `UI\Store.dat`, the `StoreText` of the section whose `AssetName` is the skin
  (`[Plane 3]` `StoreText = "PLANE (3)"` `AssetName="BiPlane Skin 3"`). It is written in the case
  that `UI\Challenges.dat`'s unlock lines use ("Plane (2) is now unlocked."): `Plane (1)` …
  `Helicopter (4)`. MX vs. ATV's tables give no other name. The livery names (Pitts, Spitfire) exist
  only as texture names.
- **about**: the model (one: MX Unleashed's shape and figures; two: its own), the garage logo from
  `Veh\Vehicles.dat` (`[BiPlane Skin 1]` `Logo="THQ"`, which draws `UI\Tex\logo_bkg_<Logo>`), the
  store `Price`, and the challenge whose `Prize` is the skin (biplane4: the Airplane Challenge;
  kiowa4: the Helicopter Challenge).
- The skin is matched to its model by number: `BiPlane Skin n` is `biplane<n>`, the same numbering
  MD80d's garage sets use (`Veh_BiPlane<n>.res` holds `biplane<n>_ui.psg`).

modcheck now checks each skin in `skins[]`:

- `machine` and `skin` are given and not empty, and no pair appears twice;
- `model` is the number 1 or 2;
- `psg`, `hull` and `rays` are named and in the pack, and `ui` is in the pack where it is named;
- `name` and `about` are non-empty text where given.

`ui` used to be required. Now it is optional, because openmx's preview falls back to the psg.

## Proof

- `cargo test --release -p import-atv --test fliers -- --ignored` (DISCX_ISO=iso of the main checkout)
  passes. It asserts all eight names and two abouts, and modcheck passes the built pack (174 files
  byte for byte).
- `tests/disc.rs::a_machine_packs_skins_are_held_to_the_standard` passes. It runs a good pack, a
  pack without name/about/ui, and one failing case for each rule.
- In openmx (slot s3), with the new mod.json installed and a garage.txt picking biplane3/kiowa4,
  `openmx --headless --track free09 --frames 30 --garage --mods mods` prints
  `fliers: P1 biplane Plane (3) (biplane3), from mxvatv_fliers, model 2` and
  `fliers: P1 kiowa Helicopter (4) (kiowa4), from mxvatv_fliers, model 2`.
- `./check.sh`: every crate passes except `import-music --test built`. Its two tests fail on
  `.vig` byte counts (840 vs 868, 1400 vs 1428), measured against the openmx that the symlink points
  at. import-music is untouched here, and its use of modcheck does not reach machine packs.

## Not verified

- **Whether MX vs. ATV's garage shows `StoreText` as the skin's name** was not checked against the
  game running in PCSX2 (T7). The garage may show only the logo.
- **Challenge 8's text says it unlocks "Plane (2)", but its `Prize` is `BiPlane Skin 4`.** The about
  follows `Prize`. Which one the game actually gives was not checked.
- **The `Logo` meaning** (a sponsor logo in the garage) is inferred from `UI\Tex\logo_bkg_*` names,
  not watched.
