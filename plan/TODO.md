# TODO — open tasks, in order

One line is one task; ids are permanent. `T` tooling and the standard · `M` a map source.
Checkboxes change only through `agent-runner plan` / `done`.

Survey so far (2026-10-21): MX vs. ATV Unleashed and Untamed use MX Unleashed's container, `.res`, PLN v7
`.trn` and `.col`, and openmx's readers take them (one terrain check aside: openmx MD1). Unleashed's scenes
are the same INI; Untamed's are binary `.bpb`. `notes/mxvatv.md`, `notes/mxvatv-later.md`.

### Tooling

- [ ] **M20** Untamed's 360 and backflip in PCSX2 (M19 follow-up): which PS2 button is MAPKEY_STUNT_MOD (SLUS_217.01 0x4fd630 reads the key set's +0x18), and a capture of each on a lip with the bike's attitude per frame, as M19 did for MX vs. ATV Unleashed (notes/tricks.md, extracted/mxvatv/watch/air.toml); also a clean landing of a 360 on a jump long enough for one, and the wreck reason a landing mid-clip gives (0xb or 0xf)
- [ ] **M21** do MX vs. ATV's 360 and backflip run at once? (user, 2026-10-23: openmx should do 360 backflips and 360 front flips, openmx MD20). From SLUS_211.04 (0x4e53d0 the 360, 0x4e4f20 the backflip) and Untamed (0x4fd630, 0x4fd0d0): whether one arming excludes the other (the stick's x for the 360 and y for the backflip on the same R1 + lip, a diagonal), whether both torques go into the body on one frame, and how the two landing tests (35 degrees of heading, the backflip's upright clip) combine. Then try it in PCSX2 (T7's capture: R1 + stick back and to the side off a big lip, the bike's attitude per frame) and say whether the game does a 360 backflip, and if not what stops it. notes/tricks.md, a section 'Both at once' (after M19) → notes/tricks.md | t: none (research; the capture's rows are the proof) | j: mxvatv-360-flips
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
