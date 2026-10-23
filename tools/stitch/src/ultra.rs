//! The ultra map (C5): whole worlds, each game's giant map, joined into one world mod.
//!
//! Each world goes in whole. Its tiles and its joins are copied byte for byte and moved by one offset,
//! so inside it the ground, the look and the drops are that world's exactly. Between the worlds is new
//! ground, made the way C1 makes it between tiles, with the world in the tile's place:
//!
//! - A world is one [`Source`]: its interior is its whole rectangle, its ring its outer edge as its own
//!   joins have it (each sample's height, the last cell's slope across and along, surface and look).
//!   Stitch leaves join ground all round a world (its pad), so that edge is always join ground.
//! - The new joins (`u…`) are the blend of the worlds' extensions, with C1's weights, radii and seam
//!   gates ([`Source::edge_gates`]), on the worlds' common lattice. They cover only the ground outside
//!   every world: the gaps between them and the rest of the bounding rectangle. On a world's edge they
//!   have its height, and the first cell out has its last cell's slope (the pinned ring), as at a tile's
//!   seam. Past the gates they are held to `max_slope`, and the gaps widen until they are.
//! - The look of a new join is the worlds' shares times each world's own look at the nearest point of
//!   its edge, so it fades from one world's blend of tiles to the other's with no step.
//!
//! The layout keeps each world in one piece and puts the worlds in a grid (a row for three), the
//! columns chosen for the least area, in a chain from the first world to the one nearest it in edge
//! height and so on. A world's offset must keep every tile's origin on a whole leaf (§3): a multiple of
//! each of its tiles' leaf sizes. A world with a tile whose spacing has no small ratio to the lattice
//! (Untamed's Nat_OR, 4.499999 ft) can only stay where it is, so it is placed first, at the origin.

use crate::build::{GATE_SHARE, Params, bytes_255, cell_slope, gap_for, ratio};
use crate::jsonw::{J, o, s};
use crate::{Edge, Ring, Source, Tile, shepard};
use game_sim::track::Terrain;
use modcheck::json::Json;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A component world's join, as listed in its mod.json and read for its outer band.
struct Part {
    name: String,
    at: [f64; 2],
    samples: [u32; 2],
    spacing: f64,
}

/// One built world, read for joining: its place-independent facts and its outer edge.
pub struct World {
    pub id: String,
    pub dir: PathBuf,
    pub meta: Json,
    /// The world's join lattice, feet.
    pub spacing: f64,
    /// Lattice cells a side: the world is `[0, size × spacing]` on each axis.
    pub size: [u32; 2],
    /// Its tiles, in its order.
    pub tiles: Vec<String>,
    /// Its joins, in its order (which is precedence).
    joins: Vec<Part>,
    /// The offsets it may take are multiples of this, feet; `None`: only the origin (a tile whose
    /// spacing has no small ratio to the lattice).
    pub quantum: Option<f64>,
    /// Its outer edge, as a tile's ring: 0 x = 0, 1 z = 0, 2 x = max, 3 z = max.
    pub ring: Ring,
    /// Per edge sample, the look there: (index into `tiles`, byte), the bytes summing to 255.
    looks: [Vec<Vec<(usize, u8)>>; 4],
    /// Per edge, the heights one lattice step inside.
    inner: [Vec<f64>; 4],
}

fn nums(j: Option<&Json>) -> Vec<f64> {
    j.and_then(Json::arr).unwrap_or_default().iter().filter_map(Json::num).collect()
}

/// A Json value as the writer's tree (mod.json's strings, numbers, arrays and objects; true and false as
/// 1 and 0, which no field copied here has).
fn to_j(v: &Json) -> J {
    match v {
        Json::Null => J::A(Vec::new()),
        Json::Bool(b) => J::N(if *b { 1.0 } else { 0.0 }),
        Json::Num(n) => J::N(*n),
        Json::Str(t) => J::S(t.clone()),
        Json::Arr(a) => J::A(a.iter().map(to_j).collect()),
        Json::Obj(kv) => J::O(kv.iter().map(|(k, v)| (k.clone(), to_j(v))).collect()),
    }
}

/// The prefix a world's joins get in the ultra map: its id less `world_`, and `_`.
pub fn prefix(world: &str) -> String {
    format!("{}_", world.strip_prefix("world_").unwrap_or(world))
}

impl World {
    /// Read a world mod folder built by stitch: its tiles' spacings (for the offsets it may take), and
    /// its outer edge from the joins that reach it.
    pub fn load(dir: &Path) -> Result<World, String> {
        let id = dir.file_name().map(|f| f.to_string_lossy().into_owned()).ok_or_else(|| format!("{}: not a folder name", dir.display()))?;
        let text = std::fs::read_to_string(dir.join("mod.json")).map_err(|e| format!("{id}/mod.json: {e}"))?;
        let meta = Json::parse(&text).map_err(|e| format!("{id}/mod.json: {e}"))?;
        if meta.get("kind").and_then(Json::str) != Some("world") {
            return Err(format!("{id}: not a world mod"));
        }
        let st = meta.get("stitch").ok_or_else(|| format!("{id}: no stitch field (not built by stitch)"))?;
        if st.get("worlds").is_some() {
            return Err(format!("{id}: already a world of worlds"));
        }
        let spacing = st.get("spacing").and_then(Json::num).ok_or_else(|| format!("{id}: no stitch.spacing"))?;

        // The tiles: their spacings set the offsets the world may take; their interiors must stay off the band.
        let mut tiles = Vec::new();
        let mut leaves = Vec::new();
        let mut anchored = Vec::new();
        let mut rects = Vec::new();
        for t in meta.get("tiles").and_then(Json::arr).unwrap_or_default() {
            let name = t.get("tile").and_then(Json::str).ok_or_else(|| format!("{id}: a tile with no name"))?.to_owned();
            let (at, keep) = (nums(t.get("at")), nums(t.get("keep")));
            if at.len() != 2 || keep.len() != 4 {
                return Err(format!("{id}: tiles[{name}]: at or keep malformed"));
            }
            let tile = Tile::load(&dir.join("tiles").join(&name))?;
            let sp = tile.spacing;
            match ratio(sp, spacing) {
                Some(_) => leaves.push(16.0 * sp),
                None => anchored.push(format!("{name} ({sp} ft)")),
            }
            rects.push([at[0] + keep[0] * sp, at[1] + keep[1] * sp, at[0] + keep[2] * sp, at[1] + keep[3] * sp]);
            tiles.push(name);
        }
        let quantum = if anchored.is_empty() {
            let base = 16.0 * spacing;
            let whole = |d: f64, leaf: f64| (d / leaf - (d / leaf).round()).abs() < 1e-9;
            Some((1..=4096).map(|m| f64::from(m) * base).find(|d| leaves.iter().all(|l| whole(*d, *l))).ok_or_else(|| format!("{id}: no common leaf multiple"))?)
        } else {
            eprintln!("stitch: {id}: no small ratio to the {spacing} ft lattice: {}; the world can only stay at the origin", anchored.join(", "));
            None
        };

        // The joins, and the world's extent from the lattice's.
        let mut joins = Vec::new();
        let mut far = [0.0f64; 2];
        for jn in meta.get("joins").and_then(Json::arr).unwrap_or_default() {
            let name = jn.get("join").and_then(Json::str).ok_or_else(|| format!("{id}: a join with no name"))?.to_owned();
            let (at, samples) = (nums(jn.get("at")), nums(jn.get("samples")));
            if at.len() != 2 || samples.len() != 2 {
                return Err(format!("{id}: joins[{name}]: at or samples malformed (built before C1's samples field?)"));
            }
            let bytes = std::fs::read(dir.join("joins").join(&name).join(format!("{name}.ps2.trn"))).map_err(|e| format!("{id}/joins/{name}: {e}"))?;
            let t = Terrain::parse(&bytes).map_err(|e| format!("{id}/joins/{name}: {e}"))?;
            let sp = f64::from(t.scale.0);
            let p = Part { name, at: [at[0], at[1]], samples: [samples[0] as u32, samples[1] as u32], spacing: sp };
            if (sp - spacing).abs() < 1e-9 {
                for ax in 0..2 {
                    far[ax] = far[ax].max(p.at[ax] + f64::from(p.samples[ax] - 1) * sp);
                }
            }
            joins.push(p);
        }
        let size = far.map(|f| (f / spacing).round() as u32);
        if size[0] < 4 || size[1] < 4 {
            return Err(format!("{id}: no lattice joins"));
        }
        let (wx, wz) = (f64::from(size[0]) * spacing, f64::from(size[1]) * spacing);
        for (t, r) in tiles.iter().zip(&rects) {
            if r[0] < 2.0 * spacing || r[1] < 2.0 * spacing || r[2] > wx - 2.0 * spacing || r[3] > wz - 2.0 * spacing {
                return Err(format!("{id}: {t}'s interior reaches the world's outer band"));
            }
        }

        // The outer band: the samples on the edge and one step inside, from the first join covering each.
        let band = |p: &Part| {
            let r = [p.at[0], p.at[1], p.at[0] + f64::from(p.samples[0] - 1) * p.spacing, p.at[1] + f64::from(p.samples[1] - 1) * p.spacing];
            (r, r[0] <= spacing + 1e-6 || r[1] <= spacing + 1e-6 || r[2] >= wx - spacing - 1e-6 || r[3] >= wz - spacing - 1e-6)
        };
        let mut near: Vec<(usize, [f64; 4], Vec<u8>, Vec<u8>, Vec<usize>)> = Vec::new();
        for (i, p) in joins.iter().enumerate() {
            let (r, on) = band(p);
            if !on {
                continue;
            }
            let jd = dir.join("joins").join(&p.name);
            let bytes = std::fs::read(jd.join(format!("{}.ps2.trn", p.name))).map_err(|e| format!("{id}/joins/{}: {e}", p.name))?;
            let blend = std::fs::read(jd.join(format!("{}.blend", p.name))).map_err(|e| format!("{id}/joins/{}: {e}", p.name))?;
            let jn = &meta.get("joins").and_then(Json::arr).unwrap_or_default()[i];
            let between: Vec<usize> = jn
                .get("between")
                .and_then(Json::arr)
                .unwrap_or_default()
                .iter()
                .filter_map(Json::str)
                .map(|b| tiles.iter().position(|t| t == b).ok_or_else(|| format!("{id}: joins[{}] is between {b}, not a tile", p.name)))
                .collect::<Result<_, _>>()?;
            near.push((i, r, bytes, blend, between));
        }
        let parsed: Vec<Terrain> = near.iter().map(|n| Terrain::parse(&n.2).expect("parsed above")).collect();
        let sample = |ix: u32, iz: u32| -> Result<(f64, u8, Vec<(usize, u8)>), String> {
            let (x, z) = (f64::from(ix) * spacing, f64::from(iz) * spacing);
            for (k, (i, r, _, blend, between)) in near.iter().enumerate() {
                if x < r[0] - 1e-6 || x > r[2] + 1e-6 || z < r[1] - 1e-6 || z > r[3] + 1e-6 {
                    continue;
                }
                let p = &joins[*i];
                let f = [(x - p.at[0]) / p.spacing, (z - p.at[1]) / p.spacing];
                if f.iter().any(|v| (v - v.round()).abs() > 1e-6) {
                    return Err(format!("{id}: ({x}, {z}) on its outer band is ground of {}, which has no sample there", p.name));
                }
                let (lx, lz) = (f[0].round() as u32, f[1].round() as u32);
                let (cx, cz) = (lx.min(p.samples[0] - 2), lz.min(p.samples[1] - 2));
                let t = &parsed[k];
                let q = t.cell(cx, cz, 1).ok_or_else(|| format!("{id}/joins/{}: no cell {cx},{cz}", p.name))?;
                let c = ((lx - cx) + 2 * (lz - cz)) as usize;
                let h = f64::from((t.scale * q.height[c]).0);
                let nb = between.len();
                let at = (lx + lz * p.samples[0]) as usize * nb;
                let look = between.iter().zip(&blend[at..at + nb]).filter(|(_, b)| **b > 0).map(|(t, b)| (*t, *b)).collect();
                return Ok((h, q.surface[c], look));
            }
            Err(format!("{id}: ({x}, {z}) on its outer band is no join's ground"))
        };
        let (nx, nz) = (size[0], size[1]);
        let s = spacing;
        let h = |x: u32, z: u32| sample(x, z).map(|v| v.0);
        let mut edges: [Vec<Edge>; 4] = Default::default();
        let mut looks: [Vec<Vec<(usize, u8)>>; 4] = Default::default();
        let mut inner: [Vec<f64>; 4] = Default::default();
        for e in 0..4 {
            let n = if e % 2 == 0 { nz } else { nx };
            for a in 0..=n {
                let c = a.min(n - 1);
                // (the sample, the one inside, and the two along that make the cell's slope along the edge)
                let (on, ins, a0, a1) = match e {
                    0 => ((0, a), (1, a), (0, c), (0, c + 1)),
                    1 => ((a, 0), (a, 1), (c, 0), (c + 1, 0)),
                    2 => ((nx, a), (nx - 1, a), (nx, c), (nx, c + 1)),
                    _ => ((a, nz), (a, nz - 1), (c, nz), (c + 1, nz)),
                };
                let (hv, surface, look) = sample(on.0, on.1)?;
                let hi = h(ins.0, ins.1)?;
                let across = if e < 2 { (hi - hv) / s } else { (hv - hi) / s };
                let along = (h(a1.0, a1.1)? - h(a0.0, a0.1)?) / s;
                let (gx, gz) = if e % 2 == 0 { (across, along) } else { (along, across) };
                edges[e].push(Edge { h: hv, gx, gz, surface });
                looks[e].push(look);
                inner[e].push(hi);
            }
        }
        drop(parsed);
        Ok(World { id, dir: dir.to_path_buf(), meta, spacing, size, tiles, joins, quantum, ring: Ring::new(edges), looks, inner })
    }

    /// The world's rectangle at an offset.
    fn rect(&self, at: [f64; 2]) -> [f64; 4] {
        [at[0], at[1], at[0] + f64::from(self.size[0]) * self.spacing, at[1] + f64::from(self.size[1]) * self.spacing]
    }

    /// The world as a source of the blend, at an offset, with its gates.
    fn source(&self, index: usize, at: [f64; 2], gates: [f64; 4]) -> Source {
        Source { tile: index, rect: self.rect(at), at, spacing: self.spacing, keep: [0, 0, self.size[0], self.size[1]], ring: self.ring.clone(), gates }
    }

    /// Height at a lattice sample anywhere in the world (world feet, local), from the first of its
    /// lattice joins with a sample there, read off its file (kept in `files` for the next ask): what a
    /// new piece that reaches into the world (C11) holds there, so its samples are the world's own.
    fn ground(&self, x: f64, z: f64, files: &mut std::collections::HashMap<String, Vec<u8>>) -> Option<f64> {
        for p in &self.joins {
            if (p.spacing - self.spacing).abs() > 1e-9 {
                continue;
            }
            let f = [(x - p.at[0]) / p.spacing, (z - p.at[1]) / p.spacing];
            if f.iter().zip(p.samples).any(|(v, n)| *v < -1e-6 || *v > f64::from(n - 1) + 1e-6 || (v - v.round()).abs() > 1e-6) {
                continue;
            }
            if !files.contains_key(&p.name) {
                let bytes = std::fs::read(self.dir.join("joins").join(&p.name).join(format!("{}.ps2.trn", p.name))).ok()?;
                files.insert(p.name.clone(), bytes);
            }
            let t = Terrain::parse(&files[&p.name]).ok()?;
            let (lx, lz) = (f[0].round() as u32, f[1].round() as u32);
            let (cx, cz) = (lx.min(p.samples[0] - 2), lz.min(p.samples[1] - 2));
            let q = t.cell(cx, cz, 1)?;
            return Some(f64::from((t.scale * q.height[((lx - cx) + 2 * (lz - cz)) as usize]).0));
        }
        None
    }

    /// Height at a lattice sample on the edge or one step inside (local sample units); `None` deeper.
    fn band_height(&self, ix: u32, iz: u32) -> Option<f64> {
        let [nx, nz] = self.size;
        match (ix, iz) {
            (0, z) => Some(self.ring.edges[0][z as usize].h),
            (x, 0) => Some(self.ring.edges[1][x as usize].h),
            (x, z) if x == nx => Some(self.ring.edges[2][z as usize].h),
            (x, z) if z == nz => Some(self.ring.edges[3][x as usize].h),
            (1, z) => Some(self.inner[0][z as usize]),
            (x, 1) => Some(self.inner[1][x as usize]),
            (x, z) if x == nx - 1 => Some(self.inner[2][z as usize]),
            (x, z) if z == nz - 1 => Some(self.inner[3][x as usize]),
            _ => None,
        }
    }
}

/// The worlds' places and the new ground's sources.
pub struct UltraPlan {
    pub spacing: f64,
    /// The lattice's last sample on each axis: the ultra map is `[0, size × spacing]`.
    pub size: [u32; 2],
    /// Each world's offset, feet.
    pub at: Vec<[f64; 2]>,
    /// Each world's grid cell (column, row).
    pub cell: Vec<(usize, usize)>,
    pub sources: Vec<Source>,
    pub radii: Vec<f64>,
    pub fade: f64,
    pub widen: f64,
}

/// The ground at one lattice point outside every world's interior (or on its edge, or one step in).
struct Pt {
    h: f64,
    surface: u8,
    /// Each world's share.
    w: Vec<f64>,
    /// Each world's nearest edge sample: (edge, index).
    foot: Vec<(usize, usize)>,
    /// Distance to the nearest world.
    near: f64,
}

/// Round up to a multiple of `q`.
fn up(v: f64, q: f64) -> f64 {
    (v / q - 1e-9).ceil() * q + 0.0
}

/// The chain the grid is filled with: the first world (or the one that must stay at the origin), then
/// each time the one left whose mean edge height is nearest the last one's.
fn chain(worlds: &[World]) -> Result<Vec<usize>, String> {
    let anchored: Vec<usize> = (0..worlds.len()).filter(|&i| worlds[i].quantum.is_none()).collect();
    if anchored.len() > 1 {
        return Err(format!(
            "{} each have a tile off the lattice ratio, so each can only stay at the origin",
            anchored.iter().map(|&i| worlds[i].id.as_str()).collect::<Vec<_>>().join(" and ")
        ));
    }
    let mut order = vec![anchored.first().copied().unwrap_or(0)];
    let mut left: Vec<usize> = (0..worlds.len()).filter(|i| *i != order[0]).collect();
    while !left.is_empty() {
        let last = worlds[*order.last().expect("one")].ring.mean();
        let k = (0..left.len()).min_by(|a, b| (worlds[left[*a]].ring.mean() - last).abs().total_cmp(&(worlds[left[*b]].ring.mean() - last).abs())).expect("one left");
        order.push(left.remove(k));
    }
    Ok(order)
}

/// Fill a grid of `cols` columns snake-wise: each world's (column, row).
fn grid(order: &[usize], cols: usize) -> (Vec<Vec<Option<usize>>>, Vec<(usize, usize)>) {
    let rows = order.len().div_ceil(cols);
    let mut g = vec![vec![None; cols]; rows];
    let mut cell = vec![(0, 0); order.len()];
    for (i, &w) in order.iter().enumerate() {
        let (r, mut c) = (i / cols, i % cols);
        if r % 2 == 1 {
            c = cols - 1 - c;
        }
        g[r][c] = Some(w);
        cell[w] = (c, r);
    }
    (g, cell)
}

/// Lay the worlds out: the grid whose bounding box (at the narrowest gaps) is smallest, each world at
/// its column's and row's start rounded up to its quantum, each gap as wide as its facing edges need
/// (build's rule, with the worlds' gates).
pub fn plan(worlds: &[World], gates: &[[f64; 4]], p: &Params, widen: f64) -> Result<UltraPlan, String> {
    if worlds.len() < 2 {
        return Err("an ultra map needs two worlds or more".into());
    }
    let step = worlds[0].spacing;
    if let Some(w) = worlds.iter().find(|w| (w.spacing - step).abs() > 1e-9) {
        return Err(format!("{} is on a {} ft lattice and {} on {} ft: joins of two lattices between worlds are not made", worlds[0].id, step, w.id, w.spacing));
    }
    let order = chain(worlds)?;
    let dims = |w: usize| [f64::from(worlds[w].size[0]) * step, f64::from(worlds[w].size[1]) * step];
    let area = |cols: usize| {
        let (g, _) = grid(&order, cols);
        let width: f64 = (0..cols).map(|c| g.iter().filter_map(|r| r[c]).map(|w| dims(w)[0]).fold(0.0, f64::max)).sum::<f64>() + (cols - 1) as f64 * p.min_gap;
        let depth: f64 = g.iter().map(|r| r.iter().flatten().map(|&w| dims(w)[1]).fold(0.0, f64::max)).sum::<f64>() + (g.len() - 1) as f64 * p.min_gap;
        width * depth
    };
    let cols = (1..=worlds.len()).min_by(|a, b| area(*a).total_cmp(&area(*b)).then(a.cmp(b))).expect("one");
    let (g, cell) = grid(&order, cols);
    let rows = g.len();
    let mut at = vec![[0.0; 2]; worlds.len()];
    let mut far = [0.0f64; 2];
    for axis in 0..2 {
        let lines = if axis == 0 { cols } else { rows };
        let members = |l: usize| -> Vec<usize> { if axis == 0 { g.iter().filter_map(|r| r[l]).collect() } else { g[l].iter().flatten().copied().collect() } };
        let mut cursor = 0.0;
        for l in 0..lines {
            let mut end = cursor;
            for w in members(l) {
                at[w][axis] = match worlds[w].quantum {
                    Some(q) => up(cursor, q),
                    None if cursor == 0.0 => 0.0,
                    None => return Err(format!("{} can only stay at the origin, and the layout puts it at {cursor} ft", worlds[w].id)),
                };
                end = end.max(at[w][axis] + dims(w)[axis]);
            }
            far[axis] = far[axis].max(end);
            let mut gap = p.min_gap * widen;
            if l + 1 < lines {
                let pairs: Vec<(Option<usize>, Option<usize>)> =
                    if axis == 0 { g.iter().map(|r| (r[l], r[l + 1])).collect() } else { (0..cols).map(|c| (g[l][c], g[l + 1][c])).collect() };
                for (a, b) in pairs {
                    if let (Some(a), Some(b)) = (a, b) {
                        let dh = worlds[a].ring.edge_mean(axis + 2) - worlds[b].ring.edge_mean(axis);
                        let room = |w: usize, e: usize| gates[w][e] - (step + 2.0 * p.fade);
                        gap = gap.max(gap_for(dh, p, room(a, axis + 2) + room(b, axis), widen));
                    }
                }
            }
            cursor = end + gap;
        }
    }
    let size = far.map(|f| crate::build::whole_samples((f / step).round() as u32, step));
    let sources: Vec<Source> = worlds.iter().enumerate().map(|(i, w)| w.source(i, at[i], gates[i])).collect();
    let radii = crate::build::radii(&sources, size, step);
    Ok(UltraPlan { spacing: step, size, at, cell, sources, radii, fade: p.fade, widen })
}

/// One new join piece to compute: lattice origin and cells a side.
#[derive(Clone, Debug)]
pub struct UPiece {
    pub name: String,
    pub origin: [u32; 2],
    pub cells: [u32; 2],
    /// The rectangle of new ground it is cut from, `[x0, z0, x1, z1]` in lattice cells: the piece
    /// reaches past it (C11, [`crate::build::cuts`]), and where that is deep inside a world its ground is
    /// the rectangle's nearest.
    pub inner: [u32; 4],
}

impl UltraPlan {
    /// The world whose closed rectangle holds a world point.
    fn inside(&self, x: f64, z: f64) -> Option<usize> {
        self.sources.iter().position(|s| s.contains(x, z))
    }

    /// The ground at a lattice point: a world's own on its edge or one step in, else the blend of the
    /// worlds' extensions (build's `Plan::blend`, every world a candidate). `None` deeper in a world.
    fn at(&self, worlds: &[World], x: f64, z: f64) -> Option<Pt> {
        let n = self.sources.len();
        let step = self.spacing;
        let foot = |src: &Source| -> (usize, usize) {
            let [x0, z0, x1, z1] = src.keep.map(f64::from);
            let qx = ((x - src.at[0]) / src.spacing).clamp(x0, x1);
            let qz = ((z - src.at[1]) / src.spacing).clamp(z0, z1);
            let d = |a: f64, b: f64| (a - b).abs() < 1e-9;
            if d(qx, x0) {
                (0, qz.round() as usize)
            } else if d(qx, x1) {
                (2, qz.round() as usize)
            } else if d(qz, z0) {
                (1, qx.round() as usize)
            } else {
                (3, qx.round() as usize)
            }
        };
        if let Some(i) = self.inside(x, z) {
            let src = &self.sources[i];
            let (ix, iz) = (((x - src.at[0]) / step).round() as u32, ((z - src.at[1]) / step).round() as u32);
            let h = worlds[i].band_height(ix, iz)?;
            let f = foot(src);
            let mut w = vec![0.0; n];
            w[i] = 1.0;
            let surface = src.ring.edges[f.0][f.1].surface;
            return Some(Pt { h, surface, w, foot: self.sources.iter().map(foot).collect(), near: 0.0 });
        }
        let ds: Vec<f64> = self.sources.iter().map(|s| s.extend_distance(x, z)).collect();
        let ni = (0..n).min_by(|a, b| ds[*a].total_cmp(&ds[*b])).expect("a world");
        let feet: Vec<(usize, usize)> = self.sources.iter().map(foot).collect();
        let surface = self.sources[ni].ring.edges[feet[ni].0][feet[ni].1].surface;
        let reach: Vec<Option<crate::Reach>> =
            (0..n).map(|i| (i == ni || ds[i] - step < self.radii[i]).then(|| self.sources[i].extend(x, z, step, self.fade))).collect();
        let nearest = reach[ni].expect("the nearest is always extended");
        let mut w = vec![0.0; n];
        if nearest.d <= step * (1.0 + 1e-9) {
            w[ni] = 1.0;
            return Some(Pt { h: nearest.h, surface, w, foot: feet, near: nearest.d });
        }
        let mut sum = 0.0;
        for (i, r) in reach.iter().enumerate() {
            if let Some(r) = r {
                w[i] = shepard(r.d - step, self.radii[i]);
                sum += w[i];
            }
        }
        if sum == 0.0 {
            w[ni] = 1.0;
            sum = 1.0;
        }
        let mut h = 0.0;
        for (i, r) in reach.iter().enumerate() {
            if let Some(r) = r {
                w[i] /= sum;
                h += w[i] * r.h;
            }
        }
        Some(Pt { h, surface, w, foot: feet, near: nearest.d })
    }

    /// How far a point is past every world's seam gate (build's `Plan::clear`).
    fn clear(&self, x: f64, z: f64) -> f64 {
        self.sources.iter().map(|s| s.distance(x, z) - s.gate_at(x, z)).fold(f64::INFINITY, f64::min)
    }

    /// The new join pieces: the ground outside every world, cut along every world's edges into
    /// rectangles and each of those into chunks, `u<column>_<row>_<x>_<z>`. A rectangle's chunks start
    /// a whole foot before it (where it is not at the map's edge) and run past its far end
    /// ([`crate::build::cuts`], `Over`; `Back` on the map's own far edge), so openmx's last row and
    /// column of each are someone's.
    pub fn pieces(&self, chunk: u32) -> Vec<UPiece> {
        let step = self.spacing;
        let mut xs = vec![0, self.size[0]];
        let mut zs = vec![0, self.size[1]];
        for s in &self.sources {
            let r = s.rect.map(|v| (v / step).round() as u32);
            xs.extend([r[0], r[2]]);
            zs.extend([r[1], r[3]]);
        }
        for v in [&mut xs, &mut zs] {
            v.sort_unstable();
            v.dedup();
        }
        let mut out = Vec::new();
        for (b, zw) in zs.windows(2).enumerate() {
            for (a, xw) in xs.windows(2).enumerate() {
                let mid = [(f64::from(xw[0]) + f64::from(xw[1])) / 2.0 * step, (f64::from(zw[0]) + f64::from(zw[1])) / 2.0 * step];
                if self.inside(mid[0], mid[1]).is_some() {
                    continue;
                }
                let whole = |o: u32| (f64::from(o) * step).fract() == 0.0;
                let before = |v: u32| (1..=v).map(|k| v - k).find(|o| whole(*o)).unwrap_or(0);
                let inner = [xw[0], zw[0], xw[1], zw[1]];
                // Over, but not past the map's own outer edge.
                let end = |v: u32, n: u32| if v == n { crate::build::End::Back } else { crate::build::End::Over };
                for (jz, (z0, cz)) in crate::build::cuts(before(zw[0]), zw[1], chunk, end(zw[1], self.size[1]), &whole).into_iter().enumerate() {
                    for (jx, (x0, cx)) in crate::build::cuts(before(xw[0]), xw[1], chunk, end(xw[1], self.size[0]), &whole).into_iter().enumerate() {
                        out.push(UPiece { name: format!("u{a:02}_{b:02}_{jx:02}_{jz:02}"), origin: [x0, z0], cells: [cx, cz], inner });
                    }
                }
            }
        }
        out
    }

    /// The steepest new ground on a coarse lattice (every `stride` samples), past every world's gate.
    pub fn coarse_slope(&self, worlds: &[World], stride: u32) -> (f64, [f64; 2]) {
        let step = self.spacing * f64::from(stride);
        let (nx, nz) = (self.size[0] / stride, self.size[1] / stride);
        let row = (nx + 1) as usize;
        // (height, clear); NaN height inside a world.
        let mut h = vec![(f64::NAN, 0.0); row * (nz as usize + 1)];
        let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
        let rows_each = (nz as usize + 1).div_ceil(threads).max(1);
        std::thread::scope(|sc| {
            for (part, out) in h.chunks_mut(rows_each * row).enumerate() {
                sc.spawn(move || {
                    for (i, o) in out.iter_mut().enumerate() {
                        let (kx, kz) = (i % row, part * rows_each + i / row);
                        let (x, z) = (kx as f64 * step, kz as f64 * step);
                        if self.inside(x, z).is_none() {
                            *o = (self.at(worlds, x, z).map_or(f64::NAN, |p| p.h), self.clear(x, z));
                        }
                    }
                });
            }
        });
        let mut worst = (0.0, [0.0; 2]);
        for kz in 0..nz as usize {
            for kx in 0..nx as usize {
                let c = [h[kx + kz * row], h[kx + 1 + kz * row], h[kx + (kz + 1) * row], h[kx + 1 + (kz + 1) * row]];
                if c.iter().any(|p| p.0.is_nan() || p.1 < step) {
                    continue;
                }
                let g = cell_slope(c.map(|p| p.0), step);
                if g > worst.0 {
                    worst = (g, [kx as f64 * step, kz as f64 * step]);
                }
            }
        }
        worst
    }
}

/// One new join piece computed.
pub struct Done {
    pub name: String,
    pub at: [f64; 2],
    pub samples: [u32; 2],
    /// Indices into the ultra map's tiles.
    pub between: Vec<usize>,
    pub trn: Vec<u8>,
    pub blend: Vec<u8>,
    pub far: f64,
    pub near: f64,
    pub clamped: usize,
}

/// Compute one new piece. `tile_of[w][t]` is world `w`'s tile `t` in the ultra map's tile list.
fn piece(plan: &UltraPlan, worlds: &[World], tile_of: &[Vec<usize>], n_tiles: usize, spec: &UPiece) -> Result<Done, String> {
    let step = plan.spacing;
    let files = std::cell::RefCell::new(std::collections::HashMap::new());
    let (sw, sd) = (spec.cells[0] as usize + 1, spec.cells[1] as usize + 1);
    let (mw, md) = (sw + 2, sd + 2);
    let at = [f64::from(spec.origin[0]) * step, f64::from(spec.origin[1]) * step];
    let mut h = vec![0.0; mw * md];
    let mut near = vec![0.0; sw * sd];
    let mut clear = vec![0.0; sw * sd];
    let mut surfaces = vec![0u8; sw * sd];
    let mut shares = vec![0.0f64; sw * sd * n_tiles];
    for mz in 0..md {
        for mx in 0..mw {
            let (kx, kz) = (mx as f64 - 1.0, mz as f64 - 1.0);
            let (x, z) = (at[0] + kx * step, at[1] + kz * step);
            // Past its rectangle, inside a world (C11's overlap): the world's height where the band
            // has one, and the look and surface of the rectangle's nearest point, whose foot on the
            // world's edge is the right one.
            let r = spec.inner.map(|v| f64::from(v) * step);
            let edge = || plan.at(worlds, x.clamp(r[0], r[2]), z.clamp(r[1], r[3]));
            let deep = |wi: usize| worlds[wi].ground(x - plan.at[wi][0], z - plan.at[wi][1], &mut files.borrow_mut());
            let pt = match (plan.inside(x, z), plan.at(worlds, x, z)) {
                (None, Some(pt)) => Some(pt),
                (Some(_), Some(pt)) => edge().map(|e| Pt { h: pt.h, ..e }),
                (Some(wi), None) => edge().map(|e| Pt { h: deep(wi).unwrap_or(e.h), ..e }),
                (None, None) => edge(),
            }
            .ok_or_else(|| format!("{}: ({x}, {z}) is deep inside a world", spec.name))?;
            h[mx + mz * mw] = pt.h;
            if (1..=sw).contains(&mx) && (1..=sd).contains(&mz) {
                let i = (mx - 1) + (mz - 1) * sw;
                near[i] = pt.near;
                clear[i] = plan.clear(x, z);
                surfaces[i] = pt.surface & 0x7f;
                let sh = &mut shares[i * n_tiles..(i + 1) * n_tiles];
                for (wi, wv) in pt.w.iter().enumerate() {
                    if *wv == 0.0 {
                        continue;
                    }
                    let (e, k) = pt.foot[wi];
                    for &(t, b) in &worlds[wi].looks[e][k] {
                        sh[tile_of[wi][t]] += wv * f64::from(b) / 255.0;
                    }
                }
            }
        }
    }
    let clamped = h.iter().filter(|v| **v < 0.0).count();
    let feet: Vec<f32> = (0..sd).flat_map(|z| (0..sw).map(move |x| (x, z))).map(|(x, z)| h[x + 1 + (z + 1) * mw].max(0.0) as f32).collect();
    let (mut grid, hs) = trnwrite::Grid::from_feet(sw as u32, sd as u32, &feet, step as f32)?;
    grid.surfaces = surfaces;
    grid.normals = (0..sd)
        .flat_map(|z| (0..sw).map(move |x| (x, z)))
        .map(|(x, z)| {
            let at = |dx: i64, dz: i64| h[(x as i64 + 1 + dx) as usize + (z as i64 + 1 + dz) as usize * mw];
            let gx = (at(1, 0) - at(-1, 0)) / (2.0 * step);
            let gz = (at(0, 1) - at(0, -1)) / (2.0 * step);
            let len = (gx * gx + 1.0 + gz * gz).sqrt();
            [(-gx / len * 127.0) as i8, (1.0 / len * 127.0) as i8, (-gz / len * 127.0) as i8]
        })
        .collect();
    let trn = trnwrite::build(&grid, &trnwrite::Options::new(step as f32, hs))?;
    let (bytes, _) = trnwrite::write(&trn)?;
    let (mut far, mut near_s) = (0.0f64, 0.0f64);
    for z in 0..sd - 1 {
        for x in 0..sw - 1 {
            let c = [x + z * sw, x + 1 + z * sw, x + (z + 1) * sw, x + 1 + (z + 1) * sw];
            let g = cell_slope(c.map(|i| h[(i % sw) + 1 + (i / sw + 1) * mw]), step);
            if c.iter().all(|&i| clear[i] > 0.0) {
                far = far.max(g);
            } else {
                near_s = near_s.max(g);
            }
        }
    }
    // The look: every tile with a share anywhere in the piece, in the ultra map's tile order.
    let mut between: Vec<usize> = (0..n_tiles).filter(|&t| (0..sw * sd).any(|i| shares[i * n_tiles + t] * 255.0 >= 0.5)).collect();
    if between.len() < 2 {
        // modcheck wants two or more: the first other tile, at 0 everywhere.
        let other = (0..n_tiles).find(|t| !between.contains(t)).ok_or("one tile only")?;
        between.push(other);
        between.sort_unstable();
    }
    let mut blend = Vec::with_capacity(sw * sd * between.len());
    for i in 0..sw * sd {
        let w: Vec<f64> = between.iter().map(|&t| shares[i * n_tiles + t]).collect();
        blend.extend(bytes_255(&w));
    }
    Ok(Done { name: spec.name.clone(), at, samples: [sw as u32, sd as u32], between, trn: bytes, blend, far, near: near_s, clamped })
}

/// A file into place: a hard link where the file system allows it (the bytes are the world's), a copy
/// where not.
fn link_or_copy(from: &Path, to: &Path) -> Result<(), String> {
    if let Some(p) = to.parent() {
        std::fs::create_dir_all(p).map_err(|e| format!("{}: {e}", p.display()))?;
    }
    if std::fs::hard_link(from, to).is_ok() {
        return Ok(());
    }
    std::fs::copy(from, to).map(|_| ()).map_err(|e| format!("{}: {e}", to.display()))
}

/// What an ultra build came to.
pub struct Built {
    pub dir: PathBuf,
    pub plan: UltraPlan,
    pub worlds: Vec<String>,
    pub tiles: usize,
    pub copied_joins: usize,
    pub pieces: usize,
    pub far_slope: f64,
    pub near_slope: f64,
}

/// Build the ultra map `p.id` from built world folders into `out/<id>/`, replacing one already there.
/// `p.sky` names a tile of any world (default: the first world's sky); the spawn is the first world's.
pub fn build(world_dirs: &[PathBuf], out: &Path, p: &Params) -> Result<Built, String> {
    let t0 = std::time::Instant::now();
    let worlds: Vec<World> = world_dirs.iter().map(|d| World::load(d)).collect::<Result<_, _>>()?;
    let mut tiles: Vec<(usize, String)> = Vec::new();
    let mut tile_of: Vec<Vec<usize>> = Vec::new();
    for (wi, w) in worlds.iter().enumerate() {
        let mut map = Vec::new();
        for t in &w.tiles {
            if let Some((other, _)) = tiles.iter().find(|(_, x)| x == t) {
                return Err(format!("{t} is in both {} and {}", worlds[*other].id, w.id));
            }
            map.push(tiles.len());
            tiles.push((wi, t.clone()));
        }
        tile_of.push(map);
    }
    let mut prefixes: Vec<String> = Vec::new();
    for w in &worlds {
        let pf = prefix(&w.id);
        if prefixes.contains(&pf) {
            return Err(format!("{} is given twice", w.id));
        }
        prefixes.push(pf);
    }
    for w in &worlds {
        eprintln!(
            "stitch: {}: {} tiles, {} joins, {:.0} × {:.0} ft at {} ft, edge height {:.1} ft (mean; x0 {:.1}, z0 {:.1}, x1 {:.1}, z1 {:.1}), offsets in steps of {}",
            w.id,
            w.tiles.len(),
            w.joins.len(),
            f64::from(w.size[0]) * w.spacing,
            f64::from(w.size[1]) * w.spacing,
            w.spacing,
            w.ring.mean(),
            w.ring.edge_mean(0),
            w.ring.edge_mean(1),
            w.ring.edge_mean(2),
            w.ring.edge_mean(3),
            w.quantum.map_or("none (the origin only)".into(), |q| format!("{q} ft"))
        );
    }
    eprintln!("stitch: worlds read in {:.1} s", t0.elapsed().as_secs_f64());
    let t0 = std::time::Instant::now();
    let step = worlds[0].spacing;
    let base = 2.0 * step + 3.0 * p.fade;
    let gates: Vec<[f64; 4]> = worlds.iter().enumerate().map(|(i, w)| w.source(i, [0.0; 2], [0.0; 4]).edge_gates(step, p.fade, GATE_SHARE * p.max_slope, base)).collect();
    for (w, g) in worlds.iter().zip(&gates) {
        if g.iter().any(|v| *v > base) {
            eprintln!("stitch: {}: seam gates {:?} ft (C1's {base:.0}): its outer edge stays steep that far out", w.id, g.map(|v| v.round()));
        }
    }
    eprintln!("stitch: seam gates measured in {:.1} s", t0.elapsed().as_secs_f64());

    let dir = out.join(&p.id);
    if dir.exists() {
        let old = std::fs::read_to_string(dir.join("mod.json")).unwrap_or_default();
        if !old.contains("\"world\"") {
            return Err(format!("{} exists and is not a world mod: not replacing it", dir.display()));
        }
        std::fs::remove_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let joins_dir = dir.join("joins");
    let mut widen = 1.0;
    let (plan, results) = loop {
        let plan = plan(&worlds, &gates, p, widen)?;
        let specs = plan.pieces(p.chunk);
        eprintln!(
            "stitch: widen {widen:.3}: ultra map {:.0} × {:.0} ft, {} worlds at {}, {} new pieces, radius {:.0}–{:.0} ft",
            f64::from(plan.size[0]) * step,
            f64::from(plan.size[1]) * step,
            worlds.len(),
            worlds.iter().zip(&plan.at).map(|(w, a)| format!("{} ({:.0}, {:.0})", w.id, a[0], a[1])).collect::<Vec<_>>().join(", "),
            specs.len(),
            plan.radii.iter().copied().fold(f64::INFINITY, f64::min),
            plan.radii.iter().copied().fold(0.0, f64::max)
        );
        let t0 = std::time::Instant::now();
        let (coarse, at) = plan.coarse_slope(&worlds, 4);
        eprintln!("stitch: coarse pass {coarse:.3} steepest, {:.1} s", t0.elapsed().as_secs_f64());
        if coarse > p.max_slope {
            eprintln!("stitch: {coarse:.3} at ({:.0}, {:.0}) on the coarse lattice, past {}: widening the gaps", at[0], at[1], p.max_slope);
        } else {
            let _ = std::fs::remove_dir_all(&joins_dir);
            let t0 = std::time::Instant::now();
            let next = AtomicUsize::new(0);
            let err: Mutex<Option<String>> = Mutex::new(None);
            let results: Mutex<Vec<Done>> = Mutex::new(Vec::new());
            let threads = if p.threads == 0 { std::thread::available_parallelism().map_or(4, |n| n.get()) } else { p.threads };
            std::thread::scope(|sc| {
                for _ in 0..threads.min(specs.len().max(1)) {
                    sc.spawn(|| {
                        loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            if i >= specs.len() || err.lock().expect("lock").is_some() {
                                break;
                            }
                            let r = piece(&plan, &worlds, &tile_of, tiles.len(), &specs[i]).and_then(|mut d| {
                                let jd = joins_dir.join(&d.name);
                                std::fs::create_dir_all(&jd).map_err(|e| format!("{}: {e}", jd.display()))?;
                                std::fs::write(jd.join(format!("{}.ps2.trn", d.name)), &d.trn).map_err(|e| e.to_string())?;
                                std::fs::write(jd.join(format!("{}.blend", d.name)), &d.blend).map_err(|e| e.to_string())?;
                                (d.trn, d.blend) = (Vec::new(), Vec::new());
                                results.lock().expect("lock").push(d);
                                Ok(())
                            });
                            if let Err(e) = r {
                                *err.lock().expect("lock") = Some(e);
                            }
                        }
                    });
                }
            });
            if let Some(e) = err.into_inner().expect("lock") {
                return Err(e);
            }
            let mut results = results.into_inner().expect("lock");
            results.sort_by(|a, b| a.name.cmp(&b.name));
            eprintln!("stitch: {} new pieces solved and written in {:.1} s", results.len(), t0.elapsed().as_secs_f64());
            let far = results.iter().map(|r| r.far).fold(0.0, f64::max);
            if far <= p.max_slope {
                break (plan, results);
            }
            let worst = results.iter().max_by(|a, b| a.far.total_cmp(&b.far)).expect("a piece");
            eprintln!("stitch: new ground {far:.3} steep away from the seams in {} (at {:.0}, {:.0}), past {}: widening the gaps", worst.name, worst.at[0], worst.at[1], p.max_slope);
        }
        widen *= 1.25;
        if widen > 20.0 {
            return Err("the ground between the worlds stays too steep however wide the gaps".into());
        }
    };

    // Each world whole: its tiles and joins, the same bytes, moved.
    let t0 = std::time::Instant::now();
    let mut tiles_j = Vec::new();
    let mut joins_j = Vec::new();
    let mut drops = Vec::new();
    let mut todo: Vec<String> = vec![
        "joins/u*: generated ground between whole worlds (tools/stitch, C5), no source game's: a closed-form blend of the \
         worlds' outer edges, as each world's own joins blend its tiles' edges; the look is each world's edge look under the \
         worlds' shares (openmx MD9)"
            .into(),
        "each world is whole: its tiles and joins are its own mod's, byte for byte, moved by its offset (stitch.worlds); its \
         joins are named <world>_<join>"
            .into(),
    ];
    todo.extend(p.todo.iter().cloned());
    let mut games: Vec<String> = Vec::new();
    let mut perms: Vec<String> = Vec::new();
    let mut gates_j = Vec::new();
    let mut collars_j = Vec::new();
    let mut copied = 0;
    for (wi, w) in worlds.iter().enumerate() {
        let d = plan.at[wi];
        let m = &w.meta;
        for t in m.get("tiles").and_then(Json::arr).unwrap_or_default() {
            let name = t.get("tile").and_then(Json::str).unwrap_or_default();
            let a = nums(t.get("at"));
            tiles_j.push(o(vec![("tile", s(name)), ("at", J::A(vec![J::N(a[0] + d[0]), J::N(a[1] + d[1])])), ("keep", to_j(t.get("keep").expect("read in load")))]));
            let from = w.dir.join("tiles").join(name);
            for rel in crate::files(&from)? {
                link_or_copy(&from.join(&rel), &dir.join("tiles").join(name).join(&rel))?;
            }
        }
        let pf = prefix(&w.id);
        for jn in m.get("joins").and_then(Json::arr).unwrap_or_default() {
            let name = jn.get("join").and_then(Json::str).unwrap_or_default();
            let a = nums(jn.get("at"));
            let new = format!("{pf}{name}");
            joins_j.push(o(vec![
                ("join", s(&new)),
                ("at", J::A(vec![J::N(a[0] + d[0]), J::N(a[1] + d[1])])),
                ("between", to_j(jn.get("between").expect("a join's between"))),
                ("samples", to_j(jn.get("samples").expect("read in load"))),
            ]));
            let (from, to) = (w.dir.join("joins").join(name), dir.join("joins").join(&new));
            for ext in ["ps2.trn", "blend"] {
                link_or_copy(&from.join(format!("{name}.{ext}")), &to.join(format!("{new}.{ext}")))?;
            }
            copied += 1;
        }
        drops.extend(m.get("drop").and_then(Json::arr).unwrap_or_default().iter().map(to_j));
        for l in m.get("todo").and_then(Json::arr).unwrap_or_default().iter().filter_map(Json::str) {
            if !todo.iter().any(|x| x == l) {
                todo.push(l.to_owned());
            }
        }
        for g in m.get("source").and_then(|s| s.get("games")).and_then(Json::arr).unwrap_or_default().iter().filter_map(Json::str) {
            if !games.iter().any(|x| x == g) {
                games.push(g.to_owned());
            }
        }
        perms.push(m.get("permission").and_then(Json::str).unwrap_or("").to_owned());
        let st = m.get("stitch").expect("checked in load");
        gates_j.extend(st.get("gates").and_then(Json::arr).unwrap_or_default().iter().map(to_j));
        for c in st.get("collars").and_then(Json::arr).unwrap_or_default() {
            let r = nums(c.get("rect"));
            let mut kv: Vec<(String, J)> = match to_j(c) {
                J::O(kv) => kv,
                _ => Vec::new(),
            };
            for (k, v) in kv.iter_mut() {
                if k == "rect" && r.len() == 4 {
                    *v = J::A(vec![J::N(r[0] + d[0]), J::N(r[1] + d[1]), J::N(r[2] + d[0]), J::N(r[3] + d[1])]);
                }
            }
            collars_j.push(J::O(kv));
        }
    }
    let names: Vec<&str> = tiles.iter().map(|(_, t)| t.as_str()).collect();
    for r in &results {
        joins_j.push(o(vec![
            ("join", s(&r.name)),
            ("at", J::A(vec![J::N(r.at[0]), J::N(r.at[1])])),
            ("between", J::A(r.between.iter().map(|&t| s(names[t])).collect())),
            ("samples", J::A(vec![J::N(f64::from(r.samples[0])), J::N(f64::from(r.samples[1]))])),
        ]));
    }
    let clamped: usize = results.iter().map(|r| r.clamped).sum();
    if clamped > 0 {
        todo.push(format!("joins/u*: {clamped} samples of the blend fell below 0 ft and were raised to 0 (a raw height is unsigned)"));
    }
    for (w, g) in worlds.iter().zip(&gates) {
        if g.iter().any(|v| *v > base) {
            todo.push(format!(
                "{}: its outer edge keeps the ground beside it steeper than max_slope {} for up to {:.0} ft out (seam gates x0 z0 x1 z1: {}); \
                 the new joins are held to max_slope only past them (C9's rule, on a world)",
                w.id,
                p.max_slope,
                g.iter().copied().fold(0.0, f64::max),
                g.map(|v| format!("{v:.0}")).join(" ")
            ));
        }
    }
    let first = &worlds[0].meta;
    let sky = match &p.sky {
        Some(sk) if names.contains(&sk.as_str()) => sk.clone(),
        Some(sk) => return Err(format!("sky {sk} is not a tile of any of the worlds")),
        None => first.get("sky").and_then(Json::str).unwrap_or(names[0]).to_owned(),
    };
    let spawn = to_j(first.get("spawn").ok_or_else(|| format!("{}: no spawn", worlds[0].id))?);
    if !matches!(&spawn, J::O(kv) if kv.iter().any(|(k, _)| k == "tile")) {
        return Err(format!("{}: its spawn is not a tile's (a position would need moving)", worlds[0].id));
    }
    let permission = if perms.iter().all(|x| *x == perms[0]) { perms[0].clone() } else { "own-disc".into() };
    let far_slope = results.iter().map(|r| r.far).fold(0.0, f64::max);
    let near_slope = results.iter().map(|r| r.near).fold(0.0, f64::max);
    let worlds_j: Vec<J> = worlds
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let src = &plan.sources[i];
            o(vec![
                ("world", s(&w.id)),
                ("prefix", s(prefix(&w.id))),
                ("at", J::A(plan.at[i].iter().map(|v| J::N(*v)).collect())),
                ("rect", J::A(src.rect.iter().map(|v| J::N(*v)).collect())),
                ("cell", J::A(vec![J::N(plan.cell[i].0 as f64), J::N(plan.cell[i].1 as f64)])),
                ("edges", J::A((0..4).map(|e| J::N((w.ring.edge_mean(e) * 10.0).round() / 10.0)).collect())),
                ("gates", J::A(gates[i].iter().map(|v| J::N(*v)).collect())),
            ])
        })
        .collect();
    let meta = o(vec![
        ("standard", J::N(1.0)),
        ("kind", s("world")),
        ("id", s(&p.id)),
        ("name", s(&p.name)),
        ("source", o(vec![("game", s("Combined")), ("games", J::A(games.into_iter().map(J::S).collect()))])),
        ("version", s(&p.version)),
        ("author", s(crate::build::AUTHOR)),
        ("permission", s(permission)),
        ("tiles", J::A(tiles_j)),
        ("joins", J::A(joins_j)),
        ("drop", J::A(drops)),
        ("sky", s(sky)),
        ("spawn", spawn),
        ("todo", J::A(todo.into_iter().map(J::S).collect())),
        (
            "stitch",
            o(vec![
                ("spacing", J::N(step)),
                ("max_slope", J::N(p.max_slope)),
                ("fade", J::N(p.fade)),
                ("radius", J::N(plan.radii.iter().copied().fold(0.0, f64::max))),
                ("radius_min", J::N(plan.radii.iter().copied().fold(f64::INFINITY, f64::min))),
                ("widen", J::N(plan.widen)),
                ("collars", J::A(collars_j)),
                ("gates", J::A(gates_j)),
                // C5's own: each world's place, for stitch check (its seams, its gates, its joins' sources).
                ("worlds", J::A(worlds_j)),
                ("far_slope", J::N((far_slope * 1e4).round() / 1e4)),
                ("near_slope", J::N((near_slope * 1e4).round() / 1e4)),
            ]),
        ),
    ]);
    std::fs::write(dir.join("mod.json"), meta.pretty()).map_err(|e| format!("{}: {e}", dir.display()))?;
    eprintln!("stitch: {} tiles and {copied} joins of the worlds put in place in {:.1} s", tiles.len(), t0.elapsed().as_secs_f64());
    Ok(Built {
        dir,
        worlds: worlds.iter().map(|w| w.id.clone()).collect(),
        tiles: tiles.len(),
        copied_joins: copied,
        pieces: results.len(),
        far_slope,
        near_slope,
        plan,
    })
}
