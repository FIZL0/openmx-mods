# TODO — open tasks, in order

One line is one task; ids are permanent. `T` tooling and the standard · `M` a map source.
Checkboxes change only through `agent-runner plan` / `done`.

Survey so far (2026-10-21): MX vs. ATV Unleashed and Untamed use MX Unleashed's container, `.res`, PLN v7
`.trn` and `.col`, and openmx's readers take them (one terrain check aside: openmx MD1). Unleashed's scenes
are the same INI; Untamed's are binary `.bpb`. `notes/mxvatv.md`, `notes/mxvatv-later.md`.

### Tooling

- [ ] **T5** `tools/discx`: a Rust crate (root workspace, the first one) that opens a disc image from `iso/`, identifies it by the boot ELF in SYSTEM.CNF (AGENTS.md's table), finds its PSR in the ISO 9660 root, and lists or extracts members by glob, `.res` members included (`discx ls ISO 'Env\*\*.res'`, `discx x ISO GLOB extracted/<game>/`, `discx res FILE.res`). Use openmx's `game_sim::archive::{Psr, Res}` by path, not a second reader; it replaces the ad-hoc dd/bsdtar steps in the notes → Cargo.toml · tools/discx/ | t: tools/discx tests — an ISO 9660 image with a small PSR and a `.res` built in the test lists and extracts byte-exact | j: discx
- [ ] **T6** `tools/survey`: per disc, every map's terrain, scene and collision through openmx's readers, one table row per map (levels, leaves, spacing, side, blocks, embedded textures, scene keys, models, collision placements, `.col` refused and why). Until openmx MD1 lands, read the terrain header's texture table itself and pass `Terrain::parse` a view without it, and say so in the output. Write the tables into notes/mxvatv.md and notes/mxvatv-later.md (all 56 and all 49 maps, every `.col`) (after T5) → tools/survey/ · notes/mxvatv.md · notes/mxvatv-later.md | t: tools/survey tests — a generated `.res` with a terrain, a scene and a `.col` gives the expected row | j: survey
- [ ] **T3** the grid limits, from MX Unleashed's own code: can its loader and terrain query take what the other games ship — 80×80 leaves at 2 levels (6400 against free06's 2304), 4.5 ft spacing, TerrainWidth 5, a non-empty texture table between the blocks and the tree? Read the `.trn` header loader (openmx journal 2026-10-15-d6, `./kit fn 0x1d9930`), the tree walks and `terrain_point_query`/`terrain_cell_read` (research/oracle.txt) for fixed-size buffers, caps and anything sized from TerrainWidth; and the edge/radar code that reads terrain +0x530..+0x53c. Also: where the height band lives (game-sim's TerrainNode.height_scale vs game-render terrain.rs's "not known" fit). Result in notes/mxu-terrain-limits.md, and anything the port must change in notes/openmx-asks.md (openmx MD3 waits on it) → notes/mxu-terrain-limits.md · notes/openmx-asks.md | t: none — a research note; every limit named with the instruction it comes from | j: mxu-terrain-limits
- [ ] **T1** `standard/OPENMX-TRACK-STANDARD.md`: what a track mod is, for openmx MD2 to load. A folder `mods/tracks/<id>/` holding `mod.json` (id, name, source game or data set, version, author/permission, `todo` of known gaps) and the files a track `.res` holds, loose: `<id>.scn`, `<id>.ps2.trn`, `Col\*.col`, textures, sky. Say which scene keys a mod must have and which the port ignores, and how a mod names its spawn. Base its shape on opentennis-mods' `standard/HST-COURT-UMPIRE-STANDARD.md` (github.com/FIZL0/opentennis-mods) (after T3) → standard/OPENMX-TRACK-STANDARD.md | t: none — the openmx MD2 test builds a mod by it | j: track-standard
- [ ] **T2** `tools/trnwrite`: a height grid (+ optional texture pages) → a PLN v7 `.ps2.trn` and a `.scn`. Build the 16×16 tree, the leaf blocks (heights, normals, surface bytes), share identical leaf blocks as the disc does (free06: 2304 leaves over 842 blocks), and write the texture table when given pages. Read the result back with openmx's `Terrain`. Round trip: every terrain on all three discs read → rewritten → read gives the same cells (heights, normals, surfaces) (after T5, T3) → tools/trnwrite/ | t: tools/trnwrite tests — generated grids round-trip; `--ignored` disc round-trip over iso/ | j: trnwrite
- [ ] **T4** `build.sh`: find the discs in `iso/` by boot ELF, run each importer, package to `out/mods/tracks/<source>_<map>/` per the standard, and check each with openmx's readers (after T1, T5) → build.sh | t: build.sh --check on whatever discs are present | j: build
- [ ] **T7** accuracy harness: compare a mod against its source game running. Set up PCSX2 for each source disc with ../ps2-decomp-kit (a kit config per game under extracted/<game>/, PINE on), find each game's terrain query and its camera in RAM (as openmx's research did for MXU: terrain_point_query, the camera), and write tools/accuracy: (1) heights — read the original's own ground height and surface at ≥1,000 points per map over PINE and compare with openmx's Terrain on the imported file, which must be exact since the terrain is copied; (2) the look — screenshot pairs from the same poses, the original (./kit shot) and openmx (--shot from a pose, openmx MD10), side by side in out/accuracy/<map>/ for a person or an agent to read. Prove it on one MX vs. ATV Unleashed map end to end (after T5, M2) → tools/accuracy/ · notes/accuracy.md | t: tools/accuracy tests on generated data; the one map's heights all equal | j: accuracy-harness

### MX vs. ATV Unleashed

- [ ] **M8** which of the 56 maps are freeride, and what each is: read every `Env\<MAP>\<MAP>.atyList` against `Env\Freeride.aty` and the other `.aty`s, each scene's SceneName, and the UI's map names (`UI\` strings or `TrackList.res`). Table in notes/mxvatv.md: map, name, activities, size; and the first mods to make (after T5) → notes/mxvatv.md | t: none — every row from a file named in the note | j: mxvatv-maps
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

### Music

### Combined maps

One giant map per game out of its non-racing maps (no stadium with crowd seats: no SX, FS or other arena
maps), their edge berms removed and the maps joined smoothly; then one ultra map out of every game's giant
map and MX Unleashed's. Inside each source map the ground stays the original's; only the joins change.

### Stretch

Waiting on a disc in `iso/` or on a map author's permission; nothing here runs unattended.

- [ ] **M3** ATV Offroad Fury 2 (PS2, Rainbow) then 1, 3, 4 (3–4 are Climax): terrain format and free-roam areas. Rainbow's ATVOF2 may share the PSR/PLN family. `notes/atvof.md`. Needs the discs.
- [ ] **M11** MX vs. ATV Reflex / Alive (PS3/360): freeride maps; Reflex's deformable terrain is likely a heightfield. `notes/mxvatv-later.md`. Needs the discs.
- [ ] **M5** Motocross Madness 2 (PC): heightmap terrain, open Baja maps. `notes/mcm2.md`. Needs the game files.
- [ ] **M6** MX Bikes community freeride/compound maps: meshes rasterised to a height grid; overhangs and bridges become props. Only with the map author's permission. `notes/mxbikes.md`.
- [~] **M7** MX Simulator community tracks (heightmap PNG + texture, near 1:1). BLOCKED: the track site answers "PHP has been disabled on this server"; unblocks when it or a mirror is back. Author permission needed per track.
