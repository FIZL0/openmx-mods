# TODO — open tasks, in order

One line is one task; ids are permanent. `T` tooling and the standard · `M` a map source.
Checkboxes change only through `agent-runner plan` / `done`.

Survey so far (2026-10-21): MX vs. ATV Unleashed and Untamed use MX Unleashed's container, `.res`, PLN v7
`.trn` and `.col`, and openmx's readers take them (one terrain check aside: openmx MD1). Unleashed's scenes
are the same INI; Untamed's are binary `.bpb`. `notes/mxvatv.md`, `notes/mxvatv-later.md`.

### Tooling

- [ ] **M18** Elevation surfaces past the NDVI test: tell Ocotillo's sandy washes from its badlands (texture, slope or a soils set such as SSURGO), and give water (Hatfield's river) zone 7 rather than dirt; see notes/elevation.md Surfaces (M16)
- [~] **T9** the accuracy pairs' openmx half: once openmx has a track from a mod folder (MD2) and --shot-pose (MD10), set tools/accuracy pair's --openmx-args default to their real flags, settle the fov's axis with MD10 (notes/accuracy.md, Poses), and take Lago's (mxvatv_nat05) eight pairs end to end; read them and write what differs in notes/accuracy.md (after T7) → tools/accuracy/ · notes/accuracy.md | t: tools/accuracy tests — the openmx command line built from a pose | j: accuracy-pairs BLOCKED: openmx has neither MD2 (a track from a mod folder, by command-line flag) nor MD10 (--shot-pose); both open in openmx's plan/TODO.md at b63486a. Unblocks when both land: set look::OPENMX_ARGS to their flags, settle the fov axis, take Lago's 8 pairs with accuracy pair, write the differences in notes/accuracy.md

### MX vs. ATV Unleashed


### MX vs. ATV Untamed

### Real elevation

### Accuracy

A map is done when it matches the game it came from, not when it loads. Terrain, collision, placements and
scene values are copied as the disc has them, never resampled or approximated; what the port cannot show yet
goes in mod.json's `todo` and notes/openmx-asks.md.
- [ ] **M12** MX vs. ATV Unleashed accuracy pass: every imported map (M2) through T7 — heights exact at ≥1,000 points, and screenshot pairs from at least five poses each (spawn, the map's landmarks, a ridge looking across). Fix every difference in the importer (a prop missing, a texture wrong, fog, sky, lights, ecosystem density); what only the port can fix goes to notes/openmx-asks.md and the map's mod.json todo. A table per map in notes/mxvatv.md: matches / differs / why (after M2, T7) → tools/import-mxvatv/ · notes/mxvatv.md · notes/openmx-asks.md | t: tools/accuracy over every imported map | j: mxvatv-accuracy
- [ ] **M13** MX vs. ATV Untamed accuracy pass: the same as M12 for every imported Untamed map (M4), on Untamed running in PCSX2 (after M4, T7, M12) → tools/import-mxvatv/ · notes/mxvatv-later.md · notes/openmx-asks.md | t: tools/accuracy over every imported Untamed map | j: untamed-accuracy

### Music

### Combined maps

One giant map per game out of its non-racing maps (no stadium with crowd seats: no SX, FS or other arena
maps), their edge berms removed and the maps joined smoothly; then one ultra map out of every game's giant
map and MX Unleashed's. Inside each source map the ground stays the original's; only the joins change.
- [ ] **C8** tools/stitch: a tile with missing terrain cells. mxvatv_nat04 is 1024×1280 samples but has no cells past x≈896 on its rows, yet find_keep keeps x to 990, and ring/ground_in read the missing cells as 0 ft (unwrap_or), so its join edge drops 400 ft (near-seam slope 63, coarse 1.37 that no widening fixes; stitch check: 2,800 'no cell' failures). Cut each side to the cells that exist, or end the interior at the hole, and say so in todo (C7 journal) → tools/stitch/ | t: tools/stitch tests — a tile with a missing root cell column: its keep stops at the hole, stitch check passes | j: stitch-holes
- [ ] **C9** tools/stitch: the rideable gate never passes on the real MX vs. ATV world. A steep tile edge (mc09's steepest edge cell is 3.1; nat17, nat02) carries a slope of 0.6–1.0 just past the gate's 2 lattice + 3 fade, and widening the gaps cannot change it (29 maps without nat04 fail at 0.726 171 ft from mc09 at every widening; with --fade 96, 0.608 at 315 ft). Make the gate or the extension fit a steep edge (the fade or the blur scaled by the edge's slope, or a gate distance per edge) so C2's world builds at max_slope 0.5 (C7 journal) → tools/stitch/ | t: tools/stitch tests — a map with a 3:1 edge beside a flat one builds at max_slope 0.5 | j: stitch-steep-edges
- [ ] **C2** MX vs. ATV Unleashed giant map: every non-racing map from M8's table (no stadium with crowd seats: not SX, FS or other arenas; MC and the open national-style maps if they qualify by M8) joined by C1 into one world. Ride every seam in openmx; list each map's place in notes/mxvatv.md (after C1, M2, M8, M12) → notes/mxvatv.md | t: tools/stitch --check on the world: every seam continuous, every interior identical to its imported map | j: mxvatv-giant-map
- [ ] **C3** MX vs. ATV Untamed giant map: the same as C2 for Untamed's non-racing maps (no SX, FS, MT or other stadium maps) (after C1, M4, M13) → notes/mxvatv-later.md | t: tools/stitch --check on the world | j: untamed-giant-map
- [ ] **C5** the ultra map: every game's giant map (C2, C3, C4, and each later game's as it lands: ATV Offroad Fury M3, Reflex/Alive M11, …) joined by C1 into one world, MX Unleashed's included. Plan the layout so each game's world stays whole and the joins between games are as smooth as those inside one. Re-run whenever a new game's giant map lands (after C2, C3, C4) → notes/ultra-map.md | t: tools/stitch --check on the world | j: ultra-map
- [ ] **C10** world_mxu back to the default max_slope 0.5 once C9 lands: drop max_slope and max_slope_why from tools/stitch/worlds/world_mxu.json, rebuild, and stitch check it (C4 built it at 8 because mxu_nat17's corner mountain carries 7.77 and mxu_nat21 0.63 past the gate). While the spec needs a raised max_slope, the world's mod.json todo does not say so: let a spec carry todo lines into the world's (C4 journal) (after C9) → tools/stitch/worlds/ | t: tools/stitch --check on world_mxu at 0.5 | j: mxu-world-slope

### Stretch

Waiting on a disc in `iso/` or on a map author's permission; nothing here runs unattended.

- [ ] **M3** ATV Offroad Fury 2 (PS2, Rainbow) then 1, 3, 4 (3–4 are Climax): terrain format and free-roam areas. Rainbow's ATVOF2 may share the PSR/PLN family. `notes/atvof.md`. Needs the discs.
- [ ] **M11** MX vs. ATV Reflex / Alive (PS3/360): freeride maps; Reflex's deformable terrain is likely a heightfield. `notes/mxvatv-later.md`. Needs the discs.
- [ ] **M5** Motocross Madness 2 (PC): heightmap terrain, open Baja maps. `notes/mcm2.md`. Needs the game files.
- [ ] **M6** MX Bikes community freeride/compound maps: meshes rasterised to a height grid; overhangs and bridges become props. Only with the map author's permission. `notes/mxbikes.md`.
- [~] **M7** MX Simulator community tracks (heightmap PNG + texture, near 1:1). BLOCKED: the track site answers "PHP has been disabled on this server"; unblocks when it or a mirror is back. Author permission needed per track.
