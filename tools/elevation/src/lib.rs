//! Real elevation to a track mod: a lidar DEM (USGS 3DEP, GeoTIFF) cut to a window of whole root cells,
//! turned into feet and written by T2's `trnwrite`, with an aerial image (NAIP) draped over it as the
//! ground's colour, and proved by reading everything back with openmx's readers.
//!
//! What is kept and what is not (the accuracy rule, AGENTS.md):
//!
//! - **The grid is the DEM's.** One terrain sample is one DEM pixel, at the pixel's centre; nothing is
//!   resampled, rotated or reprojected. The terrain's spacing is the pixel's size in feet. A grid whose
//!   pixels are not square on the ground (a geographic one, SRTM's or 3DEP's arc-second products) is
//!   refused, since it could only be used by resampling it.
//! - **x is east, z is north.** openmx's camera (`game-render::mat::look_at`: x = up × forward) is
//!   left-handed, so looking along +z with +x to the right is looking north with east to the right: the
//!   map is not mirrored. Sample row z = 0 is the window's southern row, which is the DEM's last.
//! - **Heights are the DEM's, in feet, above a datum.** The terrain's raw heights are unsigned 16-bit
//!   with one scale per map, so the lowest whole metre (or foot) under the window is subtracted first
//!   and kept in `mod.json`. The relief is then quantised to 65,535 steps (`trnwrite::Grid::from_feet`).
//! - **The picture is resampled**, unavoidably: the image's pixels are not the terrain's, and the game
//!   draws 8-bit textures. Bilinear onto the texture's pixel centres, then median cut to 256 colours.

pub mod dem;
pub mod drape;
pub mod geotiff;
pub mod sites;
pub mod sky;
pub mod surfaces;

use dem::{Mosaic, Overlap};
use drape::Rgb;
use game_sim::track::Terrain;
use geotiff::{FOOT, GeoTiff, METRE, US_FOOT};
use std::path::{Path, PathBuf};
use trnwrite::scene::SceneText;
use trnwrite::{Grid, Options, build, write};

/// One international foot, in metres.
pub const FOOT_M: f64 = 0.3048;

/// What to build.
#[derive(Clone, Debug)]
pub struct Request {
    /// The mod's id (`elev_<map>`), which is also the stem of its scene and terrain (§2).
    pub id: String,
    pub name: String,
    pub dem: Vec<PathBuf>,
    /// Aerial images in the DEM's projection, the first that covers a pixel colouring it. None: no drape.
    pub naip: Vec<PathBuf>,
    /// The window's centre, in the DEM's model coordinates (UTM easting, northing).
    pub centre: (f64, f64),
    /// Root cells across (x, east) and along (z, north): `256 × cells + 1` samples a side.
    pub cells: (u32, u32),
    /// The drape's size in pixels a side.
    pub texture: u32,
    /// The surface id written on every sample, unless `ground` classifies them.
    pub surface: u8,
    /// What kind of place it is and the averaged NDVI from which its ground is green (M16): each
    /// sample's surface id from the aerial picture (`surfaces::Ground::classify`). None, or no picture:
    /// `surface` everywhere.
    pub ground: Option<(surfaces::Ground, f32)>,
    /// `source.data` and `source.date` (§1).
    pub data: String,
    pub date: String,
    /// The disc map whose sky, fog and lights the scene takes (M17). None: the scene has none, and the
    /// mod holds only public-domain data.
    pub sky: Option<sky::Donor>,
}

/// What the build found, for the log, `mod.json` and the journal.
#[derive(Clone, Debug, PartialEq)]
pub struct Facts {
    pub epsg: Option<u16>,
    /// Samples across and along.
    pub samples: (u32, u32),
    /// The DEM pixel (column, row) under sample (0, 0) (the south-west corner), in the first DEM's frame.
    pub corner_pixel: (i64, i64),
    /// The model coordinate of sample (0, 0)'s centre.
    pub corner: (f64, f64),
    /// The DEM's pixel size in its own units, and in feet as the terrain's scale.
    pub pixel: f64,
    pub scale: f32,
    /// Metres (or the DEM's vertical unit) subtracted from every height, and that unit in metres.
    pub datum: f64,
    pub vertical_unit_m: f64,
    /// Lowest and highest heights over the datum, in feet.
    pub low_ft: f32,
    pub high_ft: f32,
    pub height_scale: f32,
    /// One raw step in feet, and the largest difference found reading the terrain back.
    pub step_ft: f64,
    pub worst_ft: f64,
    pub overlap: Overlap,
    pub leaves: usize,
    pub blocks: usize,
    pub spawn: [f32; 3],
    /// Samples by surface id, ascending.
    pub surfaces: Vec<(u8, u64)>,
}

/// A built mod: its files (paths in the folder, `/`-separated) and the facts.
pub struct Built {
    pub files: Vec<(String, Vec<u8>)>,
    pub facts: Facts,
    pub todo: Vec<String>,
}

pub fn unit_m(code: Option<u16>, what: &str) -> Result<f64, String> {
    match code {
        // 3DEP's tiles say metre for the horizontal and nothing for the vertical: its product
        // specification is metres above NAVD88.
        None | Some(METRE) => Ok(1.0),
        Some(FOOT) => Ok(FOOT_M),
        Some(US_FOOT) => Ok(1200.0 / 3937.0),
        Some(c) => Err(format!(
            "{what} unit code {c} is not metre, foot or US survey foot"
        )),
    }
}

/// The DEM window: `samples` heights, rows south to north, x east, in the DEM's units, and where it is.
pub fn cut(
    m: &mut Mosaic,
    centre: (f64, f64),
    samples: (u32, u32),
) -> Result<(Vec<f32>, (i64, i64), Overlap), String> {
    let g = m.geo.clone();
    if g.model == 2 {
        return Err(
            "a geographic (latitude/longitude) grid: its pixels are not square on the ground, and \
                    making them so would resample it"
                .into(),
        );
    }
    if g.dx != g.dy {
        return Err(format!(
            "pixels of {} × {}: the terrain has one spacing",
            g.dx, g.dy
        ));
    }
    if g.bands != 1 {
        return Err(format!("{} bands: a DEM has one", g.bands));
    }
    let (ic, jc) = m.pixel_of(centre.0, centre.1);
    let (nx, nz) = samples;
    let (i0, j0) = (ic - i64::from(nx / 2), jc - i64::from(nz / 2));
    let (px, overlap) = m.window(i0, j0, nx, nz)?;
    let missing = px.iter().filter(|v| v.is_nan()).count();
    if missing > 0 {
        return Err(format!(
            "{missing} of the window's {} pixels have no data (outside the DEMs or nodata): move it or add tiles",
            px.len()
        ));
    }
    // Flip the rows: sample row z is DEM row j0 + nz - 1 - z.
    let mut out = Vec::with_capacity(px.len());
    for z in 0..nz as usize {
        let row = nz as usize - 1 - z;
        out.extend_from_slice(&px[row * nx as usize..(row + 1) * nx as usize]);
    }
    Ok((out, (i0, j0 + i64::from(nz) - 1), overlap))
}

/// Where a request's window lies without reading it: sample (0, 0)'s centre in model coordinates, the
/// DEM's pixel in its units, the samples a side, and the EPSG code. The same window [`cut`] reads.
pub fn window_of(req: &Request) -> Result<((f64, f64), f64, (u32, u32), Option<u16>), String> {
    let m = Mosaic::open(&req.dem)?;
    let g = m.geo.clone();
    let samples = (256 * req.cells.0 + 1, 256 * req.cells.1 + 1);
    let (ic, jc) = m.pixel_of(req.centre.0, req.centre.1);
    let (i0, j0) = (ic - i64::from(samples.0 / 2), jc - i64::from(samples.1 / 2));
    let corner = (g.x(i0 as f64), g.y((j0 + i64::from(samples.1) - 1) as f64));
    Ok((corner, g.dx, samples, g.epsg))
}

/// The terrain's spacing for a pixel of `metres`: the f32 that the scene's `TerrainWidthScale` (written
/// with six decimals by `trnwrite::scene`) reads back as, so the scene and the file agree exactly
/// (the standard's §5 check compares them bit for bit). 1 m gives 3.280840 ft against 3.2808399: 0.1 mm
/// over 4 km.
pub fn spacing_ft(metres: f64) -> f32 {
    format!("{:.6}", (metres / FOOT_M) as f32).parse().unwrap()
}

/// Build the mod's files.
pub fn build_mod(req: &Request) -> Result<Built, String> {
    if !req.id.starts_with("elev_")
        || req.id.len() > 40
        || !req
            .id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    {
        return Err(format!(
            "id {:?}: `elev_` and [a-z0-9_], at most 40 (§1)",
            req.id
        ));
    }
    if !(1..=16).contains(&req.cells.0) || !(1..=16).contains(&req.cells.1) {
        return Err(format!(
            "{:?} root cells: 1 to 16 a side (notes/mxu-terrain-limits.md)",
            req.cells
        ));
    }
    let mut m = Mosaic::open(&req.dem)?;
    let g = m.geo.clone();
    let h_unit = unit_m(g.linear_units, "horizontal")?;
    let v_unit = unit_m(g.vertical_units, "vertical")?;
    let samples = (256 * req.cells.0 + 1, 256 * req.cells.1 + 1);
    let (raw, corner_pixel, overlap) = cut(&mut m, req.centre, samples)?;

    let pixel_m = g.dx * h_unit;
    let scale = spacing_ft(pixel_m);
    let low = raw.iter().copied().fold(f32::INFINITY, f32::min);
    let datum = f64::from(low).floor();
    let feet: Vec<f32> = raw
        .iter()
        .map(|v| ((f64::from(*v) - datum) * v_unit / FOOT_M) as f32)
        .collect();
    let (mut grid, hs) = Grid::from_feet(samples.0, samples.1, &feet, scale)?;
    let corner = (g.x(corner_pixel.0 as f64), g.y(corner_pixel.1 as f64));
    let classified = match req.ground {
        Some((ground, green)) if !req.naip.is_empty() => {
            let seen = surfaces::seen(&req.naip, corner, g.dx, samples, g.epsg)?;
            let metres: Vec<f32> = raw.iter().map(|v| (f64::from(*v) * v_unit) as f32).collect();
            let covers = ground.classify_with_dem(&seen, &metres, samples.0 as usize, pixel_m, green);
            grid.surfaces = covers.iter().map(|c| c.id()).collect();
            Some((ground, green))
        }
        _ => {
            grid.surfaces = vec![req.surface; feet.len()];
            None
        }
    };
    let mut counts = std::collections::BTreeMap::<u8, u64>::new();
    for id in &grid.surfaces {
        *counts.entry(*id).or_default() += 1;
    }
    let mut o = Options::new(scale, hs);
    o.name = trnwrite::DISC_NAME.to_owned();
    let t = build(&grid, &o)?;
    let (trn, w) = write(&t)?;
    let step_ft = f64::from(scale) * f64::from(hs);
    let worst_ft = prove_terrain(&trn, &grid, &feet, scale, hs)?;

    // The start: the window's centre sample, on the ground, facing north.
    let (cx, cz) = (samples.0 / 2, samples.1 / 2);
    let spawn = [
        cx as f32 * scale,
        feet[(cx + cz * samples.0) as usize],
        cz as f32 * scale,
    ];
    let mut scene = SceneText::for_terrain(&t, &req.id, spawn, [0.0, 0.0, 1.0]).to_text();
    if let Some(d) = &req.sky {
        scene = d.scene_with(&scene)?;
        d.prove(&scene)?;
    }

    let facts = Facts {
        epsg: g.epsg,
        samples,
        corner_pixel,
        corner,
        pixel: g.dx,
        scale,
        datum,
        vertical_unit_m: v_unit,
        low_ft: feet.iter().copied().fold(f32::INFINITY, f32::min),
        high_ft: feet.iter().copied().fold(0.0, f32::max),
        height_scale: hs,
        step_ft,
        worst_ft,
        overlap,
        leaves: w.leaves,
        blocks: w.blocks,
        spawn,
        surfaces: counts.into_iter().collect(),
    };

    let dir = format!("tracks/{}", req.id);
    let mut files = vec![
        (format!("{dir}/{}.scn", req.id), scene.into_bytes()),
        (format!("{dir}/{}.ps2.trn", req.id), trn),
    ];
    let mut todo = vec![
        format!(
            "{id}.ps2.trn: heights are the DEM's over a datum of {datum} {unit} (mod.json elevation.datum), quantised to steps of {step:.4} ft",
            id = req.id,
            unit = if v_unit == 1.0 { "m" } else { "ft" },
            step = step_ft
        ),
        surface_todo(req, &facts, classified),
        format!(
            "{}.ps2.trn: normals are trnwrite's central differences, not the original tool's (unknown) formula",
            req.id
        ),
        format!(
            "{}.ps2.trn: the scale is {scale} ft, the six-decimal value the scene writes, against {:.7} ft for the DEM's {pixel_m} m",
            req.id,
            pixel_m / FOOT_M
        ),
        format!(
            "{}.scn: the start is the window's centre sample facing north, chosen by tools/elevation",
            req.id
        ),
        "no props, collision objects, ecosystem, detail map (<stem>_d.tga) or baked light map (<stem>_shadow_NN.pst)".to_owned(),
    ];
    match &req.sky {
        None => todo.push(format!(
            "{}.scn: no sky (CubeFile), [Fog] or [Light<n>]: the data has none, the port's defaults draw it",
            req.id
        )),
        Some(d) => {
            todo.push(format!(
                "{id}.scn: CubeFile={cube} and [{sections}] are {game}'s {map} ({name}) scene's ({res}:{scene}), copied line for line, and {path} is its sky, copied byte for byte: \
                 the data has no sky, fog or lights, so they are a disc map's of the same kind (mod.json sky_from), not the place's",
                id = req.id,
                cube = d.cube_file,
                sections = d.sections.join("], ["),
                game = sky::GAME,
                map = d.map,
                name = d.name,
                res = d.res,
                scene = d.scene,
                path = d.cube_path(),
            ));
            todo.push(format!(
                "{}.scn: the sun's Position and TargetPosition are in {}'s frame (its 5760-ft map), not this map's: their direction is kept, and where the game puts the light or its shadow from the points themselves is not checked",
                req.id, d.map
            ));
            todo.push(format!(
                "{}: {}'s bytes, so this mod is own-disc; built without a sky it is public-domain",
                d.cube_path(),
                sky::GAME
            ));
            files.push((format!("{dir}/{}", d.cube_path()), d.cube.clone()));
        }
    }
    if req.naip.is_empty() {
        todo.push("no ground picture: built without an aerial image".to_owned());
    } else {
        let extent = (
            (samples.0 - 1) as f32 * scale,
            (samples.1 - 1) as f32 * scale,
        );
        let img = drape_image(req, &facts, g.epsg)?;
        let stem = format!("{}_naip", req.id);
        let pst = drape::pst(&img);
        let psm = drape::psm(&stem, extent);
        prove_drape(&pst, &psm, &stem, &img, extent)?;
        files.push((format!("{dir}/{stem}.pst"), pst));
        files.push((
            format!("{dir}/terrainscreenspacemesh.psm"),
            psm.into_bytes(),
        ));
        todo.push(format!(
            "{stem}.pst: the aerial image resampled (bilinear) to {t}×{t} over the map ({:.2} m a pixel) and cut to 256 colours; \
             whether the game takes a whole-map texture this size is not checked",
            (samples.0 - 1) as f64 * pixel_m / f64::from(req.texture),
            t = req.texture
        ));
        todo.push(
            "terrainscreenspacemesh.psm: one pass, free09's ShaderPass_1 keys with its mask's whole-map UV scale; read by game-render's splat, not checked in the game".to_owned(),
        );
    }
    let json = mod_json(req, &facts, &todo);
    files.insert(0, ("mod.json".to_owned(), json.into_bytes()));
    Ok(Built { files, facts, todo })
}

/// The surfaces' line in `todo`: what each sample's id is and why (M16, `notes/elevation.md`).
fn surface_todo(req: &Request, f: &Facts, classified: Option<(surfaces::Ground, f32)>) -> String {
    let all: u64 = f.surfaces.iter().map(|(_, n)| n).sum();
    let shares: Vec<String> = f
        .surfaces
        .iter()
        .map(|(id, n)| {
            let what = match *id {
                surfaces::SAND => " (zone 5, sandy dirt)",
                surfaces::GRASS => " (zone 4, off-track grass)",
                surfaces::DIRT => " (zone 1, dirt)",
                surfaces::WATER => " (zone 7, shallow water)",
                _ => "",
            };
            format!("{id}{what} {:.1}%", 100.0 * *n as f64 / all as f64)
        })
        .collect();
    let base = "MX Unleashed's own freeride maps ride every painted layer (sand, rock, grass) as zone 1, \
                so these ids go past what the original does";
    match classified {
        Some((ground, green)) => {
            let how = match ground {
                surfaces::Ground::Dunes => format!(
                    "bare ground is sand and green (NAIP's NDVI averaged over {} m from {green}) is dirt, the scrub flats between dunes",
                    2 * surfaces::RADIUS + 1
                ),
                surfaces::Ground::Desert => format!(
                    "the site is desert badlands and washes with nothing green in the picture, so the DEM tells them apart: \
                     sand where the ground is under {flat}° and either open (under {open}% of the {sq} m square round it over {steep}°) \
                     or, inside the badlands, paler in the picture than that square by {paler} of 255 (a wash floor); the rest is dirt. \
                     The thresholds were set by eye and checked against SSURGO's soil units, which are too coarse to use for the washes; \
                     a paved road on open ground is sand too",
                    flat = surfaces::WASH_FLAT,
                    open = surfaces::OPEN * 100.0,
                    sq = 2 * surfaces::OPEN_RADIUS + 1,
                    steep = surfaces::STEEP,
                    paler = surfaces::PALER,
                ),
                surfaces::Ground::Forest => format!(
                    "green (NAIP's NDVI averaged over {} m from {green}) is grass, the rest dirt; the picture sees the canopy, \
                     not the ground under it",
                    2 * surfaces::RADIUS + 1
                ),
            };
            format!(
                "{}.ps2.trn: surface ids from the aerial picture, {}: {how}. {water}Rock rides as dirt (the bikes' .phy has no rock zone); {base}",
                req.id,
                shares.join(", "),
                water = if f.surfaces.iter().any(|(id, _)| *id == surfaces::WATER) {
                    "Water is the DEM's hydro-flattened surface where the picture is dark and not green: it rides as zone 7's grip \
                     on the lidar's flat water surface, with no water plane, splash or depth (the drape shows the river as photographed). "
                } else {
                    ""
                },
            )
        }
        None => format!(
            "{}.ps2.trn: every surface id is {}, not taken from the ground: {base}",
            req.id, req.surface
        ),
    }
}

/// Read the written terrain back with openmx's `Terrain` and check every sample through every cell the
/// game can ask for at step 1: its local height is `height_scale × raw` bit for bit, its world height
/// (× the file's scale) is the DEM's feet within half a step, and its surface is the grid's. Returns
/// the largest difference from the DEM in feet.
pub fn prove_terrain(
    trn: &[u8],
    g: &Grid,
    feet: &[f32],
    scale: f32,
    hs: f32,
) -> Result<f64, String> {
    use ps2_float::F;
    let t = Terrain::parse(trn).map_err(|e| format!("openmx's Terrain::parse: {e}"))?;
    if t.scale != F(scale) {
        return Err(format!("the file's scale is {:?}, not {scale}", t.scale));
    }
    let half = f64::from(scale) * f64::from(hs) / 2.0;
    let mut worst = 0f64;
    for sz in 0..g.depth - 1 {
        for sx in 0..g.width - 1 {
            let q = t
                .cell(sx, sz, 1)
                .ok_or_else(|| format!("no cell at {sx}, {sz}"))?;
            for (i, (dx, dz)) in [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().enumerate() {
                let n = ((sx + dx) + (sz + dz) * g.width) as usize;
                let want = F(hs) * F::from(i32::from(g.heights[n]));
                if q.height[i].to_bits() != want.to_bits() {
                    return Err(format!(
                        "sample {}, {}: local height {:?}, not {want:?}",
                        sx + dx,
                        sz + dz,
                        q.height[i]
                    ));
                }
                if q.surface[i] != g.surfaces[n] & 0x7f {
                    return Err(format!(
                        "sample {}, {}: surface {}",
                        sx + dx,
                        sz + dz,
                        q.surface[i]
                    ));
                }
                let world = f64::from((t.scale * q.height[i]).0);
                let d = (world - f64::from(feet[n])).abs();
                worst = worst.max(d);
                // Half a step, and the f32 rounding of two products near the top of the relief.
                if d > half + f64::from(feet[n]).abs() * 1e-6 + 1e-6 {
                    return Err(format!(
                        "sample {}, {}: {world} ft against the DEM's {} ft",
                        sx + dx,
                        sz + dz,
                        feet[n]
                    ));
                }
            }
        }
    }
    Ok(worst)
}

/// The drape as the texture's pixels: each one's centre found on the map, then in the first aerial
/// image that has it.
pub fn drape_image(req: &Request, f: &Facts, epsg: Option<u16>) -> Result<Rgb, String> {
    let n = req.texture;
    if !(16..=1024).contains(&n) || !n.is_power_of_two() {
        return Err(format!(
            "a {n}-pixel drape: a power of two from 16 to 1024 (the GS's largest)"
        ));
    }
    // Model coordinates of the map: sample s at corner + s × pixel; the texture spans samples 0..=last.
    let last = ((f.samples.0 - 1) as f64, (f.samples.1 - 1) as f64);
    let at = |u: u32, v: u32| -> (f64, f64) {
        let sx = (f64::from(u) + 0.5) / f64::from(n) * last.0;
        let sz = (1.0 - (f64::from(v) + 0.5) / f64::from(n)) * last.1;
        (f.corner.0 + sx * f.pixel, f.corner.1 + sz * f.pixel)
    };
    let (west, east) = (f.corner.0 - f.pixel, f.corner.0 + (last.0 + 1.0) * f.pixel);
    let (south, north) = (f.corner.1 - f.pixel, f.corner.1 + (last.1 + 1.0) * f.pixel);
    let mut pixels: Vec<Option<[u8; 3]>> = vec![None; (n * n) as usize];
    for path in &req.naip {
        let mut r = GeoTiff::open(path)?;
        let g = r.geo.clone();
        if g.epsg != epsg {
            return Err(format!(
                "{}: EPSG {:?}, the DEM is {epsg:?}",
                path.display(),
                g.epsg
            ));
        }
        if g.bands < 3 {
            return Err(format!(
                "{}: {} bands, a colour image has at least 3",
                path.display(),
                g.bands
            ));
        }
        // The image's pixels under the map, with one to spare for the filter.
        let i0 = (((west - g.left) / g.dx).floor() as i64 - 1).max(0);
        let j0 = (((g.top - north) / g.dy).floor() as i64 - 1).max(0);
        let i1 = (((east - g.left) / g.dx).ceil() as i64 + 1).min(i64::from(g.width));
        let j1 = (((g.top - south) / g.dy).ceil() as i64 + 1).min(i64::from(g.height));
        if i0 >= i1 || j0 >= j1 {
            continue;
        }
        let (w, h) = ((i1 - i0) as u32, (j1 - j0) as u32);
        let mut px = r.window(i0 as u32, j0 as u32, w, h)?;
        // No data: the tag's value in every band, or, with no tag, 0 in every band. NAIP's quarter quads
        // carry no tag, and their corners outside the quad's slanted edges are filled with 0.
        let nd = g.nodata.unwrap_or(0.0);
        for p in px.chunks_exact_mut(g.bands as usize) {
            if p.iter().all(|v| f64::from(*v) == nd) {
                p.fill(f32::NAN);
            }
        }
        for v in 0..n {
            for u in 0..n {
                let k = (u + v * n) as usize;
                if pixels[k].is_some() {
                    continue;
                }
                let (x, y) = at(u, v);
                let fi = (x - g.left) / g.dx - 0.5 - i0 as f64;
                let fj = (g.top - y) / g.dy - 0.5 - j0 as f64;
                if let Some(c) = drape::bilinear(&px, w, h, g.bands as usize, fi, fj) {
                    pixels[k] = Some(c.map(|v| v.round().clamp(0.0, 255.0) as u8));
                }
            }
        }
    }
    let missing = pixels.iter().filter(|p| p.is_none()).count();
    if missing > 0 {
        return Err(format!(
            "{missing} of the drape's {} pixels are under no aerial image: add the neighbouring ones",
            n * n
        ));
    }
    Ok(Rgb {
        width: n,
        height: n,
        pixels: pixels.into_iter().map(Option::unwrap).collect(),
    })
}

/// The drape through game-render: `pst::Texture::parse` decodes the picture to its own palette
/// colours, and `splat::Splat::parse` reads one pass with the whole-map UV scale.
pub fn prove_drape(
    pst: &[u8],
    psm: &str,
    stem: &str,
    img: &Rgb,
    extent: (f32, f32),
) -> Result<(), String> {
    use game_render::pst::Texture;
    use game_render::splat::Splat;
    let tex = Texture::parse(pst).map_err(|e| format!("game-render's Texture::parse: {e}"))?;
    if (tex.width, tex.height) != (img.width as usize, img.height as usize) {
        return Err(format!("the .pst decodes {}×{}", tex.width, tex.height));
    }
    let palette = drape::median_cut(&img.pixels);
    let idx = drape::index(&img.pixels, &palette);
    for (i, &p) in idx.iter().enumerate() {
        let c = palette[p as usize];
        if tex.rgba[4 * i..4 * i + 4] != [c[0], c[1], c[2], 255] {
            return Err(format!(
                "pixel {i}: decoded {:?}, palette {c:?}",
                &tex.rgba[4 * i..4 * i + 4]
            ));
        }
    }
    let mut asked = Vec::new();
    let s = Splat::parse(psm, |name| {
        asked.push(name.to_owned());
        Some(tex.clone())
    });
    if s.passes.len() != 1 || asked != [stem.to_ascii_lowercase()] {
        return Err(format!(
            "splat read {} passes asking for {asked:?}",
            s.passes.len()
        ));
    }
    let p = &s.passes[0];
    let (u, v) = (p.scale[0] * extent.0, p.scale[1] * extent.1);
    if (u - 1.0).abs() > 1e-5 || (v + 1.0).abs() > 1e-5 || p.blend.is_some() || p.angle != 0.0 {
        return Err(format!(
            "splat's pass: scale {:?}, blend {:?}, angle {}",
            p.scale, p.blend, p.angle
        ));
    }
    Ok(())
}

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// `mod.json` (§1), with the tool's own `elevation` block: where the window is and the datum, so the
/// map can be put back on the ground.
pub fn mod_json(req: &Request, f: &Facts, todo: &[String]) -> String {
    let list = |v: &[String]| {
        v.iter()
            .map(|s| format!("    {}", json_str(s)))
            .collect::<Vec<_>>()
            .join(",\n")
    };
    let files = |v: &[PathBuf]| -> Vec<String> {
        v.iter()
            .map(|p| {
                p.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            })
            .collect()
    };
    let dem = files(&req.dem);
    let naip = files(&req.naip);
    format!(
        "{{\n  \"standard\": 1,\n  \"kind\": \"track\",\n  \"id\": {id},\n  \"name\": {name},\n  \"source\": {{\n    \"game\": \"Real elevation\",\n    \"data\": {data},\n    \"date\": {date}\n  }},\n  \"version\": \"1.3.0\",\n  \"author\": \"openmx-mods tools/elevation 0.1\",\n  \"permission\": {permission},\n{sky_from}  \"elevation\": {{\n    \"epsg\": {epsg},\n    \"dem\": [{dem}],\n    \"naip\": [{naip}],\n    \"corner\": [{cx:.6}, {cy:.6}],\n    \"corner_pixel\": [{pi}, {pj}],\n    \"pixel\": {pixel},\n    \"samples\": [{sx}, {sz}],\n    \"datum\": {datum},\n    \"vertical_unit_m\": {vu},\n    \"surfaces\": {{{surfaces}}},\n    \"axes\": \"x east, z north; sample (0, 0) is the south-west corner\"\n  }},\n  \"todo\": [\n{todo}\n  ]\n}}\n",
        id = json_str(&req.id),
        name = json_str(&req.name),
        data = json_str(&req.data),
        date = json_str(&req.date),
        epsg = f.epsg.map_or("null".to_owned(), |e| e.to_string()),
        dem = dem
            .iter()
            .map(|s| json_str(s))
            .collect::<Vec<_>>()
            .join(", "),
        naip = naip
            .iter()
            .map(|s| json_str(s))
            .collect::<Vec<_>>()
            .join(", "),
        cx = f.corner.0,
        cy = f.corner.1,
        pi = f.corner_pixel.0,
        pj = f.corner_pixel.1,
        pixel = f.pixel,
        sx = f.samples.0,
        sz = f.samples.1,
        datum = f.datum,
        vu = f.vertical_unit_m,
        surfaces = f
            .surfaces
            .iter()
            .map(|(id, n)| format!("\"{id}\": {n}"))
            .collect::<Vec<_>>()
            .join(", "),
        todo = list(todo),
        permission = json_str(if req.sky.is_some() { "own-disc" } else { "public-domain" }),
        sky_from = req.sky.as_ref().map_or(String::new(), |d| format!(
            "  \"sky_from\": {{\n    \"game\": {},\n    \"boot\": {},\n    \"map\": {},\n    \"name\": {},\n    \"members\": [{}],\n    \"scene\": {},\n    \"cube\": {},\n    \"sections\": [{}]\n  }},\n",
            json_str(sky::GAME),
            json_str(sky::BOOT),
            json_str(&d.map),
            json_str(&d.name),
            json_str(&d.res),
            json_str(&d.scene),
            json_str(&d.cube_member),
            d.sections.iter().map(|s| json_str(s)).collect::<Vec<_>>().join(", "),
        )),
    )
}

/// Write the files under `out` (the mod's folder), replacing any earlier build of it.
pub fn write_mod(out: &Path, b: &Built) -> Result<(), String> {
    if out.exists() {
        // Only ever an earlier mod: a folder without a mod.json is somebody else's.
        if !out.join("mod.json").is_file() {
            return Err(format!(
                "{} exists and is not a mod folder; not replacing it",
                out.display()
            ));
        }
        std::fs::remove_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    for (p, bytes) in &b.files {
        let path = out.join(p);
        std::fs::create_dir_all(path.parent().unwrap())
            .map_err(|e| format!("{}: {e}", path.display()))?;
        std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(())
}
