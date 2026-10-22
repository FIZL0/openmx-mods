# TODO — open tasks, in order

One line is one task; ids are permanent. `T` tooling and the standard · `M` a map source.
Checkboxes change only through `agent-runner plan` / `done`.

Survey so far (2026-10-21): MX vs. ATV Unleashed and Untamed use MX Unleashed's container, `.res`, PLN v7
`.trn` and `.col`, and openmx's readers take them (one terrain check aside: openmx MD1). Unleashed's scenes
are the same INI; Untamed's are binary `.bpb`. `notes/mxvatv.md`, `notes/mxvatv-later.md`.

### Tooling

- [ ] **C7** tools/stitch at C2/C5 scale: limit each join point's candidates to the tiles near it (a spatial grid, and a radius per neighbourhood rather than twice the world's farthest gap) and measure time and join size on a 30-map world before C2 runs it → tools/stitch/ | t: tools/stitch tests — the same world built with and without the pruning is identical within the height quantum | j: stitch-scale
- [ ] **C6** tools/stitch: joins beside a tile whose spacing is not the world lattice's (3.5 ft stadium/SX maps against a 4.5 ft lattice) at that tile's own spacing, so its seams are exact at every one of its samples, not only at shared lattice points (mc06 in a 4.5 ft world steps 0.51 ft between them; notes/stitch.md, Limits) → tools/stitch/ | t: tools/stitch tests — a 3.5 ft map beside a 4.5 ft one: zero between-lattice seam points, every seam point continuous | j: stitch-mixed-spacing
- [~] **T9** the accuracy pairs' openmx half: once openmx has a track from a mod folder (MD2) and --shot-pose (MD10), set tools/accuracy pair's --openmx-args default to their real flags, settle the fov's axis with MD10 (notes/accuracy.md, Poses), and take Lago's (mxvatv_nat05) eight pairs end to end; read them and write what differs in notes/accuracy.md (after T7) → tools/accuracy/ · notes/accuracy.md | t: tools/accuracy tests — the openmx command line built from a pose | j: accuracy-pairs BLOCKED: openmx has neither MD2 (a track from a mod folder, by command-line flag) nor MD10 (--shot-pose); both open in openmx's plan/TODO.md at b63486a. Unblocks when both land: set look::OPENMX_ARGS to their flags, settle the fov axis, take Lago's 8 pairs with accuracy pair, write the differences in notes/accuracy.md
- [ ] **M17** Elevation mods' sky, fog and lights: give tools/elevation's scenes a sky (CubeFile), [Fog] and [Light<n>] taken from a disc map of the same kind (a desert map for the dunes, a forest one for Hatfield), named in mod.json and its todo, since the data has none (M1 todo)
- [ ] **M16** Elevation mods' surfaces: find which of the bike's .phy surface ids MXU's freeride ground uses for sand, dirt, rock and grass (free06–11's surface bytes against their splat layers), then give tools/elevation's maps real surface ids (dunes sand; Hatfield/Ocotillo from NAIP or a land-cover set), replacing today's all-0 (M1 todo)

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
- [ ] **C2** MX vs. ATV Unleashed giant map: every non-racing map from M8's table (no stadium with crowd seats: not SX, FS or other arenas; MC and the open national-style maps if they qualify by M8) joined by C1 into one world. Ride every seam in openmx; list each map's place in notes/mxvatv.md (after C1, M2, M8, M12) → notes/mxvatv.md | t: tools/stitch --check on the world: every seam continuous, every interior identical to its imported map | j: mxvatv-giant-map
- [ ] **C3** MX vs. ATV Untamed giant map: the same as C2 for Untamed's non-racing maps (no SX, FS, MT or other stadium maps) (after C1, M4, M13) → notes/mxvatv-later.md | t: tools/stitch --check on the world | j: untamed-giant-map
- [ ] **C4** MX Unleashed's giant map with the same tool: its freeride and other non-stadium maps from MX.PSR (free*, nat*; not sx*), imported as they are and joined by C1, so the ultra map (C5) has MXU in the same form as the others. openmx's P71 splices MXU's freeride maps inside the port; say in the journal how the two compare and whether P71 can load this world instead (after C1, T5) → notes/mxu.md | t: tools/stitch --check on the world | j: mxu-giant-map
- [ ] **C5** the ultra map: every game's giant map (C2, C3, C4, and each later game's as it lands: ATV Offroad Fury M3, Reflex/Alive M11, …) joined by C1 into one world, MX Unleashed's included. Plan the layout so each game's world stays whole and the joins between games are as smooth as those inside one. Re-run whenever a new game's giant map lands (after C2, C3, C4) → notes/ultra-map.md | t: tools/stitch --check on the world | j: ultra-map

### Stretch

Waiting on a disc in `iso/` or on a map author's permission; nothing here runs unattended.

- [ ] **M3** ATV Offroad Fury 2 (PS2, Rainbow) then 1, 3, 4 (3–4 are Climax): terrain format and free-roam areas. Rainbow's ATVOF2 may share the PSR/PLN family. `notes/atvof.md`. Needs the discs.
- [ ] **M11** MX vs. ATV Reflex / Alive (PS3/360): freeride maps; Reflex's deformable terrain is likely a heightfield. `notes/mxvatv-later.md`. Needs the discs.
- [ ] **M5** Motocross Madness 2 (PC): heightmap terrain, open Baja maps. `notes/mcm2.md`. Needs the game files.
- [ ] **M6** MX Bikes community freeride/compound maps: meshes rasterised to a height grid; overhangs and bridges become props. Only with the map author's permission. `notes/mxbikes.md`.
- [~] **M7** MX Simulator community tracks (heightmap PNG + texture, near 1:1). BLOCKED: the track site answers "PHP has been disabled on this server"; unblocks when it or a mirror is back. Author permission needed per track.
