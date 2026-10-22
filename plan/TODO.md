# TODO — open tasks, in order

One line is one task; ids are permanent. `T` tooling and the standard · `M` a map source.
Checkboxes change only through `agent-runner plan` / `done`.

Survey so far (2026-10-21): MX vs. ATV Unleashed and Untamed use MX Unleashed's container, `.res`, PLN v7
`.trn` and `.col`, and openmx's readers take them (one terrain check aside: openmx MD1). Unleashed's scenes
are the same INI; Untamed's are binary `.bpb`. `notes/mxvatv.md`, `notes/mxvatv-later.md`.

### Tooling

- [~] **T9** the accuracy pairs' openmx half: once openmx has a track from a mod folder (MD2) and --shot-pose (MD10), set tools/accuracy pair's --openmx-args default to their real flags, settle the fov's axis with MD10 (notes/accuracy.md, Poses), and take Lago's (mxvatv_nat05) eight pairs end to end; read them and write what differs in notes/accuracy.md (after T7) → tools/accuracy/ · notes/accuracy.md | t: tools/accuracy tests — the openmx command line built from a pose | j: accuracy-pairs BLOCKED: openmx has neither MD2 (a track from a mod folder, by command-line flag) nor MD10 (--shot-pose); both open in openmx's plan/TODO.md at b63486a. Unblocks when both land: set look::OPENMX_ARGS to their flags, settle the fov axis, take Lago's 8 pairs with accuracy pair, write the differences in notes/accuracy.md

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
- [ ] **C3** MX vs. ATV Untamed giant map: the same as C2 for Untamed's non-racing maps (no SX, FS, MT or other stadium maps) (after C1, M4, M13) → notes/mxvatv-later.md | t: tools/stitch --check on the world | j: untamed-giant-map
- [ ] **C5** the ultra map: every game's giant map (C2, C3, C4, and each later game's as it lands: ATV Offroad Fury M3, Reflex/Alive M11, …) joined by C1 into one world, MX Unleashed's included. Plan the layout so each game's world stays whole and the joins between games are as smooth as those inside one. Re-run whenever a new game's giant map lands (after C2, C3, C4) → notes/ultra-map.md | t: tools/stitch --check on the world | j: ultra-map

### Stretch

Waiting on a disc in `iso/` or on a map author's permission; nothing here runs unattended.

- [ ] **M3** ATV Offroad Fury 2 (PS2, Rainbow) then 1, 3, 4 (3–4 are Climax): terrain format and free-roam areas. Rainbow's ATVOF2 may share the PSR/PLN family. `notes/atvof.md`. Needs the discs.
- [ ] **M11** MX vs. ATV Reflex / Alive (PS3/360): freeride maps; Reflex's deformable terrain is likely a heightfield. `notes/mxvatv-later.md`. Needs the discs.
- [ ] **M5** Motocross Madness 2 (PC): heightmap terrain, open Baja maps. `notes/mcm2.md`. Needs the game files.
- [ ] **M6** MX Bikes community freeride/compound maps: meshes rasterised to a height grid; overhangs and bridges become props. Only with the map author's permission. `notes/mxbikes.md`.
- [~] **M7** MX Simulator community tracks (heightmap PNG + texture, near 1:1). BLOCKED: the track site answers "PHP has been disabled on this server"; unblocks when it or a mirror is back. Author permission needed per track.
