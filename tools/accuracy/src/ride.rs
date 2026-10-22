//! A mod ridden in openmx (T10): the shell started on the mod headless, a few seconds of the start
//! ridden, and what that says against the map.
//!
//! Four checks a map, and one shot:
//!
//! - **loads**: openmx exits cleanly, says it took the mod (`mods: <id> (...)`), and refuses nothing (no
//!   `left out` line, nothing on stderr).
//! - **heights**: the ground the shell builds for the mod — the folder through `Res::folder`, the one
//!   `.scn`, the terrain it names looked up by suffix as `track_mods::terrain_bytes` does, the scene's
//!   layout — asked as the ride asks it (`World::ground`) at the harness's points, against the harness's
//!   port on the mod's `.trn` ([`crate::look::height_at`]); and, where there is a RAM image of the source
//!   game, against the original's own query in all four modes ([`crate::heights::compare`]).
//! - **start**: the dirt under the start that openmx prints is the shell ground's, the start is over
//!   the terrain and not under it, and at the end of the ride the bike is on the ground.
//! - **edge**: openmx's `map:` line, its edge (MD3), against the scene's four terrain numbers (the
//!   original's), and the terrain file's own box beside it.
//! - **look**: a `--shot-pose` shot (MD10) from the overview pose, for the ground's textures (MD4); on a
//!   map whose `.toy` has an animated toy with collision (the windmills, MD5), one at that toy too.
//!
//! openmx's shell is a binary, which this repo cannot link: the shell's load is copied here from
//! `crates/openmx/src/track_mods.rs` and `main.rs`, and the log is how the run is read.

use crate::heights::{self, Oracle};
use crate::look;
use crate::modfiles::TrackMod;
use crate::pose::Pose;
use crate::source::{self, Normals};
use game_render::disc::Res;
use game_sim::biplane::Area;
use game_sim::track::{Scene, Terrain};
use game_sim::world::{Ground, World};
use modcheck::json::Json;
use ps2_float::{F, Vf};
use std::fmt::Write as _;
use std::path::Path;

/// openmx's command line for a ride: `look::OPENMX_ARGS` with `{frames}` frames, not two, so the bike
/// rides on after the shot (the shot is the first frame).
pub const RIDE_ARGS: &str = "--mods {mods} --track {id} --headless --no-sound --frames {frames} --size 640x480 --overlay none --graphics original --shot {shot} --shot-pose {pose}";

/// How many frames a ride is: openmx's headless ride takes 180 in 6 s of its clock.
pub const FRAMES: u32 = 180;

/// How far the bike may be from the ground under it at the end of the ride, in feet: its origin rides
/// a little over the dirt on its wheels (Lago: 0.1 ft), and a bike that fell through or sits on a prop
/// is many feet off.
pub const ON_THE_GROUND: f32 = 3.0;

/// The ground openmx's world query answers off the terrain (`world::NO_GROUND`).
pub const NO_GROUND: f32 = -1.0e6;

/// openmx's command line for `pose` with `frames` frames.
pub fn ride_args(folder: &Path, shot: &Path, pose: &Pose, frames: u32) -> Vec<String> {
    look::openmx_args(&RIDE_ARGS.replace("{frames}", &frames.to_string()), folder, shot, pose)
}

/// The mod as openmx's shell loads it.
pub struct Shell {
    pub scene: Scene,
    /// The `.scn`'s member name.
    pub scn: String,
    pub terrain: Vec<u8>,
}

impl Shell {
    /// The folder through `Res::folder`, as `track_mods::check` and the shell's track load read it: the
    /// one `.scn`, the terrain its `TerrainFile` names (as written, then with `.ps2`), and `mod.json`'s
    /// `spawn` put into the scene through the scene's own reader.
    pub fn load(dir: &Path) -> Result<Shell, String> {
        let res = Res::folder(dir)?;
        let scenes: Vec<&str> = res.entries().iter().map(|e| e.name.as_str()).filter(|n| n.to_ascii_lowercase().ends_with(".scn")).collect();
        let scn = match scenes.as_slice() {
            [one] => one.to_string(),
            [] => return Err("no .scn".into()),
            more => return Err(format!("{} .scn files, and a track has one", more.len())),
        };
        let mut scene = Scene::parse(&res.read(&scn)?).map_err(|e| format!("{scn}: {e}"))?;
        let terrain = terrain_bytes(&res, &scene.terrain_file)?;
        Terrain::parse(&terrain).map_err(|e| format!("{}: {e}", scene.terrain_file))?;
        if let Some((at, dir)) = spawn(dir)? {
            let text = format!("[SceneInfo]\r\nDefaultStartPosition={at}\r\nDefaultStartDirection={dir}\r\n[Environment]\r\nTerrainFile=-\r\n");
            let read = Scene::parse(text.as_bytes()).map_err(|e| format!("mod.json spawn: {e}"))?;
            scene.start = read.start;
            scene.start_direction = read.start_direction;
        }
        Ok(Shell { scene, scn, terrain })
    }

    pub fn parse(&self) -> Terrain<'_> {
        Terrain::parse(&self.terrain).expect("parsed once in load")
    }

    pub fn start(&self) -> [f32; 3] {
        self.scene.start.map(|c| c.0)
    }

    /// The map's edge as the shell's `off_map` measures it (MD3): the scene's four numbers.
    pub fn edge(&self) -> (f32, f32) {
        let t = &self.scene.terrain;
        let a = Area::new(F(t.width as f32), F(t.breadth as f32), t.width_scale, t.breadth_scale);
        (a.width.0, a.depth.0)
    }
}

/// `track_mods::terrain_bytes`: the name as written, then with the platform tag.
fn terrain_bytes(res: &Res, named: &str) -> Result<Vec<u8>, String> {
    if let Ok(bytes) = res.read_suffix(named) {
        return Ok(bytes);
    }
    let tagged = match named.rsplit_once('.') {
        Some((stem, ext)) => format!("{stem}.ps2.{ext}"),
        None => return Err(format!("no file ending in {named}")),
    };
    res.read_suffix(&tagged).map_err(|_| format!("no file ending in {named} or {tagged}"))
}

/// `mod.json`'s spawn as the text of its numbers, comma-joined, or none.
fn spawn(dir: &Path) -> Result<Option<(String, String)>, String> {
    let text = std::fs::read_to_string(dir.join("mod.json")).map_err(|e| format!("{}/mod.json: {e}", dir.display()))?;
    let json = Json::parse(&text).map_err(|e| format!("mod.json: {e}"))?;
    let Some(s) = json.get("spawn") else { return Ok(None) };
    let three = |key: &str| -> Result<String, String> {
        match s.get(key) {
            Some(Json::Arr(v)) if v.len() == 3 => {
                // The shell keeps the numbers' text; f64's shortest form reads back as the same number.
                let n: Vec<String> = v.iter().filter_map(Json::num).map(|n| n.to_string()).collect();
                if n.len() == 3 { Ok(n.join(",")) } else { Err(format!("mod.json: spawn {key} is not three numbers")) }
            }
            _ => Err(format!("mod.json: spawn {key} is not three numbers")),
        }
    };
    Ok(Some((three("position")?, three("direction")?)))
}

/// The ride's query, as the shell asks it for the dirt under the start: straight down from high up.
pub fn ground_y(ground: &mut Ground, x: f32, z: f32) -> f32 {
    World::ground(ground, Vf::new(x, 1.0e4, z, 1.0)).0.y()
}

/// The shell's ground against the harness's at `points`, bit for bit in `y`. The harness's is
/// `look::height_at` on the mod's `.trn` as `TrackMod` reads it; the shell's is `World::ground` on the
/// terrain and layout `Shell::load` found.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Against {
    pub points: usize,
    /// Points where the two differ, apart from `strip`.
    pub differ: usize,
    /// Points where openmx has no ground and the harness has: `world::Ground::at` stops a sample short of
    /// the far edges (`sx + 1 >= samples`), which is inside the shoot-back margin (`off_map::MARGIN`).
    pub strip: usize,
    pub first: Option<String>,
}

pub fn shell_against_harness(shell: &Shell, m: &TrackMod, points: &[[f32; 2]]) -> Against {
    let (st, ht) = (shell.parse(), m.parse());
    let mut sg = Ground::new(&st, &shell.scene.terrain);
    let hg = Ground::new(&ht, &m.layout);
    let field = hg.field();
    let (w, d) = shell.edge();
    let margin = game_sim::off_map::MARGIN.0;
    let mut a = Against { points: points.len(), ..Default::default() };
    for &[x, z] in points {
        let s = ground_y(&mut sg, x, z);
        let h = look::height_at(&field, x, z);
        if s.to_bits() == h.to_bits() {
            continue;
        }
        if s <= NO_GROUND && (x > w - margin || z > d - margin) {
            a.strip += 1;
        } else {
            a.differ += 1;
            a.first.get_or_insert_with(|| format!("at {x}, {z}: openmx {s}, harness {h}"));
        }
    }
    a
}

impl Against {
    /// The words for the table, after the original's count.
    fn words(&self) -> String {
        let mut s = String::new();
        if self.differ > 0 {
            let _ = write!(s, "; {} of {} differ from the harness ({})", self.differ, self.points, self.first.as_deref().unwrap_or(""));
        }
        if self.strip > 0 {
            let _ = write!(s, "; {} in the last sample strip have no ground in openmx's ride query (inside the edge margin)", self.strip);
        }
        s
    }
}

/// What one openmx run printed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Log {
    /// The `mods:` line's id: the mod was taken.
    pub loaded: Option<String>,
    /// `start: x, y, z facing ...`, to the foot.
    pub start: Option<[f32; 3]>,
    /// `the dirt under the start line is at ...`.
    pub dirt: Option<f32>,
    /// `map: W x D feet to its edge`.
    pub edge: Option<(f32, f32)>,
    /// `ridden for ... over N frames, ending at x, y, z`.
    pub frames: Option<u32>,
    pub end: Option<[f32; 3]>,
    /// `left out` lines and everything on stderr.
    pub refused: Vec<String>,
    /// The collision meshes `solid:` did not load, by the name it gives.
    pub unsolid: Vec<String>,
    /// Every `wreck: reason N`.
    pub wrecks: Vec<u32>,
}

fn numbers(s: &str) -> Vec<f32> {
    s.split(|c: char| c == ',' || c.is_whitespace()).filter_map(|w| w.parse().ok()).collect()
}

/// Read a ride's stdout and stderr.
pub fn read_log(stdout: &str, stderr: &str) -> Log {
    let mut log = Log::default();
    for line in stdout.lines() {
        let line = line.trim();
        if let Some(r) = line.strip_prefix("mods: ") {
            if r.contains("left out") {
                log.refused.push(line.to_owned());
            } else if let Some(id) = r.split_whitespace().next() {
                log.loaded = Some(id.to_owned());
            }
        } else if let Some(r) = line.strip_prefix("start: ") {
            if let [x, y, z, ..] = numbers(r.split(" facing").next().unwrap_or(""))[..] {
                log.start = Some([x, y, z]);
            }
        } else if let Some(r) = line.strip_prefix("the dirt under the start line is at ") {
            log.dirt = r.trim_end_matches(" feet").parse().ok();
        } else if let Some(r) = line.strip_prefix("map: ") {
            if let [w, d, ..] = numbers(&r.replace(" x ", " "))[..] {
                log.edge = Some((w, d));
            }
        } else if let Some(r) = line.strip_prefix("ridden for ") {
            if let Some((head, end)) = r.split_once(" ending at ") {
                log.frames = head.split_whitespace().rev().nth(1).and_then(|n| n.parse().ok());
                if let [x, y, z] = numbers(end)[..] {
                    log.end = Some([x, y, z]);
                }
            }
        } else if let Some(r) = line.strip_prefix("wreck: reason ") {
            if let Some(n) = r.split_whitespace().next().and_then(|n| n.parse().ok()) {
                log.wrecks.push(n);
            }
        } else if let Some((name, _)) = line.split_once(".col: ") {
            // Under `solid: a of b collision meshes (n)`, one line a mesh that did not load.
            log.unsolid.push(format!("{name}.col"));
        } else if line.contains("left out") {
            log.refused.push(line.to_owned());
        }
    }
    log.unsolid.dedup();
    log.refused.extend(stderr.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_owned));
    log
}

/// The first animated toy of the map's `.toy` with collision (`HasCollision=TRUE`), by its name and
/// position: what MD5's class-1 collision is for.
pub fn collision_toy(dir: &Path) -> Option<(String, [f32; 3])> {
    let res = Res::folder(dir).ok()?;
    let toy = res.entries().iter().find(|e| e.name.to_ascii_lowercase().ends_with(".toy"))?;
    let text = String::from_utf8_lossy(&res.read(&toy.name).ok()?).into_owned();
    let mut cur: Option<(Option<String>, Option<[f32; 3]>, bool)> = None;
    let mut found = None;
    let mut done = |c: &mut Option<(Option<String>, Option<[f32; 3]>, bool)>| {
        if let Some((Some(n), Some(p), true)) = c.take() {
            found.get_or_insert((n, p));
        }
    };
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            done(&mut cur);
            let name = line.trim_matches(|c| c == '[' || c == ']').to_ascii_lowercase();
            let toy = name.strip_prefix("animatedtoy").is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
            cur = toy.then_some((None, None, false));
            continue;
        }
        let Some(c) = cur.as_mut() else { continue };
        let Some((k, v)) = line.split_once('=') else { continue };
        match k.trim().to_ascii_lowercase().as_str() {
            "name" => c.0 = Some(v.trim().to_owned()),
            "position" => {
                if let [x, y, z] = numbers(v)[..] {
                    c.1 = Some([x, y, z]);
                }
            }
            "hascollision" => c.2 = v.trim().eq_ignore_ascii_case("true"),
            _ => {}
        }
    }
    done(&mut cur);
    found
}

/// One map's verdicts and the numbers behind them.
#[derive(Clone, Debug, Default)]
pub struct Verdict {
    pub id: String,
    pub loads: (bool, String),
    pub heights: (bool, String),
    pub start: (bool, String),
    pub edge: (bool, String),
}

impl Verdict {
    pub fn ok(&self) -> bool {
        self.loads.0 && self.heights.0 && self.start.0 && self.edge.0
    }

    /// One row of notes/accuracy.md's table: `| map | loads | heights | start | edge |`.
    pub fn row(&self) -> String {
        let cell = |(ok, why): &(bool, String)| if *ok { format!("ok: {why}") } else { format!("**no**: {why}") };
        format!("| `{}` | {} | {} | {} | {} |", self.id, cell(&self.loads), cell(&self.heights), cell(&self.start), cell(&self.edge))
    }
}

/// The heights: the shell's ground against the harness's at `n` points (by `heights::points`, over the
/// terrain's box), and with `ram` the original's own queries against the shell's ground in all four
/// modes, `n` points each.
pub fn heights_verdict(shell: &Shell, m: &TrackMod, ram: Option<&Path>, n: usize, seed: u64) -> Result<(bool, String), String> {
    let terrain = shell.parse();
    let ground = Ground::new(&terrain, &shell.scene.terrain);
    let field = ground.field();
    match ram {
        Some(image) => {
            let src = source::find(&m.boot).ok_or_else(|| format!("{}: no source game has boot ELF {:?}", m.id, m.boot))?;
            let bytes = heights::load_ram(image)?;
            let (key, scale) = m.root_key();
            let at = crate::ram::find_terrain(&bytes, &src, &key, scale);
            let [at] = at[..] else { return Err(format!("{}: the terrain object: {at:x?}", m.id)) };
            let mut oracle = Oracle::new(bytes, src, at);
            let checks = heights::object_checks(&oracle, &src, &field);
            let (lo, hi) = heights::inner_box(&oracle, &src);
            let points = heights::points(n, seed, lo, hi, terrain.scale.0);
            let modes = heights::modes(oracle.own_mode());
            let r = heights::compare(&mut oracle, &field, Normals::Raw, &modes, &points, 0.0);
            let mine = shell_against_harness(shell, m, &points);
            let bad = r.differ("height") + r.differ("ride height");
            let box_ok = checks.iter().all(|c| c.1);
            let mut why = format!("{} of {} points differ from the original{}", bad, r.total_points(), mine.words());
            // The box is not a height: openmx's Ground takes the scene's size, the game the file's leaves
            // (M12's ask), and the heights agree over both.
            if !box_ok {
                let bad: Vec<&str> = checks.iter().filter(|c| !c.1).map(|c| c.0.as_str()).collect();
                let _ = write!(why, "; {}", bad.join("; "));
            }
            Ok((bad == 0 && mine.differ == 0, why))
        }
        None => {
            let (ex, ez) = look::terrain_box(m, &m.parse());
            let points = heights::points(n * 4, seed, [0.0, 0.0], [ex, ez], terrain.scale.0);
            let mine = shell_against_harness(shell, m, &points);
            let why = format!("no RAM image: {} points against the harness{}", points.len(), mine.words());
            Ok((mine.differ == 0, why))
        }
    }
}

/// The wreck openmx's off-map test gives (`state::Wreck::CANNONBALL`).
pub const CANNONBALL: u32 = 13;

/// What a wreck's reason is (`air::Wreck`, `state::Wreck`).
pub fn wreck_name(reason: u32) -> &'static str {
    match reason {
        1 => "over the bars",
        2 | 3 => "off to one side",
        4 => "off the back",
        5 => "landed too hard",
        6 => "hit the scenery",
        7 => "hit a rider",
        8 => "nose-dived",
        11 => "landed on the rear",
        12 => "no hands on the bars",
        CANNONBALL => "off the map",
        14 => "nobody driving",
        _ => "?",
    }
}

/// The start and the end of the ride, against the shell's ground. The original stands the rider on
/// the ground over a start that is under it (M12, M13), and its Free Ride starts riding: so the bike
/// must end the ride on the ground, not wrecked. Where the scene's start is under the dirt, the row says
/// so.
pub fn start_verdict(shell: &Shell, log: &Log) -> (bool, String) {
    let terrain = shell.parse();
    let mut ground = Ground::new(&terrain, &shell.scene.terrain);
    let [sx, sy, sz] = shell.start();
    let under = ground_y(&mut ground, sx, sz);
    let mut ok = true;
    let mut why = Vec::new();
    if under <= NO_GROUND {
        why.push(format!("no terrain under the start ({sx:.0}, {sz:.0})"));
    } else if sy < under - 0.5 {
        why.push(format!("the scene's start is {:.1} ft under the dirt", under - sy));
    } else {
        why.push(format!("the start is {:.1} ft over the dirt", sy - under));
    }
    match log.dirt {
        Some(d) if (d - under).abs() <= 0.05 || (under <= NO_GROUND && d <= NO_GROUND) => {}
        Some(d) => {
            ok = false;
            why.push(format!("openmx's dirt {d:.1} is not the shell ground's {under:.1}"));
        }
        None => {
            ok = false;
            why.push("openmx printed no dirt under the start".into());
        }
    }
    match (log.end, log.frames) {
        (Some([x, y, z]), Some(f)) => {
            let g = ground_y(&mut ground, x, z);
            let gap = y - g;
            if g <= NO_GROUND || gap.abs() > ON_THE_GROUND {
                ok = false;
                why.push(format!("after {f} frames the bike is {gap:+.1} ft off the ground at {x:.0}, {y:.0}, {z:.0}"));
            } else {
                why.push(format!("after {f} frames on the ground ({gap:+.1} ft)"));
            }
        }
        _ => {
            ok = false;
            why.push("no ride".into());
        }
    }
    if !log.wrecks.is_empty() {
        ok = false;
        let reasons: Vec<String> = log.wrecks.iter().map(|&w| format!("{w} ({})", wreck_name(w))).collect();
        why.push(format!("wrecked: reason {}", reasons.join(", ")));
    }
    (ok, why.join("; "))
}

/// openmx's edge against the scene's numbers (MD3), with the terrain's own box and the scene's
/// `TerrainBorderWidth` beside it. The game holds `256 x width x scale + 2 x border` next to the four
/// numbers (found in the RAM images, T10); what reads it was not decompiled. A ride thrown off the map
/// at the start is not the original's (it rides from there).
pub fn edge_verdict(shell: &Shell, m: &TrackMod, log: &Log) -> (bool, String) {
    let (w, d) = shell.edge();
    let border = shell.scene.terrain.border_width.0;
    let (bx, bz) = look::terrain_box(m, &m.parse());
    let mut notes = Vec::new();
    if (bx - w).abs() >= 0.5 || (bz - d).abs() >= 0.5 {
        notes.push(format!("the terrain's box is {bx:.0} x {bz:.0}"));
    }
    if border > 0.0 {
        notes.push(format!("border {border:.0}: the game holds {:.0} x {:.0}", w + 2.0 * border, d + 2.0 * border));
    }
    let mut ok = true;
    let mut why = match log.edge {
        // openmx prints it to the foot.
        Some((lw, ld)) if (lw - w).abs() <= 0.5 && (ld - d).abs() <= 0.5 => format!("{lw:.0} x {ld:.0}, the scene's"),
        Some((lw, ld)) => {
            ok = false;
            format!("openmx {lw:.0} x {ld:.0}, the scene's {w:.0} x {d:.0}")
        }
        None => {
            ok = false;
            "openmx printed no map edge".into()
        }
    };
    if log.wrecks.contains(&CANNONBALL) {
        ok = false;
        notes.push("thrown off the map from the start".into());
    }
    for n in notes {
        why.push_str("; ");
        why.push_str(&n);
    }
    (ok, why)
}

/// The loads verdict: taken, nothing refused, and every collision mesh that did not load named in the
/// mod's `todo` (`todo`, its text).
pub fn loads_verdict(id: &str, log: &Log, exit_ok: bool, todo: &str) -> (bool, String) {
    let mut why = Vec::new();
    if !exit_ok {
        why.push("openmx failed".to_owned());
    }
    match &log.loaded {
        Some(got) if got == id => {}
        Some(got) => why.push(format!("openmx took {got}")),
        None => why.push("openmx did not take the mod".into()),
    }
    why.extend(log.refused.iter().take(3).cloned());
    let low = todo.to_ascii_lowercase();
    let (named, unnamed): (Vec<&String>, Vec<&String>) = log.unsolid.iter().partition(|n| low.contains(&n.to_ascii_lowercase()));
    for n in &unnamed {
        why.push(format!("{n} not loaded and not in todo"));
    }
    let ok = why.is_empty();
    let mut text = if ok { "taken, nothing refused".to_owned() } else { why.join("; ") };
    if !named.is_empty() {
        let names: Vec<&str> = named.iter().map(|n| n.as_str()).collect();
        let _ = write!(text, "; {} not loaded, in todo", names.join(", "));
    }
    (ok, text)
}
