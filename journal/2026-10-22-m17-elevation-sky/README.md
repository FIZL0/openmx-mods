# M17: sky, fog and lights for the elevation mods

The DEM and the aerial image have no sky, fog or lights, so M1's scenes had none. Each site now takes all
three, as one set, from a MX vs. ATV Unleashed map of the same kind. The set is the scene's `CubeFile`,
its `[Fog]`, `[Lights]` and `[Light<n>]` sections copied line for line, and the `.cub` copied byte for byte
from the map's own archive. The choices and the reasons are in `notes/elevation.md`, *Sky, fog and lights*:

| site | donor | sky |
|---|---|---|
| elev_glamis | Nat03 Glamis | sky16 |
| elev_ocotillo | Nat09 Deadman Wash (desert; ocotillo and cactus ecosystem) | sky31 |
| elev_littlesahara | MC04 Glamis South (dunes) | sky16, MC04's own lights and fog end |
| elev_hatfield | Nat15 Smuggler's Notch (Appalachian forest) | sky93 |

## What changed

- `tools/elevation/src/sky.rs` (new): `Donor::take(disc, map, name)` opens `Env\<map>\<map>.res`
  with discx and reads its one `.scn`. `Donor::from_scene` collects the sections and the `CubeFile`,
  then finds the `.cub` the way game-render's `Sky::load` does (`<stem>.ps2.cub`, then the name as
  written) and checks that `Sky::parse` decodes it. It refuses a scene with no `CubeFile`, no `[Fog]`
  or no `[Light<n>]`, and a sky that is missing or does not decode. `scene_with` appends `CubeFile` to
  the generated scene's `[Environment]` and the sections after it. `prove` compares the result with
  the donor through game-render's `scene::Scene::parse`: the fog's numbers bit for bit, every light,
  and the `CubeFile` finding the copied file.
- `lib.rs`: `Request.sky: Option<Donor>`. With a donor, `build_mod` writes the scene with the donor's
  sections and proves it, and adds the `.cub` at its member path (`tracks/<id>/Skies/skyNN.ps2.cub`).
  It adds three `todo` entries in place of "no sky": the donor and what was copied, the sun's points
  being in the donor's frame, and own-disc. `mod.json` gets `sky_from` and `permission: own-disc`.
  `version` is now 1.1.0 for every build, because mod.json changed even without a sky (the no-sky
  `todo` entry moved).
- `sites.rs`: each site names its donor. `main.rs`: the data step opens the MX vs. ATV Unleashed disc
  (found by boot ELF in `--iso`, else `$DISCX_ISO`, else `iso/`) and gives each site its donor. With no
  disc it says so and builds without a sky, and `--sky none` does the same on purpose. `build` takes
  `--sky MAP,NAME`.
- `build.sh`: one line, `export DISCX_ISO=$iso`, so `./build.sh --iso DIR` reaches the data step.
- `Cargo.toml`: depends on `discx` (by path, the repo's own).

## How it was proved

- `cargo test -p elevation`: `tests/sky.rs` adds 2 tests, and the 7 old ones still pass.
  - The first uses a scene in the disc's layout and a generated 96-packet `.cub` that `Sky::parse` takes.
    The sections come out line for line (the `;  The Sun` comment and `NumberOfLights = 2` kept), and
    `[StaticModels]`/`[Model1]`/`EcosystemFile` are not copied. It also checks the five refusals.
  - The second builds a one-cell mod with the donor. The `.cub` is in the files with the same bytes.
    `CubeFile` sits in `[Environment]`. game-render reads the fog (120,94,64; 5, 768, 1, 0.8 as the
    original's strtod gives it) and two lights (ambient, then the directional sun with its colour).
    game-sim's `track::Scene` still parses the scene. `prove` refuses a changed haziness, a changed
    light colour and a changed CubeFile. `mod.json` has `own-disc` and `sky_from`, and modcheck passes.
- Mutation check, reverted: making `scene_with` drop the `Type=` lines failed the build in the test
  ("the scene's lights [] are not Nat99's …").
- **The four real sites**, built against the disc image (`DISCX_ISO` = the main checkout's `iso/`, read
  only). All four build and every in-build proof passes (`modcheck: 4 ok`). For each one I extracted
  the donor's `.cub` separately with `discx x`, and the mod's copy is identical (`cmp`) to the archive
  member. It is also identical to the loose `Skies\skyNN.ps2.cub` in the PSR root. The mod scene's
  Fog/Lights/Light<n> lines `diff` clean against the donor scene extracted with discx.
- `./build.sh --check --iso <main iso/> --out <scratch>`: every step ran, and `modcheck: 102 ok,
  0 failed`. Its four elevation mods are identical (`diff -r`) to the direct run.
- `elevation --out … --sky none`: every file except `mod.json` is byte-identical to M1's builds in
  `out/`. `mod.json` differs only in `version` and in where the no-sky `todo` entry sits.
- The four sky builds replace M1's in this worktree's `out/mods/tracks/` (not in git).

## What failed on the way

- The first check that a donor has a `[Light<n>]` accepted `[Lights]` (6 characters, "light" plus
  "s"). The refusal test caught it, and the check now excludes `Fog` and `Lights` by name.
- A test expected haziness 0.8 exactly. game-render reads 0.80 as the original does, 0.79999995, so
  the test now compares within 1e-6. The proof itself compares the two parses bit for bit, so it is
  unaffected.
- A second test asserted that no todo said "no sky". The donor's own entry says "the data has no sky",
  so the assertion now matches the no-donor entry's exact wording.
- The scratchpad hit its disk quota after the full build.sh run (about 100 mods). The next `--sky none`
  run failed to write, and its two "FAIL" lines came from that. I cleared the scratchpad and the run
  then passed.

## Not verified

- **Nothing was seen in the game, PCSX2 or the port.** The port cannot load a mod until MD2. I did not
  render the skies, even with game-render. They are copied as they are, and the choice of donor rests
  on the map's name, its ecosystem members and its fog colour, not on looking at it.
- Whether the original game uses the sun's `Position`/`TargetPosition` as points (for shadows, say) and
  not only as a direction. They are in the donor's 5760-ft frame, and the elevation maps are 6719 ft.
  This is in each `todo`.
- Whether MD2 finds a mod's `.cub` under `Skies/` by suffix. The standard's lookup rule (§2) says it
  will, but the loader does not exist yet.
- That Smuggler's Notch looks like southern West Virginia, or Deadman Wash like Ocotillo Wells, beyond
  the names, the ecosystem model names and the fog colour.
- The donors' `[Environment]` keys that go with lighting but were not copied (`EcoAmbientScale`,
  `AmountOfTransmittedLight`): whether the original reads them for the ground's light was not checked.
- MX Unleashed's maps were not looked at as donors.
