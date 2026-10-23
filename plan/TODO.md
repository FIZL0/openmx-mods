# TODO — open tasks, in order

One line is one task; ids are permanent. `T` tooling and the standard · `M` a map source.
Checkboxes change only through `agent-runner plan` / `done`.

Survey so far (2026-10-21): MX vs. ATV Unleashed and Untamed use MX Unleashed's container, `.res`, PLN v7
`.trn` and `.col`, and openmx's readers take them (one terrain check aside: openmx MD1). Unleashed's scenes
are the same INI; Untamed's are binary `.bpb`. `notes/mxvatv.md`, `notes/mxvatv-later.md`.

### Tooling

- [ ] **M20** Untamed's 360 and backflip in PCSX2 (M19 follow-up): which PS2 button is MAPKEY_STUNT_MOD (SLUS_217.01 0x4fd630 reads the key set's +0x18), and a capture of each on a lip with the bike's attitude per frame, as M19 did for MX vs. ATV Unleashed (notes/tricks.md, extracted/mxvatv/watch/air.toml); also a clean landing of a 360 on a jump long enough for one, and the wreck reason a landing mid-clip gives (0xb or 0xf)
- [ ] **M22** import-mxvatv's find_suffix todo after openmx c66d022 (found in M21): openmx's Res::find_suffix now prefers a member the name matches whole and falls back to a plain suffix only when none does. suffix_picks (tools/import-mxvatv/src/lib.rs) still says 'a plain suffix, no path boundary', and notes/openmx-asks.md's 'MD2, lookups on a segment boundary' (T11) is half answered: the wrong-file case (thq.psm → BaleTHQ.psm) is gone, but the stand-in case (treebb.psm → BigTreeBB.psm when the map has no TreeBB.psm) remains. M21 only changed the two tests to expect the new lookup. Fix the comment, rewrite the ask to the stand-in case, and re-import the maps whose todo named a wrong file → tools/import-mxvatv/src/lib.rs · notes/openmx-asks.md | t: cargo test -p import-mxvatv | j: suffix-after-c66d022
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
