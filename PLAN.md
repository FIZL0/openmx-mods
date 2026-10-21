# PLAN

One line is one task. Ids are permanent. `T` tooling and the standard · `M` a map source.
Do T1–T3 first: every source ends in the same writer, and the real-elevation import (M1) is how it is proved.

## Tooling

- [ ] **T1** Write `standard/OPENMX-TRACK-STANDARD.md`: what a freeride track mod is (`mod.json` with id,
  name, source, `todo`; a height grid with spacing and height scale; ground texture; optional props as
  `.col` + placements; spawn). Base it on opentennis-mods' `standard/HST-COURT-UMPIRE-STANDARD.md`.
- [ ] **T2** `tools/trnwrite`: Rust, a height grid + texture → `.trn` + `.scn`, using `../openmx` `game-sim`
  to read the result back. Produce the leaf tree, normals and surface bytes; share identical leaf blocks the
  way the disc does (`free06`: 2304 leaves over 842 blocks). Round-trip test: read `free06`, write it, read
  it again, same heights.
- [ ] **T3** Find the grid limits: the largest leaf count and spacing the game's loader and the port take, and
  where the per-track height band lives (`terrain.rs`: not in `.trn`, `.scn` or `.toy`; fitted per track).
  This decides how much of each source map fits, and whether the standard carries its own scale.
- [ ] **T4** `build.sh`: find sources under `iso/` by disc ID (as opentennis-mods does), unpack, run each
  importer, package to `out/mods/<source>_<map>/`.

## Sources

- [ ] **M1** Real elevation: USGS 3DEP GeoTIFF / SRTM → height grid. Glamis dunes first (sand, ~1 m data),
  then Ocotillo Wells, Little Sahara (UT), Hatfield-McCoy. Ground texture from the matching NAIP aerial
  image. Public domain, so these mods can ship whole. (after T2)
- [ ] **M2** MX vs ATV Unleashed (PS2): Rainbow Studios, same studio as MX Unleashed. First check its disc for
  `.res`/`.trn`/`.scn` lookalikes; if they match, its freeride areas need only a repack. `notes/mxvatv.md`.
- [ ] **M3** ATV Offroad Fury 2 (PS2, Rainbow) then 1, 3, 4 (3–4 are Climax): find the terrain format and
  the free-roam areas. `notes/atvof.md`.
- [ ] **M4** MX vs ATV Untamed / Reflex / Alive: freeride maps. Reflex has deformable terrain, so likely a
  heightfield. `notes/mxvatv-later.md`.
- [ ] **M5** Motocross Madness 2 (PC): heightmap terrain, open Baja maps. `notes/mcm2.md`.
- [ ] **M6** MX Bikes community freeride/compound maps: meshes, so rasterise to a height grid; overhangs and
  bridges become props. Only with the map author's permission. `notes/mxbikes.md`.
- [~] **M7** MX Simulator community tracks (heightmap PNG + texture, near 1:1). Blocked: the track site
  answers "PHP has been disabled on this server"; unblocks when it or a mirror is back. Author permission
  needed per track.
