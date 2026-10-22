//! The map joiner: several track mods laid out as one world mod (standard §3), none of them resampled.
//!
//! Each map is a **tile**: its track mod folder copied whole, placed at an offset `at` that is a whole
//! number of its leaves. Its edge berm band is found from its terrain and its scene ([`Tile::find_keep`]),
//! and only the interior `keep` is used. The gaps between the interiors are new ground, the **joins**:
//! PLN terrains written with T2's writer on one lattice for the whole world ([`build`]), whose heights
//! are a blend solved in closed form over the gap:
//!
//! - Every tile is extended past each edge of its interior by its own ground's height and slope at that
//!   edge, the slope fading out over a taper ([`Source::extend`]).
//! - The extensions are blended with modified Shepard weights of the distance to each interior,
//!   `((R − d′)₊ / (R d′))²`, `d′` the distance less one lattice step. A weight's gradient is zero
//!   where it reaches 1, so the blend meets every interior with the interior's own height and slope.
//! - The first lattice ring outside an interior is pinned to the extension itself, so the first join
//!   cell has the tile's last cell's slope exactly (up to the join's height quantum).
//!
//! The same weights are the join's look (`<join>.blend`, MD9): 255 for a tile at its edge, fading to the
//! others across the gap, never a hard edge. The surface byte is the nearest tile's at its edge.
//!
//! [`check`] reads a world back with openmx's readers and proves the seams, the slopes and the interiors.

pub mod build;
pub mod check;
pub mod jsonw;

use game_sim::track::{Scene, Terrain};
use modcheck::json::Json;
use ps2_float::F;
use std::path::{Path, PathBuf};

/// The game's off-map margin: within this of the edge a rider is thrown back (game-sim
/// `off_map::MARGIN`, MXU `0x002522c0`; 150 on both the bike's and the biplane's test). A tile's own edge
/// is no edge in a world, so at least this much of each side is cut.
pub const SHOOTOFF: f64 = 150.0;
/// How far past the shootoff band the berm is looked for, at most.
pub const MAX_BERM: f64 = 400.0;
/// The berm ends where the edge profile stops falling inward faster than this (feet per foot).
pub const BERM_FALL: f64 = 0.02;
/// The profile is compared over this many samples.
const BERM_WINDOW: usize = 4;

/// One track mod folder read for joining.
pub struct Tile {
    /// The folder's name, which is the mod's id and the tile's name in the world.
    pub id: String,
    pub dir: PathBuf,
    pub meta: Json,
    /// The scene's path in the folder, `/`-separated.
    pub scene_file: String,
    pub scene_text: String,
    pub scene: Scene,
    pub terrain_file: String,
    /// The terrain as `Terrain::parse` takes it (through trnwrite's MD1 view when it has pages).
    terrain: Vec<u8>,
    /// Feet a sample (the terrain's own scale).
    pub spacing: f64,
    pub scale: F,
    /// The last sample on each axis: the field is samples `0..=n`, its edge `n × spacing`.
    pub n: (u32, u32),
}

/// Every file under a folder, relative and `/`-separated, in sorted order.
pub fn files(dir: &Path) -> Result<Vec<String>, String> {
    fn walk(root: &Path, at: &Path, out: &mut Vec<String>) -> Result<(), String> {
        let rd = std::fs::read_dir(at).map_err(|e| format!("{}: {e}", at.display()))?;
        for e in rd {
            let e = e.map_err(|e| format!("{}: {e}", at.display()))?;
            let p = e.path();
            if p.is_dir() {
                walk(root, &p, out)?;
            } else {
                let rel = p.strip_prefix(root).expect("under the root");
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out)?;
    out.sort();
    Ok(out)
}

/// The files a name finds as the port finds them: the end of the path, compared in lowercase (§2).
fn lookup<'a>(files: &'a [String], name: &str) -> Vec<&'a String> {
    let want = name.replace('\\', "/").to_ascii_lowercase();
    files
        .iter()
        .filter(|f| {
            let f = f.to_ascii_lowercase();
            f == want || f.ends_with(&format!("/{want}"))
        })
        .collect()
}

impl Tile {
    /// Read a track mod folder: its mod.json, its one scene, the terrain the scene names.
    pub fn load(dir: &Path) -> Result<Tile, String> {
        let id = dir
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .ok_or_else(|| format!("{}: not a folder name", dir.display()))?;
        let text = std::fs::read_to_string(dir.join("mod.json")).map_err(|e| format!("{id}/mod.json: {e}"))?;
        let meta = Json::parse(&text).map_err(|e| format!("{id}/mod.json: {e}"))?;
        if meta.get("kind").and_then(Json::str) != Some("track") {
            return Err(format!("{id}: not a track mod (a world's tile is a track, §3)"));
        }
        let all = files(dir)?;
        let scenes: Vec<&String> = all.iter().filter(|f| f.to_ascii_lowercase().ends_with(".scn")).collect();
        let [scene_file] = scenes.as_slice() else {
            return Err(format!("{id}: {} .scn files, a track has one", scenes.len()));
        };
        let scene_bytes = std::fs::read(dir.join(scene_file)).map_err(|e| format!("{id}/{scene_file}: {e}"))?;
        let scene = Scene::parse(&scene_bytes).map_err(|e| format!("{id}/{scene_file}: {e}"))?;
        let scene_text = String::from_utf8(scene_bytes).map_err(|_| format!("{id}/{scene_file}: not UTF-8"))?;
        // As the shell looks it up: the name as written, then with the platform tag (§2).
        let named = scene.terrain_file.clone();
        let tagged = match named.rsplit_once('.') {
            Some((stem, ext)) => format!("{stem}.ps2.{ext}"),
            None => format!("{named}.ps2"),
        };
        let terrain_file = [named.as_str(), tagged.as_str()]
            .iter()
            .find_map(|n| match lookup(&all, n).as_slice() {
                [one] => Some((*one).clone()),
                _ => None,
            })
            .ok_or_else(|| format!("{id}: the terrain {named} is not in the folder (or is there twice)"))?;
        let raw = std::fs::read(dir.join(&terrain_file)).map_err(|e| format!("{id}/{terrain_file}: {e}"))?;
        let terrain = trnwrite::openmx_view(&raw).map_err(|e| format!("{id}/{terrain_file}: {e}"))?.into_owned();
        let t = Terrain::parse(&terrain).map_err(|e| format!("{id}/{terrain_file}: {e}"))?;
        let root = t.root().ok_or_else(|| format!("{id}/{terrain_file}: no root node"))?;
        let (mut cx, mut cz) = (0, 0);
        for z in 0..16 {
            for x in 0..16 {
                if t.child(&root, x, z).is_some() {
                    cx = cx.max(x + 1);
                    cz = cz.max(z + 1);
                }
            }
        }
        if t.levels == 0 || cx == 0 {
            return Err(format!("{id}/{terrain_file}: an empty tree"));
        }
        let per = 16u32.pow(t.levels);
        let (scale, spacing) = (t.scale, f64::from(t.scale.0));
        drop(t);
        Ok(Tile {
            id,
            dir: dir.to_path_buf(),
            meta,
            scene_file: (*scene_file).clone(),
            scene_text,
            scene,
            terrain_file,
            terrain,
            spacing,
            scale,
            n: (cx * per, cz * per),
        })
    }

    pub fn terrain(&self) -> Terrain<'_> {
        Terrain::parse(&self.terrain).expect("parsed when the tile was loaded")
    }

    /// The `source.game` of the tile's mod.json.
    pub fn game(&self) -> String {
        self.meta.get("source").and_then(|s| s.get("game")).and_then(Json::str).unwrap_or("").to_owned()
    }

    /// One corner of a cell as world feet: the terrain's local height times its scale, in the game's
    /// single floats (game-sim `world::Ground::at` at a corner).
    fn corner(&self, t: &Terrain, cx: u32, cz: u32, dx: u32, dz: u32) -> Option<f64> {
        let q = t.cell(cx, cz, 1)?;
        Some(f64::from((self.scale * q.height[(dx + 2 * dz) as usize]).0))
    }

    /// A sample's height from whichever cell holds it (the field's samples, for the berm profile).
    fn sample(&self, t: &Terrain, x: u32, z: u32) -> f64 {
        let (cx, cz) = (x.min(self.n.0 - 1), z.min(self.n.1 - 1));
        self.corner(t, cx, cz, x - cx, z - cz).unwrap_or(0.0)
    }

    /// The ground at a point inside (or on the edge of) `keep`, in the tile's sample units: the cell is
    /// the one inside `keep` (so the far edges read the last kept cell, not the cut one), blended as
    /// game-sim's `Ground::at` blends it. Height in world feet, and the nearest corner's surface.
    pub fn ground_in(&self, t: &Terrain, keep: [u32; 4], fx: f64, fz: f64) -> Option<(f64, u8)> {
        let cx = (fx.floor().max(0.0) as u32).clamp(keep[0], keep[2] - 1);
        let cz = (fz.floor().max(0.0) as u32).clamp(keep[1], keep[3] - 1);
        let q = t.cell(cx, cz, 1)?;
        let (tx, tz) = (F((fx - f64::from(cx)) as f32), F((fz - f64::from(cz)) as f32));
        let w = [(F::ONE - tx) * (F::ONE - tz), tx * (F::ONE - tz), (F::ONE - tx) * tz, tx * tz];
        let mut y = F::ZERO;
        for i in 0..4 {
            y = y + w[i] * q.height[i];
        }
        let nearest = usize::from(tx > F(0.5)) + 2 * usize::from(tz > F(0.5));
        Some((f64::from((self.scale * y).0), q.surface[nearest]))
    }

    /// The interior left after the edge berm is cut, `[x0, z0, x1, z1]` in samples, half-open.
    ///
    /// Each side loses at least the shootoff band (150 ft, or the scene's `TerrainBorderWidth` if
    /// wider), then as long as the side's profile (the median height at each distance from the edge)
    /// keeps falling inward faster than [`BERM_FALL`]: that is the berm's inner slope. At most
    /// [`MAX_BERM`] feet past the shootoff band, and at most a quarter of the map.
    pub fn find_keep(&self) -> [u32; 4] {
        let t = self.terrain();
        let s = self.spacing;
        let border = f64::from(self.scene.terrain.border_width.0).max(0.0);
        let m0 = (SHOOTOFF.max(border) / s).ceil() as usize;
        let mut band = [0u32; 4];
        for (side, b) in band.iter_mut().enumerate() {
            // side 0: x = d, 1: z = d, 2: x = n − d, 3: z = n − d
            let across = if side % 2 == 0 { self.n.0 } else { self.n.1 };
            let dmax = (m0 + (MAX_BERM / s).ceil() as usize).min(across as usize / 4);
            let profile = self.profile(&t, side, dmax + BERM_WINDOW, m0 as u32);
            let mut d = m0.min(dmax);
            while d < dmax && profile[d] - profile[d + BERM_WINDOW] > BERM_FALL * BERM_WINDOW as f64 * s {
                d += 1;
            }
            *b = d as u32;
        }
        [band[0], band[1], self.n.0 - band[2], self.n.1 - band[3]]
    }

    /// One side's edge profile: for each distance `d` in samples from the side (0: x = d, 1: z = d,
    /// 2: x = n − d, 3: z = n − d), the median height along it, leaving `skip` samples at either end.
    pub fn profile(&self, t: &Terrain, side: usize, last: usize, skip: u32) -> Vec<f64> {
        let (len, across) = if side % 2 == 0 { (self.n.1, self.n.0) } else { (self.n.0, self.n.1) };
        let lo = skip.min(len / 2);
        let hi = len.saturating_sub(skip).max(lo + 1);
        (0..=last)
            .map(|d| {
                let d = (d as u32).min(across);
                let mut v: Vec<f64> = (lo..hi)
                    .map(|a| match side {
                        0 => self.sample(t, d, a),
                        1 => self.sample(t, a, d),
                        2 => self.sample(t, across - d, a),
                        _ => self.sample(t, a, across - d),
                    })
                    .collect();
                v.sort_by(f64::total_cmp);
                v[v.len() / 2]
            })
            .collect()
    }

    /// Every placement the scene's readers take, with its section name and position, in file order.
    /// The rule is `track::Scene::parse`'s: a numbered `[Model<n>]` or `[Object<n>]` with a file.
    pub fn placements(&self) -> Result<Vec<(String, [f64; 3])>, String> {
        let mut names: [Vec<String>; 2] = [Vec::new(), Vec::new()];
        let mut current: Option<(String, usize, bool)> = None;
        let flush = |c: &mut Option<(String, usize, bool)>, names: &mut [Vec<String>; 2]| {
            if let Some((name, kind, true)) = c.take() {
                names[kind].push(name);
            }
        };
        for raw in self.scene_text.lines() {
            let line = raw.trim();
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                flush(&mut current, &mut names);
                let low = name.to_ascii_lowercase();
                let numbered = |p: &str| {
                    low.strip_prefix(p).is_some_and(|r| !r.is_empty() && r.bytes().all(|b| b.is_ascii_digit()))
                };
                current = if numbered("model") {
                    Some((name.to_owned(), 0, false))
                } else if numbered("object") {
                    Some((name.to_owned(), 1, false))
                } else {
                    None
                };
                continue;
            }
            if let (Some(c), Some((k, v))) = (current.as_mut(), line.split_once('='))
                && matches!(k.trim().to_ascii_lowercase().as_str(), "col" | "slt")
            {
                c.2 = !v.trim().is_empty();
            }
        }
        flush(&mut current, &mut names);
        let lists = [&self.scene.models, &self.scene.collision];
        let mut out = Vec::new();
        for kind in 0..2 {
            if names[kind].len() != lists[kind].len() {
                return Err(format!(
                    "{}/{}: {} sections against Scene::parse's {} placements",
                    self.id,
                    self.scene_file,
                    names[kind].len(),
                    lists[kind].len()
                ));
            }
            for (n, p) in names[kind].iter().zip(lists[kind]) {
                out.push((n.clone(), p.position.map(|c| f64::from(c.0))));
            }
        }
        Ok(out)
    }

    /// Where a rider starts on this tile alone: mod.json's `spawn` (§2b) or the scene's.
    pub fn spawn(&self) -> [f64; 3] {
        let from_json = self.meta.get("spawn").and_then(|s| s.get("position")).and_then(Json::arr).and_then(|a| {
            let v: Vec<f64> = a.iter().filter_map(Json::num).collect();
            (v.len() == 3).then(|| [v[0], v[1], v[2]])
        });
        from_json.unwrap_or(self.scene.start.map(|c| f64::from(c.0)))
    }

    /// The ground along the four edges of `keep`, one entry per sample (see [`Ring`]).
    pub fn ring(&self, keep: [u32; 4]) -> Ring {
        let t = self.terrain();
        let s = self.spacing;
        let [x0, z0, x1, z1] = keep;
        let at = |x: u32, z: u32| -> Edge {
            let (cx, cz) = (x.clamp(x0, x1 - 1), z.clamp(z0, z1 - 1));
            let (dx, dz) = (x - cx, z - cz);
            let c = |a, b| self.corner(&t, cx, cz, a, b).unwrap_or(0.0);
            let surf = t.cell(cx, cz, 1).map_or(0, |q| q.surface[(dx + 2 * dz) as usize]);
            Edge { h: c(dx, dz), gx: (c(1, dz) - c(0, dz)) / s, gz: (c(dx, 1) - c(dx, 0)) / s, surface: surf }
        };
        let edges = [
            (z0..=z1).map(|z| at(x0, z)).collect::<Vec<Edge>>(),
            (x0..=x1).map(|x| at(x, z0)).collect(),
            (z0..=z1).map(|z| at(x1, z)).collect(),
            (x0..=x1).map(|x| at(x, z1)).collect(),
        ];
        let prefix = edges.clone().map(|e| {
            let mut acc = [0.0; 3];
            let mut out = vec![acc];
            for w in e.windows(2) {
                let (a, b) = ([w[0].h, w[0].gx, w[0].gz], [w[1].h, w[1].gx, w[1].gz]);
                acc = [0, 1, 2].map(|k| acc[k] + (a[k] + b[k]) / 2.0);
                out.push(acc);
            }
            out
        });
        Ring { edges, prefix }
    }
}

/// The ground at one sample on an interior's edge: height in feet, the slope of the kept cell it
/// belongs to in x and z (feet per foot), and the surface byte (id) there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    pub h: f64,
    pub gx: f64,
    pub gz: f64,
    pub surface: u8,
}

/// The four edges of an interior: 0 x = x0 and 2 x = x1 (along z), 1 z = z0 and 3 z = z1 (along x).
#[derive(Clone, Debug)]
pub struct Ring {
    pub edges: [Vec<Edge>; 4],
    /// Per edge, the integral of (height, x slope, z slope) from its start to each sample.
    prefix: [Vec<[f64; 3]>; 4],
}

impl Ring {
    /// An edge at a fractional sample along it, the height and slopes blended linearly (which is what the
    /// cell's own blend gives on its edge), the surface the nearest sample's.
    fn along(&self, edge: usize, a: f64) -> Edge {
        let e = &self.edges[edge];
        let a = a.clamp(0.0, (e.len() - 1) as f64);
        let i = (a.floor() as usize).min(e.len() - 2);
        let t = a - i as f64;
        let (p, q) = (e[i], e[i + 1]);
        Edge {
            h: p.h + (q.h - p.h) * t,
            gx: p.gx + (q.gx - p.gx) * t,
            gz: p.gz + (q.gz - p.gz) * t,
            surface: if t > 0.5 { q.surface } else { p.surface },
        }
    }

    /// The integral of (height, x slope, z slope) along an edge from its start to `a` samples, the edge
    /// held at its end values past either end. Exact for the linear pieces.
    fn integral(&self, edge: usize, a: f64) -> [f64; 3] {
        let e = &self.edges[edge];
        let p = &self.prefix[edge];
        let v = |x: &Edge| [x.h, x.gx, x.gz];
        let last = (e.len() - 1) as f64;
        if a <= 0.0 {
            return v(&e[0]).map(|c| c * a);
        }
        if a >= last {
            let end = v(&e[e.len() - 1]);
            return [0, 1, 2].map(|i| p[e.len() - 1][i] + end[i] * (a - last));
        }
        let i = a.floor() as usize;
        let at = self.along(edge, a);
        let (lo, mid) = (v(&e[i]), [at.h, at.gx, at.gz]);
        [0, 1, 2].map(|k| p[i][k] + (lo[k] + mid[k]) / 2.0 * (a - i as f64))
    }

    /// (height, x slope, z slope) averaged over `a ± sigma` samples along an edge.
    pub fn blurred(&self, edge: usize, a: f64, sigma: f64) -> [f64; 3] {
        if sigma < 1e-9 {
            let e = self.along(edge, a);
            return [e.h, e.gx, e.gz];
        }
        let (hi, lo) = (self.integral(edge, a + sigma), self.integral(edge, a - sigma));
        [0, 1, 2].map(|k| (hi[k] - lo[k]) / (2.0 * sigma))
    }

    pub fn mean(&self) -> f64 {
        let all: Vec<f64> = self.edges.iter().flatten().map(|e| e.h).collect();
        all.iter().sum::<f64>() / all.len() as f64
    }

    pub fn edge_mean(&self, edge: usize) -> f64 {
        self.edges[edge].iter().map(|e| e.h).sum::<f64>() / self.edges[edge].len() as f64
    }
}

/// A placed interior as the blend sees it: its rectangle in world feet and its edge ground.
#[derive(Clone, Debug)]
pub struct Source {
    pub tile: usize,
    /// `[x0, z0, x1, z1]` in world feet, closed.
    pub rect: [f64; 4],
    pub at: [f64; 2],
    pub spacing: f64,
    pub keep: [u32; 4],
    pub ring: Ring,
}

/// What one tile's extension says at a point outside its interior.
#[derive(Clone, Copy, Debug)]
pub struct Reach {
    /// Distance to the interior, feet.
    pub d: f64,
    /// The extended height.
    pub h: f64,
    pub surface: u8,
}

impl Source {
    /// Distance from a world point to the closed interior.
    pub fn distance(&self, x: f64, z: f64) -> f64 {
        let dx = (self.rect[0] - x).max(x - self.rect[2]).max(0.0);
        let dz = (self.rect[1] - z).max(z - self.rect[3]).max(0.0);
        dx.hypot(dz)
    }

    pub fn contains(&self, x: f64, z: f64) -> bool {
        const E: f64 = 1e-6;
        x >= self.rect[0] - E && x <= self.rect[2] + E && z >= self.rect[1] - E && z <= self.rect[3] + E
    }

    /// The distance [`Source::extend`] finds, by the same arithmetic to the bit (for the pruning, which
    /// must decide on exactly the distance the weight is taken from).
    pub fn extend_distance(&self, x: f64, z: f64) -> f64 {
        let s = self.spacing;
        let [x0, z0, x1, z1] = self.keep.map(f64::from);
        let fx = (x - self.at[0]) / s;
        let fz = (z - self.at[1]) / s;
        let (qx, qz) = (fx.clamp(x0, x1), fz.clamp(z0, z1));
        ((fx - qx) * s).hypot((fz - qz) * s)
    }

    /// The tile's ground carried past its edge: the edge's height plus its slope along the way out,
    /// times `taper(d)` (which is `d` for the first `step` and then levels off over `fade` feet).
    pub fn extend(&self, x: f64, z: f64, step: f64, fade: f64) -> Reach {
        let s = self.spacing;
        let [x0, z0, x1, z1] = self.keep.map(f64::from);
        let fx = (x - self.at[0]) / s;
        let fz = (z - self.at[1]) / s;
        let (qx, qz) = (fx.clamp(x0, x1), fz.clamp(z0, z1));
        let (ux, uz) = ((fx - qx) * s, (fz - qz) * s);
        let d = ux.hypot(uz);
        // The edge, blurred along its length the more the farther out: none on the pinned ring, then
        // σ = d′² / (d′ + fade) feet, so a hill cut by the edge does not run on as a ridge forever.
        let dp = (d - step).max(0.0);
        let sigma = dp * dp / (dp + fade) / s;
        let (vert, horiz) = (if fx < x0 { Some(0) } else if fx > x1 { Some(2) } else { None }, if fz < z0 { Some(1) } else if fz > z1 { Some(3) } else { None });
        let (e, surface) = match (vert, horiz) {
            (Some(v), Some(h)) => {
                // A corner: the two edges' ends, by the angle out of it (each alone on its side's line).
                let a = self.ring.blurred(v, qz - z0, sigma);
                let b = self.ring.blurred(h, qx - x0, sigma);
                let (wa, wb) = (ux * ux / (d * d), uz * uz / (d * d));
                ([0, 1, 2].map(|i| wa * a[i] + wb * b[i]), self.ring.along(v, qz - z0).surface)
            }
            (Some(v), None) => (self.ring.blurred(v, qz - z0, sigma), self.ring.along(v, qz - z0).surface),
            (None, Some(h)) => (self.ring.blurred(h, qx - x0, sigma), self.ring.along(h, qx - x0).surface),
            (None, None) => {
                let e = self.ring.along(0, qz - z0);
                ([e.h, e.gx, e.gz], e.surface)
            }
        };
        if d == 0.0 {
            return Reach { d, h: e[0], surface };
        }
        let slope = (e[1] * ux + e[2] * uz) / d;
        let taper = if d <= step { d } else { step + fade * (1.0 - (-(d - step) / fade).exp()) };
        Reach { d, h: e[0] + slope * taper, surface }
    }
}

/// Modified Shepard weight for a distance past the pinned ring (`d′ > 0`) within the radius.
pub fn shepard(d: f64, radius: f64) -> f64 {
    if d >= radius {
        0.0
    } else {
        let w = (radius - d) / (radius * d);
        w * w
    }
}
