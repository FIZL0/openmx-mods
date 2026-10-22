# Real elevation (M1)

`tools/elevation` turns a USGS 3DEP lidar DEM (GeoTIFF) and a NAIP aerial image into a track mod
(`elev_<site>`). The data is public domain, so these mods may ship whole in `out/` releases. They are not
in git.

## Sources

- **DEM: USGS 3DEP 1 m** ("Digital Elevation Model (DEM) 1 meter"). Listed by the National Map's API, for
  example `https://tnmaccess.nationalmap.gov/api/v1/products?datasets=Digital%20Elevation%20Model%20(DEM)%201%20meter&bbox=W,S,E,N&outputFormat=JSON`.
  Files are on `prd-tnm.s3.amazonaws.com/StagedProducts/Elevation/1m/Projects/<project>/TIFF/`. A tile is
  10 km square in UTM (NAD83, EPSG 269xx), 10,012 pixels a side with 6 pixels of overlap on each edge,
  f32 metres (NAVD88; the files carry no vertical keys), LZW with the floating-point predictor, 512×512
  tiles, nodata −999999. The corner is a quarter millimetre off the metre (669993.999734), the same on
  every tile of a project, so tiles of one project mosaic without moving a pixel.
- **Picture: USDA NAIP**, 0.6 m, four bands (R, G, B, near infrared), u8, deflate, in the same UTM zone
  as the DEM. Fetched from Microsoft Planetary Computer: the STAC search
  `https://planetarycomputer.microsoft.com/api/stac/v1/search?collections=naip&bbox=…` lists the
  quarter quads, and the blob URL needs the anonymous token from
  `https://planetarycomputer.microsoft.com/api/sas/v1/token/naip` appended. The quads carry no nodata tag.
  Their corners outside the quad's slanted edges are 0 in every band, and the tool treats those as missing.
- **SRTM and 3DEP's arc-second products** are geographic grids. Their pixels are not square on the
  ground, so they cannot become a terrain (one spacing for x and z) without resampling, and the tool
  refuses them. All four sites have 1 m lidar, so none of them needs SRTM.

Downloads go in `extracted/elevation/dem/` and `extracted/elevation/naip/` (ignored).

## How a map is made

- One terrain sample per DEM pixel, at the pixel's centre. The terrain's spacing is the pixel in feet:
  3.280840 ft, the six-decimal value the scene writes (3.2808399 exactly; 0.1 mm over 4 km). The standard's
  check compares the scene and the file bit for bit.
- A map is `256 × cells + 1` pixels a side, which is whole root cells (TerrainWidth = cells), so the
  scene and the tree agree with no padding. The default is 8×8 cells: 2049 samples, 2048 m ≈ 6719 ft a
  side, against free06's 6912 ft. That is 16,384 leaves and 16,449 blocks (every leaf has its own relief,
  so none is shared), about 36 MB. The limits from `notes/mxu-terrain-limits.md`: at most 16 cells a side
  (4097 samples, which would be 65,536 leaves plus 257 upper blocks, past the u16 block index unless some
  were shared). The 4000-leaf draw list binds only past a fog end of about 1600 ft at this spacing (leaves are
  52.5 ft), and the scenes here have no fog section, so the game's default applies (not checked).
- **x is east, z is north**, and sample (0, 0) is the south-west corner. openmx's camera is left-handed
  (`game-render::mat::look_at`), so this is not mirrored. `mod.json`'s `elevation` block gives the
  corner's UTM coordinate, the pixel, and the datum, so a point in the map converts back to the ground:
  `E = corner.x + x / scale × pixel`, `N = corner.y + z / scale × pixel`, `elevation (m) = datum + y × 0.3048`.
- Heights: the lowest whole metre under the window (`datum`) is subtracted, the rest is converted to
  feet and quantised to 65,535 steps of the relief (`trnwrite::Grid::from_feet`). The step is 0.006 ft on
  the dunes and 0.025 ft on Hatfield-McCoy's 470 m of relief. The build reads the file back with openmx's
  `Terrain` and checks every sample of every cell against the DEM within half a step.
- The ground's colour: the NAIP image resampled (bilinear) onto a 1024×1024 texture over the whole map
  (2 m a pixel at 8 cells), cut to 256 colours (median cut), and written as a `.pst`. A
  `terrainscreenspacemesh.psm` lays it once over the map, as free09's whole-map masks are laid
  (`UScale = 1/extent`, `VScale = −1/extent`, so the first row is north). game-render's `splat` reads it.
  Whether MX Unleashed itself takes a one-pass psm and a 1 MB texture is not known.
- The start is the centre sample, facing north. Surface ids are all 0, because which of the bike's `.phy`
  surfaces is sand is not known. Normals come from trnwrite's central differences.

## The four sites (built 2026-10-22)

| id | DEM tile (project) | centre (UTM) | NAIP | datum | relief |
|---|---|---|---|---|---|
| `elev_glamis` | `USGS_1M_11_x67y365` (CA_SaltonSea_EarthMRI_2021_D21) | 676634, 3648229 (11N) | 2022 `m_3211508_nw` | 60 m | 0.5–393.1 ft |
| `elev_ocotillo` | `USGS_1M_11_x58y367` (CA_SaltonSea_EarthMRI_2021_D21) | 584090, 3666421 (11N) | 2022 `m_3311664_nw`, `m_3311656_sw` | 26 m | 2.9–373.4 ft |
| `elev_littlesahara` | `USGS_1M_12_x38y440` (UT_StatewideCenSouth_2020_A20) | 381994, 4397606 (12N) | 2021 `m_3911222_nw`, `m_3911221_ne` | 1556 m | 2.3–390.1 ft |
| `elev_hatfield` | `USGS_1M_17_x42y417` (WV_FEMAR3_Southcentral_2018_D19) | 425114, 4166421 (17N) | 2020 `m_3708118_sw` | 233 m | 0.3–1536.7 ft |

How the windows were chosen: `elevation preview` (a hillshade) to see the ground, and `elevation relief`
to rank 2 km windows by high minus low.

- **Glamis**: the two tiles at Glamis itself are flat creosote ground and the Mesquite gold mine (the
  mine is the top-relief window there). The dunes run north-west to south-east through x66/x67 ×
  y365/y366. The chosen window is the top-relief one in that belt, south of Highway 78, with 120 m of dunes.
- **Ocotillo Wells**: the badlands ridge in x58y367, the top-relief window of the tile, with a paved road
  across its north half.
- **Little Sahara**: the dune field around Sand Mountain in the tile's west half. The tile's top-relief
  windows are the mountains east of the dunes, so they were not used.
- **Hatfield-McCoy**: the top-relief window in the hills north-east of Gilbert, WV (the Rockhouse
  system's country). Whether the trails run inside it was not checked against a trail map. The lidar is
  bare earth: it shows benches and tracks under the forest that the aerial picture, which is all
  canopy, hides. The drape is the canopy's colour over bare ground.

Rebuild a site with the command in its journal entry (`journal/2026-10-22-m1-elevation/`).
