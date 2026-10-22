# M1: tools/elevation

Real elevation to track mods. A USGS 3DEP 1 m lidar DEM (GeoTIFF) is cut to whole root cells, one
terrain sample per DEM pixel, and turned into feet over a datum. T2's `trnwrite` writes it, and openmx's
`Terrain` reads it back and checks every sample. A NAIP aerial image is draped over the map as a
whole-map `.pst` with a one-pass `terrainscreenspacemesh.psm`, and game-render's `pst` and `splat`
read it back. All four sites are built: `elev_glamis`, `elev_ocotillo`, `elev_littlesahara` and
`elev_hatfield`. They are in `out/mods/tracks/` of this worktree, not in git. Site table, sources and
choices: `notes/elevation.md`.

## What it does

```
elevation --out STAGE [--from DIR]   build.sh's data step: every site whose files are in extracted/elevation/
elevation sites | fetch              the four sites; download their files with curl (USGS S3, Planetary Computer)
elevation build OUTDIR --id … --dem … --centre E,N [--cells W,D] [--naip …] --data … --date …
elevation info | relief | preview    look at GeoTIFFs: geography, windows ranked by relief, a hillshade PNG
```

- `geotiff.rs`: the `tiff` crate (0.10, new to the workspace) decodes. This file adds ModelPixelScale,
  ModelTiepoint (corner or centre), GeoKeys (model, EPSG, linear and vertical units) and GDAL_NODATA. A
  window reads only the strips or tiles under it.
- `dem.rs`: a mosaic of rasters on one grid (same EPSG, pixel and bands, corners whole pixels apart
  within 1e-3). Each pixel is taken from the first raster with data there. Overlaps are counted
  and compared.
- `lib.rs`: `build_mod`. It refuses geographic grids, non-square pixels, nodata in the window, and more
  than 16 cells. Rows are flipped so that z is north. The lowest whole metre is subtracted, the rest
  converted to feet, then `Grid::from_feet`, `build`, `write`, `prove_terrain` (below), the scene
  (`SceneText`), the drape, `mod.json` (with an `elevation` block giving the UTM corner, pixel, samples,
  datum and axes) and `todo`.
- `drape.rs`: bilinear onto the texture's pixel centres, median cut to 256 colours, `.pst` (game-render's
  layout, the palette in GS order), and the psm (free09's pass keys).
- `sites.rs`: the four sites' tiles, quads and windows. The `date` is the DEM file's mtime (when it was
  fetched).

## How it was proved

- `cargo test -p elevation` (`tests/generated.rs`, 7 tests, in `./check.sh`):
  - A striped LZW GeoTIFF from the `tiff` encoder and a hand-written tiled one (partial edge tiles, as
    3DEP's are tiled) read back pixel for pixel across chunk edges, with their place and keys.
  - A two-tile mosaic with 6 px of overlap reads across the seam, and counts 60 overlap pixels, 0 differing.
    A half-pixel offset is refused, and so is another pixel size.
  - A 600×400 generated dune DEM with a marker pixel, plus a 0.5 m RGB image with coloured quadrants, is
    built as 2×1 cells (513×257, two levels). The test writes the mod and runs `modcheck::check` (ok).
    It then reads every sample of every step-1 cell from the file on disk with openmx's `Terrain` and
    compares it with the DEM pixel in feet (within half a step). The marker is the highest sample at
    (406, 228), which is east and north, not mirrored. The spawn is the centre sample on the ground. The
    `.pst` decodes in game-render with the north-east quadrant red at the top right. game-render's
    `Splat::paint` gives red at the map's north-east corner and blue at the south-west.
  - One root cell (one level) without a picture passes modcheck. Rewriting replaces it, and a folder
    without a `mod.json` is never replaced.
  - Refusals: nodata in the window, a window off the DEM, a geographic grid, 17 cells, an id without `elev_`.
  - The drape takes each pixel from the first image that has it. NAIP's 0 fill counts as missing, and a
    pixel no image covers is an error.
- Mutation checks, each reverted: dropping the row flip failed the build test (`0,0: 55.16 vs 24.06`),
  and flipping the drape's row direction failed it at the colour check.
- **Inside every build** (`prove_terrain`, `prove_drape`): the written `.trn` is parsed by openmx's
  `Terrain::parse` (no pages, so no MD1 view). For every cell at step 1, every corner's local height must
  equal `height_scale × raw` bit for bit, its world height must be within half a step of the DEM's feet,
  and its surface must be the one written. The `.pst` must decode (game-render) to exactly the palette
  colours of the indexed image, and `Splat::parse` must read one pass, no blend, scale 1/extent and
  −1/extent.
- **The four real sites** (DEM tiles and NAIP quads downloaded 2026-10-22; build output from
  `./build.sh --check --out <scratch>`, `modcheck: 4 ok, 0 failed`, byte-identical to the hand-run
  builds in `out/`):

  | id | samples | datum | relief (ft) | step (ft) | worst read-back (ft) | leaves / blocks |
  |---|---|---|---|---|---|---|
  | elev_glamis | 2049² at 3.28084 ft | 60 m | 0.47–393.11 | 0.00600 | 0.00305 | 16384 / 16449 |
  | elev_ocotillo | 2049² | 26 m | 2.86–373.39 | 0.00570 | 0.00290 | 16384 / 16449 |
  | elev_littlesahara | 2049² | 1556 m | 2.25–390.06 | 0.00595 | 0.00302 | 16384 / 16449 |
  | elev_hatfield | 2049² | 233 m | 0.30–1536.69 | 0.02345 | 0.01184 | 16384 / 16449 |

  Each window lies inside one DEM tile, so no overlap was exercised on real data.
- **Registration by eye**: for Ocotillo, Little Sahara and Hatfield, a hillshade of the DEM window
  (`elevation preview --centre --size 2049 --every 2`) was set beside the decoded drape. The road and the
  isolated hill (Ocotillo), the dune crests and the track running south (Little Sahara), and the river
  (Hatfield) sit in the same places, so the two are registered and both are north-up. Glamis's drape was
  only looked at on its own (sand and dune ridges).
- `elevation fetch` was run against a copy of the data folder with one NAIP quad removed. It fetched
  the token and the quad, and the file is byte-identical to the first download.
- `./check.sh` passes (41 tests). It had failed at first, and not because of this task. The openmx checkout
  moved during the session (its P97 merge, V49), `eco::Grove::load` gained a layout and a terrain, and
  `tools/renderprobe` (M9's) stopped compiling. main already has the fix in T9's commit 2d92cf6. I
  applied that commit's renderprobe hunk unchanged (`git show 2d92cf6 -- tools/renderprobe | git apply`)
  rather than merging main. Both sides then hold the same text, and the runner's merge stays clean.

## What failed on the way, and decisions

- **The first Glamis pick was a gold mine.** The two tiles at Glamis itself (x67y366, x68y366) are flat,
  and their top-relief window is the Mesquite mine. The hillshade showed the dune belt off to the
  south-west, so three more tiles were fetched and ranked. The chosen window has 120 m of dunes, south of
  Highway 78.
- At Little Sahara the top-relief windows are the mountains east of the dunes. The window was placed by
  eye on the dune field around Sand Mountain instead.
- My first drape test sampled `Splat::paint` 10 ft from the map's edge. The port's filter wraps there,
  which mixed in the far edge's colour. The test now samples two texels in.
- **Datum.** Raw heights are unsigned with one scale per map, and Little Sahara sits at 1556 m. Absolute
  heights would cost 0.08 ft a step there, and ground below sea level could not be written at all. So the
  lowest whole metre is subtracted, and `mod.json` keeps it. The relief, the shape of the ground, is what
  is kept exactly.
- **Scale 3.280840, not 3.2808399.** The standard's check compares the scene's six-decimal text with the
  file's f32 bit for bit, so the file takes the value the text reads back as. That is 0.1 mm over 4 km.
  This is in `todo`.
- **Size 8×8 root cells** (2 km, about free06's size) by default. 16×16 would exceed the u16 block index,
  because no leaf of real ground repeats.
- **The picture goes in a whole-map psm, not in terrain pages.** The pages (MD4) are 256×256 per
  64×64-sample tile, capped at 128, so they cover about 700 m at 1 m. MXU's freeride maps colour their
  ground through `terrainscreenspacemesh.psm` whole-map masks, which game-render's `splat` already draws.
  1024² is the GS's largest texture: 2 m a pixel against NAIP's 0.6 m.
- **NAIP years**: the one nearest each lidar collection that Planetary Computer has. Glamis and Ocotillo
  use 2022 (lidar 2021), Little Sahara 2021 (lidar 2020), and Hatfield 2020 (lidar 2018; PC's WV list
  starts at 2020).
- Surfaces are all 0 and there is no sky, fog or lights. Both are in each `todo`, and both are filed as
  follow-ups.

## Not verified

- **Nothing was loaded in the game, PCSX2 or the port.** T7 has no source game to compare an elevation
  map with. The port cannot load a mod until MD2, and does not take its psm until the ask filed here
  (`notes/openmx-asks.md`, *an elevation mod's ground paint*). Whether MX Unleashed takes a 36 MB,
  16,449-block terrain, a 1 MB 1024² `.pst`, or a one-pass psm was not tested.
- **Handedness** comes from openmx's camera code (`mat::look_at`), not from the original game. If the
  game were right-handed, the maps would be mirrored north to south.
- **Vertical units and datum**: 3DEP's files carry no vertical keys. Metres above NAVD88 is the
  product's documented convention, and I did not check it in the file's metadata (there is no XML
  sidecar here).
- The drape's registration was checked by eye on three sites, not measured. Glamis's was not compared
  with its DEM.
- The windows' choice of riding ground was not checked against any trail or OHV-area map (in particular
  that Hatfield's window holds Rockhouse trails, or that Glamis's is inside the open riding area).
- Overlap handling (6 px between 3DEP tiles) is tested on generated data only. None of the four windows
  crosses a tile edge.
- Paths not exercised: DEMs in feet or US survey feet, centre-tiepoint rasters, and the
  `--texture` sizes other than 1024 and the tests' 32/64.
- The median-cut palette's quality was judged by eye on the decoded drapes only.
- The four mods' `date` is the DEM file's mtime. Rebuilding after a re-fetch changes `mod.json`, and the
  `version` is not bumped automatically.
