# T11: textures missing on modded maps in openmx

## What was done

- Rebuilt `out/` in this slot (`DISCX_ISO=<main checkout>/iso ./build.sh --no-install`; the slot had no
  `out/`): 56 MX vs. ATV Unleashed, 40 Untamed, 4 elevation, 5 worlds. Built openmx at `e7b66dd` into
  `extracted/openmx-target` (its own `target/release` predates MD3 and MD12).
- Rode every track mod and the 27 MXU tiles `world_mxu` uses (127) in openmx headless:
  `look::OPENMX_ARGS`, one `--shot-pose` at the start and one at the biggest landmark from `accuracy
  poses --landmarks 1` (overview for the elevation maps). That makes 254 shots, read by eye on 16 contact
  sheets (scratchpad, not kept).
- `tools/import-mxvatv/tests/textures.rs` (new): `audit` opens a mod folder with `disc::Res::folder` and
  resolves, as the shell does, the sky (`Sky::load`'s lookup), the ground's
  `terrainscreenspacemesh.psm` passes (`Splat::load`'s), and every model the scene places, the
  ecosystem grows and the toys drive, with each material through `world::Assets::resolve` over
  `read_suffix`. Each miss gives its cause: no model, no `.psm`, a texture no file has, a file the reader
  refuses, or a suffix lookup that answers with another file. Two tests:
  `each_material_is_resolved_as_openmx_resolves_it_and_a_miss_says_why` (a folder made in the test),
  and `--ignored` `every_material_of_every_imported_map_resolves_in_openmx` over `out/` (or
  `OPENMX_MODS_OUT`), which prints a table and fails on any miss the mod's `todo` does not name.
- First run over `out/`: 5,162 misses, 4,983 not in any todo. 5,041 were Untamed materials with no
  `.psm`, because Untamed's materials are BPB only. That is the "textures not showing" the user saw:
  every Untamed prop, plant, start gate and billboard is white.
- **Fixed (importer):** `decode_text_kinds` decodes every `.psm.bpb`, `.toy.bpb` and `.prt.bpb` beside
  itself with tools/bpb, as `decode_scenes` does the scene (standard §2a, extended). Every name's hash
  must recompute; a file whose text name the archives already hold stops the import. One todo entry a
  kind names every decoded file, so modcheck's "not a disc member" check passes. The `*.psm.bpb` kinds
  entry now says it is decoded. Untamed's 40 mods now draw textured (re-ridden).
- **Said (importer):** `suffix_picks` asks every name the importer looked up again of the written
  folder through `Res::folder` and `find_suffix`. It adds one todo entry for names answered with
  another file (45 mods: `thq.psm` → `Tex\BaleTHQ.psm` and so on) and one for names the map lacks but
  the suffix finds (`seating_blue.pst` → `Anaheim_seating_blue.pst`). `Look` records what it was
  asked. A material with no `.psm` whose own-named texture the mod holds now says so in its todo
  entry (MX vs. ATV Unleashed's rule, `0x00167f10`; for Untamed, "not looked up").
- After: 224 misses on 127 mods, every one in its mod's todo. The ignored test passes.
- `notes/openmx-asks.md`: MD2 boundary (what it draws wrong, and `cpresets.ini` →
  `biplanecpresets.ini` on the MXU freeride tiles), a material with no `.psm`, `RenderTarget`, Untamed
  BPB answered for what is drawn, the elevation ground answered. `notes/accuracy.md`: *Textures in
  openmx (T11)*, the table. `standard/` §2a: the decoded kinds.
- Tests changed: `untamed.rs` `a_map_imports_with_its_scene_decoded_and_reads_back` expects the
  decoded `.psm` and `.toy`, not "the toys are BPB". `disc.rs` adds
  `a_name_openmx_s_suffix_lookup_answers_with_another_file_is_in_todo`.

## What was looked at and needed no fix

- The lower-case `tex\` beside `Tex\` (85 mods here) and `.pst` against `.PST`. `Res` folds case, and no
  folder has one path spelt two ways (all 127 open with `Res::folder`). mxvatv_nat13's
  `tex\StartGate02.PST` resolves.
- The elevation ground. From the rider it looked flat, but from 150 ft up it is the NAIP picture (dune
  shadows, Hatfield's trees). It is soft at 2 m a pixel with no detail map, as its todo says. Nothing
  in `tools/elevation` changed.
- The MX vs. ATV and Untamed ground: openmx MD4, filed and open. Skies: all resolve and draw.

- The brief's "files on the line" lists `tex/StartGate02.PST` as a file this task creates. It is a mod
  file under `out/` (game data, ignored), named in the brief as an example, so nothing by that name is
  in the commit.

## What failed on the way

- The first ride run gave identical start and landmark shots: `accuracy poses` prints the
  `--shot-pose` lines to stdout, and `poses.txt` holds the vector table. Fixed by reading stdout.
- `--track mxu_free06` stops at the camera preset ("no preset Race Cam 1"), because the suffix lookup
  finds `biplanecpresets.ini`. Shots of the five MXU freeride tiles were taken with `--camera "Flight
  Cam 1"`.
- A background `build.sh` was running the binary from before `suffix_picks`. I stopped it and ran it
  again. `suffix_picks` first borrowed `Look::asked` while `one` wrote it (`RefCell already
  borrowed`, caught by the tests).

## Not verified

- No comparison with the original game in PCSX2. "Missing" here means openmx draws it white, flat or
  wrong, not that it differs from a T7 shot.
- That the decoded `.prt` text is what openmx's shell particles reader wants: it is in the shell
  crate (`crates/openmx/src/particles.rs`), which this repo cannot link. No particles were looked at in
  the shots.
- That the decoded `.toy` makes openmx run Untamed's traffic. The importer reads the cars and clips
  through game-render, but no shot was taken at a toy's path.
- What the original draws for `seating_blue.pst` (FS05, SX09), and whether Untamed's loader gives a
  material with no `.psm` its own name's texture (SLUS_217.01 not decompiled here).
- The contact sheets were read by eye at 400×300; a small untextured prop seen at a distance on one of
  the 254 frames could be missed. The audit covers every material, but only the models the scene,
  ecosystem and toys name. Bike, rider and HUD are the disc's.
- The world mods (`world_*`) as worlds: openmx has no MD8, so only their tiles were ridden, one at a
  time.
