# TODO — open tasks, in order

One line is one task; ids are permanent. `T` tooling and the standard · `M` a map source.
Checkboxes change only through `agent-runner plan` / `done`.

Survey so far (2026-10-21): MX vs. ATV Unleashed and Untamed use MX Unleashed's container, `.res`, PLN v7
`.trn` and `.col`, and openmx's readers take them (one terrain check aside: openmx MD1). Unleashed's scenes
are the same INI; Untamed's are binary `.bpb`. `notes/mxvatv.md`, `notes/mxvatv-later.md`.

### Tooling

- [ ] **M20** Untamed's 360 and backflip in PCSX2 (M19 follow-up): which PS2 button is MAPKEY_STUNT_MOD (SLUS_217.01 0x4fd630 reads the key set's +0x18), and a capture of each on a lip with the bike's attitude per frame, as M19 did for MX vs. ATV Unleashed (notes/tricks.md, extracted/mxvatv/watch/air.toml); also a clean landing of a 360 on a jump long enough for one, and the wreck reason a landing mid-clip gives (0xb or 0xf)
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
- [ ] **M23** stitch cuts each map at its ridge (openmx MD27, 2026-10-23): a big world's maps meet peak to peak, not across two tiles' shootoff plateaus. find_keep stops at the berm (MXU free keep [17,17,751,751]), so every join is ~675 ft of flat 190-ft plateau with a ridge each side; the cut belongs at the profile's peak past the berm (MXU free d=87/88, ~250 ft). A prototype (find_keep moves the cut to the highest point within 1200 ft that stands >10 ft over the berm) gives keep [88,87,681,680] and one ridge in openmx's shot, but: tile at goes negative (-432) and openmx refuses it, so shift the world; the joins then leave the outer band x/z 0..432 uncovered (39 holes in openmx's new 'world holes:' line, each throws within 150 ft); the ridge-top join is ~420 ft flat (narrow it: min_gap); seams reach slope 2.27 against 0.36 elsewhere; the drop list grows to ~45 placements in the new band. Then rebuild out/mods (its world_mxu_free predates C11's fix and has 84 holes). openmx's side landed in MD27: a tile's collision shapes and models outside its keep are cut, the shell prints 'world holes:' and 'thrown: tick N at x, z', and crates/openmx/tests/world_joins.rs has an ignored ride test over $OPENMX_WORLDS/tracks/world_mxu_free (every join crossed, every outer side thrown) to run on the rebuilt world.

### Stretch

Waiting on a disc in `iso/` or on a map author's permission; nothing here runs unattended.

- [ ] **M3** ATV Offroad Fury 2 (PS2, Rainbow) then 1, 3, 4 (3–4 are Climax): terrain format and free-roam areas. Rainbow's ATVOF2 may share the PSR/PLN family. `notes/atvof.md`. Needs the discs. — reopened: T10, C11 landed
- [ ] **M11** MX vs. ATV Reflex / Alive (PS3/360): freeride maps; Reflex's deformable terrain is likely a heightfield. `notes/mxvatv-later.md`. Needs the discs. — reopened: T10, C11 landed
- [ ] **M5** Motocross Madness 2 (PC): heightmap terrain, open Baja maps. `notes/mcm2.md`. Needs the game files. — reopened: T10, C11 landed
- [ ] **M6** MX Bikes community freeride/compound maps: meshes rasterised to a height grid; overhangs and bridges become props. Only with the map author's permission. `notes/mxbikes.md`. — reopened: T10, C11 landed
- [~] **M7** MX Simulator community tracks (heightmap PNG + texture, near 1:1). BLOCKED: the track site answers "PHP has been disabled on this server"; unblocks when it or a mirror is back. Author permission needed per track.
