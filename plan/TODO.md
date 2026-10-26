# TODO — open tasks, in order

One line is one task; ids are permanent. `T` tooling and the standard · `M` a map source.
Checkboxes change only through `agent-runner plan` / `done`.

Survey so far (2026-10-21): MX vs. ATV Unleashed and Untamed use MX Unleashed's container, `.res`, PLN v7
`.trn` and `.col`, and openmx's readers take them (one terrain check aside: openmx MD1). Unleashed's scenes
are the same INI; Untamed's are binary `.bpb`. `notes/mxvatv.md`, `notes/mxvatv-later.md`.

### Tooling

- [ ] **M20** Untamed's 360 and backflip in PCSX2 (M19 follow-up): which PS2 button is MAPKEY_STUNT_MOD (SLUS_217.01 0x4fd630 reads the key set's +0x18), and a capture of each on a lip with the bike's attitude per frame, as M19 did for MX vs. ATV Unleashed (notes/tricks.md, extracted/mxvatv/watch/air.toml); also a clean landing of a 360 on a jump long enough for one, and the wreck reason a landing mid-clip gives (0xb or 0xf)
### MX vs. ATV Unleashed
- [ ] **T12** MX vs. ATV Unleashed's PC install as a source beside its PS2 ISO (openmx MD38, user 2026-10-26): discx, import-tricks and import-mxvatv take the Steam install's data/*.pak (a directory, or the paks themselves) wherever they take SLUS-21104's ISO now, read through one shared pak reader in the format openmx's tools/mxvatv_pak.py documents (its ls|x|check; research/mxvatv-pc.md: the paks hold the disc's sim data byte for byte). Source picked by what the path is, ISO stays the default; files the PC paks lack or hold differently are named in notes/mxvatv.md, not guessed → tools/discx · tools/import-tricks · tools/import-mxvatv · notes/mxvatv.md | t: with both sources present (~/repos/openmx/context/mxvatv-pc/data, iso/), every file the import tools read comes out byte-identical from either, and an import-mxvatv map and import-tricks pack built from the paks equal the ISO-built ones; tests skip, saying so, when the PC install is absent | j: mxvatv-pc-paks


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
