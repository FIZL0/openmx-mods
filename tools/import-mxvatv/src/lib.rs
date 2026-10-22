//! MX vs. ATV Unleashed's maps as track mods (standard/OPENMX-TRACK-STANDARD.md §2). Untamed's go
//! through the same import, with its binary INI decoded in front ([`untamed`]).
//!
//! ```text
//! MXVSATV.PSR ── Env\Nat03\nat03.res ─────┐
//!             ── Env\Nat03\nat03_mat.res ─┼─ unpacked, member paths kept ──> <out>/mxvatv_nat03/
//!             ── Env\Nat03\nat03_mst.res ─┘                                    mod.json
//!             ── Env\Nat03\Nat03.atyList   [Free Ride]: name, start ───────────^
//! ```
//!
//! A map's archives are copied as the disc has them; nothing is edited, resampled or dropped (§2).
//! The loose PSR members the game opens for the map because its archives lack them are copied
//! beside them ([`pull_loose`]).
//! Everything the port reads, or would read differently, is then looked at with openmx's own readers:
//!
//! - **game-sim**: `track::Scene` (spawn, terrain layout, placements), `track::Terrain` (through
//!   modcheck's MD1 view when it refuses the file, as it does every MX vs. ATV terrain), and
//!   `track::Collision` for every distinct `COL=` the scene places.
//! - **game-render**: its scene (props, sky, ecosystem), `psg::Model` for every placed prop, plant
//!   and toy car, every material those name resolved to a texture as `world::Assets` does it,
//!   `sky::Sky`, `eco::Ecosystem`, `toy::cars`, and every `.psg`, `.pst`, `.atm`, `.cub` and `.mot` in
//!   the folder by its reader.
//! - **The scene's keys** against every MX Unleashed scene's ([`mxu_keys`]): a key no MXU scene has is
//!   one the port does not read.
//!
//! The scene is never edited (§2a). What the port reads differently goes in `mod.json`: the Free
//! Ride's start when the `.atyList` gives one (`spawn`, §2b), and in `lookups` the files the scene's
//! `TerrainFile`, `CubeFile` and `EcosystemFile` come to by the standard's lookup. Every gap the
//! readers find is a `todo` entry naming the file and the reason.

pub mod mxu_keys;
pub mod untamed;

use discx::{Disc, ResFile, normalise};
use game_render::{eco, ini, psg, pst, scene as rscene, skin, sky, toy, world};
use game_sim::track::{Collision, Scene, Terrain};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

/// The disc this importer reads (AGENTS.md's table).
pub const BOOT: &str = "SLUS_211.04";
pub const GAME: &str = "MX vs. ATV Unleashed";
pub const AUTHOR: &str = "openmx-mods tools/import-mxvatv 0.1";
/// The mods' own version (§1): bump it when what the importer writes changes.
pub const VERSION: &str = "1.0.0";

/// The game a map comes from: what `mod.json`'s `source` and `id` say (§1), and where its map
/// archives sit on the PSR.
#[derive(Debug, PartialEq, Eq)]
pub struct Origin {
    pub game: &'static str,
    pub boot: &'static str,
    /// The id prefix (§1's table).
    pub prefix: &'static str,
    /// The PSR folder that holds `<MAP>\<MAP>.res`: `Env` on MX vs. ATV Unleashed, `res\env` on Untamed.
    pub archives: &'static str,
}

pub const MXVATV: Origin = Origin { game: GAME, boot: BOOT, prefix: "mxvatv_", archives: "Env" };

/// One map as the disc has it.
pub struct Map {
    /// The game it is from.
    pub origin: &'static Origin,
    /// The map's folder as the PSR spells it: `Nat03`.
    pub folder: String,
    /// Its archives, the track archive first, then `_mat` and `_mst`: (PSR member, archive).
    pub archives: Vec<(String, ResFile)>,
    /// `Env\<MAP>\<MAP>.atyList`: (PSR member, bytes).
    pub atylist: Option<(String, Vec<u8>)>,
    /// The other PSR members under `Env\<MAP>\`, which are not copied: (member, `Some(bytes)` for
    /// the ones whose path, without `Env\<MAP>\`, is also an archive member's, to compare).
    pub loose: Vec<(String, Option<Vec<u8>>)>,
    /// Every PSR member that is not a `.res`, by its base name lowercased: where a file the map names
    /// but its archives lack sits loose on the disc, for its `todo` entry.
    pub elsewhere: BTreeMap<String, Vec<String>>,
    /// The loose PSR members the game opens for this map because its archives lack them (M15,
    /// [`pull_loose`]); copied into the mod beside the archives' members and listed in
    /// `source.members`.
    pub pulled: Vec<Pulled>,
    /// The Free Ride as read for the map elsewhere than `atylist` (Untamed's `Env\Master.atyList.bpb`,
    /// [`untamed::free_ride`]); `atylist` is then `None`.
    pub ride: Option<FreeRide>,
}

/// A loose PSR member the game opens for a map: the member as the PSR spells it, its bytes, and the
/// map's file that names it.
pub struct Pulled {
    pub member: String,
    pub bytes: Vec<u8>,
    pub why: String,
}

/// The maps on the disc: every folder `Env\<MAP>\` with a `<map>.res`, in PSR order.
pub fn map_folders(disc: &Disc) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for e in disc.matching(r"Env\*\*.res") {
        let parts: Vec<&str> = e.name.split(['\\', '/']).collect();
        let (folder, file) = (parts[1], parts[2]);
        if file.eq_ignore_ascii_case(&format!("{folder}.res")) && !out.iter().any(|f| f.eq_ignore_ascii_case(folder)) {
            out.push(folder.to_string());
        }
    }
    out
}

/// Read one map's archives and `.atyList` off the disc.
pub fn load_map(disc: &mut Disc, folder: &str) -> Result<Map, String> {
    let mut map = load(disc, folder, &MXVATV)?;
    map.pulled = pull_loose(disc, &map.archives)?;
    Ok(map)
}

/// A map's archives (`<archives>\<MAP>\<MAP>.res`, `_mat`, `_mst`), the PSR members under
/// `Env\<MAP>\` and its `.atyList` there, if any. Nothing is pulled yet.
pub fn load(disc: &mut Disc, folder: &str, origin: &'static Origin) -> Result<Map, String> {
    let under: Vec<_> = disc.matching(&format!(r"Env\{folder}\**")).into_iter().cloned().collect();
    let mut archives = Vec::new();
    for suffix in ["", "_mat", "_mst"] {
        let want = format!(r"{}\{folder}\{folder}{suffix}.res", origin.archives);
        if let Some(e) = disc.find(&want).cloned() {
            archives.push((e.name.clone(), disc.res(&e)?));
        } else if suffix.is_empty() {
            return Err(format!("{want}: not on the disc"));
        }
    }
    let mut atylist = None;
    let mut loose = Vec::new();
    for e in &under {
        if archives.iter().any(|(n, _)| n == &e.name) {
            continue;
        }
        let one_deep = e.name.split(['\\', '/']).count() == 3;
        if one_deep && e.name.to_ascii_lowercase().ends_with(".atylist") && atylist.is_none() {
            atylist = Some((e.name.clone(), disc.read(e)?));
        } else {
            let also = archives.iter().any(|(_, r): &(String, ResFile)| r.entries().iter().any(|m| normalise(&m.name) == normalise(&e.name)));
            loose.push((e.name.clone(), if also { Some(disc.read(e)?) } else { None }));
        }
    }
    let mut elsewhere: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for e in disc.entries() {
        if !e.name.to_ascii_lowercase().ends_with(".res") {
            elsewhere.entry(base(&e.name).to_ascii_lowercase()).or_default().push(e.name.clone());
        }
    }
    Ok(Map { origin, folder: folder.to_string(), archives, atylist, loose, elsewhere, pulled: Vec::new(), ride: None })
}

impl Map {
    /// Where on the PSR a file the archives lack is, as a todo entry says it.
    fn where_else(&self, name: &str) -> String {
        let b = base(name).to_ascii_lowercase();
        match self.elsewhere.get(&b) {
            None => "on no part of the PSR outside a .res".into(),
            Some(v) => {
                let shown: Vec<&str> = v.iter().take(3).map(String::as_str).collect();
                let more = if v.len() > 3 { format!(" and {} more", v.len() - 3) } else { String::new() };
                format!("loose on the PSR as {}{more}", shown.join(", "))
            }
        }
    }
}

/// The texture names a `.psm` gives that are no file (SLUS_211.04 `0x00167720`).
pub(crate) const NOT_FILES: [&str; 4] = ["RenderTarget", "RenderTarget2", "FrameBuffer", "DisplayBuffer"];

/// The loose PSR members the game opens for a map (M15; notes/mxvatv.md, *Loose PSR members*).
///
/// SLUS_211.04's resource manager looks a name up, as a whole lowercased path, among the archives
/// it has loaded (`0x00120750`, `0x0011ff70`). When none holds it, `0x0011dcf0` opens the PSR member
/// at that path (`0x001c6a70`, any case), so a loose member stands in for a missing archive
/// member. Three kinds of request reach past a map's archives on the disc, and each is followed as
/// the game makes it:
///
/// - the ecosystem's species, `Eco\<model>.psg`: GAME_REL.BIN `0x0046d040` sets the model folder to
///   `Eco\` around the model load (`0x0018b690`), which swaps the name's extension for `.psg`;
/// - the map's material table, `MST\Env\<stem>.mst` (`0x0019db30`): each line names a material,
///   opened as `<line>.psm` (`0x001a0e90`). Its `#include`s name the shared tables, not the map's;
/// - those materials' textures: the `.psm`'s folder and the `TextureMap_<n>` of each
///   `[ShaderPass_<p>]` (`0x00167720`), or the material's own name when it has no `.psm`
///   (`0x00167f10`), tried as `.pst`, `.pss`, `.tga` and `.atm` in turn (`0x00170500`).
///
/// A path an archive holds is the archive's. One only the PSR holds, loose, is pulled.
pub fn pull_loose(disc: &mut Disc, archives: &[(String, ResFile)]) -> Result<Vec<Pulled>, String> {
    let mut held: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for (a, (_, r)) in archives.iter().enumerate() {
        for (i, e) in r.entries().iter().enumerate() {
            held.entry(normalise(&e.name)).or_insert((a, i));
        }
    }
    let mut psr: BTreeMap<String, usize> = BTreeMap::new();
    for (i, e) in disc.entries().iter().enumerate() {
        if !e.name.to_ascii_lowercase().ends_with(".res") {
            psr.entry(normalise(&e.name)).or_insert(i);
        }
    }
    let mut o = Opener { disc, archives, held, psr, pulled: Vec::new() };

    let scenes: Vec<String> = o.held.keys().filter(|k| k.ends_with(".scn")).cloned().collect();
    let [scn] = scenes.as_slice() else {
        return Ok(Vec::new()); // import_into refuses the map, and says why
    };
    let stem = base(scn).trim_end_matches(".scn").to_string();
    let text = ini::text(&o.held_bytes(scn)?);
    let sections = ini::parse(&text);

    // The ecosystem's species.
    let env = sections.iter().find(|s| s.name.eq_ignore_ascii_case("Environment"));
    if let Some(file) = env.and_then(|e| e.get("EcosystemFile")).map(str::trim).filter(|f| !f.is_empty()) {
        let esb = normalise(&with_ext(file, ".esb"));
        let hits: Vec<String> = o.held.keys().filter(|k| **k == esb || k.ends_with(&format!("/{esb}"))).cloned().collect();
        if let [esb] = hits.as_slice()
            && let Ok(e) = eco::Ecosystem::parse(&o.held_bytes(esb)?, 1.0)
        {
            for s in &e.species {
                let model = s.model.trim_end_matches('\0');
                o.open(&with_ext(&format!("Eco\\{model}"), ".psg"), &format!("species {model} of {esb}"))?;
            }
        }
    }

    // The material table, and each material's textures.
    let mst = format!("mst/env/{stem}.mst");
    if o.held.contains_key(&mst) {
        let table = ini::text(&o.held_bytes(&mst)?);
        for line in table.lines() {
            let m = line.split(';').next().unwrap_or("").trim();
            if m.is_empty() || m.starts_with('#') {
                continue;
            }
            let why = format!("material {m} in {mst}");
            let psm = with_ext(m, ".psm");
            let mut textures = Vec::new();
            match o.open(&psm, &why)? {
                None => textures.push(m.to_string()),
                Some(bytes) => {
                    let dir = &psm[..psm.rfind('\\').map_or(0, |i| i + 1)];
                    let secs = ini::parse(&ini::text(&bytes));
                    let get = |sec: &str, key: &str| {
                        secs.iter().find(|s| s.name.eq_ignore_ascii_case(sec)).and_then(|s| s.get(key)).map(|v| v.trim().to_string())
                    };
                    let passes: usize = get("General", "TotalShaderPasses").and_then(|v| v.parse().ok()).unwrap_or(0);
                    for p in 1..=passes {
                        let pass = format!("ShaderPass_{p}");
                        let maps: usize = get(&pass, "totalTextureMaps").and_then(|v| v.parse().ok()).unwrap_or(1);
                        for n in 1..=maps {
                            if let Some(t) = get(&pass, &format!("TextureMap_{n}")).filter(|t| !t.is_empty())
                                && !NOT_FILES.iter().any(|x| x.eq_ignore_ascii_case(&t))
                            {
                                textures.push(format!("{dir}{t}"));
                            }
                        }
                    }
                }
            }
            for t in textures {
                for ext in [".pst", ".pss", ".tga", ".atm"] {
                    if o.open(&with_ext(&t, ext), &format!("texture {t} of {why}"))?.is_some() {
                        break;
                    }
                }
            }
        }
    }

    let dir = scn.rfind('/').map_or("", |i| &scn[..i]);
    pull_physics(&mut o, dir, &stem)?;
    pull_shadows(&mut o)?;
    pull_particles(&mut o, dir)?;
    Ok(o.pulled)
}

/// The machines' particle sheets, by the machine (GAME_REL.BIN `0x004a6810`'s switch on its type), and
/// the two zone tables (`atvparticlezones.dat` for an ATV, `mxparticlezones.dat` for the rest).
pub const PARTICLE_SHEETS: [&str; 13] = [
    "mxparticles.dat",
    "mxaiparticles.dat",
    "atvparticles.dat",
    "atvaiparticles.dat",
    "biplaneparticles.dat",
    "helicopterparticles.dat",
    "buggyparticles.dat",
    "monsterparticles.dat",
    "bajaparticles.dat",
    "golfparticles.dat",
    "SRparticles.dat",
    "mxparticlezones.dat",
    "atvparticlezones.dat",
];

/// The map's own particle sheets (M12, seen in the game's name pool on MC10). `0x004a6810` opens the
/// machine's sheet and zone table in the map's folder (`%s%s`, the folder and the name) and takes
/// `Part\`'s only when the map has none. Every name the switch can give is pulled where the map's
/// folder has it loose, whichever machine rides.
fn pull_particles(o: &mut Opener, dir: &str) -> Result<(), String> {
    for name in PARTICLE_SHEETS {
        o.open(&format!("{dir}/{name}"), &format!("the machine's particles, {name} in the map's folder (0x004a6810)"))?;
    }
    Ok(())
}

/// The physics objects the map's toys name (M12, seen in the game's name pool in a ride on Lago).
/// SLUS_211.04 `0x002171b0` reads `[PhysicsCollections]` of `<map>.toy`: the first
/// `NumberOfPhysicsCollections` sections `[PhysicsCollection<n>]`, each a `Name`, opened as
/// `<map folder>\<Name>.pco` (`%s%s.pco`). A `Name` with a `*` is a group matched against the scene's
/// models, which no map uses. GAME_REL.BIN `0x004922a0` then opens each `.pco`'s parts'
/// collision, `Col\<Name>m.col` and `Col\<Name>f.col` (its name with `m.col` and `f.col` put on).
fn pull_physics(o: &mut Opener, dir: &str, stem: &str) -> Result<(), String> {
    let toy = format!("{dir}/{stem}.toy");
    if !o.held.contains_key(&toy) {
        return Ok(());
    }
    let secs = ini::parse(&ini::text(&o.held_bytes(&toy)?));
    let get = |secs: &[ini::Section], sec: &str, key: &str| {
        secs.iter().find(|s| s.name.eq_ignore_ascii_case(sec)).and_then(|s| s.get(key)).map(|v| v.trim().trim_matches('"').to_string())
    };
    let n: usize = get(&secs, "PhysicsCollections", "NumberOfPhysicsCollections").and_then(|v| v.parse().ok()).unwrap_or(0);
    for i in 1..=n {
        let Some(name) = get(&secs, &format!("PhysicsCollection{i}"), "Name").filter(|v| !v.is_empty() && !v.contains('*')) else {
            continue;
        };
        let why = format!("physics collection {name} of {toy}");
        let Some(pco) = o.open(&format!("{dir}/{name}.pco"), &why)? else { continue };
        let parts = ini::parse(&ini::text(&pco));
        let k: usize = get(&parts, "PhysicsParts", "NumberOfPhysicsParts").and_then(|v| v.parse().ok()).unwrap_or(0);
        for p in 1..=k {
            if let Some(part) = get(&parts, &format!("PhysicsPart{p}"), "Name").filter(|v| !v.is_empty()) {
                for end in ["m.col", "f.col"] {
                    o.open(&format!("Col\\{part}{end}"), &format!("part {part} of {name}.pco"))?;
                }
            }
        }
    }
    Ok(())
}

/// How many shadow sets the game makes (M12: `0x0031bc90` is handed 6 on every map ridden).
pub const SHADOW_SETS: usize = 6;

/// The shadow materials: `0x0031bc90` opens `shadowCaster<n>`, `shadowReceiver<n>` and
/// `terrainShadowReceiver<n>` for each of the [`SHADOW_SETS`] (`Tex\`, as `MST\TerrainShadow.mst`
/// spells them), on every map.
fn pull_shadows(o: &mut Opener) -> Result<(), String> {
    for n in 1..=SHADOW_SETS {
        for m in ["shadowCaster", "terrainShadowReceiver", "shadowReceiver"] {
            o.open(&format!("Tex\\{m}{n}.psm"), &format!("shadow set {n} ({m}%d, 0x0031bc90)"))?;
        }
    }
    Ok(())
}

/// A name with its extension swapped, as the game does it: from the last dot, or added when there
/// is none.
pub(crate) fn with_ext(name: &str, ext: &str) -> String {
    match name.rfind('.') {
        Some(d) => format!("{}{ext}", &name[..d]),
        None => format!("{name}{ext}"),
    }
}

/// Opens a path the way the game's resource manager does: the map's archives, then the PSR.
pub(crate) struct Opener<'a> {
    disc: &'a mut Disc,
    archives: &'a [(String, ResFile)],
    /// Every archive member by its normalised path: (archive, entry).
    pub(crate) held: BTreeMap<String, (usize, usize)>,
    /// Every PSR member that is not a `.res`, by its normalised path.
    psr: BTreeMap<String, usize>,
    pub(crate) pulled: Vec<Pulled>,
}

impl<'a> Opener<'a> {
    pub(crate) fn new(disc: &'a mut Disc, archives: &'a [(String, ResFile)]) -> Opener<'a> {
        let mut held: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        for (a, (_, r)) in archives.iter().enumerate() {
            for (i, e) in r.entries().iter().enumerate() {
                held.entry(normalise(&e.name)).or_insert((a, i));
            }
        }
        let mut psr: BTreeMap<String, usize> = BTreeMap::new();
        for (i, e) in disc.entries().iter().enumerate() {
            if !e.name.to_ascii_lowercase().ends_with(".res") {
                psr.entry(normalise(&e.name)).or_insert(i);
            }
        }
        Opener { disc, archives, held, psr, pulled: Vec::new() }
    }

    pub(crate) fn held_bytes(&self, key: &str) -> Result<Vec<u8>, String> {
        let (a, i) = self.held[key];
        let (member, r) = &self.archives[a];
        let e = &r.entries()[i];
        r.read(e).map_err(|err| format!("{member}:{}: {err}", e.name))
    }

    pub(crate) fn open(&mut self, path: &str, why: &str) -> Result<Option<Vec<u8>>, String> {
        let key = normalise(path);
        if self.held.contains_key(&key) {
            return self.held_bytes(&key).map(Some);
        }
        if let Some(p) = self.pulled.iter().find(|p| normalise(&p.member) == key) {
            return Ok(Some(p.bytes.clone()));
        }
        let Some(&i) = self.psr.get(&key) else {
            return Ok(None);
        };
        let e = self.disc.entries()[i].clone();
        let bytes = self.disc.read(&e)?;
        self.pulled.push(Pulled { member: e.name.clone(), bytes: bytes.clone(), why: why.to_string() });
        Ok(Some(bytes))
    }
}

/// One file of the mod: its path in the folder (`/`-separated, the member's spelling), its bytes, and
/// the archive member it came from.
struct File {
    path: String,
    bytes: Vec<u8>,
    from: String,
}

/// What an import made.
#[derive(Debug)]
pub struct Imported {
    pub id: String,
    pub dir: PathBuf,
    pub name: String,
    pub files: usize,
    pub todo: Vec<String>,
    /// The spawn `mod.json` gives, if the `.atyList` has one: (position, direction, from).
    pub spawn: Option<(String, String, String)>,
    /// The loose PSR members copied beside the archives' ([`pull_loose`]).
    pub pulled: Vec<String>,
}

/// The mod's id: `mxvatv_` and the map's folder, lowercased (§1).
pub fn id_for(folder: &str) -> Result<String, String> {
    id_with(MXVATV.prefix, folder)
}

/// The mod's id: the game's prefix and the map's folder, lowercased (§1).
pub fn id_with(prefix: &str, folder: &str) -> Result<String, String> {
    let id = format!("{prefix}{}", folder.to_ascii_lowercase());
    if id.len() > 40 || !id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') {
        return Err(format!("{folder}: {id:?} is not an id of 1-40 of [a-z0-9_]"));
    }
    Ok(id)
}

/// Import one map into `out/<id>/`, replacing what is there. The folder is written whole or not at
/// all: an error removes it.
pub fn import(map: &Map, out: &Path) -> Result<Imported, String> {
    let id = id_with(map.origin.prefix, &map.folder)?;
    let dir = out.join(&id);
    let r = import_into(map, &id, &dir);
    if r.is_err() {
        let _ = std::fs::remove_dir_all(&dir);
    }
    r
}

fn import_into(map: &Map, id: &str, dir: &Path) -> Result<Imported, String> {
    let mut files = unpack(map)?;
    let mut todo = Todo::default();
    todo.extend(decode_scenes(&mut files)?);
    for (member, r) in &map.archives {
        if r.padding > 0 {
            todo.push(format!(
                "{member}: Res::parse refuses it as the disc has it: {} zero bytes pad its directory, which the header's length counts; \
                 its members were read through a view without them (discx ResFile::unpadded), so the port needs openmx's reader to accept the padding to open it",
                r.padding
            ));
        }
    }
    if dir.exists() {
        std::fs::remove_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    for f in &files {
        let p = dir.join(&f.path);
        std::fs::create_dir_all(p.parent().expect("a member has a parent")).map_err(|e| format!("{}: {e}", p.display()))?;
        std::fs::write(&p, &f.bytes).map_err(|e| format!("{}: {e}", p.display()))?;
    }

    let look = Look { files: &files };

    // The scene: the folder's only `.scn` (§2).
    let scenes: Vec<&File> = files.iter().filter(|f| f.path.to_ascii_lowercase().ends_with(".scn")).collect();
    let scn = match scenes.as_slice() {
        [s] => *s,
        [] => return Err(format!("{}: no .scn in the archives", map.folder)),
        many => return Err(format!("{}: {} .scn files: {}", map.folder, many.len(), many.iter().map(|f| f.path.as_str()).collect::<Vec<_>>().join(", "))),
    };
    let scn_name = base(&scn.path).to_string();
    let scene = Scene::parse(&scn.bytes).map_err(|e| format!("{}: Scene::parse: {e}", scn.path))?;
    let text = ini::text(&scn.bytes);
    let sections = ini::parse(&text);

    // The terrain: the name as written, then with the platform tag (§2).
    let mut names = vec![scene.terrain_file.clone()];
    if let Some(dot) = scene.terrain_file.rfind('.') {
        names.push(format!("{}.ps2{}", &scene.terrain_file[..dot], &scene.terrain_file[dot..]));
    }
    let mut trn = None;
    for n in &names {
        if let Some(f) = look.one(n).map_err(|e| format!("{}: TerrainFile {e}", scn.path))? {
            trn = Some(f);
            break;
        }
    }
    let trn = trn.ok_or_else(|| format!("{}: TerrainFile={} finds no file (tried {})", scn.path, scene.terrain_file, names.join(", ")))?;
    let view;
    let terrain = match Terrain::parse(&trn.bytes) {
        Ok(t) => t,
        Err(refusal) => {
            let v = modcheck::terrain::md1_view(&trn.bytes).map_err(|e| format!("{}: Terrain::parse: {refusal}; no MD1 view: {e}", trn.path))?;
            todo.push(format!(
                "{}: Terrain::parse refuses it ({refusal}): {} texture pages ({} bytes) sit between the blocks and the tree (openmx MD1); \
                 read here through modcheck's MD1 view, and the ground is not drawn from those pages until openmx MD4",
                trn.path, v.pages, v.gap
            ));
            view = v.bytes;
            Terrain::parse(&view).map_err(|e| format!("{}: Terrain::parse refuses its MD1 view too: {e}", trn.path))?
        }
    };
    let facts = terrain_facts(&terrain).map_err(|e| format!("{}: {e}", trn.path))?;
    layout(&scn_name, &trn.path, &scene, &facts, &mut todo);

    // Collision: one `.col` per distinct `COL=` (§2).
    let mut placed: BTreeMap<String, (String, usize)> = BTreeMap::new();
    for p in &scene.collision {
        placed.entry(p.file.to_ascii_lowercase()).or_insert_with(|| (p.file.clone(), 0)).1 += 1;
    }
    for (name, n) in placed.values() {
        let mut f = look.one(name);
        if matches!(f, Ok(None)) && !name.contains('.') {
            f = look.one(&format!("{name}.col"));
        }
        match f {
            Err(e) => todo.push(format!("{scn_name}: COL={e}")),
            Ok(None) => todo.push(format!(
                "{name}: placed by {scn_name} ({n} [Object] sections) and in none of the map's archives ({}); the port skips it. \
                 What the game does with it is not known",
                map.where_else(&if name.contains('.') { name.clone() } else { format!("{name}.col") })
            )),
            Ok(Some(f)) => {
                if let Err(e) = Collision::parse(&f.bytes) {
                    let md = if e.contains("class 1") { " (openmx MD5)" } else { "" };
                    todo.push(format!("{}: Collision::parse: {e}{md}; placed by {scn_name} {n} times, the port skips it", f.path));
                }
            }
        }
    }

    scene_keys(&scn_name, &sections, &mut todo);
    let env = sections.iter().find(|s| s.name.eq_ignore_ascii_case("Environment"));
    if let Some(d) = env.and_then(|s| s.get("DetailMapTilingMultiplier")) {
        if d.trim() != "8" {
            todo.push(format!(
                "{scn_name}: DetailMapTilingMultiplier={} where game-render's terrain assumes 8, MX Unleashed's (openmx MD4)",
                d.trim()
            ));
        }
    }
    let leaves = (facts.samples.0 / 16, facts.samples.1 / 16);
    if leaves.0 > 48 || leaves.1 > 48 {
        todo.push(format!(
            "{}: {} × {} leaves at {} ft, bigger than MX Unleashed's free06 (48 × 48 at 9 ft): edge, radar, draw distance and height band need openmx MD3",
            trn.path, leaves.0, leaves.1, facts.scale
        ));
    }

    let mut lookups = vec![("scene", scn.path.clone()), ("terrain", trn.path.clone())];
    render(map, &look, &scn_name, &text, &scene, &terrain, &mut lookups, &mut todo);
    kinds(&files, &scn_name, &mut todo);

    // The Free Ride: its name and start from the `.atyList` (M8, §2b).
    let mut name = String::new();
    let mut spawn = None;
    let ride = match (&map.ride, &map.atylist) {
        (Some(fr), _) => Some(fr.clone()),
        (None, Some((member, bytes))) => Some(free_ride(member, bytes)?),
        (None, None) => None,
    };
    match ride {
        None => todo.push(format!(r"Env\{}: no .atyList, so no Free Ride name or start; the name is the folder's", map.folder)),
        Some(fr) => {
            name = fr.name;
            spawn = fr.spawn;
            todo.extend(fr.todo);
        }
    }
    if name.trim().is_empty() {
        name = map.folder.clone();
    }
    if spawn.is_none() {
        todo.push(format!(
            "{scn_name}: the spawn is the scene's DefaultStartPosition {} facing {}: the .atyList's [Free Ride] gives no StartPosition, \
             and where the game starts that Free Ride was not looked up (T7)",
            vec3(&sections, "SceneInfo", "DefaultStartPosition"),
            vec3(&sections, "SceneInfo", "DefaultStartDirection")
        ));
    }

    // Where the Free Ride starts on MX vs. ATV Unleashed (M12, every map ridden in PCSX2): at the
    // .atyList's StartPosition where it gives one (MC01, FS05), else at the scene's DefaultStartPosition,
    // the ride camera 14 ft behind it. So the entry above is no gap there, except where that position is
    // under the ground (FS02, FS06, Nat10, Nat14, SM01-03): the game stands the rider on the ground over it.
    if spawn.is_none() && map.origin.boot == BOOT {
        let unknown = format!("{scn_name}: the spawn is the scene's DefaultStartPosition ");
        todo.0.retain(|t| !t.starts_with(&unknown));
        let field = game_sim::world::Ground::new(&terrain, &scene.terrain);
        let field = field.field();
        let [x, y, z] = scene.start.map(|c| c.0);
        let ground = game_sim::contact_search::ground(&field, ps2_float::Vf::new(x, 0.0, z, 1.0)).point.y();
        if y < ground - 1.0 {
            todo.push(format!(
                "{scn_name}: the spawn, the scene's DefaultStartPosition {}, is {:.1} ft under the ground there ({ground:.2}); \
                 the game stands the rider on the ground over it (M12, seen in PCSX2), and whether openmx does was not checked",
                vec3(&sections, "SceneInfo", "DefaultStartPosition"),
                ground - y
            ));
        }
    }

    let mut uncopied = 0;
    let mut same = 0;
    let mut by: BTreeMap<String, usize> = BTreeMap::new();
    for (l, bytes) in &map.loose {
        if map.pulled.iter().any(|p| &p.member == l) {
            continue;
        }
        if let Some(b) = bytes {
            let twin = files.iter().find(|f| normalise(&f.path) == normalise(l)).expect("load_map found it");
            if &twin.bytes == b {
                same += 1;
                continue;
            }
            todo.push(format!("{l}: a loose member at the same path as {}, with different bytes; the archive's is copied", twin.from));
        }
        uncopied += 1;
        *by.entry(ext_of(l)).or_default() += 1;
    }
    if uncopied > 0 {
        let kinds: Vec<String> = by.iter().map(|(k, n)| format!("{n} {k}")).collect();
        todo.push(format!(
            r"Env\{}\: {uncopied} loose members beside the archives are not copied ({}){}: race, AI, camera and replay files, the activities' .aty and Anim\; the port does not read them",
            map.folder,
            kinds.join(", "),
            if same > 0 { format!("; {same} more are the same bytes as the archive member at their path") } else { String::new() }
        ));
    }

    let mut members: Vec<String> = map.archives.iter().map(|(m, _)| m.clone()).collect();
    members.extend(map.pulled.iter().map(|p| p.member.clone()));
    let loose: Vec<(&str, &str)> = map.pulled.iter().map(|p| (p.member.as_str(), p.why.as_str())).collect();
    let json = mod_json(map.origin, id, &name, &map.folder, &members, spawn.as_ref(), &lookups, &loose, &todo.0);
    std::fs::write(dir.join("mod.json"), json).map_err(|e| format!("{}: {e}", dir.display()))?;
    let pulled = map.pulled.iter().map(|p| p.member.clone()).collect();
    Ok(Imported { id: id.to_string(), dir: dir.to_path_buf(), name, files: files.len(), todo: todo.0, spawn, pulled })
}

/// Every member of every archive at its own path. The same path twice is one file when the bytes are
/// the same, and stops the import when they are not (§2); the same path spelt two ways stops it too,
/// since the folder could then hold only one of them.
fn unpack(map: &Map) -> Result<Vec<File>, String> {
    let mut files: Vec<File> = Vec::new();
    let mut at: BTreeMap<String, usize> = BTreeMap::new();
    for (member, res) in &map.archives {
        for e in res.entries() {
            let bytes = res.read(e).map_err(|err| format!("{member}:{}: {err}", e.name))?;
            let path = e.path();
            let from = format!("{member}:{}", e.name);
            if path.split('/').any(|s| s.is_empty() || s == "." || s == "..") {
                return Err(format!("{from}: not a path a folder can hold"));
            }
            let key = path.to_ascii_lowercase();
            match at.get(&key) {
                Some(&k) => {
                    let first = &files[k];
                    if first.bytes != bytes {
                        return Err(format!("{path}: {} and {from} differ; the import does not choose one (§2)", first.from));
                    }
                    if first.path != path {
                        return Err(format!("{} ({}) and {path} ({from}): the same file spelt two ways", first.path, first.from));
                    }
                }
                None => {
                    at.insert(key, files.len());
                    files.push(File { path, bytes, from });
                }
            }
        }
    }
    // The loose members the game opens for the map ([`pull_loose`]): none is at an archive member's
    // path, so none can clash with one, but the same path spelt two ways in the PSR would.
    for p in &map.pulled {
        let path = p.member.replace('\\', "/");
        let key = path.to_ascii_lowercase();
        if let Some(&k) = at.get(&key) {
            return Err(format!("{} ({}) and {path} ({}): the same file spelt two ways", files[k].path, files[k].from, p.member));
        }
        at.insert(key, files.len());
        files.push(File { path, bytes: p.bytes.clone(), from: p.member.clone() });
    }
    Ok(files)
}

/// A binary scene (`<MAP>.scn.bpb`, MX vs. ATV Untamed's) decoded by `tools/bpb` to the INI text the
/// port reads, written beside it as `<MAP>.scn` (§2a); the `.bpb` stays as the disc has it. Each
/// decode is a todo entry naming both files.
fn decode_scenes(files: &mut Vec<File>) -> Result<Vec<String>, String> {
    let mut todo = Vec::new();
    let mut add = Vec::new();
    for f in files.iter() {
        let Some(path) = f.path.strip_suffix(".bpb").filter(|p| p.to_ascii_lowercase().ends_with(".scn")) else {
            continue;
        };
        let b = bpb::parse(&f.bytes).map_err(|e| format!("{}: bpb: {e}", f.from))?;
        let bad = b.bad_hashes();
        if !bad.is_empty() {
            return Err(format!("{}: {} names whose hash does not recompute: {}", f.from, bad.len(), bad.join(", ")));
        }
        if files.iter().any(|g| g.path.eq_ignore_ascii_case(path)) {
            return Err(format!("{path}: the archives hold it and {} too; the import does not choose one", f.path));
        }
        todo.push(format!(
            "{path}: not a disc member: {} (the disc's binary scene, kept) decoded to INI text by tools/bpb (§2a), \
             every name's hash recomputed; the port reads only text scenes",
            f.path
        ));
        add.push(File { path: path.to_string(), bytes: b.to_ini(), from: format!("{} decoded by tools/bpb", f.from) });
    }
    files.extend(add);
    Ok(todo)
}

/// The todo list, without repeats, in the order found.
#[derive(Default)]
struct Todo(Vec<String>);

impl Todo {
    fn push(&mut self, s: String) {
        if !self.0.contains(&s) {
            self.0.push(s);
        }
    }
    fn extend(&mut self, v: Vec<String>) {
        for s in v {
            self.push(s);
        }
    }
}

/// Lookups the way §2 has them: the end of the path, case and slash folded, on a `/` boundary. A name
/// that finds two files is an error (the port's plain suffix would take the first; modcheck fails it).
struct Look<'a> {
    files: &'a [File],
}

impl<'a> Look<'a> {
    fn all(&self, name: &str) -> Vec<&'a File> {
        let want = name.replace('\\', "/").trim().trim_start_matches('/').to_ascii_lowercase();
        if want.is_empty() {
            return Vec::new();
        }
        self.files
            .iter()
            .filter(|f| {
                let p = f.path.to_ascii_lowercase();
                p == want || p.ends_with(&format!("/{want}"))
            })
            .collect()
    }

    fn one(&self, name: &str) -> Result<Option<&'a File>, String> {
        match self.all(name).as_slice() {
            [] => Ok(None),
            [f] => Ok(Some(f)),
            many => Err(format!("{name} finds {} files: {}", many.len(), many.iter().map(|f| f.path.as_str()).collect::<Vec<_>>().join(", "))),
        }
    }

    /// The bytes a name finds, for game-render's readers; two files are none, and said elsewhere.
    fn read(&self, name: &str) -> Option<Vec<u8>> {
        self.one(name).ok().flatten().map(|f| f.bytes.clone())
    }
}

pub(crate) fn base(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn ext_of(path: &str) -> String {
    let b = base(path);
    // A BPB (Untamed's binary INI) is its kind and `.bpb`: `.psm.bpb`.
    if let Some(stem) = b.strip_suffix(".bpb").or_else(|| b.strip_suffix(".BPB"))
        && let Some(d) = stem.rfind('.')
    {
        return b[d..].to_ascii_lowercase();
    }
    match b.rfind('.') {
        Some(d) => b[d..].to_ascii_lowercase(),
        None => "(no extension)".into(),
    }
}

/// What the layout check needs from a terrain.
struct Facts {
    scale: f32,
    levels: u32,

    samples: (u32, u32),
}

fn terrain_facts(t: &Terrain) -> Result<Facts, String> {
    let root = t.root().ok_or("the terrain has no root node")?;
    if t.levels == 0 || t.levels > 4 {
        return Err(format!("{} tree levels", t.levels));
    }
    let (mut cx_max, mut cz_max) = (0, 0);
    for cz in 0..16 {
        for cx in 0..16 {
            if t.child(&root, cx, cz).is_some() {
                cx_max = cx_max.max(cx + 1);
                cz_max = cz_max.max(cz + 1);
            }
        }
    }
    let side = 16u32.pow(t.levels);
    Ok(Facts { scale: t.scale.0, levels: t.levels, samples: (cx_max * side, cz_max * side) })
}

/// §2a: the scene's layout against its terrain. The scene is the disc's and is never edited, so a
/// disagreement is a `todo` entry naming the scene and the key, which modcheck looks for.
fn layout(scn: &str, trn: &str, scene: &Scene, t: &Facts, todo: &mut Todo) {
    let l = &scene.terrain;
    let side = 16u32.pow(t.levels);
    for (key, n, have) in [("TerrainWidth", l.width, t.samples.0), ("TerrainBreadth", l.breadth, t.samples.1)] {
        if u64::from(n) * 256 != u64::from(have) {
            todo.push(format!(
                "{scn}: {key}={n} on the disc, the terrain is {have} samples ({} root cells of {side}) {}; the scene is kept as it is (§2a)",
                have / side,
                if key == "TerrainWidth" { "wide" } else { "deep" }
            ));
        }
        if n > 16 {
            todo.push(format!("{scn}: {key}={n} is over 16 (§2a)"));
        }
    }
    for (key, v) in [("TerrainWidthScale", l.width_scale.0), ("TerrainBreadthScale", l.breadth_scale.0)] {
        if v != t.scale {
            todo.push(format!("{scn}: {key}={v} on the disc, {trn}'s own scale is {} ft; the scene is kept as it is (§2a)", t.scale));
        }
    }
}

/// A section's kind: its name with its number cut, lowercased (`[Model224]` is `model`).
pub fn section_kind(name: &str) -> String {
    name.trim().trim_end_matches(|c: char| c.is_ascii_digit()).to_ascii_lowercase()
}

/// Every `(section kind, key)` a scene's text uses, lowercased, as [`mxu_keys::MXU_SCENE_KEYS`] lists
/// them.
pub fn scene_key_set(text: &str) -> BTreeSet<(String, String)> {
    let mut out = BTreeSet::new();
    for s in ini::parse(text).iter().filter(|s| !s.name.is_empty()) {
        for (k, _) in &s.keys {
            out.insert((section_kind(&s.name), k.to_ascii_lowercase()));
        }
    }
    out
}

/// The scene's keys against MX Unleashed's: one entry per key (section kind and key) that no MXU
/// scene has, with how many sections carry it and its commonest values.
fn scene_keys(scn: &str, sections: &[ini::Section], todo: &mut Todo) {
    let known: HashSet<(&str, &str)> = mxu_keys::MXU_SCENE_KEYS.iter().copied().collect();
    let sections_known: HashSet<&str> = mxu_keys::MXU_SCENE_KEYS.iter().map(|(s, _)| *s).collect();
    // (kind, key lowercased) -> (kind as spelt, key as spelt, sections, values)
    let mut new: BTreeMap<(String, String), (String, String, usize, BTreeMap<String, usize>)> = BTreeMap::new();
    let mut new_sections: BTreeMap<String, usize> = BTreeMap::new();
    for s in sections.iter().filter(|s| !s.name.is_empty()) {
        let kind = s.name.trim().trim_end_matches(|c: char| c.is_ascii_digit()).to_string();
        let lk = section_kind(&s.name);
        if !sections_known.contains(lk.as_str()) {
            *new_sections.entry(kind.clone()).or_default() += 1;
            continue;
        }
        let mut seen = HashSet::new();
        for (k, v) in &s.keys {
            let kl = k.to_ascii_lowercase();
            if known.contains(&(lk.as_str(), kl.as_str())) {
                continue;
            }
            let e = new.entry((lk.clone(), kl.clone())).or_insert_with(|| (kind.clone(), k.clone(), 0, BTreeMap::new()));
            if seen.insert(kl) {
                e.2 += 1;
            }
            *e.3.entry(v.clone()).or_default() += 1;
        }
    }
    for (kind, n) in new_sections {
        todo.push(format!("{scn}: {n} [{kind}] sections, a section no MX Unleashed scene has; the port does not read them"));
    }
    for ((lk, _), (kind, key, n, values)) in new {
        let mut vs: Vec<(&String, &usize)> = values.iter().collect();
        vs.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        let shown: Vec<String> = vs.iter().take(4).map(|(v, c)| format!("{v} ×{c}")).collect();
        let more = if vs.len() > 4 { format!(", and {} other values", vs.len() - 4) } else { String::new() };
        let numbered = if matches!(lk.as_str(), "model" | "object" | "light") { "<n>" } else { "" };
        let so = match lk.as_str() {
            "model" => ", and draws every one of these props whatever it says",
            "object" => ", and places every one of these collision shapes whatever it says",
            _ => "",
        };
        todo.push(format!(
            "{scn}: {key}= in {n} [{kind}{numbered}] sections ({}{more}): no MX Unleashed scene has this key, so the port does not read it{so}",
            shown.join(", ")
        ));
    }
}

/// game-render's side: the scene as it reads it, the sky, the ecosystem, every prop's model and its
/// materials' textures, the toys, and every file of a kind it has a reader for.
#[allow(clippy::too_many_arguments)]
fn render(map: &Map, look: &Look, scn: &str, text: &str, scene: &Scene, terrain: &Terrain, lookups: &mut Vec<(&'static str, String)>, todo: &mut Todo) {
    let rs = rscene::Scene::parse(text);
    let env = ini::parse(text).into_iter().find(|s| s.name.eq_ignore_ascii_case("Environment"));

    // The sky, as `sky::Sky::load` finds it: `<stem>.ps2.cub`, then the name as written.
    match env.as_ref().and_then(|e| e.get("CubeFile")).map(|c| c.trim().to_string()).filter(|c| !c.is_empty()) {
        None => todo.push(format!("{scn}: no CubeFile, so the port draws no sky")),
        Some(cube) => {
            let lower = cube.to_ascii_lowercase();
            let stem = lower.strip_suffix(".cub").unwrap_or(&lower);
            let tried = [format!("{stem}.ps2.cub"), lower.clone()];
            let mut found = None;
            for t in &tried {
                match look.one(t) {
                    Ok(Some(f)) => {
                        found = Some(f);
                        break;
                    }
                    Ok(None) => {}
                    Err(e) => {
                        todo.push(format!("{scn}: CubeFile {e}"));
                        break;
                    }
                }
            }
            match found {
                None => todo.push(format!("{scn}: CubeFile={cube} finds no file (tried {}); the port draws no sky", tried.join(", "))),
                Some(f) => {
                    lookups.push(("sky", f.path.clone()));
                    if let Err(e) = sky::Sky::parse(&f.bytes) {
                        todo.push(format!("{}: sky::Sky::parse: {e}; the port draws no sky", f.path));
                    }
                }
            }
        }
    }

    // The ecosystem, as `eco::Grove::load` reads it.
    let mut models: BTreeMap<String, usize> = BTreeMap::new();
    let mut billboards: BTreeSet<String> = BTreeSet::new();
    if let Some(file) = rs.ecosystem_file.as_deref() {
        let mut found = look.one(file);
        if matches!(found, Ok(None)) && !file.contains('.') {
            found = look.one(&format!("{file}.esb"));
            if let Ok(Some(f)) = &found {
                todo.push(format!(
                    "{scn}: EcosystemFile={file} names no file as written, and {} is {file}.esb; game-render's eco reader takes the name as \
                     written, so the port draws no plants here (the game swaps or adds the extension: GAME_REL.BIN 0x0046f070)",
                    f.path
                ));
            }
        }
        match found {
            Err(e) => todo.push(format!("{scn}: EcosystemFile {e}")),
            Ok(None) => todo.push(format!("{scn}: EcosystemFile={file} finds no file ({}); the port draws no plants", map.where_else(file))),
            Ok(Some(f)) => {
                lookups.push(("ecosystem", f.path.clone()));
                match eco::Ecosystem::parse(&f.bytes, eco::extent(&scene.terrain, terrain)) {
                    Err(e) => todo.push(format!("{}: eco::Ecosystem::parse: {e}; the port draws no plants", f.path)),
                    Ok(e) => {
                        for s in &e.species {
                            *models.entry(s.model_name()).or_default() += 0;
                            // As `eco::Grove::new` names it: the texture less its `.tga`.
                            billboards.insert(s.billboard.split('.').next().unwrap_or("").to_ascii_lowercase());
                        }
                    }
                }
            }
        }
    }
    for p in &rs.models {
        *models.entry(p.model.clone()).or_default() += 1;
    }

    // The toys, as `toy::Traffic::load` takes them; Untamed's are BPB, decoded here to be read.
    let stem = &scn[..scn.len() - 4];
    let toys = match look.one(&format!("{stem}.toy")).ok().flatten() {
        Some(f) => Some((f.path.clone(), f.bytes.clone())),
        None => look.one(&format!("{stem}.toy.bpb")).ok().flatten().and_then(|f| {
            let b = bpb::parse(&f.bytes).ok()?;
            todo.push(format!(
                "{}: the toys are BPB, and the port looks for {stem}.toy text, so it runs none; read here through tools/bpb's decode",
                f.path
            ));
            Some((f.path.clone(), b.to_ini()))
        }),
    };
    if let Some((path, bytes)) = toys {
        let f = File { path, bytes, from: String::new() };
        lookups.push(("toys", f.path.clone()));
        for car in toy::cars(&ini::text(&f.bytes)) {
            *models.entry(car.stem().to_string()).or_default() += 0;
            if car.has_physics {
                todo.push(format!("{}: {} has HasPhysics, which the port's traffic leaves out", f.path, car.model));
                continue;
            }
            match look.read(&car.mot).ok_or_else(|| "no such file".to_string()).and_then(|b| skin::Clip::parse(&b)) {
                Err(e) => todo.push(format!("{}: {}'s MOTFileName={}: {e}; the port leaves the car out", f.path, car.model, car.mot)),
                Ok(c) => {
                    if toy::Path::from_clip(&c).is_none() {
                        todo.push(format!("{}: {}'s {} has no Body channel; the port leaves the car out", f.path, car.model, car.mot));
                    }
                }
            }
        }
    }

    // Each model, and each of its materials down to a texture, as `world::Assets` resolves them.
    let mut assets = world::Assets::default();
    // A `.psm` the folder has only as Untamed's BPB is read through its decode, so that its textures
    // are still looked for; the BPB kinds' todo entry says the port reads none of them.
    let reader = |name: &str| look.read(name).or_else(|| look.read(&format!("{name}.bpb")).and_then(|b| bpb::parse(&b).ok()).map(|b| b.to_ini()));
    let mut no_psm: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut no_tex: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut missing: Vec<String> = Vec::new();
    for (name, placed) in &models {
        let file = format!("{name}.psg");
        match look.one(&file) {
            Err(e) => todo.push(format!("{scn}: SLT={e}")),
            Ok(None) => {
                let placed = if *placed > 0 { format!("{placed} placed, ") } else { String::new() };
                missing.push(format!("{file} ({placed}{})", map.where_else(&file)));
            }
            Ok(Some(f)) => match psg::Model::parse(&f.bytes) {
                Err(_) => {} // said below, with every .psg in the folder
                Ok(m) => {
                    for mat in &m.materials {
                        material(map, &mut assets, &reader, look, mat, &f.path, &mut no_psm, &mut no_tex);
                    }
                }
            },
        }
    }
    for b in &billboards {
        material(map, &mut assets, &reader, look, b, "the ecosystem's billboards", &mut no_psm, &mut no_tex);
    }
    if !missing.is_empty() {
        todo.push(format!(
            "{scn}: {} models the scene, its ecosystem or its toys name are in no archive of the map, so the port draws nothing there: {}",
            missing.len(),
            missing.join(", ")
        ));
    }
    for (what, list, why) in [
        ("no .psm", &no_psm, "the port draws it with the default surface and no texture"),
        ("a .psm whose texture is in no archive of the map", &no_tex, "the port draws it untextured"),
    ] {
        if !list.is_empty() {
            let shown: Vec<String> = list.iter().map(|(m, by)| format!("{m}; named by {}", by.iter().cloned().collect::<Vec<_>>().join(", "))).collect();
            todo.push(format!("{} materials have {what}; {why}: {}", list.len(), shown.join(" | ")));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn material(
    map: &Map,
    assets: &mut world::Assets,
    reader: &dyn Fn(&str) -> Option<Vec<u8>>,
    look: &Look,
    mat: &str,
    by: &str,
    no_psm: &mut BTreeMap<String, BTreeSet<String>>,
    no_tex: &mut BTreeMap<String, BTreeSet<String>>,
) {
    if mat.trim().is_empty() {
        return;
    }
    let by = base(by).to_string();
    if reader(&format!("{mat}.psm")).is_none() {
        no_psm.entry(format!("{mat}.psm, {}", map.where_else(&format!("{mat}.psm")))).or_default().insert(by);
        return;
    }
    let psm = reader(&format!("{mat}.psm")).expect("just read");
    let m = rscene::Material::parse(&ini::text(&psm));
    if m.texture.is_none() && m.animation.is_none() {
        return; // a material with no texture to find: drawn as its .psm says
    }
    if assets.resolve(reader, mat).0.is_none() {
        let wanted: Vec<String> = world::texture_files(mat, &m).iter().map(|t| format!("{t}, {}", map.where_else(t))).collect();
        // The name read another way: without its spaces, as the original's text reader drops them
        // (`scene::slt_name`), or without what follows its first dot (`Nat05_liverystable_detail.dxm`).
        let mut near = String::new();
        if let Some(t) = &m.texture {
            let squeezed: String = t.chars().filter(|c| !c.is_whitespace()).collect();
            let cut = squeezed.split('.').next().unwrap_or("").to_string();
            for (v, how) in [(squeezed.clone(), "without its spaces"), (cut, "without its spaces and what follows its first dot")] {
                if v != *t
                    && let Ok(Some(f)) = look.one(&format!("{v}.pst"))
                {
                    near = format!(" (the archive has {}: the name {how}, which game-render's Material::parse does not try)", f.path);
                    break;
                }
            }
        }
        no_tex.entry(format!("{mat}.psm wants {}{near}", wanted.join(" or "))).or_default().insert(by);
    }
}

/// Every file of a kind a reader of openmx's takes, through that reader; and one entry per kind that
/// none takes.
fn kinds(files: &[File], scn: &str, todo: &mut Todo) {
    let stem = scn[..scn.len() - 4].to_ascii_lowercase();
    let mut unread: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    let mut binary: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for f in files {
        let ext = ext_of(&f.path);
        let refused = match ext.as_str() {
            // Untamed's binary INI: decoded by tools/bpb, which no reader of openmx's is.
            k if k.ends_with(".bpb") => {
                binary.entry(ext.clone()).or_default().push(&f.path);
                match bpb::parse(&f.bytes) {
                    Err(e) => Some(("tools/bpb", e)),
                    Ok(b) if !b.bad_hashes().is_empty() => Some(("tools/bpb", format!("names whose hash does not recompute: {}", b.bad_hashes().join(", ")))),
                    Ok(_) => None,
                }
            }
            ".psg" => psg::Model::parse(&f.bytes).err().map(|e| ("psg::Model::parse", e)),
            ".pst" => pst::Texture::parse(&f.bytes).err().map(|e| ("pst::Texture::parse", e)),
            ".atm" => pst::Texture::parse_animation(&f.bytes).err().map(|e| ("pst::Texture::parse_animation", e)),
            ".cub" => sky::Sky::parse(&f.bytes).err().map(|e| ("sky::Sky::parse", e)),
            ".mot" => skin::Clip::parse(&f.bytes).err().map(|e| ("skin::Clip::parse", e)),
            // Read above: the scene, the terrain, the collision the scene places, the ecosystem, the
            // toys, and the materials the models name.
            ".scn" | ".trn" | ".col" | ".esb" | ".toy" | ".psm" => None,
            ".aud" => {
                if base(&f.path).eq_ignore_ascii_case(&format!("{stem}.aud")) {
                    todo.push(format!(
                        "{}: the track's sounds; the port reads them only at tracks\\<stem>\\<stem>.aud, so a mod's needs openmx MD2's suffix lookup",
                        f.path
                    ));
                } else {
                    unread.entry(ext).or_default().push(&f.path);
                }
                None
            }
            _ => {
                unread.entry(ext).or_default().push(&f.path);
                None
            }
        };
        if let Some((reader, e)) = refused {
            todo.push(format!("{}: {reader}: {e}; kept as the disc has it", f.path));
        }
    }
    for (ext, list) in binary {
        let shown: Vec<&str> = list.iter().take(3).copied().collect();
        let more = if list.len() > 3 { format!(", and {} more", list.len() - 3) } else { String::new() };
        todo.push(format!(
            "*{ext} ({} files: {}{more}): MX vs. ATV Untamed's binary INI (BPB), which no reader in game-sim or game-render takes, so the port \
             reads none of them (openmx ask: a BPB reader); each decodes with tools/bpb, every hash recomputing; kept as the disc has it",
            list.len(),
            shown.join(", ")
        ));
    }
    for (ext, list) in unread {
        let shown: Vec<&str> = list.iter().take(3).copied().collect();
        let more = if list.len() > 3 { format!(", and {} more", list.len() - 3) } else { String::new() };
        todo.push(format!(
            "*{ext} ({} files: {}{more}): no reader in game-sim or game-render takes this kind; kept as the disc has it",
            list.len(),
            shown.join(", ")
        ));
    }
}

/// A `[section]`'s three numbers as the file has them, for a todo entry.
fn vec3(sections: &[ini::Section], section: &str, key: &str) -> String {
    sections
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(section))
        .and_then(|s| s.get(key))
        .map(|v| format!("({})", v.trim()))
        .unwrap_or_else(|| "(none)".into())
}

/// What a map's `.atyList` says about its Free Ride.
#[derive(Clone, Debug)]
pub struct FreeRide {
    pub name: String,
    /// (position, direction, from) as JSON arrays and the key they came from.
    pub spawn: Option<(String, String, String)>,
    pub todo: Vec<String>,
}

/// Read the `.atyList`'s `[General]` `DisplayName` and the `[Free Ride]` activity (the section an
/// `Activity` line names whose `Type` is `free ride`, M8).
pub fn free_ride(member: &str, bytes: &[u8]) -> Result<FreeRide, String> {
    free_ride_of(member, bytes, "General")
}

/// [`free_ride`] with the list's `[General]` under another name: Untamed's one `Master.atyList` has a
/// `[<MAP> General]` for every map.
pub fn free_ride_of(member: &str, bytes: &[u8], general: &str) -> Result<FreeRide, String> {
    let sections = ini::parse(&ini::text(bytes));
    let unquote = |v: &str| v.trim().trim_matches('"').trim().to_string();
    let general = sections.iter().find(|s| s.name.eq_ignore_ascii_case(general));
    let name = general.and_then(|s| s.get("DisplayName")).map(unquote).unwrap_or_default();
    let mut todo = Vec::new();
    let activities: Vec<String> = general
        .map(|g| g.keys.iter().filter(|(k, _)| k.eq_ignore_ascii_case("Activity")).map(|(_, v)| unquote(v)).collect())
        .unwrap_or_default();
    let fr = activities.iter().find_map(|a| {
        sections
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(a) && s.get("Type").is_some_and(|t| unquote(t).eq_ignore_ascii_case("free ride")))
    });
    let Some(fr) = fr else {
        todo.push(format!("{member}: no activity of Type \"free ride\"; the game may not offer this map as a Free Ride"));
        return Ok(FreeRide { name, spawn: None, todo });
    };
    let section = fr.name.clone();
    let spawn = match (fr.get("StartPosition"), fr.get("StartDirection")) {
        (Some(p), Some(d)) => Some((
            json_vec3(p).map_err(|e| format!("{member} [{section}] StartPosition={p}: {e}"))?,
            json_vec3(d).map_err(|e| format!("{member} [{section}] StartDirection={d}: {e}"))?,
            format!("{member} [{section}] StartPosition, StartDirection"),
        )),
        (Some(_), None) | (None, Some(_)) => {
            todo.push(format!("{member} [{section}]: one of StartPosition and StartDirection without the other; not used"));
            None
        }
        (None, None) => None,
    };
    if let Some(a) = fr.get("ATYFile") {
        todo.push(format!(
            "{member} [{section}] ATYFile={}: the Free Ride's own activity file, not copied and not read by the port",
            a.trim()
        ));
    }
    if fr.get("AllowRadar").is_some_and(|v| !ini::switch(&unquote(v))) {
        todo.push(format!("{member} [{section}] AllowRadar=FALSE: the game shows no radar in this Free Ride; the port does not read it"));
    }
    Ok(FreeRide { name, spawn, todo })
}

/// `2465.528320,401.135345,1255.091919` as a JSON array of the same three numbers, as written (an
/// `f` suffix dropped). Each must be a decimal number JSON can hold and a single can read.
fn json_vec3(v: &str) -> Result<String, String> {
    let parts: Vec<String> = v.split(',').map(|p| p.trim().trim_end_matches(['f', 'F']).to_string()).collect();
    if parts.len() != 3 {
        return Err(format!("{} numbers, not 3", parts.len()));
    }
    for p in &parts {
        let json_ok = {
            let b = p.strip_prefix('-').unwrap_or(p).as_bytes();
            let int_end = b.iter().position(|c| !c.is_ascii_digit()).unwrap_or(b.len());
            let int_ok = int_end > 0 && !(int_end > 1 && b[0] == b'0');
            let rest = &b[int_end..];
            let frac_ok = rest.is_empty() || (rest[0] == b'.' && rest.len() > 1 && rest[1..].iter().all(u8::is_ascii_digit));
            int_ok && frac_ok
        };
        if !json_ok || p.parse::<f32>().is_err() {
            return Err(format!("{p:?} is not a decimal number"));
        }
    }
    Ok(format!("[{}]", parts.join(", ")))
}

/// A JSON string.
fn q(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

#[allow(clippy::too_many_arguments)]
fn mod_json(
    origin: &Origin,
    id: &str,
    name: &str,
    folder: &str,
    members: &[String],
    spawn: Option<&(String, String, String)>,
    lookups: &[(&str, String)],
    loose: &[(&str, &str)],
    todo: &[String],
) -> String {
    let list = |v: &[String], indent: &str| -> String {
        if v.is_empty() {
            return "[]".into();
        }
        let items: Vec<String> = v.iter().map(|s| format!("{indent}  {}", q(s))).collect();
        format!("[\n{}\n{indent}]", items.join(",\n"))
    };
    let mut o = String::from("{\n");
    o += "  \"standard\": 1,\n  \"kind\": \"track\",\n";
    o += &format!("  \"id\": {},\n  \"name\": {},\n", q(id), q(name));
    o += &format!("  \"source\": {{\n    \"game\": {},\n    \"boot\": {},\n    \"members\": {}\n  }},\n", q(origin.game), q(origin.boot), list(members, "    "));
    o += &format!("  \"version\": {},\n  \"author\": {},\n  \"permission\": \"own-disc\",\n", q(VERSION), q(AUTHOR));
    if let Some((p, d, from)) = spawn {
        o += &format!("  \"spawn\": {{\"position\": {p}, \"direction\": {d}, \"from\": {}}},\n", q(from));
    }
    o += &format!("  \"map\": {},\n", q(folder));
    let l: Vec<String> = lookups.iter().map(|(k, v)| format!("{}: {}", q(k), q(v))).collect();
    o += &format!("  \"lookups\": {{{}}},\n", l.join(", "));
    if !loose.is_empty() {
        let l: Vec<String> = loose.iter().map(|(m, why)| format!("    {}: {}", q(m), q(why))).collect();
        o += &format!("  \"loose\": {{\n{}\n  }},\n", l.join(",\n"));
    }
    o += &format!("  \"todo\": {}\n}}\n", list(todo, "  "));
    o
}

