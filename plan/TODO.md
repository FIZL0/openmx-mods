# TODO — open tasks, in order

One line is one task; ids are permanent. `T` tooling and the standard · `M` a map source.
Checkboxes change only through `agent-runner plan` / `done`.

Survey so far (2026-10-21): MX vs. ATV Unleashed and Untamed use MX Unleashed's container, `.res`, PLN v7
`.trn` and `.col`, and openmx's readers take them (one terrain check aside: openmx MD1). Unleashed's scenes
are the same INI; Untamed's are binary `.bpb`. `notes/mxvatv.md`, `notes/mxvatv-later.md`.

### Tooling

- [ ] **T9** the accuracy pairs' openmx half: once openmx has a track from a mod folder (MD2) and --shot-pose (MD10), set tools/accuracy pair's --openmx-args default to their real flags, settle the fov's axis with MD10 (notes/accuracy.md, Poses), and take Lago's (mxvatv_nat05) eight pairs end to end; read them and write what differs in notes/accuracy.md (after T7) → tools/accuracy/ · notes/accuracy.md | t: tools/accuracy tests — the openmx command line built from a pose | j: accuracy-pairs

### MX vs. ATV Unleashed


### MX vs. ATV Untamed

### Real elevation

- [ ] **M1** USGS 3DEP / SRTM → a height grid → T2's writer. Glamis dunes first (~1 m lidar), then Ocotillo Wells, Little Sahara (UT), Hatfield-McCoy; ground texture from the matching NAIP image. Public domain, so these mods may ship whole in `out/` releases (not in git). Fit to the limits T3 found (after T2, T1) → tools/elevation/ | t: tools/elevation tests on a generated GeoTIFF | j: elevation

### Accuracy

A map is done when it matches the game it came from, not when it loads. Terrain, collision, placements and
scene values are copied as the disc has them, never resampled or approximated; what the port cannot show yet
goes in mod.json's `todo` and notes/openmx-asks.md.
- [ ] **M13** MX vs. ATV Untamed accuracy pass: the same as M12 for every imported Untamed map (M4), on Untamed running in PCSX2 (after M4, T7, M12) → tools/import-mxvatv/ · notes/mxvatv-later.md · notes/openmx-asks.md | t: tools/accuracy over every imported Untamed map | j: untamed-accuracy

### Music

### Combined maps

One giant map per game out of its non-racing maps (no stadium with crowd seats: no SX, FS or other arena
maps), their edge berms removed and the maps joined smoothly; then one ultra map out of every game's giant
map and MX Unleashed's. Inside each source map the ground stays the original's; only the joins change.
- [ ] **C1** tools/stitch, the map joiner: lay several imported maps out as one world without resampling any of them. Each map stays a terrain of its own, as the disc has it (spacing 3.5, 4.5 or 9 ft, its own height scale), placed at an offset in a world of tiles (openmx MD8 loads such a world). Per map, find its edge berm band (from the terrain and the scene's border/shootoff values; MXU's edge code is in openmx's oracle: terrain +0x530..), cut it, and fill each join with new ground that meets both sides with matching height and slope (a smooth blend solved over the band, rideable: no step, no slope past what the bike climbs), with collision and the surface ids of the nearer map. Choose the layout so neighbours' base heights are close (they range 50–555 ft); props in a cut band are dropped and listed. The output is one world mod per the standard (T1's world section) (after T2, T1) → tools/stitch/ | t: tools/stitch tests — two generated maps with berms join with continuous height and slope at every seam point, and each interior is byte-identical to its source | j: stitch — user, 2026-10-22: the smooth transition is the point — the join ground must blend in look as well as shape: give it texture/detail weights that fade from one map's ground to the other's across the band (for openmx MD9), never a hard edge; and the world mod names one global skybox, sun, ambient and fog (default: the first map's) for openmx MD9's Global sky setting
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
