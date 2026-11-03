# TODO — open tasks, in order

One line is one task; ids are permanent. `T` tooling and the standard · `M` a map source.
Checkboxes change only through `agent-runner plan` / `done`.

Survey so far (2026-10-21): MX vs. ATV Unleashed and Untamed use MX Unleashed's container, `.res`, PLN v7
`.trn` and `.col`, and openmx's readers take them (one terrain check aside: openmx MD1). Unleashed's scenes
are the same INI; Untamed's are binary `.bpb`. `notes/mxvatv.md`, `notes/mxvatv-later.md`.

### Tooling

- [ ] **T14** (follow-up, critical) Every mod ships its own picture (user 2026-11-03: 'we should prebake world images when mods are uploaded and not regen them'; openmx P282). Each importer and tools/stitch writes previews/track.png into the mod it builds (openmx's track_previews::own_picture reads it first): the overview still shot once here with openmx's still_command framing (512x288, --overview, --world-ground all), so a player's machine never shoots one. Worlds above all: world_ultra's still takes 8.4 GB and 64 s, which OOM-killed the Steam Deck. Backfill every mod already built and republish it to the store → tools/stitch · tools/import-mxvatv · tools/survey | t: every tracks/<id>/ in mods/ has previews/track.png, 512x288, and modcheck fails a mod without one | j: baked-pictures
- [ ] **T13** stitch builds on openmx's stitcher (after openmx P232; user 2026-10-29: 'can we have the real one here and just have mxmods depend on the stitching?'). openmx P232 moves tools/stitch's core (lib.rs, build.rs, check.rs seams, jsonw.rs) and tools/trnwrite into openmx as crates/stitch and crates/trnwrite, and ships world_mxu and world_mxu_free built from the player's disc. Here: delete the moved code, path-depend on openmx/crates/stitch and openmx/crates/trnwrite (as game-sim is today), keep ultra.rs, the CLI and the import step; stop installing world_mxu and world_mxu_free (openmx makes them), keep world_mxvatv, world_untamed and world_ultra and any new world. build.sh and check.sh green; the other worlds rebuild byte-identical -> tools/stitch · tools/trnwrite · build.sh | t: stitch tests on the openmx crates; world_mxvatv/untamed/ultra joins unchanged
### MX vs. ATV Unleashed


### MX vs. ATV Untamed

### Real elevation

### Accuracy

A map is done when it matches the game it came from, not when it loads. Terrain, collision, placements and
scene values are copied as the disc has them, never resampled or approximated; what the port cannot show yet
goes in mod.json's `todo` and notes/openmx-asks.md.

### Music

### Combined maps

One giant map per game out of its non-racing maps (no stadium with crowd seats: no SX, FS or other arena
maps), their edge berms removed and the maps joined smoothly; then one ultra map out of every game's giant
map and MX Unleashed's. Inside each source map the ground stays the original's; only the joins change.

### Stretch

Waiting on a disc in `iso/` or on a map author's permission; nothing here runs unattended.

- [ ] **M3** ATV Offroad Fury 2 (PS2, Rainbow) then 1, 3, 4 (3–4 are Climax): terrain format and free-roam areas. Rainbow's ATVOF2 may share the PSR/PLN family. `notes/atvof.md`. Needs the discs. — reopened: T10, C11 landed
- [ ] **M11** MX vs. ATV Reflex / Alive (PS3/360): freeride maps; Reflex's deformable terrain is likely a heightfield. `notes/mxvatv-later.md`. Needs the discs. — reopened: T10, C11 landed
- [ ] **M5** Motocross Madness 2 (PC): heightmap terrain, open Baja maps. `notes/mcm2.md`. Needs the game files. — reopened: T10, C11 landed
- [ ] **M6** MX Bikes community freeride/compound maps: meshes rasterised to a height grid; overhangs and bridges become props. Only with the map author's permission. `notes/mxbikes.md`. — reopened: T10, C11 landed
- [~] **M7** MX Simulator community tracks (heightmap PNG + texture, near 1:1). BLOCKED: the track site answers "PHP has been disabled on this server"; unblocks when it or a mirror is back. Author permission needed per track.
