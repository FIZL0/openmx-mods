# TODO — open tasks, in order

One line is one task; ids are permanent. `T` tooling and the standard · `M` a map source.
Checkboxes change only through `agent-runner plan` / `done`.

Survey so far (2026-10-21): MX vs. ATV Unleashed and Untamed use MX Unleashed's container, `.res`, PLN v7
`.trn` and `.col`, and openmx's readers take them (one terrain check aside: openmx MD1). Unleashed's scenes
are the same INI; Untamed's are binary `.bpb`. `notes/mxvatv.md`, `notes/mxvatv-later.md`.

### Tooling

### MX vs. ATV Unleashed
- [ ] **M19** MX vs. ATV's tricks for openmx (user, 2026-10-23: every trick MX vs. ATV adds over MX Unleashed, native in openmx, plus its whips, 360s and front flips on R1). From both discs (MXVSATV.PSR, MX08PS2.PSR; MXU's is rider\anim\stunts.ini, 57 stunts, openmx crates/game-sim/src/trick.rs): each game's stunt table (names, display names, HoldTime/SafeTime/Duration or what replaces them, scores), its gesture map (which buttons and stick make which trick: MXStuntMapping_16x9.lyt.bpb is the pause screen's chart), the rider animations each new trick plays, and how the whip, the 360 (flat spin) and the front flip are driven — the air control's torque, which buttons hold it, the limits and the landing test. The ELF parts need a Ghidra project for SLUS_211.04 (and SLUS_217.01 where it differs) under extracted/<game>/ghidra (AGENTS.md, Decompiling); prove the stunt tables by reading them back and the air moves against the game in PCSX2 (T7: a whip, a 360 and a front flip captured, the bike's attitude per frame). Write notes/tricks.md: the list of tricks MXU lacks, per game, and what openmx needs for each; add a trick-pack folder to the standard (mods/tricks/<game>/: the table and the animations as the disc has them) and build it with build.sh (after T7) → notes/tricks.md · standard/ · tools/ | t: tests reading each game's stunt table back | j: mxvatv-tricks — user, 2026-10-23: R1 is MXU's trick modifier and stays so; openmx adds the whip, 360 and front flip under R1 held only, so the pump for a backflip never starts a front flip. Write down exactly how each MX vs. ATV game separates its front flip and spin input from the preload pump and from its other R1 tricks


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
