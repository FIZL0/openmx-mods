//! The ground, point by point: the original's own query on the original's RAM against openmx's port of
//! it on the mod's file.
//!
//! The original is run by openmx's R5900 interpreter (`ps2-oracle`) over a RAM image of the source game
//! in a ride on the map: its terrain object, its tree and blocks as its loader built them, and its query
//! and cell reader, which are code in that same image. The port is `game_sim::contact_search::ground`
//! (and `nearest_surface`) over a `TerrainField` of the mod's `.trn`. The terrain is copied, so every
//! bit of every answer must agree.
//!
//! Two queries are compared where the source has both: the buffer query (point, normal, the cell's four
//! surfaces, the corner weights), and the ride's query (height, normal, the nearest corner's surface).
//! Each runs in the four ways a terrain object can be set — cells one or two samples wide, split one way
//! or by parity — over the same points; the map's own setting is reported first.

use crate::ram;
use crate::source::{Normals, Source};
use game_sim::contact_search::{Cell, Ground as Hit, Heightfield, TerrainField, ground, nearest_surface};
use ps2_float::F;
use ps2_float::Vf;
use ps2_oracle::Ee;

/// How many instructions one query may take: a cell read is a few hundred.
const BUDGET: u64 = 200_000;
/// What the out-fields are filled with before a call, so a field the original did not write shows.
const JUNK: u32 = 0x7fc0_dead;

/// One way the terrain object can be set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mode {
    pub coarse: bool,
    pub split_one_way: bool,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} sample{} a cell, split {}",
            if self.coarse { 2 } else { 1 },
            if self.coarse { "s" } else { "" },
            if self.split_one_way { "one way" } else { "by parity" }
        )
    }
}

/// The four modes, `own` first.
pub fn modes(own: Mode) -> Vec<Mode> {
    let mut all = vec![own];
    for coarse in [false, true] {
        for split_one_way in [true, false] {
            let m = Mode { coarse, split_one_way };
            if m != own {
                all.push(m);
            }
        }
    }
    all
}

/// What the ride's query gives back.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ride {
    pub y: f32,
    pub normal: Vf,
    pub surface: u8,
}

/// The original's side: a terrain answering the two queries the way the source game does.
pub trait Original {
    fn query(&mut self, mode: Mode, at: Vf) -> Hit;
    /// `None` where the source has no ride query (or it is not known).
    fn ride(&mut self, mode: Mode, at: Vf) -> Option<Ride>;
}

/// The source game's own code on its own RAM.
pub struct Oracle {
    ee: Ee,
    src: Source,
    terrain: u32,
    current: Option<Mode>,
}

impl Oracle {
    pub fn new(ram: Vec<u8>, src: Source, terrain: u32) -> Oracle {
        Oracle { ee: Ee::with_ram(ram), src, terrain, current: None }
    }

    /// The object's own setting, as the game left it.
    pub fn own_mode(&self) -> Mode {
        let o = self.src.object;
        Mode {
            coarse: self.ee.u32_at(self.terrain + o.coarse) != 0,
            split_one_way: self.ee.u32_at(self.terrain + o.split) == 1,
        }
    }

    /// Whether the game takes its normals from its packed table (which [`Over127`] does not model).
    pub fn packed_normals(&self) -> bool {
        match self.src.normals {
            Normals::Over127 { packed } => self.ee.u32_at(packed) != 0,
            Normals::Raw => false,
        }
    }

    /// A float field of the terrain object.
    pub fn field(&self, off: u32) -> f32 {
        self.ee.f32_at(self.terrain + off)
    }

    pub fn quad(&self, off: u32) -> [f32; 4] {
        std::array::from_fn(|k| self.ee.f32_at(self.terrain + off + 4 * k as u32))
    }

    fn set(&mut self, mode: Mode) {
        if self.current == Some(mode) {
            return;
        }
        let o = self.src.object;
        self.ee.set_u32(self.terrain + o.coarse, u32::from(mode.coarse));
        self.ee.set_u32(self.terrain + o.split, if mode.split_one_way { 1 } else { 0 });
        // The ten-cell cache is keyed by the sample alone, not the width: empty it on every change.
        self.ee.set_u32(self.terrain + o.cache_ready, 0);
        self.current = Some(mode);
    }

    fn put(&mut self, at: u32, v: [u32; 4]) {
        for (k, w) in v.into_iter().enumerate() {
            self.ee.set_u32(at + 4 * k as u32, w);
        }
    }

    fn get(&self, at: u32) -> Vf {
        Vf::new(self.ee.f32_at(at), self.ee.f32_at(at + 4), self.ee.f32_at(at + 8), self.ee.f32_at(at + 12))
    }
}

impl Original for Oracle {
    fn query(&mut self, mode: Mode, at: Vf) -> Hit {
        self.set(mode);
        let buf = self.ee.scratch(0);
        self.put(buf, at.to_bits());
        self.put(buf + 0x10, [JUNK; 4]);
        self.put(buf + 0x20, [JUNK, 0, 0, 0]);
        self.put(buf + 0x30, [1.0f32.to_bits(); 4]);
        self.ee.call(self.src.query, &[u64::from(self.terrain), u64::from(buf)], &[], BUDGET);
        Hit {
            point: self.get(buf),
            normal: self.get(buf + 0x10),
            surfaces: self.ee.u32_at(buf + 0x20).to_le_bytes(),
            weights: self.get(buf + 0x30),
        }
    }

    fn ride(&mut self, mode: Mode, at: Vf) -> Option<Ride> {
        let f = self.src.ride_query?;
        self.set(mode);
        let (point, normal, surface) = (self.ee.scratch(0), self.ee.scratch(1), self.ee.scratch(2));
        self.put(point, at.to_bits());
        self.put(normal, [JUNK; 4]);
        self.put(surface, [0xff, 0, 0, 0]);
        let args = [self.terrain, point, normal, 0, surface].map(u64::from);
        self.ee.call(f, &args, &[], BUDGET);
        Some(Ride { y: self.ee.f32_at(point + 4), normal: self.get(normal), surface: self.ee.u8_at(surface) })
    }
}

/// The port's answer to the ride's query.
pub fn port_ride(field: &dyn Heightfield, at: Vf) -> Ride {
    let g = ground(field, at);
    Ride { y: g.point.y(), normal: g.normal, surface: nearest_surface(field, at).unwrap_or(0xff) }
}

/// openmx's field with the one step MX vs. ATV's cell reader adds ([`Normals::Over127`]): the corners'
/// normals are the file's bytes over 127. Heights and surfaces are the port's own.
pub struct Over127<'a> {
    pub field: TerrainField<'a>,
}

impl Heightfield for Over127<'_> {
    fn min(&self) -> Vf {
        self.field.min
    }
    fn max(&self) -> Vf {
        self.field.max
    }
    fn per_sample(&self) -> F {
        self.field.per_sample
    }
    fn height_scale(&self) -> F {
        self.field.height_scale
    }
    fn coarse(&self) -> bool {
        self.field.coarse
    }
    fn split_one_way(&self) -> bool {
        self.field.split_one_way
    }
    fn cell(&self, ix: i32, iz: i32, step: i32) -> Option<Cell> {
        let mut c = self.field.cell(ix, iz, step)?;
        let t = self.field.terrain;
        let (node, lx, lz) = t.leaf(u32::try_from(ix).ok()?, u32::try_from(iz).ok()?)?;
        let block = t.block(node.block)?;
        let k = F::ONE / F(127.0);
        let side = node.vertical_scale * k;
        let s = step as u32;
        for (i, (dx, dz)) in [(0, 0), (s, 0), (0, s), (s, s)].into_iter().enumerate() {
            let n = block.raw_normal(lx + dx, lz + dz)?.map(|b| F::from(i32::from(b)));
            c.normals[i] = Vf::new((n[0] * side).0, (n[1] * k).0, (n[2] * side).0, 1.0);
        }
        Some(c)
    }
}

/// The port the original is held against: openmx as it is, or openmx with the source's own normal
/// step.
pub fn port_with<'a>(field: TerrainField<'a>, normals: Normals) -> Box<dyn Heightfield + 'a> {
    match normals {
        Normals::Raw => Box::new(field),
        Normals::Over127 { .. } => Box::new(Over127 { field }),
    }
}

/// `n` points spread over `[lo, hi]` in x and z, the same every time for the same seed: three in four
/// anywhere, one in eight on a sample (where the cell index turns over), one in eight on a cell's edge.
pub fn points(n: usize, seed: u64, lo: [f32; 2], hi: [f32; 2], spacing: f32) -> Vec<[f32; 2]> {
    let mut s = seed | 1;
    let mut next = move || {
        // xorshift64*
        s ^= s >> 12;
        s ^= s << 25;
        s ^= s >> 27;
        s.wrapping_mul(0x2545_f491_4f6c_dd1d)
    };
    let mut unit = move || (next() >> 40) as f32 / (1u64 << 24) as f32;
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        let kind = out.len() % 8;
        let mut p = [lo[0] + unit() * (hi[0] - lo[0]), lo[1] + unit() * (hi[1] - lo[1])];
        if kind == 6 || kind == 7 {
            let i = if kind == 6 { 0 } else { 1 };
            p[i] = (p[i] / spacing).floor() * spacing;
            if kind == 7 {
                p[0] = (p[0] / spacing).floor() * spacing;
            }
        }
        if p.iter().zip(lo.iter().zip(hi.iter())).all(|(v, (l, h))| v >= l && v <= h) {
            out.push(p);
        }
    }
    out
}

/// A disagreement, with what each side said.
#[derive(Clone, Debug, PartialEq)]
pub struct Mismatch {
    pub mode: Mode,
    pub at: [f32; 2],
    pub what: &'static str,
    pub original: String,
    pub port: String,
}

/// What is compared, in report order.
pub const FIELDS: [&str; 7] = [
    "height",
    "normal",
    "four surfaces",
    "weights",
    "ride height",
    "ride normal",
    "ride surface",
];

/// One mode's tally.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tally {
    pub points: usize,
    /// Points the query found a cell under.
    pub cells: usize,
    pub rides: usize,
    /// Points at which a field differed, by [`FIELDS`] index.
    pub differ: [usize; 7],
    /// Points with any field differing.
    pub mismatches: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub modes: Vec<(Mode, Tally)>,
    /// The first disagreements, at most [`Report::KEEP`].
    pub mismatches: Vec<Mismatch>,
}

impl Report {
    pub const KEEP: usize = 12;

    pub fn total_mismatches(&self) -> usize {
        self.modes.iter().map(|(_, t)| t.mismatches).sum()
    }

    pub fn total_points(&self) -> usize {
        self.modes.iter().map(|(_, t)| t.points).sum()
    }

    /// Points at which one field differed, over every mode.
    pub fn differ(&self, field: &str) -> usize {
        let i = FIELDS.iter().position(|f| *f == field).expect("a field name");
        self.modes.iter().map(|(_, t)| t.differ[i]).sum()
    }
}

fn bits(v: Vf) -> String {
    format!("{:?} ({:08x?})", v.0, v.to_bits())
}

fn one(f: f32) -> String {
    format!("{f} ({:08x})", f.to_bits())
}

/// Every point in every mode, both queries, every field compared bit for bit.
pub fn compare(original: &mut dyn Original, port: &TerrainField, normals: Normals, modes: &[Mode], points: &[[f32; 2]], probe_y: f32) -> Report {
    let mut report = Report::default();
    for &mode in modes {
        let mut field = TerrainField { terrain: port.terrain, ..*port };
        field.coarse = mode.coarse;
        field.split_one_way = mode.split_one_way;
        let port = port_with(field, normals);
        let mut t = Tally::default();
        for &p in points {
            let at = Vf::new(p[0], probe_y, p[1], 1.0);
            let (o, q) = (original.query(mode, at), ground(port.as_ref(), at));
            t.points += 1;
            t.cells += usize::from(q.point.y() != at.y());
            let mut seen: Vec<(usize, String, String)> = Vec::new();
            if o.point.to_bits() != q.point.to_bits() {
                seen.push((0, bits(o.point), bits(q.point)));
            }
            if o.normal.to_bits() != q.normal.to_bits() {
                seen.push((1, bits(o.normal), bits(q.normal)));
            }
            if o.surfaces != q.surfaces {
                seen.push((2, format!("{:?}", o.surfaces), format!("{:?}", q.surfaces)));
            }
            if o.weights.to_bits() != q.weights.to_bits() {
                seen.push((3, bits(o.weights), bits(q.weights)));
            }
            if let Some(r) = original.ride(mode, at) {
                t.rides += 1;
                let pr = port_ride(port.as_ref(), at);
                if r.y.to_bits() != pr.y.to_bits() {
                    seen.push((4, one(r.y), one(pr.y)));
                }
                if r.normal.to_bits() != pr.normal.to_bits() {
                    seen.push((5, bits(r.normal), bits(pr.normal)));
                }
                if r.surface != pr.surface {
                    seen.push((6, r.surface.to_string(), pr.surface.to_string()));
                }
            }
            t.mismatches += usize::from(!seen.is_empty());
            for (i, o, q) in seen {
                t.differ[i] += 1;
                if report.mismatches.len() < Report::KEEP {
                    report.mismatches.push(Mismatch { mode, at: p, what: FIELDS[i], original: o, port: q });
                }
            }
        }
        report.modes.push((mode, t));
    }
    report
}

/// The terrain object's own numbers against the port's field: the same box, scale and step, or the
/// points would not be the same points.
pub fn object_checks(o: &Oracle, src: &Source, port: &TerrainField) -> Vec<(String, bool)> {
    let ob = src.object;
    let (omax, pmax) = (o.quad(ob.max), port.max);
    let omin = o.quad(ob.min);
    vec![
        (
            format!("samples per unit: original {:08x}, port {:08x}", o.field(ob.per_sample).to_bits(), port.per_sample.to_bits()),
            o.field(ob.per_sample).to_bits() == port.per_sample.to_bits(),
        ),
        (
            format!("height scale: original {}, port {}", o.field(ob.height_scale), port.height_scale.0),
            o.field(ob.height_scale).to_bits() == port.height_scale.to_bits(),
        ),
        (
            format!("box x, z: original {:?}..{:?}, port {:?}..{:?}", [omin[0], omin[2]], [omax[0], omax[2]], [port.min.x(), port.min.z()], [pmax.x(), pmax.z()]),
            omin[0] == port.min.x() && omin[2] == port.min.z() && omax[0] == pmax.x() && omax[2] == pmax.z(),
        ),
    ]
}

/// The box the points are drawn from: the original's, inside its border where its query clamps to one
/// (a point past it is moved onto it there, and the port has no such step: see `notes/accuracy.md`).
pub fn inner_box(o: &Oracle, src: &Source) -> ([f32; 2], [f32; 2]) {
    let (min, max) = (o.quad(src.object.min), o.quad(src.object.max));
    let b = src.object.border.map_or(0.0, |off| o.field(off));
    ([min[0] + b, min[2] + b], [max[0] - b, max[2] - b])
}

/// Read an image: exactly 32 MB.
pub fn load_ram(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let ram = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if ram.len() != ram::RAM_BYTES {
        return Err(format!("{}: {} bytes, not a 32 MB EE RAM image", path.display(), ram.len()));
    }
    Ok(ram)
}
