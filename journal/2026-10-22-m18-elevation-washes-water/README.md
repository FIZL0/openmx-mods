# M18: Ocotillo's sandy washes, and water

M16 gave the elevation maps their surface ids from NAIP's vegetation index. That test could not tell
Ocotillo's sandy washes from its badlands (nothing there is green), and it left Hatfield's river as dirt.
This task tells both apart using the DEM, with the picture's brightness and near infrared to confirm.
The full account is in `notes/elevation.md`, *Surfaces past the NDVI test (M18)*.

## Results

| site | before (M16) | now |
|---|---|---|
| `elev_ocotillo` | dirt 100% | sand 56.7%, dirt 43.3% |
| `elev_hatfield` | grass 96.5%, dirt 3.5% | grass 96.3%, dirt 2.7%, water 0.9% (37,986 samples) |
| `elev_glamis` | sand 99.9%, dirt 0.1% | unchanged |
| `elev_littlesahara` | sand 64.8%, dirt 35.2% | unchanged |

## What changed

- `tools/elevation/src/surfaces.rs`:
  - `WATER = 6` (zone 7, "shallow water or mud") and `Cover::Water`.
  - `box_mean`: a summed-area mean that skips samples with no data.
  - `slopes`: the slope in degrees, on the DEM averaged over 5 × 5 samples.
  - `washes`: the desert's sand. A sample is sand where it is flat (under 8°) and either open (under 10%
    of the 51 m square round it is over 15°) or paler than that square by 1.5 counts. The answer is then a
    5 × 5 majority vote.
  - `water`: 4-connected flat patches (under 0.3°) of at least 200 samples, with median NIR under 40
    and median NDVI under 0.
  - `Ground::classify_with_dem`: M16's `classify`, then the washes (desert only), then water (every
    kind of ground). `classify` itself is unchanged.
- `lib.rs`:
  - `build_mod` hands the window's heights in metres to `classify_with_dem`.
  - The surfaces `todo` line explains the desert rule and the water, and names zone 7.
  - `unit_m` is now `pub`, for `cover`.
  - mod.json `version` is now 1.3.0.
- `main.rs`: `elevation cover` reads the DEM window too, draws water blue and counts it.
- `notes/elevation.md`: the M16 table and the Ocotillo bullet point to the new section. The new section
  has the rule, the SSURGO check table and the water findings.

## How it was proved

- `cargo test -p elevation`: 15 pass (7 + 2 + 6). Three of them are new, in `tests/surfaces.rs`:
  - A generated desert with an open flat, ridged badlands, a pale flat wash and a dark flat bench.
    The flat and the wash are sand; the bench and the ridges are dirt. With the picture made uniform,
    the wash is dirt.
  - Generated water: a dark flat river becomes water. These do not: a flat green field, a flat pale car
    park (bare but bright in NIR), a dark puddle of 50 samples, and sloping ground.
  - A built terrain with a flat river band, read back with openmx's `Terrain`. Every sample 5 rows
    inside the band is id 6, and none 5 rows outside it is. The `todo` names zone 7 and the
    hydro-flattening.
  - The M16 build test now expects desert sand: its generated ground is an open 0.6° flat.
- Mutation checks, each reverted (`diff` against a saved copy):
  - Dropping the dark-NIR condition fails the water test (the car park) and two others.
  - Dropping the paler branch fails the desert test.
  - Dropping the open branch fails the desert test and the build test.
  - Not writing `Cover::Water` fails both water tests.
  - Not smoothing the slope fails nothing; see Not verified.
- All four sites were built with
  `DISCX_ISO=/home/rycs/repos/openmx-mods/iso target/release/elevation --out <scratch>`: exit 0, no FAIL,
  every sample's id read back through `Terrain` in `prove_terrain`, and the mod.json counts above.
- By eye: `elevation cover` for Ocotillo and Hatfield beside the NAIP picture and a slope picture.
  Ocotillo's fan and the pale braided washes in the badlands are sand. Hatfield's water follows the
  DEM's flat band, which is wider and cleaner than the dark pixels in the picture, because trees
  overhang the banks.
- SSURGO: I queried Soil Data Access (`sdmdataaccess.sc.egov.usda.gov`, a read-only public service) for
  the map units and polygons over Ocotillo's window and rasterised them in a scratch script.
  - The share of each unit called sand is in the notes: the sand units 97–100%, Badland 17.6%, gullied
    land 55.7%.
  - Point queries agree: the fan is *Carrizo very gravelly sand*, the badlands *Badland*.
  - The scripts and the polygons are in the session scratchpad, not in git.
- The thresholds were explored with numpy on a dump of each window (DEM plus NAIP means), written by a
  throwaway example binary that I deleted before committing. The Rust then gave the same shares as the
  prototype (Ocotillo 56.7%, Hatfield water 0.9%).

## Tried and dropped

- **Slope plus "below the local mean height" for washes** (slope under 5°, more than 0.3 m under the
  31 m mean). It got the fan but only scraps of the badlands' washes: the floors are narrow and tilted,
  and the 5 × 5 vote removes them.
- **Brightness on its own.** The sunlit side of every ridge is as pale as a wash. Brightness counts only
  on flat ground.
- **SSURGO as the source.** Its polygons (1:24,000) do not resolve the washes inside Badland and gullied
  land. Its cobbly-loam polygon covers flat fan as well as the hill it is named for. It would also need
  a new fetch, a WKT reader and a UTM projection, for a coarser answer than the DEM gives. It is used as
  the check.
- **Water from the picture alone** (NIR under 60 on flat ground). This gave a ragged river: the banks'
  overhanging trees are bright in NIR. The DEM's flat patch gives the lidar's bank.

## Not verified

- **How it rides.** Nothing was run in PCSX2 or the port (T7). I have not checked whether zone 5 in the
  washes and zone 7 on the river feel right, or are noticeable.
- **Water past the original.** Zone 7 is the bike's grip only. There is no water plane, splash, depth or
  sound. What else the game does with id 6 (sound, particles, AI) was not decompiled; M16 left the
  same open question for its ids.
- **Ground truth.** The thresholds were set by eye on one site each: the desert rule on Ocotillo, the
  water rule on Hatfield. SSURGO checks the coarse split only. Nobody has checked whether a given wash's
  floor is really sand on the ground.
- **The road** across Ocotillo's fan is classed as sand (flat, open and pale). I left it, because the
  tool does not map roads.
- **The slope's 5 × 5 smoothing** is not guarded by a test: removing it changes no test result. It is
  a tuning choice that the generated shapes cannot see. The real-site shares were measured with it.
- **Other sites.** The water rule found nothing at Glamis, Little Sahara or Ocotillo. Those sites have
  no flat patch of 200 samples, so the dark and NDVI conditions were never tested there. A site with a
  lake, or a sunlit flat car park, has not been tried.
- **`build.sh --check`** was not run in full. I ran the elevation step directly, and it is the only step
  this task changes.
