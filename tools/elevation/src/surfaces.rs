//! Which of the bike's `.phy` surfaces the ground is (M16).
//!
//! A terrain sample's surface byte (its low seven bits) indexes the tire's per-surface tables
//! (`NumberOfSurfaces = 16`, `game_sim::phy::Surface`). The bikes' `.phy` files name each row in a
//! comment, "Friction zone 1 - (black) Standard on-track dirt" to "Friction zone 16", numbered from 1;
//! the byte is the zone's number less one. Which bytes MX Unleashed's own freeride ground carries, and
//! under which of its painted layers, is what [`survey`] counts: every sample of a freeride terrain
//! against the layer its `terrainscreenspacemesh.psm` shows most at that sample. `elevation surfaces`
//! prints it for free06–free11, and `notes/elevation.md` has the table.
//!
//! [`Cover`] is what the elevation maps are given from it: one id per kind of ground.

use game_render::ini;
use game_render::pst::Texture;
use game_sim::track::Terrain;
use std::collections::BTreeMap;

/// The surface ids (zone number less one) the elevation maps use, from the bikes' `.phy` comments and
/// the freeride survey (`notes/elevation.md`, *Surfaces*).
pub const DIRT: u8 = 0;
/// Zone 4, "(green) standard off track zone ... presumably grassy areas".
pub const GRASS: u8 = 3;
/// Zone 5, "(light brown) Sandy dirt ... sandy slippery section".
pub const SAND: u8 = 4;
/// Zone 7, "shallow water or mud" (M18): rivers and ponds.
pub const WATER: u8 = 6;

/// What one map's survey found.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Survey {
    /// The base layer, then each masked layer in the order they are painted.
    pub layers: Vec<String>,
    /// Samples by (surface id, the index in `layers` that shows most there).
    pub count: BTreeMap<(u8, usize), u64>,
    /// Samples whose surface byte has its top bit set (not part of the id).
    pub top_bit: u64,
    pub samples: u64,
}

/// The layers a `.psm` paints: the first pass's texture is the base, and each `*_Mask` pass's alpha
/// lays the texture of the pass after it. Returns (base, [(mask, layer, mask's UV scale)]).
pub fn layers(psm: &str) -> Option<(String, Vec<(String, String, [f32; 2])>)> {
    let sections = ini::parse(psm);
    let mut passes: Vec<(String, [f32; 2])> = Vec::new();
    for s in &sections {
        let name = s.name.to_ascii_lowercase();
        let Some(n) = name.strip_prefix("shaderpass_") else {
            continue;
        };
        if n.contains('_') {
            continue;
        }
        let Some(map) = s.get("TextureMap_1") else {
            continue;
        };
        let mut scale = [1.0, 1.0];
        for a in &sections {
            let an = a.name.to_ascii_lowercase();
            let uv = a
                .get("Algorithm")
                .is_some_and(|v| v.trim().eq_ignore_ascii_case("UVAnimation"));
            if uv && an.strip_prefix(&name).is_some_and(|r| r.starts_with("_animator_")) {
                scale = [
                    a.number("UScale").unwrap_or(1.0),
                    a.number("VScale").unwrap_or(1.0),
                ];
            }
        }
        passes.push((map.trim().to_ascii_lowercase(), scale));
    }
    let base = passes.first()?.0.clone();
    let mut masked = Vec::new();
    for w in passes.windows(2) {
        if w[0].0.ends_with("_mask") {
            masked.push((w[0].0.clone(), w[1].0.clone(), w[0].1));
        }
    }
    Some((base, masked))
}

/// Count every sample of a `samples`-a-side terrain at `scale` feet by (its surface id, the layer that
/// shows most). A layer's share is its mask's alpha times what the masks painted after it leave
/// (the GS's `(Cs - Cd) * Ad + Cd`, pass after pass); the base gets what all of them leave. The masks
/// are read bilinear, at the sample's world position times the mask pass's UV scale, wrapped.
pub fn survey(
    trn: &[u8],
    psm: &str,
    mut texture: impl FnMut(&str) -> Option<Texture>,
    samples: u32,
    scale: f32,
) -> Result<Survey, String> {
    let t = Terrain::parse(trn).map_err(|e| format!("Terrain::parse: {e}"))?;
    let (base, masked) = layers(psm).ok_or("the .psm has no passes")?;
    let mut masks = Vec::new();
    for (m, _, uv) in &masked {
        masks.push((texture(m).ok_or(format!("no {m}.pst"))?, *uv));
    }
    let mut s = Survey {
        layers: std::iter::once(base)
            .chain(masked.iter().map(|(_, l, _)| l.clone()))
            .collect(),
        ..Survey::default()
    };
    fn top_bits(n: &trnwrite::Node) -> u64 {
        let here = if n.is_leaf() {
            n.block.surfaces.iter().filter(|&&b| b & 0x80 != 0).count() as u64
        } else {
            0
        };
        here + n.children.iter().flatten().map(top_bits).sum::<u64>()
    }
    s.top_bit = top_bits(&trnwrite::read(trn)?.root);
    for sz in 0..samples {
        for sx in 0..samples {
            // The cell at (sx, sz) when there is one, else the one before it on that side.
            let (cx, dx) = if sx + 1 < samples { (sx, 0) } else { (sx - 1, 1) };
            let (cz, dz) = if sz + 1 < samples { (sz, 0) } else { (sz - 1, 1) };
            let q = t
                .cell(cx, cz, 1)
                .ok_or(format!("no cell at {cx}, {cz}"))?;
            let id = q.surface[(dx + 2 * dz) as usize];
            let (x, z) = (sx as f32 * scale, sz as f32 * scale);
            let mut left = 1.0f32;
            let mut best = (0usize, 0.0f32);
            for (i, (tex, uv)) in masks.iter().enumerate().rev() {
                let a = f32::from(tex.sample_smooth_parts(x * uv[0], z * uv[1], false, true)[3]) / 255.0;
                let share = a * left;
                if share > best.1 {
                    best = (i + 1, share);
                }
                left *= 1.0 - a;
            }
            if left > best.1 {
                best = (0, left);
            }
            *s.count.entry((id, best.0)).or_default() += 1;
            s.samples += 1;
        }
    }
    Ok(s)
}

/// Every leaf block sample's surface id (top bit off) in a terrain's tree, counted: for maps with no
/// painted layers to set them against (the nationals and supercross tracks). A shared block counts
/// once per leaf that uses it.
pub fn ids(trn: &[u8]) -> Result<BTreeMap<u8, u64>, String> {
    fn walk(n: &trnwrite::Node, out: &mut BTreeMap<u8, u64>) {
        if n.is_leaf() {
            for b in n.block.surfaces {
                *out.entry(b & 0x7f).or_default() += 1;
            }
        }
        for c in n.children.iter().flatten() {
            walk(c, out);
        }
    }
    let mut out = BTreeMap::new();
    walk(&trnwrite::read(trn)?.root, &mut out);
    Ok(out)
}

impl Survey {
    /// The table: one row per surface id, a column per layer, each cell the samples' share.
    pub fn table(&self) -> String {
        let mut out = format!("{:>4} {:>8}", "id", "samples");
        for l in &self.layers {
            out += &format!(" {:>16}", l);
        }
        out.push('\n');
        let ids: std::collections::BTreeSet<u8> = self.count.keys().map(|k| k.0).collect();
        for id in ids {
            let row: Vec<u64> = (0..self.layers.len())
                .map(|l| self.count.get(&(id, l)).copied().unwrap_or(0))
                .collect();
            let n: u64 = row.iter().sum();
            out += &format!("{id:>4} {n:>8}");
            for c in row {
                out += &format!(" {:>15.1}%", 100.0 * c as f64 / n as f64);
            }
            out.push('\n');
        }
        out += &format!("top bit set on {} block samples\n", self.top_bit);
        out
    }
}

/// The kind of ground at an elevation map's sample, from its aerial picture or the site.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cover {
    Sand,
    Dirt,
    Grass,
    Water,
}

impl Cover {
    pub fn id(self) -> u8 {
        match self {
            Cover::Sand => SAND,
            Cover::Dirt => DIRT,
            Cover::Grass => GRASS,
            Cover::Water => WATER,
        }
    }
}

/// One terrain sample's ground as the aerial picture sees it: the mean of the image's pixels whose
/// centres fall in the sample's square (a DEM pixel), in red, green, blue and near infrared, 0..255.
pub type Seen = [f32; 4];

/// The aerial images' pixels averaged over every terrain sample's square, rows south to north as the
/// grid has them. A sample takes the first image that has any pixel in its square; None where no image
/// does. The images need a fourth band (NAIP's near infrared): the vegetation test reads it.
pub fn seen(
    naip: &[std::path::PathBuf],
    corner: (f64, f64),
    pixel: f64,
    samples: (u32, u32),
    epsg: Option<u16>,
) -> Result<Vec<Option<Seen>>, String> {
    let (nx, nz) = (samples.0 as usize, samples.1 as usize);
    let mut sum = vec![[0f64; 4]; nx * nz];
    let mut n = vec![0u32; nx * nz];
    // The image a sample was first found in: a later image adds nothing there.
    let mut owner = vec![u8::MAX; nx * nz];
    let half = pixel / 2.0;
    let (west, east) = (corner.0 - half, corner.0 + (nx as f64 - 0.5) * pixel);
    let (south, north) = (corner.1 - half, corner.1 + (nz as f64 - 0.5) * pixel);
    for (k, path) in naip.iter().enumerate() {
        let mut r = crate::geotiff::GeoTiff::open(path)?;
        let g = r.geo.clone();
        if g.epsg != epsg {
            return Err(format!("{}: EPSG {:?}, the DEM is {epsg:?}", path.display(), g.epsg));
        }
        if g.bands < 4 {
            return Err(format!(
                "{}: {} bands: the surfaces need near infrared (NAIP's fourth)",
                path.display(),
                g.bands
            ));
        }
        let i0 = ((west - g.left) / g.dx).floor().max(0.0) as i64;
        let j0 = ((g.top - north) / g.dy).floor().max(0.0) as i64;
        let i1 = (((east - g.left) / g.dx).ceil() as i64).min(i64::from(g.width));
        let j1 = (((g.top - south) / g.dy).ceil() as i64).min(i64::from(g.height));
        if i0 >= i1 || j0 >= j1 {
            continue;
        }
        let (w, h) = ((i1 - i0) as usize, (j1 - j0) as usize);
        let bands = g.bands as usize;
        let nd = g.nodata.unwrap_or(0.0);
        // A strip of rows at a time, to keep the window small.
        let strip = 512usize;
        let mut taken = vec![false; nx * nz];
        for s0 in (0..h).step_by(strip) {
            let sh = strip.min(h - s0);
            let px = r.window(i0 as u32, (j0 as usize + s0) as u32, w as u32, sh as u32)?;
            for jj in 0..sh {
                let y = g.y((j0 as usize + s0 + jj) as f64);
                let sz = ((y - corner.1) / pixel).round();
                if sz < 0.0 || sz >= nz as f64 {
                    continue;
                }
                for ii in 0..w {
                    let x = g.x((i0 as usize + ii) as f64);
                    let sx = ((x - corner.0) / pixel).round();
                    if sx < 0.0 || sx >= nx as f64 {
                        continue;
                    }
                    let p = &px[(ii + jj * w) * bands..(ii + jj * w) * bands + 4];
                    if px[(ii + jj * w) * bands..(ii + jj * w + 1) * bands]
                        .iter()
                        .all(|v| f64::from(*v) == nd)
                    {
                        continue;
                    }
                    let at = sx as usize + sz as usize * nx;
                    if owner[at] != u8::MAX && owner[at] != k as u8 {
                        continue;
                    }
                    taken[at] = true;
                    for b in 0..4 {
                        sum[at][b] += f64::from(p[b]);
                    }
                    n[at] += 1;
                }
            }
        }
        for (o, t) in owner.iter_mut().zip(&taken) {
            if *t && *o == u8::MAX {
                *o = k as u8;
            }
        }
    }
    Ok(sum
        .iter()
        .zip(&n)
        .map(|(s, &c)| (c > 0).then(|| s.map(|v| (v / f64::from(c)) as f32)))
        .collect())
}

/// The normalised difference vegetation index, (NIR − red) / (NIR + red): above about 0.2 something
/// green covers the ground.
pub fn ndvi(s: &Seen) -> f32 {
    let (r, nir) = (s[0], s[3]);
    if r + nir <= 0.0 { 0.0 } else { (nir - r) / (nir + r) }
}

/// The kind of place a site is, which says what its bare ground is: the picture tells green from bare,
/// not sand from dirt or rock (all three are pale and grey-brown from above).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ground {
    /// A dune field: bare ground is sand, and the scrub on the flats between dunes stands on dirt.
    Dunes,
    /// Desert badlands and washes: bare ground is dirt (or rock, which rides as dirt); the sandy washes
    /// and open flats are told from the badlands by the DEM and the picture's brightness (M18,
    /// [`washes`]).
    Desert,
    /// Forested hills: green is grass (zone 4, the off-track zone), bare ground is dirt.
    Forest,
}

/// How far round a sample the NDVI is averaged, in samples: a 5 × 5 square (5 m at 1 m), so a lone
/// bush, a crown's shadow or a slip face's few pixels do not make a sample of their own.
pub const RADIUS: usize = 2;

impl Ground {
    pub fn parse(s: &str) -> Option<Ground> {
        match s {
            "dunes" => Some(Ground::Dunes),
            "desert" => Some(Ground::Desert),
            "forest" => Some(Ground::Forest),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Ground::Dunes => "dunes",
            Ground::Desert => "desert",
            Ground::Forest => "forest",
        }
    }

    /// A sample's cover from whether its ground is green.
    pub fn cover(self, green: bool) -> Cover {
        match (self, green) {
            (Ground::Dunes, false) => Cover::Sand,
            (Ground::Dunes, true) => Cover::Dirt,
            (Ground::Desert, _) => Cover::Dirt,
            (Ground::Forest, false) => Cover::Dirt,
            (Ground::Forest, true) => Cover::Grass,
        }
    }

    /// Every sample's cover (rows south to north, `w` a row): green where the NDVI averaged over
    /// [`RADIUS`] reaches `threshold` (the site's [`Ground::green`] unless exploring). Samples with no
    /// picture are left out of the average, and with none round them are bare.
    pub fn classify(self, seen: &[Option<Seen>], w: usize, threshold: f32) -> Vec<Cover> {
        let h = seen.len() / w;
        // A summed-area table of the NDVI and of how many samples have one.
        let mut sum = vec![0f64; (w + 1) * (h + 1)];
        let mut cnt = vec![0u32; (w + 1) * (h + 1)];
        for z in 0..h {
            for x in 0..w {
                let (v, c) = seen[x + z * w].as_ref().map_or((0.0, 0), |s| (f64::from(ndvi(s)), 1));
                let at = (x + 1) + (z + 1) * (w + 1);
                sum[at] = v + sum[at - 1] + sum[at - (w + 1)] - sum[at - (w + 2)];
                cnt[at] = c + cnt[at - 1] + cnt[at - (w + 1)] - cnt[at - (w + 2)];
            }
        }
        let mut out = Vec::with_capacity(w * h);
        for z in 0..h {
            for x in 0..w {
                let (x0, x1) = (x.saturating_sub(RADIUS), (x + RADIUS + 1).min(w));
                let (z0, z1) = (z.saturating_sub(RADIUS), (z + RADIUS + 1).min(h));
                let box_of = |t: &dyn Fn(usize) -> f64| {
                    t(x1 + z1 * (w + 1)) - t(x0 + z1 * (w + 1)) - t(x1 + z0 * (w + 1)) + t(x0 + z0 * (w + 1))
                };
                let n = box_of(&|i| f64::from(cnt[i]));
                let green = n > 0.0 && box_of(&|i| sum[i]) / n >= f64::from(threshold);
                out.push(self.cover(green));
            }
        }
        out
    }
}

// M18: what the picture's vegetation test cannot tell, from the DEM's shape and the picture's
// brightness: a desert's sandy washes and open flats from its badlands, and water from the ground.
// `notes/elevation.md`, *Surfaces past the NDVI test (M18)*, says how each number was chosen.

/// The DEM is averaged over a 5 × 5-sample square before its slope is taken, so a lidar bump or a
/// creosote bush's few centimetres do not make a slope of their own.
pub const SLOPE_RADIUS: usize = 2;
/// A desert sample is flat enough to be a wash floor or an open flat under this slope (degrees).
pub const WASH_FLAT: f32 = 8.0;
/// Steep ground (degrees): the badlands' faces. On Ocotillo's SSURGO units half of Badland is over 15°
/// and nine tenths of Carrizo gravelly sand under 5°.
pub const STEEP: f32 = 15.0;
/// How far round a sample the share of steep ground is counted, in samples (a 51 × 51 square), and
/// under which share the ground is open (an alluvial flat, not badlands).
pub const OPEN_RADIUS: usize = 25;
pub const OPEN: f32 = 0.1;
/// Inside badlands a flat sample is a wash when its picture, averaged over [`RADIUS`], is this much
/// paler (in 0..255 counts, red, green and blue averaged) than the [`OPEN_RADIUS`] square round it:
/// the washes' sand is the palest ground in the picture.
pub const PALER: f32 = 1.5;
/// Water: 3DEP's lidar DEMs are hydro-flattened, so a river is a flat band bank to bank. A connected
/// patch of samples under this slope (degrees), at least [`WATER_SAMPLES`] of them, whose picture is
/// dark in near infrared (median under [`WATER_NIR`]) and not green (median NDVI under 0) is water.
pub const WATER_FLAT: f32 = 0.3;
pub const WATER_SAMPLES: usize = 200;
pub const WATER_NIR: f32 = 40.0;

/// The mean of `v` over the samples with `Some` in a `(2r + 1)²` square round each sample (clipped at
/// the edges), None where the square has none. Rows of `w`.
pub fn box_mean(v: &[Option<f64>], w: usize, r: usize) -> Vec<Option<f64>> {
    let h = v.len() / w;
    let mut sum = vec![0f64; (w + 1) * (h + 1)];
    let mut cnt = vec![0u32; (w + 1) * (h + 1)];
    for z in 0..h {
        for x in 0..w {
            let (a, c) = v[x + z * w].map_or((0.0, 0), |a| (a, 1));
            let at = (x + 1) + (z + 1) * (w + 1);
            sum[at] = a + sum[at - 1] + sum[at - (w + 1)] - sum[at - (w + 2)];
            cnt[at] = c + cnt[at - 1] + cnt[at - (w + 1)] - cnt[at - (w + 2)];
        }
    }
    let mut out = Vec::with_capacity(w * h);
    for z in 0..h {
        for x in 0..w {
            let (x0, x1) = (x.saturating_sub(r), (x + r + 1).min(w));
            let (z0, z1) = (z.saturating_sub(r), (z + r + 1).min(h));
            let corners = |t: &[f64]| t[x1 + z1 * (w + 1)] - t[x0 + z1 * (w + 1)] - t[x1 + z0 * (w + 1)] + t[x0 + z0 * (w + 1)];
            let n = cnt[x1 + z1 * (w + 1)] + cnt[x0 + z0 * (w + 1)] - cnt[x0 + z1 * (w + 1)] - cnt[x1 + z0 * (w + 1)];
            out.push((n > 0).then(|| corners(&sum) / f64::from(n)));
        }
    }
    out
}

/// Every sample's slope in degrees: the heights (metres, rows of `w`, `pixel` metres apart) averaged
/// over [`SLOPE_RADIUS`], then central differences (one-sided at the edges).
pub fn slopes(metres: &[f32], w: usize, pixel: f64) -> Vec<f32> {
    let h = metres.len() / w;
    let v: Vec<Option<f64>> = metres.iter().map(|m| Some(f64::from(*m))).collect();
    let s: Vec<f64> = box_mean(&v, w, SLOPE_RADIUS).into_iter().map(|m| m.unwrap()).collect();
    let d = |a: usize, b: usize, steps: usize| (s[b] - s[a]) / (steps as f64 * pixel);
    let mut out = Vec::with_capacity(w * h);
    for z in 0..h {
        for x in 0..w {
            let (xa, xb) = (x.saturating_sub(1), (x + 1).min(w - 1));
            let (za, zb) = (z.saturating_sub(1), (z + 1).min(h - 1));
            let gx = if xb > xa { d(xa + z * w, xb + z * w, xb - xa) } else { 0.0 };
            let gz = if zb > za { d(x + za * w, x + zb * w, zb - za) } else { 0.0 };
            out.push(gx.hypot(gz).atan().to_degrees() as f32);
        }
    }
    out
}

/// The desert's sand (M18): a sample is sand where its ground is flat (under [`WASH_FLAT`]) and either
/// open (under [`OPEN`] of the square round it steep) or, inside badlands, paler than its
/// surroundings by [`PALER`] (a wash floor). The answer is then the majority of the 5 × 5 square
/// round each sample, as the vegetation test is averaged, so a lone sample does not make its own.
pub fn washes(seen: &[Option<Seen>], slope: &[f32], w: usize) -> Vec<bool> {
    let steep: Vec<Option<f64>> = slope.iter().map(|s| Some(f64::from(u8::from(*s > STEEP)))).collect();
    let rugged = box_mean(&steep, w, OPEN_RADIUS);
    let bright: Vec<Option<f64>> = seen
        .iter()
        .map(|s| s.map(|s| f64::from(s[0] + s[1] + s[2]) / 3.0))
        .collect();
    let near = box_mean(&bright, w, RADIUS);
    let round = box_mean(&bright, w, OPEN_RADIUS);
    let sand: Vec<Option<f64>> = (0..seen.len())
        .map(|k| {
            let open = rugged[k].unwrap() < f64::from(OPEN);
            let paler = match (near[k], round[k]) {
                (Some(a), Some(b)) => a - b > f64::from(PALER),
                _ => false,
            };
            Some(f64::from(u8::from(slope[k] < WASH_FLAT && (open || paler))))
        })
        .collect();
    box_mean(&sand, w, RADIUS).into_iter().map(|m| m.unwrap() >= 0.5).collect()
}

/// Water (M18): the samples of every 4-connected patch under [`WATER_FLAT`] of at least
/// [`WATER_SAMPLES`] whose picture's median near infrared is under [`WATER_NIR`] and median NDVI
/// under 0. The patch is the DEM's flattened surface, so the water's edge is the lidar's bank, not
/// the picture's (where trees overhang it).
pub fn water(seen: &[Option<Seen>], slope: &[f32], w: usize) -> Vec<bool> {
    let n = slope.len();
    let h = n / w;
    let flat: Vec<bool> = slope.iter().map(|s| *s < WATER_FLAT).collect();
    let mut done = vec![false; n];
    let mut out = vec![false; n];
    let median = |v: &mut Vec<f32>| {
        v.sort_by(f32::total_cmp);
        v.get(v.len() / 2).copied()
    };
    for start in 0..n {
        if !flat[start] || done[start] {
            continue;
        }
        let mut patch = vec![start];
        done[start] = true;
        let mut i = 0;
        while i < patch.len() {
            let k = patch[i];
            i += 1;
            let (x, z) = (k % w, k / w);
            let mut near = Vec::with_capacity(4);
            if x > 0 {
                near.push(k - 1);
            }
            if x + 1 < w {
                near.push(k + 1);
            }
            if z > 0 {
                near.push(k - w);
            }
            if z + 1 < h {
                near.push(k + w);
            }
            for j in near {
                if flat[j] && !done[j] {
                    done[j] = true;
                    patch.push(j);
                }
            }
        }
        if patch.len() < WATER_SAMPLES {
            continue;
        }
        let mut nir: Vec<f32> = patch.iter().filter_map(|&k| seen[k].map(|s| s[3])).collect();
        let mut veg: Vec<f32> = patch.iter().filter_map(|&k| seen[k].as_ref().map(ndvi)).collect();
        let dark = median(&mut nir).is_some_and(|v| v < WATER_NIR);
        let bare = median(&mut veg).is_some_and(|v| v < 0.0);
        if dark && bare {
            for k in patch {
                out[k] = true;
            }
        }
    }
    out
}

impl Ground {
    /// Every sample's cover from the picture and the DEM (M18): [`Ground::classify`]'s, then for a
    /// desert its sandy washes and open flats ([`washes`]), then water wherever [`water`] finds it.
    /// `metres` are the window's heights, rows of `w`, `pixel` metres apart.
    pub fn classify_with_dem(
        self,
        seen: &[Option<Seen>],
        metres: &[f32],
        w: usize,
        pixel: f64,
        threshold: f32,
    ) -> Vec<Cover> {
        let mut out = self.classify(seen, w, threshold);
        let slope = slopes(metres, w, pixel);
        if self == Ground::Desert {
            for (c, s) in out.iter_mut().zip(washes(seen, &slope, w)) {
                if s {
                    *c = Cover::Sand;
                }
            }
        }
        for (c, s) in out.iter_mut().zip(water(seen, &slope, w)) {
            if s {
                *c = Cover::Water;
            }
        }
        out
    }
}
