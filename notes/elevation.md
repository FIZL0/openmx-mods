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
  52.5 ft). The scenes' fog is their donor map's (below), whose end is 768 or 1152 ft (not checked in the game).
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
- The start is the centre sample, facing north. Normals come from trnwrite's central differences.
- Surface ids come from the aerial picture (M16, *Surfaces* below): sand 4, dirt 0, grass 3.

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

## Sky, fog and lights (M17)

The data has no sky, fog or lights, so each site takes all three from one MX vs. ATV Unleashed map of the
same kind, as a set: the scene's `CubeFile`, its `[Fog]`, `[Lights]` and `[Light<n>]` sections copied line
for line (comments and unread keys included), and the `.cub` the `CubeFile` names copied byte for byte
from that map's archive to the same member path (`Skies/sky16.ps2.cub`). A set is never mixed or tuned:
the sky's lower half is painted in its own map's fog colour (game-render `sky`), so a sky with another
map's fog would show a seam at the horizon.

| site | donor (`Env\<map>\<map>.res`) | why | CubeFile | fog end |
|---|---|---|---|---|
| `elev_glamis` | Nat03 Glamis | the same place: the game's own Glamis national | sky16 | 768 ft |
| `elev_ocotillo` | Nat09 Deadman Wash | a desert wash map; its ecosystem is ocotillo, palo verde, prickly pear and barrel cactus | sky31 | 768 ft |
| `elev_littlesahara` | MC04 Glamis South | a second dune map (Nat03's sky and fog colour, its own lights and a 1152-ft fog end), so the two dune sites are not the same set | sky16 | 1152 ft |
| `elev_hatfield` | Nat15 Smuggler's Notch | Appalachian forest (Vermont), the same range as southern West Virginia; its ecosystem is five trees | sky93 | 768 ft |

The candidates were MX vs. ATV Unleashed's big outdoor maps, read for their `CubeFile`, fog, lights and
ecosystem members (tree, cactus and bush models). MX Unleashed's own maps were not used: the port already
has that disc, but a mod with a sky must hold the `.cub` either way (MD2 decides whether a mod reads disc
archives), and MX vs. ATV Unleashed has Glamis itself.

- **Permission.** A mod built with a sky holds a disc's bytes (the `.cub`) and the disc's scene lines, so
  its `permission` is `own-disc`. Built with `--sky none`, or with no MX vs. ATV Unleashed disc in `iso/`
  (`$DISCX_ISO`), the sites have no sky and are `public-domain` as before; every other file is the same
  either way.
- `mod.json` names the donor in `sky_from` (`game`, `boot`, `map`, `name`, `members`, `scene`, `cube`,
  `sections`), the tool's own field (standard §1), and in `todo`.
- **The sun's points.** `[Light2]`'s `Position` and `TargetPosition` are in the donor's 5760-ft frame
  (target at its middle, 2880, 400, 2880), not the elevation map's 6719-ft one. game-render uses only
  their direction, which is kept. What the original does with the points themselves (its shadow, for one)
  is not known, so this is in `todo`.
- The `[Environment]` keys that go with a sky but are not the sky (`EcoAmbientScale`,
  `AmountOfTransmittedLight`, `ParticleTextureMapName`, `Wind`) are not copied: they belong to the
  donor's ecosystem and particles, which the elevation mods do not have.
- The build proves it: game-render's `Sky::parse` decodes the `.cub`, and `scene::Scene::parse` reads the
  same fog numbers (bit for bit) and lights from the generated scene as from the donor's.

Rebuild a site with the command in its journal entry (`journal/2026-10-22-m1-elevation/`).

## Surfaces (M16)

**What the id is.** A terrain sample's surface byte (low seven bits; game-sim's `Block::surface`) indexes
the tire's per-surface tables, `NumberOfSurfaces = 16`. The bikes' `.phy` files name the rows in
comments, numbered from 1, so id = zone − 1:

| id | zone | the `.phy` comment (MX Unleashed) | rides differently from zone 1? |
|---|---|---|---|
| 0 | 1 | (black) Standard on-track dirt, "default when no friction zones present" | — |
| 1 | 2 | (white) edge track sludge | yes: rear drive 2.0 against 2.2, the side-force curves' scales and fall-offs |
| 2 | 3 | (red) off-track sludge | as zone 2, and rear `WheelDrag` 0.1 |
| 3 | 4 | (green) standard off track zone, "presumably grassy areas" | as zone 2 |
| 4 | 5 | (light brown) Sandy dirt, "sandy slippery section" | yes: front `SideForce` 0.12 against 0.165, softer curves |
| 5 | 6 | (ice blue) ice | yes: `SideForce` 0.07 |
| 6, 7 | 7, 8 | shallow / deep water or mud | as zone 2, with the front's `SideForce` 0.12 |
| 8–15 | 9–16 | "(black) Standard on-track dirt" | **no: equal to zone 1, key for key** |

That holds for all 224 bike `.phy` files on the MX Unleashed disc (every `Bike\*.res`, A and P, front and
rear). The four-wheelers in `machine.res` (baja, buggy, monster truck) have one row repeated 16 times.
MX vs. ATV Unleashed's `.phy` comments name 9–16 by paint colour ("(Yellow)", "(Blue)", "(Orange)",
"(F. Green)", "(D. Brown)") and all of them "Standard on-track dirt": they are artists' tags, not grips.

**What MX Unleashed's freeride ground uses.** `elevation surfaces` reads free06, 07, 09, 10 and 11 off the
disc. For every sample it counts the surface id against the painted layer that shows most there (each
layer's share is its mask's alpha times what the masks after it leave, as the GS blends them; masks read
bilinear at the sample). Rows are ids, columns the share of that id's samples under each layer:

| map | id | samples | under (most of) |
|---|---|---|---|
| free06 | 0 / 12 / 15 | 82,181 / 179,893 / 329,287 | dirt 78% / grass 65%, rock 26% / grass 87% |
| free07 | 0 / 12 / 13 / 15 | 24,451 / 192,928 / 44,765 / 329,217 | dry grass 100% / dry grass 57%, erode 34% / burnt ambers 68%, dark ash 32% / dry grass 88% |
| free09 | 0 / 12 / 13 / 15 | 9,403 / 23,302 / 229,439 / 329,217 | **sand 96%** / grass 90% / rock 39%, grass 33%, sand 15% / grass 86% |
| free10 | 0 / 12 / 13 / 15 | 11,339 / 200,957 / 49,608 / 329,457 | dirt 48% / dirt 48%, grass 36% / **rock 92%** / grass 86% |
| free11 | 0 / 9 / 12 / 14 / 15 | 7,615 / 1,412 / 110,375 / 142,742 / 329,217 | grass 87% / **sand 99%** / grass 70% / dirt 36%, grass 26% / grass 71% |

id 15 is the ring outside the middle 512 × 512 samples (329,217 = 769² − 512² on four maps). So the ids follow
the paint in places (free09's sand is 0, free11's is 9, free10's rock is 13), but not as one code, and
**every id the freeride maps use (0, 9, 12–15) rides as zone 1**. MX Unleashed's sand, rock and grass
all ride as dirt. Zone 5 (sand) appears on one map in the whole game (nat07, 0.4%, beside 11.6% ice), and
zone 4 (grass) on none: `elevation surfaces nat01 … sx24` counts every track's ids.

**What the elevation maps get.** The brief asks for real ground, so the maps go past the original here
(each one says so in `todo`): sand is id 4 (zone 5), grass id 3 (zone 4), dirt id 0. Rock is dirt, since
no zone is rock. Each sample's id comes from the NAIP picture over its square metre (the image's pixels
whose centres fall in it, averaged in all four bands; `surfaces::seen`). The test is vegetation: NDVI,
(NIR − red) / (NIR + red), averaged over a 5 × 5-sample square so a lone bush, a crown's shadow or a slip
face's few pixels do not make a sample of their own. The picture does not tell sand from dirt or rock
(all pale grey-brown from above), so the site's kind says what bare ground is:

| site | kind | green from (averaged NDVI) | bare | green | result |
|---|---|---|---|---|---|
| `elev_glamis` | dunes | 0.03 | sand | dirt | sand 99.9%, dirt 0.1% |
| `elev_littlesahara` | dunes | 0.0 | sand | dirt (sagebrush and juniper flats) | sand 64.8%, dirt 35.2% |
| `elev_ocotillo` | desert | — | dirt | dirt | dirt 100% (M18: sand 56.7%, dirt 43.3%) |
| `elev_hatfield` | forest | −0.2 | dirt | grass | grass 96.5%, dirt 3.5% (M18: water 0.9%, dirt 2.7%) |

NAIP's bands are uncalibrated counts, so NDVI sits lower than a reflectance one, differs by state and
year, and falls on shaded slopes. The thresholds are per site and were set by eye with
`elevation cover OUT.png SITE --picture RGB.png [--green T]`, which writes the classes and the picture
side by side:

- Glamis's sand is at about 0 everywhere. At 0.0 the slip faces turned to dirt (16%), and at −0.01 the
  whole map did; 0.03 keeps the dune field sand.
- Little Sahara at 0.0 follows the picture: the white dunes are sand, and the grey scrub strips and the
  south-west flats are dirt. At 0.03 only the junipers were (4%).
- Hatfield is all canopy. At 0.2 the shaded side of every hill was "bare" (47%). At −0.2 only the river, the
  road, the stream beds, the town and a little speckle in the darkest crowns are dirt.
- Ocotillo's NDVI is under 0.1 everywhere (badlands, washes, sparse creosote), so nothing is green. The
  washes are likely sandier than the badlands, but the picture does not separate them, so the site is dirt.
  (M18 separates them with the DEM: below.)

`elevation build` takes `--ground dunes|desert|forest --green T` for the same, or `--surface ID` for one id
everywhere. `mod.json`'s `elevation.surfaces` counts the samples per id.

## Surfaces past the NDVI test (M18)

The vegetation test cannot see two things: which of a desert's bare ground is sand, and water. Both
come from the DEM, with the picture's brightness and near infrared to confirm them
(`surfaces::washes`, `surfaces::water`, `Ground::classify_with_dem`). Every slope is taken on the DEM
averaged over 5 × 5 samples (central differences, degrees).

**Ocotillo's sand.** USDA's soil survey (SSURGO, area CA638, queried from Soil Data Access for the
window) names the ground's units: the fan north of the road is *Carrizo very gravelly sand* and
*Rositas loamy coarse sand*; the dissected ground south is *Badland* and *Sloping gullied land*. Its
polygons are mapped at 1:24,000, so the washes inside the badlands, 3 to 30 m wide, are not in them,
and its *very cobbly sandy loam, 30–65 % slopes* polygon in the north-east corner is mostly the same
flat fan. So SSURGO is the check, not the source; the tool does not fetch it. The rule:

- **Sand** where the slope is under 8° and either the ground is *open* (under 10 % of the 51 × 51-sample
  square round it is over 15°: a fan or a broad wash) or, inside the badlands, the sample's picture
  (red, green and blue averaged over 5 × 5) is more than 1.5 counts paler than that square's (a wash
  floor: the washes' sand is the palest ground in the picture, the mudstone darker). The answer is the
  majority of the 5 × 5 square round each sample, as with the NDVI. **Dirt** elsewhere.
- Slope alone (under 5°, with "below the 31 m mean height" for the badlands) was tried first: it got
  the fan but only scraps of the badlands' washes, whose floors are narrow and tilted; brightness on its
  own also marks the sunlit faces of every ridge, so it counts only on flat ground.
- Against SSURGO's units (the share of each unit the rule calls sand):

  | unit | name | of the map | sand |
  |---|---|---|---|
  | CeC | Carrizo very gravelly sand, 0–9 % slopes | 28.2 % | 97.3 % |
  | RsA | Rositas loamy coarse sand, 0–2 % | 0.4 % | 100 % |
  | RsC | Rositas loamy coarse sand, 2–9 % | 0.7 % | 99.4 % |
  | 1304 | Loamy-skeletal Ultic Haploxeralfs, very cobbly sandy loam, 30–65 % | 3.7 % | 86.6 % |
  | 1103 | Goldrose-Carsitas-Typic Petrocalcids, 2–60 % | 1.5 % | 38.1 % |
  | SrD | Sloping gullied land | 33.7 % | 55.7 % |
  | BaG | Badland | 31.8 % | 17.6 % |

  The sand units are sand, Badland is mostly dirt with its washes sand, and gullied land (more wash
  between lower ridges) is half and half. 1304 disagrees: the rule calls its flat part sand, and the
  picture shows it as the same fan as CeC beside it. The unit's 30–65 % slope class fits the hill at the
  window's edge, not the flat part, so this looks like the polygon's coarseness (not checked on the ground).
- The paved road (S22) across the fan is flat, open and pale: it is sand. It is 10 m wide.

**Water.** 3DEP's lidar DEMs are hydro-flattened: a river's surface is a flat band bank to bank. Water
is every 4-connected patch of samples under 0.3° of at least 200 samples whose picture has a median near
infrared under 40 and a median NDVI under 0 (water absorbs near infrared; a flat field or car park does
not). The patch, not the picture, gives the edge, since trees overhang the banks. Water is id 6, zone 7
("shallow water or mud"), on every kind of ground. On the four sites:

- Hatfield: one patch, the river in the north-west corner, 37,986 samples (0.9 %); no other
  patch of 200 flat samples exists on the map. Its grass and dirt shares fall to 96.3 % and 2.7 %.
- Glamis, Little Sahara and Ocotillo have no flat patch of 200 samples at all, so no water.

Zone 7 is the bike's grip on shallow water; nothing else of water is made (no plane, splash or
depth), and the drape shows the river as photographed. The thresholds are in `surfaces.rs` beside
the constants. `elevation cover OUT.png SITE` draws water blue.
