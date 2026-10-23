//! Proves a built mod folder against `standard/OPENMX-TRACK-STANDARD.md` §5, with openmx's readers.
//!
//! [`check`] reads one folder's `mod.json` (§1), then by its `kind`:
//!
//! - **track** (§2): one `.scn`, read by `track::Scene::parse`; the terrain it names, found as the shell
//!   finds it and read by `track::Terrain::parse` (through [`terrain::md1_view`] until MD1, said in a
//!   note); the scene's layout against the terrain's own root cells, scale and extent; every distinct
//!   `COL=` read by `track::Collision::parse`, or its refusal in `todo`; no two paths that differ only
//!   in case.
//! - **world** (§3): every tile as a track, every join's terrain, every `.blend` summing to 255, every
//!   `drop` naming a section that exists.
//! - **music** (§4): every song's file is there.
//! - **tricks** (§6): every table, `.mcf`, `.MLB` and `.phy` its `mod.json` names is there.
//!
//! A mod copied off a disc is also compared, byte for byte, with the members its `source.members`
//! names, read off the disc with that `source.boot` through `game_sim::archive::{Psr, Res}` (discx).
//! A file in the folder that no member accounts for must be named in `todo`.
//!
//! Every problem is a `fail`. What was checked in a weaker way than the standard asks, or not at all,
//! is a `note`, so that nothing is skipped silently.

pub mod json;
pub mod terrain;

use discx::{Disc, Game};
use game_sim::track::{Collision, Scene, Terrain};
use json::Json;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// What one mod folder came to.
#[derive(Debug, Default)]
pub struct Report {
    pub dir: PathBuf,
    pub id: Option<String>,
    pub fail: Vec<String>,
    pub note: Vec<String>,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.fail.is_empty()
    }
}

/// The discs in `iso/`, opened once per boot ELF and kept for every mod that needs them.
pub struct Discs {
    iso: Option<PathBuf>,
    open: HashMap<String, Result<Vec<Disc>, String>>,
}

impl Discs {
    pub fn new(iso: Option<PathBuf>) -> Discs {
        Discs { iso, open: HashMap::new() }
    }

    /// Every PSR of the disc with this boot ELF (Untamed has two).
    fn get(&mut self, boot: &str) -> Result<&mut Vec<Disc>, String> {
        let iso = self.iso.clone();
        let r = self.open.entry(boot.to_ascii_uppercase()).or_insert_with(|| {
            let iso = iso.ok_or("no iso folder given")?;
            let game = Game::by_boot(boot).ok_or_else(|| format!("{boot} is not a disc this repo reads"))?;
            let (image, _) = discx::images(&iso)?
                .into_iter()
                .find(|(_, b)| b.as_deref().is_some_and(|b| b.eq_ignore_ascii_case(boot)))
                .ok_or_else(|| format!("no disc with boot ELF {boot} in {}", iso.display()))?;
            game.psrs.iter().map(|psr| Disc::open_psr(&image, Some(psr))).collect()
        });
        r.as_mut().map_err(|e| e.clone())
    }
}

/// The `source.game` names of §1 and the id prefix each gives.
fn prefix_for(game: &str) -> Option<String> {
    let fixed = match game {
        "MX Unleashed" => Some("mxu_"),
        "MX vs. ATV Unleashed" => Some("mxvatv_"),
        "MX vs. ATV Untamed" => Some("untamed_"),
        "MX vs. ATV Reflex" => Some("reflex_"),
        "MX vs. ATV Alive" => Some("alive_"),
        "Real elevation" => Some("elev_"),
        "Combined" => Some("world_"),
        _ => None,
    };
    if let Some(p) = fixed {
        return Some(p.to_owned());
    }
    let n = game.strip_prefix("ATV Offroad Fury")?.trim();
    (n.is_empty() || n.bytes().all(|b| b.is_ascii_digit())).then(|| format!("atvof{}_", if n.is_empty() { "1" } else { n }))
}

/// Paths in the folder, relative, `/`-separated, spelt as on disk; `mod.json` and `previews/` left
/// out, as the identity hash leaves out `previews/` (§1).
fn files(dir: &Path) -> Result<Vec<String>, String> {
    fn walk(root: &Path, at: &Path, out: &mut Vec<String>) -> Result<(), String> {
        let rd = std::fs::read_dir(at).map_err(|e| format!("{}: {e}", at.display()))?;
        for item in rd {
            let p = item.map_err(|e| e.to_string())?.path();
            let rel = p.strip_prefix(root).expect("under root").to_string_lossy().replace('\\', "/");
            if p.is_dir() {
                walk(root, &p, out)?;
            } else {
                out.push(rel);
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out)?;
    out.retain(|f| f != "mod.json" && !f.to_ascii_lowercase().starts_with("previews/"));
    out.sort();
    Ok(out)
}

/// The files a name finds the way the port looks one up (`disc::Res::find_suffix`): the end of the
/// path, lowercased, `\` and `/` alike. Matched on a segment boundary here, so `a.col` does not find
/// `Col/rocka.col` (the port's plain suffix would: `notes/openmx-asks.md`, MD2).
fn lookup<'a>(files: &'a [String], name: &str) -> Vec<&'a String> {
    let want = name.replace('\\', "/").trim_start_matches('/').to_ascii_lowercase();
    files
        .iter()
        .filter(|f| {
            let f = f.to_ascii_lowercase();
            f == want || f.ends_with(&format!("/{want}"))
        })
        .collect()
}

fn base(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

/// Whether some `todo` entry names this file (its base name, any case).
fn in_todo(todo: &[String], file: &str) -> bool {
    let b = base(file).to_ascii_lowercase();
    todo.iter().any(|t| t.to_ascii_lowercase().contains(&b))
}

/// The parts of `mod.json` the checks below need, after §1's fields are checked.
struct Meta {
    kind: String,
    id: String,
    game: String,
    boot: Option<String>,
    members: Vec<String>,
    todo: Vec<String>,
    json: Json,
}

/// Check one mod folder.
pub fn check(dir: &Path, discs: &mut Discs) -> Report {
    let mut r = Report { dir: dir.to_path_buf(), ..Report::default() };
    check_into(dir, discs, false, &mut r);
    r
}

/// Check one track mod folder as a world's tile would be (§3): an `mxu_` id is allowed (§1), so an
/// MX Unleashed import can be read back before it is a world's.
pub fn check_tile(dir: &Path, discs: &mut Discs) -> Report {
    let mut r = Report { dir: dir.to_path_buf(), ..Report::default() };
    check_into(dir, discs, true, &mut r);
    r
}

fn check_into(dir: &Path, discs: &mut Discs, as_tile: bool, r: &mut Report) {
    let Some(meta) = mod_json(dir, as_tile, r) else { return };
    r.id = Some(meta.id.clone());
    let files = match files(dir) {
        Ok(f) => f,
        Err(e) => return r.fail.push(e),
    };
    case_collisions(&files, r);
    match meta.kind.as_str() {
        "track" => {
            track(dir, &files, &meta, r);
            copies(dir, &files, &meta, discs, r);
        }
        "world" => world(dir, &meta, discs, r),
        "music" => {
            music(dir, &meta, r);
            copies(dir, &files, &meta, discs, r);
        }
        "tricks" => {
            tricks(dir, &meta, r);
            copies(dir, &files, &meta, discs, r);
        }
        // §1: any other kind is skipped quietly by the loader, so it is not this check's to fail.
        other => r.note.push(format!("kind {other:?} is not one the standard defines; the loader skips it")),
    }
}

/// §1: `mod.json` parses and has every required field, each with the meaning v1 gives it.
fn mod_json(dir: &Path, as_tile: bool, r: &mut Report) -> Option<Meta> {
    let path = dir.join("mod.json");
    let text = match std::fs::read(&path) {
        Ok(b) => match String::from_utf8(b) {
            Ok(t) => t,
            Err(_) => {
                r.fail.push("mod.json is not UTF-8".into());
                return None;
            }
        },
        Err(e) => {
            r.fail.push(format!("mod.json: {e}"));
            return None;
        }
    };
    let j = match Json::parse(&text) {
        Ok(j) => j,
        Err(e) => {
            r.fail.push(format!("mod.json does not parse: {e}"));
            return None;
        }
    };
    if !matches!(j, Json::Obj(_)) {
        r.fail.push("mod.json is not an object".into());
        return None;
    }
    let fails = r.fail.len();
    let s = |k: &str| j.get(k).and_then(Json::str).map(str::to_owned);
    let need = |k: &str, r: &mut Report| -> String {
        s(k).unwrap_or_else(|| {
            r.fail.push(format!("mod.json: {k} is missing or not a string"));
            String::new()
        })
    };

    match j.get("standard").and_then(Json::num) {
        Some(n) if n == 1.0 => {}
        Some(n) => r.fail.push(format!("mod.json: standard is {n}; this check knows 1")),
        None => r.fail.push("mod.json: standard is missing or not a number".into()),
    }
    let kind = need("kind", r);
    let id = need("id", r);
    let name = need("name", r);
    let version = need("version", r);
    need("author", r);
    let permission = need("permission", r);

    let folder = dir.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
    if id != folder {
        r.fail.push(format!("mod.json: id {id:?} is not the folder's name {folder:?}"));
    }
    if id.is_empty() || id.len() > 40 || !id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') {
        r.fail.push(format!("mod.json: id {id:?} is not 1-40 of [a-z0-9_]"));
    }
    if name.trim().is_empty() {
        r.fail.push("mod.json: name is empty".into());
    }
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit())) {
        r.fail.push(format!("mod.json: version {version:?} is not major.minor.patch"));
    }
    if permission.trim().is_empty() {
        r.fail.push("mod.json: permission is empty".into());
    }

    let mut game = String::new();
    let mut boot = None;
    let mut members = Vec::new();
    match j.get("source") {
        Some(src @ Json::Obj(_)) => {
            game = src.get("game").and_then(Json::str).unwrap_or_default().to_owned();
            match prefix_for(&game) {
                None => r.fail.push(format!("mod.json: source.game {game:?} is not one of the standard's names")),
                Some(p) => {
                    if !id.starts_with(&p) {
                        r.fail.push(format!("mod.json: id {id:?} does not start with {p:?}, {game}'s prefix"));
                    }
                    if p == "mxu_" && !as_tile {
                        r.fail.push("mod.json: an MX Unleashed track is only a world's tile (§1)".into());
                    }
                    if (p == "world_") != (kind == "world") {
                        r.fail.push(format!("mod.json: source.game {game:?} with kind {kind:?}"));
                    }
                }
            }
            let strs = |k: &str| -> Option<Vec<String>> {
                src.get(k)?.arr()?.iter().map(|v| v.str().map(str::to_owned)).collect()
            };
            if kind == "world" {
                if strs("games").is_none_or(|g| g.is_empty()) {
                    r.fail.push("mod.json: a world's source.games is missing or empty".into());
                }
            } else if let Some(b) = src.get("boot") {
                match (b.str(), strs("members")) {
                    (Some(b), Some(m)) if !m.is_empty() => {
                        match Game::by_boot(b) {
                            Some(g) if g.title == game => {}
                            Some(g) => r.fail.push(format!("mod.json: boot {b} is {}, not {game}", g.title)),
                            None => r.note.push(format!("boot {b} is not one of AGENTS.md's discs")),
                        }
                        boot = Some(b.to_owned());
                        members = m;
                    }
                    _ => r.fail.push("mod.json: source.boot needs a string and a non-empty source.members".into()),
                }
            } else if src.get("data").and_then(Json::str).is_none() || src.get("date").and_then(Json::str).is_none() {
                r.fail.push("mod.json: source has neither boot and members nor data and date".into());
            }
        }
        _ => r.fail.push("mod.json: source is missing or not an object".into()),
    }

    let todo = match j.get("todo").and_then(Json::arr).map(|a| a.iter().map(|v| v.str().map(str::to_owned)).collect::<Option<Vec<_>>>()) {
        Some(Some(t)) => t,
        _ => {
            r.fail.push("mod.json: todo is missing or not a list of strings".into());
            Vec::new()
        }
    };
    if todo.is_empty() && j.get("checked").is_none() && kind != "world" {
        r.fail.push("mod.json: todo is [] but T7 has not checked it (no checked)".into());
    }
    if let Some(c) = j.get("checked") {
        if c.str().is_none_or(|c| c.trim().is_empty()) {
            r.fail.push("mod.json: checked is not a string".into());
        }
    }
    if let Some(sp) = j.get("spawn")
        && kind == "track"
    {
        spawn(sp, "spawn", r);
    }
    if r.fail.len() > fails {
        return None;
    }
    Some(Meta { kind, id, game, boot, members, todo, json: j })
}

/// §2b: a spawn is a position, a direction and where they were read from.
fn spawn(sp: &Json, what: &str, r: &mut Report) {
    let v3 = |k: &str| sp.get(k).and_then(Json::arr).is_some_and(|a| a.len() == 3 && a.iter().all(|x| x.num().is_some()));
    if !v3("position") || !v3("direction") {
        r.fail.push(format!("mod.json: {what} needs position and direction, three numbers each"));
    }
    if sp.get("from").and_then(Json::str).is_none_or(|f| f.trim().is_empty()) {
        r.fail.push(format!("mod.json: {what} has no from: a spawn with no source is a guess (§2b)"));
    }
}

/// §2: two files whose paths differ only in case break the mod.
fn case_collisions(files: &[String], r: &mut Report) {
    let mut seen: BTreeMap<String, &String> = BTreeMap::new();
    for f in files {
        if let Some(first) = seen.insert(f.to_ascii_lowercase(), f) {
            r.fail.push(format!("{first} and {f} differ only in case"));
        }
    }
}

/// A terrain read by openmx, directly or through the MD1 view, and what the checks need from it.
struct TerrainFacts {
    scale: f32,
    levels: u32,
    cells: (u32, u32),
    leaves_missing: usize,
}

fn read_terrain(bytes: &[u8], what: &str, r: &mut Report) -> Option<TerrainFacts> {
    let facts = |t: &Terrain| -> Result<TerrainFacts, String> {
        let root = t.root().ok_or("the terrain has no root node")?;
        let (mut cx_max, mut cz_max) = (0, 0);
        for cz in 0..16 {
            for cx in 0..16 {
                if t.child(&root, cx, cz).is_some() {
                    cx_max = cx_max.max(cx + 1);
                    cz_max = cz_max.max(cz + 1);
                }
            }
        }
        if t.levels == 0 || t.levels > 4 {
            return Err(format!("{} tree levels", t.levels));
        }
        // Every leaf in the root cells' rectangle resolves to a block, by the game's own descent.
        let side = 16u32.pow(t.levels);
        let mut missing = 0;
        for lz in (0..cz_max * side).step_by(16) {
            for lx in (0..cx_max * side).step_by(16) {
                if t.leaf(lx, lz).and_then(|(n, _, _)| t.block(n.block)).is_none() {
                    missing += 1;
                }
            }
        }
        Ok(TerrainFacts { scale: t.scale.0, levels: t.levels, cells: (cx_max, cz_max), leaves_missing: missing })
    };
    let direct = Terrain::parse(bytes).and_then(|t| facts(&t));
    match direct {
        Ok(f) => Some(f),
        Err(refusal) => match terrain::md1_view(bytes) {
            Ok(view) => match Terrain::parse(&view.bytes).and_then(|t| facts(&t)) {
                Ok(f) => {
                    r.note.push(format!(
                        "{what}: Terrain::parse refuses it ({refusal}); read through the MD1 view: \
                         {} texture pages, {} bytes, between the blocks and the tree",
                        view.pages, view.gap
                    ));
                    Some(f)
                }
                Err(e) => {
                    r.fail.push(format!("{what}: Terrain::parse refuses it ({refusal}), and its MD1 view too ({e})"));
                    None
                }
            },
            Err(why) => {
                r.fail.push(format!("{what}: Terrain::parse refuses it ({refusal}); no MD1 view: {why}"));
                None
            }
        },
    }
}

/// The one file a name finds, as `lookup` does, or the failure.
fn one<'a>(files: &'a [String], name: &str) -> Result<Option<&'a String>, String> {
    match lookup(files, name).as_slice() {
        [] => Ok(None),
        [f] => Ok(Some(f)),
        many => Err(format!("{name} finds {} files: {}", many.len(), many.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))),
    }
}

/// §2 and §2a, and §5 step 2.
fn track(dir: &Path, files: &[String], meta: &Meta, r: &mut Report) {
    let scenes: Vec<&String> = files.iter().filter(|f| f.to_ascii_lowercase().ends_with(".scn")).collect();
    let scn = match scenes.as_slice() {
        [s] => *s,
        [] => return r.fail.push("no .scn: a track mod has exactly one".into()),
        many => return r.fail.push(format!("{} .scn files, a track mod has one: {:?}", many.len(), many)),
    };
    let stem = base(scn);
    let stem = &stem[..stem.len() - 4];
    if meta.game == "Real elevation" && stem != meta.id {
        r.fail.push(format!("{scn}: a generated map's stem is its id {:?} (§2)", meta.id));
    }
    let bytes = match std::fs::read(dir.join(scn)) {
        Ok(b) => b,
        Err(e) => return r.fail.push(format!("{scn}: {e}")),
    };
    let scene = match Scene::parse(&bytes) {
        Ok(s) => s,
        Err(e) => return r.fail.push(format!("{scn}: Scene::parse: {e}")),
    };

    // The terrain: the name as written, then with the platform tag inserted (§2).
    let mut names = vec![scene.terrain_file.clone()];
    if let Some(dot) = scene.terrain_file.rfind('.') {
        names.push(format!("{}.ps2{}", &scene.terrain_file[..dot], &scene.terrain_file[dot..]));
    }
    let mut found = None;
    for n in &names {
        match one(files, n) {
            Ok(Some(f)) => {
                found = Some(f);
                break;
            }
            Ok(None) => {}
            Err(e) => return r.fail.push(format!("{scn}: TerrainFile {e}")),
        }
    }
    let Some(trn) = found else {
        return r.fail.push(format!("{scn}: TerrainFile={} finds no file (tried {})", scene.terrain_file, names.join(", ")));
    };
    match std::fs::read(dir.join(trn)) {
        Err(e) => r.fail.push(format!("{trn}: {e}")),
        Ok(tb) => {
            if let Some(t) = read_terrain(&tb, trn, r) {
                layout(scn, trn, &scene, &t, &meta.todo, r);
            }
        }
    }

    // Collision: one `.col` per distinct name the scene places (§2).
    let mut seen = HashSet::new();
    let (mut read, mut listed) = (0, 0);
    for p in &scene.collision {
        if !seen.insert(p.file.to_ascii_lowercase()) {
            continue;
        }
        let mut f = one(files, &p.file);
        if matches!(f, Ok(None)) && !p.file.contains('.') {
            f = one(files, &format!("{}.col", p.file));
        }
        let outcome = match f {
            Err(e) => Err(e),
            Ok(None) => Err(format!("{}: placed by the scene, not in the folder", p.file)),
            Ok(Some(f)) => match std::fs::read(dir.join(f)) {
                Err(e) => Err(format!("{f}: {e}")),
                Ok(b) => Collision::parse(&b).map(|_| ()).map_err(|e| format!("{f}: Collision::parse: {e}")),
            },
        };
        match outcome {
            Ok(()) => read += 1,
            Err(e) if in_todo(&meta.todo, &p.file) => {
                listed += 1;
                r.note.push(format!("{e} (in todo)"));
            }
            Err(e) => r.fail.push(format!("{e}, and todo does not name it")),
        }
    }
    r.note.push(format!(
        "{scn}: {} models, {} collision placements of {} shapes: {read} read, {listed} listed in todo",
        scene.models.len(),
        scene.collision.len(),
        seen.len()
    ));
}

/// §2a: the scene's `[Environment]` layout is the terrain's own.
///
/// `TerrainWidth` counts 256-sample units (the map edge is width × 256 × scale, `0x001a6e50`), which
/// is the tree's root cells only when it has two levels: a one-level tree (MX vs. ATV's supercross and
/// freestyle maps) has 16 root cells of 16 samples and `TerrainWidth=1`. So the samples are compared.
/// The scene is never edited (§2a), so where a disc's own scene disagrees with its terrain the mod can
/// only say so: a `todo` entry naming the scene and the key turns that fail into a note.
fn layout(scn: &str, trn: &str, scene: &Scene, t: &TerrainFacts, todo: &[String], r: &mut Report) {
    let l = &scene.terrain;
    let side = 16u32.pow(t.levels);
    let samples = (t.cells.0 * side, t.cells.1 * side);
    let stem = base(scn).to_ascii_lowercase();
    let gap = |key: &str, what: String, r: &mut Report| {
        let k = key.to_ascii_lowercase();
        if todo.iter().any(|e| {
            let e = e.to_ascii_lowercase();
            e.contains(&stem) && e.contains(&k)
        }) {
            r.note.push(format!("{what} (in todo)"));
        } else {
            r.fail.push(format!("{what}, and no todo entry names {} and {key}", base(scn)));
        }
    };
    for (key, n, have) in [("TerrainWidth", l.width, samples.0), ("TerrainBreadth", l.breadth, samples.1)] {
        if u64::from(n) * 256 != u64::from(have) {
            gap(key, format!("{scn}: {key}={n} is {} samples, but {trn} has {have} ({} root cells of {side})", u64::from(n) * 256, have / side), r);
        }
        if n > 16 {
            gap(key, format!("{scn}: {key}={n} is over 16"), r);
        }
    }
    for (key, v) in [("TerrainWidthScale", l.width_scale.0), ("TerrainBreadthScale", l.breadth_scale.0)] {
        if v != t.scale {
            gap(key, format!("{scn}: {key}={v} is not {trn}'s scale {}", t.scale), r);
        }
    }
    if t.leaves_missing > 0 {
        // The source's own tree; the game's query finds no ground there, as on the disc.
        r.note.push(format!("{trn}: {} leaves inside its root cells' rectangle have no block", t.leaves_missing));
    }
    r.note.push(format!(
        "{trn}: {} × {} root cells, {} levels, {} × {} samples at {} ft",
        t.cells.0, t.cells.1, t.levels, samples.0, samples.1, t.scale
    ));
}

/// §5 step 1: every member's bytes are in the folder at its own path, and nothing else is, unless
/// `todo` says why.
fn copies(dir: &Path, files: &[String], meta: &Meta, discs: &mut Discs, r: &mut Report) {
    let Some(boot) = &meta.boot else {
        r.note.push("not from a disc: no byte check against members".into());
        return;
    };
    let psrs = match discs.get(boot) {
        Ok(p) => p,
        Err(e) => return r.fail.push(format!("cannot prove the copy: {e}")),
    };
    let mut covered: HashSet<String> = HashSet::new();
    let mut compared = 0usize;
    let mut compare = |rel: String, want: &[u8], from: &str, r: &mut Report| {
        compared += 1;
        match std::fs::read(dir.join(&rel)) {
            Err(_) => r.fail.push(format!("{rel}: missing (from {from})")),
            Ok(have) if have == want => {}
            Ok(have) => {
                let at = have.iter().zip(want).position(|(a, b)| a != b).unwrap_or(have.len().min(want.len()));
                r.fail.push(format!("{rel}: differs from {from} ({} bytes, the member {}; first at byte {at})", have.len(), want.len()));
            }
        }
        covered.insert(rel);
    };
    for m in &meta.members {
        let mut hits = 0;
        for disc in psrs.iter_mut() {
            let entries: Vec<_> = disc.matching(m).into_iter().cloned().collect();
            for e in entries {
                hits += 1;
                if e.name.to_ascii_lowercase().ends_with(".res") {
                    match disc.res(&e) {
                        Err(err) => r.fail.push(format!("{}: {err}", e.name)),
                        Ok(res) => {
                            if res.padding > 0 {
                                r.note.push(format!(
                                    "{}: Res::parse refuses it for the {} zero bytes padding its directory; read through a view without them",
                                    e.name, res.padding
                                ));
                            }
                            for inner in res.entries().to_vec() {
                                match res.read(&inner) {
                                    Ok(want) => compare(inner.path(), &want, &format!("{}:{}", e.name, inner.name), r),
                                    Err(err) => r.fail.push(format!("{}:{}: {err}", e.name, inner.name)),
                                }
                            }
                        }
                    }
                } else {
                    match disc.read(&e) {
                        Ok(want) => compare(e.path(), &want, &e.name, r),
                        Err(err) => r.fail.push(format!("{}: {err}", e.name)),
                    }
                }
            }
        }
        if hits == 0 {
            r.fail.push(format!("source.members {m}: not on the {boot} disc"));
        }
    }
    let mut extra = Vec::new();
    for f in files {
        if !covered.contains(f) {
            if in_todo(&meta.todo, f) {
                r.note.push(format!("{f}: not a disc member, named in todo"));
            } else {
                extra.push(f.as_str());
            }
        }
    }
    if !extra.is_empty() {
        r.fail.push(format!("{} files are no member of source.members and todo does not name them: {}", extra.len(), extra.join(", ")));
    }
    r.note.push(format!("{compared} files compared byte for byte with the {boot} disc"));
}

/// §3, the parts that are this check's; seams and `keep` are C1's `--check`.
fn world(dir: &Path, meta: &Meta, discs: &mut Discs, r: &mut Report) {
    let j = &meta.json;
    let mut tiles: Vec<String> = Vec::new();
    for t in j.get("tiles").and_then(Json::arr).unwrap_or_default() {
        let Some(name) = t.get("tile").and_then(Json::str) else {
            r.fail.push("tiles[]: no tile".into());
            continue;
        };
        tiles.push(name.to_owned());
        let at_ok = t.get("at").and_then(Json::arr).is_some_and(|a| a.len() == 2 && a.iter().all(|x| x.num().is_some()));
        let keep_ok = t.get("keep").and_then(Json::arr).is_some_and(|a| {
            a.len() == 4 && a.iter().all(|x| x.num().is_some_and(|n| n >= 0.0 && n.fract() == 0.0))
        });
        if !at_ok || !keep_ok {
            r.fail.push(format!("tiles[{name}]: at needs [x, z] and keep [x0, z0, x1, z1] in whole samples"));
        }
        let mut sub = Report { dir: dir.join("tiles").join(name), ..Report::default() };
        check_into(&sub.dir.clone(), discs, true, &mut sub);
        r.fail.extend(sub.fail.into_iter().map(|f| format!("tiles/{name}: {f}")));
        r.note.extend(sub.note.into_iter().map(|n| format!("tiles/{name}: {n}")));
    }
    if tiles.is_empty() {
        r.fail.push("a world has no tiles".into());
    }
    for jn in j.get("joins").and_then(Json::arr).unwrap_or_default() {
        let Some(name) = jn.get("join").and_then(Json::str) else {
            r.fail.push("joins[]: no join".into());
            continue;
        };
        let between: Vec<&str> = jn.get("between").and_then(Json::arr).unwrap_or_default().iter().filter_map(Json::str).collect();
        if between.len() < 2 || between.iter().any(|b| !tiles.iter().any(|t| t == b)) {
            r.fail.push(format!("joins[{name}]: between must name two or more of the tiles"));
        }
        let jd = dir.join("joins").join(name);
        match std::fs::read(jd.join(format!("{name}.ps2.trn"))) {
            Err(e) => r.fail.push(format!("joins/{name}/{name}.ps2.trn: {e}")),
            Ok(b) => {
                read_terrain(&b, &format!("joins/{name}/{name}.ps2.trn"), r);
            }
        }
        match std::fs::read(jd.join(format!("{name}.blend"))) {
            Err(e) => r.fail.push(format!("joins/{name}/{name}.blend: {e}")),
            Ok(b) if between.is_empty() || b.len() % between.len() != 0 => {
                r.fail.push(format!("joins/{name}/{name}.blend: {} bytes is not whole samples of {} weights", b.len(), between.len()));
            }
            Ok(b) => {
                let bad = b.chunks(between.len()).filter(|c| c.iter().map(|&x| u32::from(x)).sum::<u32>() != 255).count();
                if bad > 0 {
                    r.fail.push(format!("joins/{name}/{name}.blend: {bad} samples do not sum to 255"));
                }
            }
        }
    }
    for d in j.get("drop").and_then(Json::arr).unwrap_or_default() {
        let (Some(tile), Some(section)) = (d.get("tile").and_then(Json::str), d.get("section").and_then(Json::str)) else {
            r.fail.push("drop[]: needs tile and section".into());
            continue;
        };
        if d.get("why").and_then(Json::str).is_none() {
            r.fail.push(format!("drop[{tile} {section}]: no why"));
        }
        let td = dir.join("tiles").join(tile);
        let want = format!("[{}]", section.to_ascii_lowercase());
        let found = files(&td).ok().into_iter().flatten().filter(|f| f.to_ascii_lowercase().ends_with(".scn")).any(|f| {
            std::fs::read(td.join(&f)).is_ok_and(|b| String::from_utf8_lossy(&b).lines().any(|l| l.trim().to_ascii_lowercase() == want))
        });
        if !found {
            r.fail.push(format!("drop: tiles/{tile}'s scene has no [{section}]"));
        }
    }
    for k in ["sky"] {
        if let Some(t) = j.get(k).and_then(Json::str)
            && !tiles.iter().any(|x| x == t)
        {
            r.fail.push(format!("{k}: {t} is not a tile"));
        }
    }
    match j.get("spawn") {
        Some(sp) if sp.get("tile").is_some() => {
            if !sp.get("tile").and_then(Json::str).is_some_and(|t| tiles.iter().any(|x| x == t)) {
                r.fail.push("spawn.tile is not a tile".into());
            }
        }
        Some(sp) => spawn(sp, "spawn", r),
        None => {}
    }
    r.note.push("seams, slopes and keep against the tile alone are C1's --check, not run here".into());
}

/// §4, what can be checked without openmx's shell crate.
fn music(dir: &Path, meta: &Meta, r: &mut Report) {
    let songs = meta.json.get("songs").and_then(Json::arr).unwrap_or_default();
    if songs.is_empty() {
        r.fail.push("a music pack has no songs".into());
    }
    for s in songs {
        let file = s.get("file").and_then(Json::str).unwrap_or_default();
        for k in ["file", "title", "artist", "from"] {
            if s.get(k).and_then(Json::str).is_none_or(|v| v.trim().is_empty()) {
                r.fail.push(format!("songs[{file}]: no {k}"));
            }
        }
        if !file.is_empty() && !dir.join(file).is_file() {
            r.fail.push(format!("songs[{file}]: the file is not in the pack"));
        }
    }
    r.note.push(
        "Adpcm::vig not run: it is in openmx's shell crate (wgpu, iroh), which this repo does not build against"
            .into(),
    );
}

/// §6, what can be checked without the tables' reader: every file `mod.json` names is in the pack.
/// The tables and clips are read back by `import-tricks check` (Table::parse, openmx's
/// `Stunts::parse` and `skin::Clip::parse`).
fn tricks(dir: &Path, meta: &Meta, r: &mut Report) {
    let j = &meta.json;
    let tables = j.get("tables").and_then(Json::arr).unwrap_or_default();
    if tables.is_empty() {
        r.fail.push("a trick pack has no tables".into());
    }
    let mut named: Vec<String> = Vec::new();
    for t in tables {
        match t.get("file").and_then(Json::str) {
            Some(f) => named.push(f.to_owned()),
            None => r.fail.push("tables[]: no file".into()),
        }
    }
    for rd in j.get("riders").and_then(Json::arr).unwrap_or_default() {
        for k in ["table", "mcf", "mlb"] {
            if let Some(f) = rd.get(k).and_then(Json::str) {
                named.push(f.to_owned());
            }
        }
    }
    named.extend(j.get("phy").and_then(Json::arr).unwrap_or_default().iter().filter_map(|v| v.str().map(str::to_owned)));
    for n in j.get("new").and_then(Json::arr).unwrap_or_default() {
        if n.get("name").and_then(Json::str).is_none_or(|v| v.trim().is_empty()) {
            r.fail.push("new[]: no name".into());
        }
    }
    for f in named {
        if !dir.join(&f).is_file() {
            r.fail.push(format!("{f}: named in mod.json, not in the pack"));
        }
    }
    r.note.push("tables and clips not read here: `import-tricks check` reads them back".into());
}

/// Every mod folder under `root`: `root` itself if it has a `mod.json`, else its children, else
/// theirs (so `out/mods`, `out/mods/tracks` and one mod all work).
pub fn find_mods(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut level = vec![root.to_path_buf()];
    for _ in 0..3 {
        let mut next = Vec::new();
        for d in level {
            if d.join("mod.json").is_file() {
                out.push(d);
            } else {
                next.extend(std::fs::read_dir(&d).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.is_dir()));
            }
        }
        level = next;
    }
    out.sort();
    out
}
