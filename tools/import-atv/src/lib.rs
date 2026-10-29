//! MX vs. ATV Unleashed's quads as a machine pack (standard §7, openmx MD37c).
//!
//! A machine pack is what openmx needs to draw, pose and hear a machine MX Unleashed does not have,
//! as the source disc has it. For the ATV that is SLUS_211.04's own sets, each found by the name the
//! ELF builds for it:
//!
//! - **the machine's archive**, `Veh\VehicleType_1.res` (`%sVehicleType_%d.res`): the 96 quad builds
//!   (`.sub`), their three chassis models and every part, the chassis' motion sets and the quad's
//!   clips, the eight `atv_*.phy`, the collision models (`WreckATVm.col` among them), the engine's
//!   `vehicle.aud` and `event.aud`, and the particles (`atvparticles.dat`, its zones, every `.emt`).
//!   Every member is copied, at its own path.
//! - **the materials and textures** the models and the builds' `[Material Changes]` name that the
//!   archive lacks: each `.psm` out of `Veh\Vehicle_mat.res` (or the PSR's loose `Veh\Tex\`), each
//!   material's `.pst` loose (or out of `Veh\Vehicle_tex.res`).
//! - **the quad's own clips**, every `Veh\Anim\ATV_*.mot` loose on the PSR (the flips are not in the
//!   archive).
//! - **the rider**, `Rider\RiderMotionSet_ATV.res` (`%sRiderMotionSet_%s.res`): the ATV rider's riding
//!   and stunt motion sets, every clip they name (most stunt clips are loose in `Rider\Anim\`), and
//!   the stunt table `ATV_Rider_Stunts.ini`.
//! - **the cameras**: `Cam\FC_ATVDefault.pb` out of `Cam\CameraPresets.res` (the file GAME_REL.BIN
//!   names for the quad's follow camera) and `Cam\FC_ATV.ini` beside it.
//! - **the sounds**: every `.VAG` the archive's `vehicle.aud` names, at `Aud\<name>`.
//!
//! Every file is the member's bytes. [`check`] reads a built pack back through openmx's readers.

use discx::{Disc, ResFile};
use game_render::scene::{Material, Vehicle};
use game_render::{Model, ini};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const AUTHOR: &str = "openmx-mods tools/import-atv 0.1";
pub const ID: &str = "mxvatv_atv";
pub const NAME: &str = "MX vs. ATV Unleashed quads";
/// The machine openmx is to ride it as.
pub const MACHINE: &str = "atv";
/// discx's key for the source.
pub const GAME: &str = "mxvatv";

/// The quad's archive: SLUS_211.04 builds `%sVehicleType_%d.res`, and type 1 is the one holding the
/// quads (every `.sub` in it is an `ATVS_*` build; types 0 the bikes, 2 the biplane, 4 to 8 the cars).
pub const ARCHIVE: &str = "Veh\\vehicletype_1.res";
/// The rider's motion sets: `%sRiderMotionSet_%s.res` with `ATV`.
pub const RIDER_SET: &str = "Rider\\RiderMotionSet_ATV.res";
pub const STUNT_TABLE: &str = "Rider\\Anim\\ATV_Rider_Stunts.ini";
/// The ATV rider's riding motions and stunt motions, both in [`RIDER_SET`].
pub const RIDER_MOTIONS: &str = "Rider\\Anim\\ATV_RiderMotions.mcf";
pub const RIDER_STUNTS: &str = "Rider\\Anim\\ATV_RiderStunts.mcf";
/// The quad's own clips, loose on the PSR (`ATV_Bike_<trick>.mot` MD21 reads, the `ATV_Air_*` poses).
pub const QUAD_CLIPS: &str = "Veh\\Anim\\ATV_*.mot";
/// The camera archive (`%sCameraPresets.res`) and the two quad camera files in it.
pub const CAMERA_SET: &str = "Cam\\CameraPresets.res";
pub const CAMERAS: &str = "Cam\\FC_ATVDefault.pb";
pub const CAMERAS_INI: &str = "Cam\\FC_ATV.ini";
pub const MATERIALS: &str = "Veh\\Vehicle_mat.res";
pub const TEXTURES: &str = "Veh\\Vehicle_tex.res";
/// The engine's sound table and the machine's event sounds, in [`ARCHIVE`].
pub const ENGINE_SOUNDS: &str = "Aud\\vehicle.aud";
pub const EVENT_SOUNDS: &str = "Aud\\event.aud";
/// The particle lists GAME_REL.BIN names for the quad (the player's and the computer riders'), and
/// the surfaces' zones, in [`ARCHIVE`].
pub const PARTICLES: &str = "Part\\atvparticles.dat";
pub const AI_PARTICLES: &str = "Part\\atvaiparticles.dat";
pub const PARTICLE_ZONES: &str = "Part\\atvparticlezones.dat";
/// The quad's wreck body (MD37: the string inside the quad's init).
pub const WRECK_COL: &str = "Veh\\Col\\WreckATVm.col";

/// What the disc is, for `mod.json`'s `source`.
#[derive(Clone, Debug)]
pub struct Origin {
    pub game: String,
    pub boot: String,
    pub psr: String,
}

/// One quad build: a `.sub`, its motion set and the chassis model that set names.
#[derive(Clone, Debug)]
pub struct Build {
    pub sub: String,
    pub mcf: String,
    pub chassis: String,
    pub parts: usize,
}

#[derive(Clone, Debug)]
pub struct Built {
    pub dir: PathBuf,
    pub members: Vec<String>,
    pub builds: Vec<Build>,
    pub todo: Vec<String>,
}

/// Where a pack member came from: the PSR itself, or a `.res` in it.
fn from_res(res: &str, member: &str) -> String {
    format!("{res}:{member}")
}

/// The disc, with the `.res` sets this pack reads opened once.
struct Source {
    disc: Disc,
    archive: ResFile,
    rider: ResFile,
    cameras: Option<ResFile>,
    materials: Option<ResFile>,
    textures: Option<ResFile>,
}

fn open_res(disc: &mut Disc, name: &str) -> Result<ResFile, String> {
    let e = disc.find(name).cloned().ok_or_else(|| format!("{name}: not on the disc"))?;
    disc.res(&e)
}

/// A member of `res` by its path, ignoring case and slash style: its own name and bytes.
fn res_find(res: &ResFile, name: &str) -> Option<(String, Vec<u8>)> {
    let want = discx::normalise(name);
    let e = res.entries().iter().find(|e| discx::normalise(&e.name) == want)?;
    Some((e.name.clone(), res.read(e).ok()?))
}

/// The first member of `res` whose file name (past the last `\`) is `leaf`, ignoring case.
fn res_leaf(res: &ResFile, leaf: &str) -> Option<(String, Vec<u8>)> {
    let e = res.entries().iter().find(|e| leaf_of(&e.name).eq_ignore_ascii_case(leaf))?;
    Some((e.name.clone(), res.read(e).ok()?))
}

fn leaf_of(name: &str) -> &str {
    name.rsplit(['\\', '/']).next().unwrap_or(name)
}

/// The pack being written: every member's bytes at its own path, once.
struct Out {
    dir: PathBuf,
    /// Lower-cased paths already written, so a member two sets carry is written once.
    have: BTreeSet<String>,
    /// Lower-cased leaf names written, for lookups by leaf.
    leaves: BTreeSet<String>,
    /// Each folder, lower-cased with a trailing `/`, as it is spelt in the pack.
    folders: std::collections::BTreeMap<String, String>,
    /// Each file and where it came from, in the order written.
    members: Vec<(String, String)>,
}

impl Out {
    fn put(&mut self, member: &str, from: &str, bytes: &[u8]) -> Result<bool, String> {
        if !self.have.insert(discx::normalise(member)) {
            return Ok(false);
        }
        // The disc spells one folder several ways (`Veh\Tex`, `veh\tex`): each is written the way it
        // was first spelt, so the pack has one folder per folder on any file system.
        let mut path = self.dir.clone();
        let parts: Vec<&str> = member.split(['\\', '/']).collect();
        let mut at = String::new();
        for p in &parts[..parts.len() - 1] {
            at.push_str(&p.to_ascii_lowercase());
            at.push('/');
            let spelt = self.folders.entry(at.clone()).or_insert_with(|| p.to_string());
            path.push(spelt.as_str());
        }
        let mut spelt: Vec<String> = path.strip_prefix(&self.dir).unwrap().iter().map(|p| p.to_string_lossy().into_owned()).collect();
        spelt.push(parts[parts.len() - 1].to_string());
        let member = spelt.join("\\");
        let member = member.as_str();
        path.push(parts[parts.len() - 1]);
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| format!("{}: {e}", path.display()))?;
        std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        self.leaves.insert(leaf_of(member).to_ascii_lowercase());
        self.members.push((member.to_owned(), from.to_owned()));
        Ok(true)
    }

    fn has_leaf(&self, leaf: &str) -> bool {
        self.leaves.contains(&leaf.to_ascii_lowercase())
    }

    fn read(&self, member: &str) -> Result<Vec<u8>, String> {
        let path = self.dir.join(member.replace('\\', "/"));
        std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// The members written whose name ends in `ext`, ignoring case.
    fn ending(&self, ext: &str) -> Vec<String> {
        self.members
            .iter()
            .map(|(m, _)| m.clone())
            .filter(|m| m.to_ascii_lowercase().ends_with(ext))
            .collect()
    }
}

impl Source {
    /// A file by leaf name: out of `res` first, then loose in the PSR under `dir`.
    fn by_leaf(&mut self, which: Which, dir: &str, leaf: &str) -> Option<(String, String, Vec<u8>)> {
        let res = match which {
            Which::Materials => self.materials.as_ref().map(|r| (r, MATERIALS)),
            Which::Textures => self.textures.as_ref().map(|r| (r, TEXTURES)),
            Which::Archive => Some((&self.archive, ARCHIVE)),
        };
        if let Some((res, name)) = res
            && let Some((member, b)) = res_leaf(res, leaf)
        {
            let from = from_res(name, &member);
            return Some((member, from, b));
        }
        let loose = format!("{dir}\\{leaf}");
        let e = self.disc.find(&loose).cloned()?;
        let b = self.disc.read(&e).ok()?;
        Some((e.name.clone(), e.name, b))
    }
}

#[derive(Clone, Copy)]
enum Which {
    Archive,
    Materials,
    Textures,
}

/// Build `root/mxvatv_atv/` off the disc. A folder already there is replaced, but only if it is this
/// tool's pack.
pub fn import(image: &Path, root: &Path) -> Result<Built, String> {
    let mut disc = Disc::open_any(image)?;
    let game = disc.game.ok_or_else(|| format!("{}: not a disc this repo knows", image.display()))?;
    if game.key != GAME {
        return Err(format!("{}: is {} ({}), not MX vs. ATV Unleashed", image.display(), game.title, game.boot));
    }
    let origin = Origin { game: game.title.to_string(), boot: game.boot.to_string(), psr: disc.psr_name.clone() };
    let archive = open_res(&mut disc, ARCHIVE)?;
    let rider = open_res(&mut disc, RIDER_SET)?;
    let cameras = open_res(&mut disc, CAMERA_SET).ok();
    let materials = open_res(&mut disc, MATERIALS).ok();
    let textures = open_res(&mut disc, TEXTURES).ok();
    let mut src = Source { disc, archive, rider, cameras, materials, textures };
    build(&mut src, &origin, root)
}

fn build(src: &mut Source, origin: &Origin, root: &Path) -> Result<Built, String> {
    let dir = root.join(ID);
    clear(&dir)?;
    let mut out = Out { dir: dir.clone(), have: BTreeSet::new(), leaves: BTreeSet::new(), folders: Default::default(), members: Vec::new() };
    let mut todo = Vec::new();

    // The machine's archive, whole.
    for e in src.archive.entries().to_vec() {
        let b = src.archive.read(&e)?;
        out.put(&e.name, &from_res(ARCHIVE, &e.name), &b)?;
    }
    // The rider's motion sets, whole, and the stunt table (the loose member; StuntInfo.res holds the
    // same bytes, import-tricks' disc test).
    for e in src.rider.entries().to_vec() {
        let b = src.rider.read(&e)?;
        out.put(&e.name, &from_res(RIDER_SET, &e.name), &b)?;
    }
    let e = src.disc.find(STUNT_TABLE).cloned().ok_or_else(|| format!("{STUNT_TABLE}: not on the disc"))?;
    let b = src.disc.read(&e)?;
    out.put(&e.name, &e.name, &b)?;
    // The cameras.
    for cam in [CAMERAS, CAMERAS_INI] {
        let got = match &src.cameras {
            Some(r) => res_find(r, cam).map(|(m, b)| (m.clone(), from_res(CAMERA_SET, &m), b)),
            None => None,
        };
        let got = got.or_else(|| {
            let e = src.disc.find(cam).cloned()?;
            let b = src.disc.read(&e).ok()?;
            Some((e.name.clone(), e.name, b))
        });
        match got {
            Some((m, from, b)) => {
                out.put(&m, &from, &b)?;
            }
            None => todo.push(format!("{}: not on the disc; the quad has no cameras of its own", leaf_of(cam))),
        }
    }

    // Every clip each motion set names (the chassis' and the rider's), and every quad clip on the
    // PSR: most of the rider's stunt clips and the quad's flips are loose in `Rider\\Anim\\` and
    // `Veh\\Anim\\`, not in the two archives.
    let mut no_clip = Vec::new();
    for mcf in out.ending(".mcf") {
        let set = game_render::rider::MotionSet::parse(&ini::text(&out.read(&mcf)?));
        let dir = match set.directory.trim().trim_end_matches(['\\', '/']) {
            "" => mcf.rsplit_once('\\').map(|(d, _)| d.to_string()).unwrap_or_default(),
            d => d.to_string(),
        };
        for m in &set.motions {
            if m.clip.is_empty() || out.has_leaf(&m.clip) {
                continue;
            }
            match src.disc.find(&format!("{dir}\\{}", m.clip)).cloned() {
                Some(e) => {
                    let b = src.disc.read(&e)?;
                    out.put(&e.name, &e.name, &b)?;
                }
                None => no_clip.push(format!("{} ({})", m.clip, file(&mcf))),
            }
        }
    }
    if !no_clip.is_empty() {
        todo.push(format!("{} clips a motion set names are not on the disc: {}", no_clip.len(), no_clip.join(", ")));
    }
    for e in src.disc.matching(QUAD_CLIPS).into_iter().cloned().collect::<Vec<_>>() {
        if !out.has_leaf(leaf_of(&e.name)) {
            let b = src.disc.read(&e)?;
            out.put(&e.name, &e.name, &b)?;
        }
    }

    // The builds, and every model they and their motion sets name.
    let mut builds = Vec::new();
    let mut wanted_models: BTreeSet<String> = BTreeSet::new();
    let mut wanted_materials: BTreeSet<String> = BTreeSet::new();
    for sub in out.ending(".sub") {
        let v = Vehicle::parse(&ini::text(&out.read(&sub)?));
        let chassis = match out.members.iter().find(|(m, _)| leaf_of(m).eq_ignore_ascii_case(&v.mcf)) {
            Some((mcf, _)) => game_render::rider::MotionSet::parse(&ini::text(&out.read(&mcf.clone())?)).geometry(),
            None => {
                todo.push(format!("{}: names {}, which the archive lacks", file(&sub), v.mcf));
                String::new()
            }
        };
        if !chassis.is_empty() {
            wanted_models.insert(chassis.to_ascii_lowercase());
        }
        for p in &v.parts {
            wanted_models.insert(p.model.to_ascii_lowercase());
        }
        for (_, to) in &v.materials {
            wanted_materials.insert(to.to_ascii_lowercase());
        }
        builds.push(Build { sub: sub.clone(), mcf: v.mcf.clone(), chassis, parts: v.parts.len() });
    }
    for m in &wanted_models {
        let leaf = format!("{m}.psg");
        if out.has_leaf(&leaf) {
            continue;
        }
        match src.by_leaf(Which::Archive, "Veh\\Anim", &leaf) {
            Some((name, from, b)) => {
                out.put(&name, &from, &b)?;
            }
            None => todo.push(format!("{leaf}: a build names it and it is on neither {} nor the PSR", leaf_of(ARCHIVE))),
        }
    }
    // Every model's materials.
    for psg in out.ending(".psg") {
        match Model::parse(&out.read(&psg)?) {
            Ok(model) => wanted_materials.extend(model.materials.iter().map(|m| m.to_ascii_lowercase())),
            Err(e) => todo.push(format!("{}: openmx's psg::Model::parse refuses it: {e}", file(&psg))),
        }
    }
    let mut no_material = Vec::new();
    for m in &wanted_materials {
        let leaf = format!("{m}.psm");
        if out.has_leaf(&leaf) {
            continue;
        }
        match src.by_leaf(Which::Materials, "Veh\\Tex", &leaf) {
            Some((name, from, b)) => {
                out.put(&name, &from, &b)?;
            }
            None => no_material.push(leaf),
        }
    }
    if !no_material.is_empty() {
        todo.push(format!(
            "{} materials a model or a build names are on neither {} nor the PSR: {}",
            no_material.len(),
            leaf_of(MATERIALS),
            no_material.join(", ")
        ));
    }
    // Every material's texture: the first of its candidate files found.
    let mut no_texture = Vec::new();
    for psm in out.ending(".psm") {
        let name = leaf_of(&psm).rsplit_once('.').map(|(s, _)| s.to_string()).unwrap_or_default();
        let mat = Material::parse(&ini::text(&out.read(&psm)?));
        let files = game_render::world::texture_files(&name, &mat);
        if files.is_empty() || files.iter().any(|f| out.has_leaf(f)) {
            continue;
        }
        let mut found = false;
        for f in &files {
            if let Some((n, from, b)) = src.by_leaf(Which::Textures, "Veh\\Tex", f) {
                out.put(&n, &from, &b)?;
                found = true;
                break;
            }
        }
        if !found {
            no_texture.push(format!("{} ({})", leaf_of(&psm), files[0]));
        }
    }
    if !no_texture.is_empty() {
        todo.push(format!(
            "{} materials' textures are on neither {} nor the PSR, so they draw untextured: {}",
            no_texture.len(),
            leaf_of(TEXTURES),
            no_texture.join(", ")
        ));
    }

    // The sounds the engine's .aud names: each `SoundN` of every list, `[Wreck]`'s too. A name with no
    // extension (`Vehicle\WRECK01`) is the `.VAG` of that name, as on the PSR. `event.aud`'s are the
    // race's (the start gate, the countdown) and not the machine's: carried, not followed.
    let mut no_sound = Vec::new();
    let aud = ini::text(&out.read(ENGINE_SOUNDS)?);
    for s in ini::parse(&aud) {
        for (k, v) in &s.keys {
            let numbered = k.to_ascii_lowercase().strip_prefix("sound").is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
            if !numbered {
                continue;
            }
            let name = format!("Aud\\{}", v.trim().trim_matches('"'));
            let tries = match leaf_of(&name).contains('.') {
                true => vec![name.clone()],
                false => vec![format!("{name}.VAG"), format!("{name}.vig")],
            };
            if tries.iter().any(|t| out.have.contains(&discx::normalise(t))) {
                continue;
            }
            match tries.iter().find_map(|t| src.disc.find(t).cloned()) {
                Some(e) => {
                    let b = src.disc.read(&e)?;
                    out.put(&e.name, &e.name, &b)?;
                }
                None => no_sound.push(format!("{} [{}] {k}", file(&name), s.name)),
            }
        }
    }
    if !no_sound.is_empty() {
        todo.push(format!("{} sounds vehicle.aud names are not on the disc: {}", no_sound.len(), no_sound.join(", ")));
    }

    // Every emitter the particle lists name.
    for list in [PARTICLES, AI_PARTICLES] {
        let Ok(b) = out.read(list) else {
            todo.push(format!("{}: not in {}", file(list), leaf_of(ARCHIVE)));
            continue;
        };
        for e in game_render::biplane::emitters(&ini::text(&b)) {
            if e.emitter.is_empty() || out.has_leaf(&e.emitter) {
                continue;
            }
            match src.by_leaf(Which::Archive, "Part", &e.emitter) {
                Some((n, from, b)) => {
                    out.put(&n, &from, &b)?;
                }
                None => todo.push(format!("{}: [{}] names {}, which is not on the disc", file(list), e.name, e.emitter)),
            }
        }
    }

    todo.push(
        "the quad's step is openmx MD37b and its pick MD37d: until both land the port loads this pack and rides none of it".into(),
    );
    todo.push(format!("{ID}: not yet looked at beside MX vs. ATV in PCSX2 (T7)"));

    let json = mod_json(origin, &out.members, &builds, &todo);
    std::fs::write(dir.join("mod.json"), json).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(Built { dir, members: out.members.into_iter().map(|(m, _)| m).collect(), builds, todo })
}

/// A member's path in the pack.
pub fn file(member: &str) -> String {
    member.replace('\\', "/")
}

/// Make way for the pack: a folder that is not there, or one whose `mod.json` is this pack's, removed.
fn clear(dir: &Path) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    let ours = std::fs::read_to_string(dir.join("mod.json"))
        .is_ok_and(|t| t.contains(&format!("\"id\": {}", json_str(ID))) && t.contains("\"kind\": \"machine\""));
    if !ours {
        return Err(format!("{}: there already, and not this tool's {ID} pack; not touched", dir.display()));
    }
    std::fs::remove_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))
}

/// A JSON string.
pub fn json_str(s: &str) -> String {
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

fn mod_json(origin: &Origin, members: &[(String, String)], builds: &[Build], todo: &[String]) -> String {
    let list = |items: Vec<String>, indent: &str| {
        if items.is_empty() {
            "[]".to_string()
        } else {
            format!("[\n{indent}  {}\n{indent}]", items.join(&format!(",\n{indent}  ")))
        }
    };
    // `source.members` are what the files came from, each once (§7): the two archives copied whole by
    // name, every other member of a `.res` as `OUTER:INNER`, and every loose file by its PSR name.
    let mut psr: Vec<String> = Vec::new();
    for (_, from) in members {
        let m = match from.split_once(':') {
            Some((res, _)) if res.eq_ignore_ascii_case(ARCHIVE) || res.eq_ignore_ascii_case(RIDER_SET) => res.to_string(),
            _ => from.clone(),
        };
        if !psr.contains(&m) {
            psr.push(m);
        }
    }
    let builds_json: Vec<String> = builds
        .iter()
        .map(|b| {
            format!(
                "{{\"sub\": {}, \"mcf\": {}, \"chassis\": {}}}",
                json_str(&file(&b.sub)),
                json_str(&b.mcf),
                json_str(&b.chassis)
            )
        })
        .collect();
    let mut phy: Vec<String> = members
        .iter()
        .map(|(m, _)| m)
        .filter(|m| m.to_ascii_lowercase().ends_with(".phy"))
        .map(|m| json_str(&file(m)))
        .collect();
    phy.sort();
    format!(
        "{{\n  \"standard\": 1,\n  \"kind\": \"machine\",\n  \"id\": {},\n  \"name\": {},\n  \"machine\": {},\n  \"source\": {{\n    \"game\": {},\n    \"boot\": {},\n    \"psr\": {},\n    \"members\": {}\n  }},\n  \"version\": \"1.0.0\",\n  \"author\": {},\n  \"permission\": \"own-disc\",\n  \"archive\": {},\n  \"builds\": {},\n  \"phy\": {},\n  \"rider\": {{\"rider\": \"ATV\", \"motions\": {}, \"stunts\": {}, \"table\": {}}},\n  \"bike_clips\": \"Veh/Anim/ATV_Bike_<name>.mot\",\n  \"cameras\": {},\n  \"cameras_ini\": {},\n  \"sounds\": {{\"engine\": {}, \"events\": {}}},\n  \"particles\": {{\"list\": {}, \"ai\": {}, \"zones\": {}}},\n  \"wreck\": {},\n  \"todo\": {}\n}}\n",
        json_str(ID),
        json_str(NAME),
        json_str(MACHINE),
        json_str(&origin.game),
        json_str(&origin.boot),
        json_str(&origin.psr),
        list(psr.iter().map(|m| json_str(m)).collect(), "    "),
        json_str(AUTHOR),
        json_str(ARCHIVE),
        list(builds_json, "  "),
        list(phy, "  "),
        json_str(&file(RIDER_MOTIONS)),
        json_str(&file(RIDER_STUNTS)),
        json_str(&file(STUNT_TABLE)),
        json_str(&file(CAMERAS)),
        json_str(&file(CAMERAS_INI)),
        json_str(&file(ENGINE_SOUNDS)),
        json_str(&file(EVENT_SOUNDS)),
        json_str(&file(PARTICLES)),
        json_str(&file(AI_PARTICLES)),
        json_str(&file(PARTICLE_ZONES)),
        json_str(&file(WRECK_COL)),
        list(todo.iter().map(|t| json_str(t)).collect(), "  "),
    )
}

/// What [`check`] read back.
#[derive(Debug, Default)]
pub struct Checked {
    pub models: usize,
    pub clips: usize,
    pub materials: usize,
    pub textures: usize,
    pub sounds: usize,
    /// Of [`Checked::sounds`], those whose header lacks the `VAGp` magic.
    pub unmarked_sounds: usize,
    pub emitters: usize,
    /// Each chassis model and its nodes, as `name<parent`.
    pub chassis: Vec<(String, Vec<String>)>,
    /// Every file openmx's reader refused, with why.
    pub refused: Vec<(String, String)>,
}

/// A built pack read back from its folder: every model, material, texture, clip and sound through
/// openmx's readers, and each chassis' node names.
pub fn check(dir: &Path) -> Result<Checked, String> {
    let res = game_render::Res::folder(dir)?;
    let mut c = Checked::default();
    let mut chassis: BTreeSet<String> = BTreeSet::new();
    for e in res.entries() {
        let lower = e.name.to_ascii_lowercase();
        if lower.ends_with(".mcf") && lower.starts_with("veh\\") {
            let set = game_render::rider::MotionSet::parse(&ini::text(&res.read(&e.name)?));
            chassis.insert(set.geometry().to_ascii_lowercase());
        }
    }
    for e in res.entries() {
        let lower = e.name.to_ascii_lowercase();
        let bytes = res.read(&e.name)?;
        let mut refuse = |why: String| c.refused.push((file(&e.name), why));
        if lower.ends_with(".psg") {
            match Model::parse(&bytes) {
                Ok(m) => {
                    c.models += 1;
                    let stem = leaf_of(&lower).trim_end_matches(".psg").to_string();
                    if chassis.contains(&stem) {
                        let named = |n: &game_render::psg::Node| match n.parent {
                            Some(p) => format!("{}<{}", n.name, m.nodes[p].name),
                            None => n.name.clone(),
                        };
                        c.chassis.push((file(&e.name), m.nodes.iter().map(named).collect()));
                    }
                }
                Err(why) => refuse(why),
            }
        } else if lower.ends_with(".mot") {
            match game_render::Clip::parse(&bytes) {
                Ok(_) => c.clips += 1,
                Err(why) => refuse(why),
            }
        } else if lower.ends_with(".pst") {
            match game_render::Texture::parse(&bytes) {
                Ok(_) => c.textures += 1,
                Err(why) => refuse(why),
            }
        } else if lower.ends_with(".psm") {
            c.materials += 1;
        } else if lower.ends_with(".vag") {
            // A VAG's 0x30-byte header (size at 0x0c, rate at 0x10, big-endian), then the data; three
            // of the quad's engine sounds have the header with its `VAGp` zeroed, as openmx's
            // `audio::Clip::vag` reads them (it never looks at the magic).
            let be = |at: usize| bytes.get(at..at + 4).map(|b| u32::from_be_bytes(b.try_into().unwrap()));
            match (be(0x0c), be(0x10)) {
                (Some(size), Some(4000..=96000)) if bytes.len() >= 0x40 && size as usize + 0x30 <= bytes.len() => {
                    c.sounds += 1;
                    if bytes.get(..4) != Some(b"VAGp") {
                        c.unmarked_sounds += 1;
                    }
                }
                _ => refuse("no VAG header".into()),
            }
        } else if lower.ends_with("particles.dat") {
            c.emitters += game_render::biplane::emitters(&ini::text(&bytes)).len();
        }
    }
    Ok(c)
}
