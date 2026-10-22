# TODO — open tasks, in order

One line is one task; ids are permanent. `T` tooling and the standard · `M` a map source.
Checkboxes change only through `agent-runner plan` / `done`.

Survey so far (2026-10-21): MX vs. ATV Unleashed and Untamed use MX Unleashed's container, `.res`, PLN v7
`.trn` and `.col`, and openmx's readers take them (one terrain check aside: openmx MD1). Unleashed's scenes
are the same INI; Untamed's are binary `.bpb`. `notes/mxvatv.md`, `notes/mxvatv-later.md`.

### Tooling

- [ ] **T2** `tools/trnwrite`: a height grid (+ optional texture pages) → a PLN v7 `.ps2.trn` and a `.scn`. Build the 16×16 tree, the leaf blocks (heights, normals, surface bytes), share identical leaf blocks as the disc does (free06: 2304 leaves over 842 blocks), and write the texture table when given pages. Read the result back with openmx's `Terrain`. Round trip: every terrain on all three discs read → rewritten → read gives the same cells (heights, normals, surfaces) (after T5, T3) → tools/trnwrite/ | t: tools/trnwrite tests — generated grids round-trip; `--ignored` disc round-trip over iso/ | j: trnwrite
- [ ] **T4** `build.sh`: find the discs in `iso/` by boot ELF, run each importer, package to `out/mods/tracks/<source>_<map>/` per the standard, and check each with openmx's readers (after T1, T5) → build.sh | t: build.sh --check on whatever discs are present | j: build
- [ ] **T7** accuracy harness: compare a mod against its source game running. Set up PCSX2 for each source disc with ../ps2-decomp-kit (a kit config per game under extracted/<game>/, PINE on), find each game's terrain query and its camera in RAM (as openmx's research did for MXU: terrain_point_query, the camera), and write tools/accuracy: (1) heights — read the original's own ground height and surface at ≥1,000 points per map over PINE and compare with openmx's Terrain on the imported file, which must be exact since the terrain is copied; (2) the look — screenshot pairs from the same poses, the original (./kit shot) and openmx (--shot from a pose, openmx MD10), side by side in out/accuracy/<map>/ for a person or an agent to read. Prove it on one MX vs. ATV Unleashed map end to end (after T5, M2) → tools/accuracy/ · notes/accuracy.md | t: tools/accuracy tests on generated data; the one map's heights all equal | j: accuracy-harness

### MX vs. ATV Unleashed

- [ ] **M2** importer: `Env\<map>.res` → a mod folder per the standard. Terrain and collision copied as they are; the scene's keys diffed against every MX Unleashed scene's, and only what the port reads differently rewritten (start position, terrain file name, sky). The props (`.psg`, `.toy`, `.esb`) and textures: whatever openmx's game-render readers take, kept; the rest listed in mod.json's `todo` (after T1, T5, M8) → tools/import-mxvatv/ | t: tools/import-mxvatv tests on a generated `.res`; `--ignored` every freeride map imports and reads back | j: import-mxvatv
- [ ] **M9** the render-side formats: the terrain's 16 embedded texture pages (65,560 bytes: a 256×256 8-bit page, 24-byte header, PSX palette), `<map>_mat.res`, `.pst`/`.psm`, `.psg` models, `.toy`, `.esb` and skies, each through openmx game-render's readers on a few maps: what reads, what differs, what is new. Notes in notes/mxvatv.md, port asks in notes/openmx-asks.md (after T5) → notes/mxvatv.md · notes/openmx-asks.md | t: none — each claim with the map and member it was checked on | j: mxvatv-render-formats

### MX vs. ATV Untamed

- [ ] **M10** decode BPB, Untamed's binary INI (`.scn.bpb`, `.toy.bpb`, `.rpy.bpb`; 7,697 members): `BPB\0`, a header, 12-byte records (a 32-bit name hash, a string-table offset, two 16-bit fields), a string table of the INI's section and key names. Get the header, the record, the value encodings and the hash from SLUS_217.01's reader (set up a Ghidra project for it with ../ps2-decomp-kit under extracted/untamed/ghidra), and write `tools/bpb` (BPB → INI text). Proof: every `.scn.bpb` on the disc decodes to an INI that openmx's `Scene::parse` reads, and every record's hash recomputes from its name (after T5) → tools/bpb/ · notes/mxvatv-later.md | t: tools/bpb tests — a BPB built in the test decodes; `--ignored` all 7,697 on the disc | j: bpb
- [ ] **M4** Untamed importer: M2's importer with M10's decoder in front, for Untamed's freeride and national maps; Untamed's `.col` (1,604) through openmx's reader first (after M2, M10) → tools/import-mxvatv/ | t: `--ignored` every chosen Untamed map imports and reads back | j: import-untamed

### Real elevation

- [ ] **M1** USGS 3DEP / SRTM → a height grid → T2's writer. Glamis dunes first (~1 m lidar), then Ocotillo Wells, Little Sahara (UT), Hatfield-McCoy; ground texture from the matching NAIP image. Public domain, so these mods may ship whole in `out/` releases (not in git). Fit to the limits T3 found (after T2, T1) → tools/elevation/ | t: tools/elevation tests on a generated GeoTIFF | j: elevation

### Accuracy

A map is done when it matches the game it came from, not when it loads. Terrain, collision, placements and
scene values are copied as the disc has them, never resampled or approximated; what the port cannot show yet
goes in mod.json's `todo` and notes/openmx-asks.md.
- [ ] **M12** MX vs. ATV Unleashed accuracy pass: every imported map (M2) through T7 — heights exact at ≥1,000 points, and screenshot pairs from at least five poses each (spawn, the map's landmarks, a ridge looking across). Fix every difference in the importer (a prop missing, a texture wrong, fog, sky, lights, ecosystem density); what only the port can fix goes to notes/openmx-asks.md and the map's mod.json todo. A table per map in notes/mxvatv.md: matches / differs / why (after M2, T7) → tools/import-mxvatv/ · notes/mxvatv.md · notes/openmx-asks.md | t: tools/accuracy over every imported map | j: mxvatv-accuracy
- [ ] **M13** MX vs. ATV Untamed accuracy pass: the same as M12 for every imported Untamed map (M4), on Untamed running in PCSX2 (after M4, T7, M12) → tools/import-mxvatv/ · notes/mxvatv-later.md · notes/openmx-asks.md | t: tools/accuracy over every imported Untamed map | j: untamed-accuracy

### Music
- [ ] **A1** music from MX vs. ATV Unleashed and Untamed as music mods. Unleashed: 20 songs, Aud\Music\*.vig in MXVSATV.PSR, titles and artists in Aud\Music\music.pb; Untamed: 22 Aud\Music\*.vig in MX08AUD.PSR (its own PSR, which openmx's Psr reads) with its playlist in that container's .bpb/.txt/.ini (M10 decodes BPB). Check each .vig against openmx's reader (crates/openmx/src/audio.rs: the 0x800-byte header, stereo, rate) and the titles against what each game's jukebox shows; package to out/mods/music/<game>/ per the standard's music section (T1): the streams as they are plus a playlist with title, artist and source. Never in git. Accuracy: the same samples the game plays, no re-encode (after T5, T1) → tools/import-music/ · notes/music.md | t: tools/import-music tests on a generated .vig and playlist; --ignored every song on both discs packages and reads back with openmx's reader | j: music-mods

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
