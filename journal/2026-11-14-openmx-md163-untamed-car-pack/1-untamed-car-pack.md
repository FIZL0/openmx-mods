# openmx MD163: MX vs. ATV Untamed's baja truck as a machine pack

`import-atv untamed-cars [OUT]` (and `--disc IMAGE --out OUT` off the Untamed disc, which build.sh's new
`SLUS_217.01 import-atv` line runs) writes `mods/machines/untamed_cars/` off SLUS_217.01. It is built on
MD39c's `import-atv cars` (branch openmx-MD39c, not yet on main), so this branch carries that commit too.

- `res\veh\VehicleType_6.res` (the baja truck, the menus' TROPHY TRUCK) is copied whole: 27 files, among
  them `Veh\baja.phy.bpb`, `Veh\Anim\baja.psg` and `Veh\Col\bajaM.col`.
- `mod.json`'s `cars` lists the baja with `"step": "untamed"`: openmx's `machine_packs::CarFiles::untamed`
  (MD161), which rides it on Untamed's car step (`atv_car::Load::untamed`). openmx offers it as
  `--mxvatv-car baja@untamed_cars`; a bare `baja` stays `mxvatv_cars`' (Unleashed's).
- Only the baja: the monster truck (type 5) is openmx MD161b, one line in `untamed_cars::CARS`.

## How it was proved

- `cargo test -p import-atv --release --test disc -- --ignored` (with `DISCX_ISO`): the new
  `the_untamed_car_pack_is_the_discs_and_openmx_reads_it` passes: modcheck byte for byte against the disc
  with no failures, the step word in the manifest, and openmx's readers read the BPB `.phy` (Weight 3500,
  as openmx MD81d's diff says: 2500 in Unleashed) and the chassis (43 nodes, the four tire nodes).
- In openmx (MD163): `crates/game-sim/tests/untamed_baja_load.rs` builds the car off this pack's
  `baja.phy.bpb` and matches openmx MD164's spawn image of the original's baja word for word;
  `crates/openmx/tests/untamed_cars.rs` rides the pack in the shell.

## Not verified

- **Not looked at beside Untamed in PCSX2 (T7)**: the pack is drawn by nothing yet, so there is nothing
  to compare by eye. It is in the pack's `todo`.
- **The archive's other members** (materials, sounds, particles) are copied but not read back by openmx:
  openmx reads only the `.phy.bpb` and the chassis model for the cars.
- **`./check.sh` is red on import-music's `tests/built.rs`** (2 tests: each `.vig` 28 bytes longer than
  openmx's reader now says) against today's openmx. This branch does not touch import-music; it is
  openmx's music reader having moved on since the test was written. Every other workspace test passes.
