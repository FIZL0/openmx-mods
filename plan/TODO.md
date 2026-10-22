# TODO — open tasks, in order

One line is one task; ids are permanent. `T` tooling and the standard · `M` a map source.
Checkboxes change only through `agent-runner plan` / `done`.

Survey so far (2026-10-21): MX vs. ATV Unleashed and Untamed use MX Unleashed's container, `.res`, PLN v7
`.trn` and `.col`, and openmx's readers take them (one terrain check aside: openmx MD1). Unleashed's scenes
are the same INI; Untamed's are binary `.bpb`. `notes/mxvatv.md`, `notes/mxvatv-later.md`.

### Tooling

- [~] **T9** the accuracy pairs' openmx half: once openmx has a track from a mod folder (MD2) and --shot-pose (MD10), set tools/accuracy pair's --openmx-args default to their real flags, settle the fov's axis with MD10 (notes/accuracy.md, Poses), and take Lago's (mxvatv_nat05) eight pairs end to end; read them and write what differs in notes/accuracy.md (after T7) → tools/accuracy/ · notes/accuracy.md | t: tools/accuracy tests — the openmx command line built from a pose | j: accuracy-pairs BLOCKED: openmx: MD2, MD10 — a track from a mod folder and --shot-pose; ./openmx-gate reopens this when both are in openmx's DONE.md
- [~] **T10** every imported map loads and rides in openmx (user, 2026-10-22: mod support verified before more games): for all 56 MX vs. ATV Unleashed and 40 Untamed mods in out/, start openmx on the mod (MD2's flag) headless, ride the start a few seconds, and check that it loads with no refusal, the terrain query equals tools/accuracy's at its 8,000 points, the start is on the ground and the edge is the map's (MD3); one --shot-pose shot per map (MD10) read by eye for the ground's textures (MD4) and the windmill (MD5). A table per map in notes/accuracy.md (openmx: loads, heights, start, edge, look); what fails goes in notes/openmx-asks.md (after T9) → tools/accuracy/ · notes/accuracy.md | t: tools/accuracy tests — the per-map openmx run built from a mod | j: openmx-every-map BLOCKED: openmx: MD2, MD3, MD4, MD5, MD10 — loading, big terrains, their ground, class-1 collision, posed shots; ./openmx-gate reopens this when all are in openmx's DONE.md

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
- [~] **C11** the combined and ultra maps in openmx (user, 2026-10-22: mod support verified before more games): load world_mxu, world_mxvatv, world_untamed and world_ultra through MD8, ride across a sample of joins, and check that every tile's ground queries equal its imported map's, the join ground equals stitch's, nothing jitters far from the origin (the ultra map's far corners), and the joins show no seam in look (MD9, --shot-pose pairs at every C9 gate, read by eye) under the Global sky. Results in notes/stitch.md (openmx); failures in notes/openmx-asks.md (after C5, T10) → tools/stitch/ · notes/stitch.md | t: tools/stitch tests — a world's openmx run built from its spec | j: openmx-worlds BLOCKED: openmx: MD8, MD9 — worlds of several terrains and their join ground's look; ./openmx-gate reopens this when both are in openmx's DONE.md

### Stretch

Waiting on a disc in `iso/` or on a map author's permission; nothing here runs unattended.

- [~] **M3** ATV Offroad Fury 2 (PS2, Rainbow) then 1, 3, 4 (3–4 are Climax): terrain format and free-roam areas. Rainbow's ATVOF2 may share the PSR/PLN family. `notes/atvof.md`. Needs the discs. BLOCKED: (after T10, C11) other games wait until mod support is verified in openmx (user, 2026-10-22): T10 and C11
- [~] **M11** MX vs. ATV Reflex / Alive (PS3/360): freeride maps; Reflex's deformable terrain is likely a heightfield. `notes/mxvatv-later.md`. Needs the discs. BLOCKED: (after T10, C11) other games wait until mod support is verified in openmx (user, 2026-10-22): T10 and C11
- [~] **M5** Motocross Madness 2 (PC): heightmap terrain, open Baja maps. `notes/mcm2.md`. Needs the game files. BLOCKED: (after T10, C11) other games wait until mod support is verified in openmx (user, 2026-10-22): T10 and C11
- [~] **M6** MX Bikes community freeride/compound maps: meshes rasterised to a height grid; overhangs and bridges become props. Only with the map author's permission. `notes/mxbikes.md`. BLOCKED: (after T10, C11) other games wait until mod support is verified in openmx (user, 2026-10-22): T10 and C11
- [~] **M7** MX Simulator community tracks (heightmap PNG + texture, near 1:1). BLOCKED: the track site answers "PHP has been disabled on this server"; unblocks when it or a mirror is back. Author permission needed per track.
