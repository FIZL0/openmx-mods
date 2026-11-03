//! MX vs. ATV Unleashed's biplane and Kiowa as a machine pack (standard §7, openmx MD80d).
//!
//! MX vs. ATV keeps MX Unleashed's two fliers and gives each four skins over two models (openmx's
//! `research/mxvatv-pc.md`, "The biplane and the Kiowa (MD80)"): `biplane`/`biplane2` and
//! `kiowa`/`kiowa2` are model one, `biplane3`/`4` and `kiowa3`/`4` model two. The pack holds, each
//! file a disc member's bytes at its own path:
//!
//! - **the biplane's archive**, `Veh\VehicleType_2.res` (`%sVehicleType_%d.res`, type 2) whole: the
//!   four skins' models and their `_ui` garage models, the prop cones, the eight `Biplane*{F,M}.col`
//!   (the grown `BiplaneM`, the new `Biplane3`/`4`), `biplane.phy`, the materials, the particle list
//!   and its emitters, `event.aud`;
//! - **the Kiowa's files**, loose on the PSR (no `.res` holds them): `kiowa.phy`, `WreckBlade.phy`,
//!   `kiowa{,2,3,4}{,_ui}.psg`, the blade and tail-rotor parts, the eight `Kiowa*{F,M}.col`,
//!   `helicopterparticles.dat` and its emitters;
//! - **the garage's sets**, `UI\Res\Veh_BiPlane1-4.res` and `Veh_Helicopter1-4.res`, every member:
//!   each skin's `_ui` model, materials and textures. They are the only copies of most skins'
//!   textures on the disc (`Pitts_body01.pst`, `SpitFire_01.pst` are in no other archive);
//! - **the materials and textures** every model names that neither of those has, loose in `Veh\Tex\`
//!   (or out of `Veh\Vehicle_mat.res` / `Vehicle_tex.res`), every texture map of every shader pass;
//! - **the cameras**: `Cam\FC_BiplaneDefault.pb`, `RC_BiplaneDefault.pb`, `FC_HeliDefault.pb` and
//!   `RC_HeliDefault.pb` out of `Cam\CameraPresets.res`;
//! - **the sounds**: `Aud\carengine.aud` (`[biplane]`'s seven keyed loops), `otherengine.aud`
//!   (`[Helicopter]`'s `EngineLoop`) and `machine.aud` (`[biplaneWreck]`, `[biplaneTireTouchdown]`,
//!   `[helicopterWreck]`), and every `.VAG` those sections name;
//! - **the pilot's clips**, `Rider\Anim\biplane_*.mot`.

use super::{Origin, Out, Which, file, from_res, json_str, leaf_of, open_res, res_find};
use discx::{Disc, ResFile};
use game_render::scene::Material;
use game_render::{Model, ini};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const AUTHOR: &str = "openmx-mods tools/import-atv fliers 0.1";
pub const ID: &str = "mxvatv_fliers";
pub const NAME: &str = "MX vs. ATV Unleashed biplane and Kiowa";
/// What openmx is to ride it as: both of MX Unleashed's fliers, each by its `skins` entries.
pub const MACHINE: &str = "fliers";

/// The biplane's archive: `%sVehicleType_%d.res` with 2 (every member a biplane file).
pub const ARCHIVE: &str = "Veh\\vehicletype_2.res";
/// The Kiowa's files and what both fliers share, loose on the PSR.
pub const LOOSE: [&str; 13] = [
    "Veh\\kiowa.phy",
    "Veh\\WreckBlade.phy",
    "Veh\\Anim\\kiowa*.psg",
    "Veh\\Anim\\bladeb*.psg",
    "Veh\\Anim\\tailrotor*.psg",
    "Veh\\Anim\\sh_biplane2.psg",
    "Veh\\Anim\\SH_BiPlane2.pst",
    "Veh\\Col\\Kiowa*.col",
    "Rider\\Anim\\biplane_*.mot",
    "Part\\helicopterparticles.dat",
    "Aud\\carengine.aud",
    "Aud\\otherengine.aud",
    "Aud\\machine.aud",
];
pub const CAMERA_SET: &str = "Cam\\CameraPresets.res";
pub const CAMERAS: [&str; 4] = ["Cam\\FC_BiplaneDefault.pb", "Cam\\RC_BiplaneDefault.pb", "Cam\\FC_HeliDefault.pb", "Cam\\RC_HeliDefault.pb"];
pub const LOOP_ENGINE: &str = "Aud\\carengine.aud";
pub const HELI_ENGINE: &str = "Aud\\otherengine.aud";
pub const MACHINE_SOUNDS: &str = "Aud\\machine.aud";
pub const EVENT_SOUNDS: &str = "Aud\\event.aud";
pub const BIPLANE_PARTICLES: &str = "Part\\biplaneparticles.dat";
pub const KIOWA_PARTICLES: &str = "Part\\helicopterparticles.dat";
/// The zones `vehicletype_2.res` carries beside the biplane's list.
pub const PARTICLE_ZONES: &str = "Part\\mxparticlezones.dat";

/// The two fliers, as `mod.json`'s `skins[].machine` names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flier {
    Biplane,
    Kiowa,
}

impl Flier {
    pub const ALL: [Flier; 2] = [Flier::Biplane, Flier::Kiowa];

    pub fn name(self) -> &'static str {
        match self {
            Flier::Biplane => "biplane",
            Flier::Kiowa => "kiowa",
        }
    }

    /// Skin `n` (1 to 4)'s model: `biplane`, `biplane2`, … (the plain name is skin 1).
    pub fn skin(self, n: usize) -> String {
        match n {
            1 => self.name().to_string(),
            n => format!("{}{n}", self.name()),
        }
    }

    /// Skin `n`'s collision pair stem: `Biplane`, `Biplane2`, … `Kiowa4`.
    fn col(self, n: usize) -> String {
        let stem = match self {
            Flier::Biplane => "Biplane",
            Flier::Kiowa => "Kiowa",
        };
        match n {
            1 => stem.to_string(),
            n => format!("{stem}{n}"),
        }
    }

    /// Skin `n`'s garage set: `UI\Res\Veh_BiPlane<n>.res`, `Veh_Helicopter<n>.res`.
    pub fn garage(self, n: usize) -> String {
        match self {
            Flier::Biplane => format!("UI\\Res\\Veh_BiPlane{n}.res"),
            Flier::Kiowa => format!("UI\\Res\\Veh_Helicopter{n}.res"),
        }
    }

    fn phy(self) -> &'static str {
        match self {
            Flier::Biplane => "Veh\\biplane.phy",
            Flier::Kiowa => "Veh\\kiowa.phy",
        }
    }

    fn cameras(self) -> (&'static str, &'static str) {
        match self {
            Flier::Biplane => (CAMERAS[0], CAMERAS[1]),
            Flier::Kiowa => (CAMERAS[2], CAMERAS[3]),
        }
    }

    fn particles(self) -> &'static str {
        match self {
            Flier::Biplane => BIPLANE_PARTICLES,
            Flier::Kiowa => KIOWA_PARTICLES,
        }
    }

    /// The `.aud` sections this flier's sounds are in, by the prefix their names start with.
    fn sound_prefix(self) -> &'static str {
        match self {
            Flier::Biplane => "biplane",
            Flier::Kiowa => "helicopter",
        }
    }
}

/// The skin's model: 1 for skins 1 and 2, 2 for 3 and 4 (the init's switch on the skin's name form).
pub fn model_of(n: usize) -> usize {
    if n <= 2 { 1 } else { 2 }
}

/// One skin, as `mod.json` lists it.
#[derive(Clone, Debug)]
pub struct Skin {
    pub machine: Flier,
    pub skin: String,
    pub model: usize,
    pub psg: String,
    pub ui: String,
    pub hull: String,
    pub rays: String,
    pub garage: String,
}

#[derive(Clone, Debug)]
pub struct Built {
    pub dir: PathBuf,
    pub members: Vec<String>,
    pub skins: Vec<Skin>,
    pub todo: Vec<String>,
}

struct Source {
    disc: Disc,
    archive: ResFile,
    cameras: Option<ResFile>,
    materials: Option<ResFile>,
    textures: Option<ResFile>,
}

impl Source {
    fn by_leaf(&mut self, which: Which, dir: &str, leaf: &str) -> Option<(String, String, Vec<u8>)> {
        let res = match which {
            Which::Materials => self.materials.as_ref().map(|r| (r, super::MATERIALS)),
            Which::Textures => self.textures.as_ref().map(|r| (r, super::TEXTURES)),
            Which::Archive => Some((&self.archive, ARCHIVE)),
        };
        if let Some((res, name)) = res
            && let Some((member, b)) = super::res_leaf(res, leaf)
        {
            let from = from_res(name, &member);
            return Some((member, from, b));
        }
        let e = self.disc.find(&format!("{dir}\\{leaf}")).cloned()?;
        let b = self.disc.read(&e).ok()?;
        Some((e.name.clone(), e.name, b))
    }
}

/// `Out::put`, and where the path is written already, a line in `clash` if the bytes differ (two
/// archives carrying one path two ways: the pack can hold only one).
fn put(out: &mut Out, clash: &mut Vec<String>, member: &str, from: &str, bytes: &[u8]) -> Result<(), String> {
    if !out.put(member, from, bytes)? {
        let at = out.members.iter().find(|(m, _)| discx::normalise(m) == discx::normalise(member)).map(|(m, f)| (m.clone(), f.clone()));
        if let Some((m, first)) = at
            && out.read(&m)? != bytes
        {
            clash.push(format!("{} from {from} differs from the copy from {first}", file(&m)));
        }
    }
    Ok(())
}

/// Build `root/mxvatv_fliers/` off the disc. A folder already there is replaced only if it is this
/// pack.
pub fn import(image: &Path, root: &Path) -> Result<Built, String> {
    let mut disc = Disc::open_any(image)?;
    let game = disc.game.ok_or_else(|| format!("{}: not a disc this repo knows", image.display()))?;
    if game.key != super::GAME {
        return Err(format!("{}: is {} ({}), not MX vs. ATV Unleashed", image.display(), game.title, game.boot));
    }
    let origin = Origin { game: game.title.to_string(), boot: game.boot.to_string(), psr: disc.psr_name.clone() };
    let archive = open_res(&mut disc, ARCHIVE)?;
    let cameras = open_res(&mut disc, CAMERA_SET).ok();
    let materials = open_res(&mut disc, super::MATERIALS).ok();
    let textures = open_res(&mut disc, super::TEXTURES).ok();
    let mut src = Source { disc, archive, cameras, materials, textures };
    build(&mut src, &origin, root)
}

fn build(src: &mut Source, origin: &Origin, root: &Path) -> Result<Built, String> {
    let dir = root.join(ID);
    clear(&dir)?;
    let mut out = Out { dir: dir.clone(), have: BTreeSet::new(), leaves: BTreeSet::new(), folders: Default::default(), members: Vec::new() };
    let mut todo = Vec::new();
    let mut clash = Vec::new();

    // The biplane's archive, whole.
    for e in src.archive.entries().to_vec() {
        let b = src.archive.read(&e)?;
        put(&mut out, &mut clash, &e.name, &from_res(ARCHIVE, &e.name), &b)?;
    }
    // The Kiowa's and the shared files, loose.
    for glob in LOOSE {
        let found: Vec<_> = src.disc.matching(glob).into_iter().cloned().collect();
        if found.is_empty() {
            todo.push(format!("{}: not on the disc", file(glob)));
        }
        for e in found {
            let b = src.disc.read(&e)?;
            put(&mut out, &mut clash, &e.name, &e.name, &b)?;
        }
    }
    // The garage's sets, every member.
    let mut garages = Vec::new();
    for f in Flier::ALL {
        for n in 1..=4 {
            let name = f.garage(n);
            let res = match src.disc.find(&name).cloned() {
                Some(e) => src.disc.res(&e)?,
                None => {
                    todo.push(format!("{}: not on the disc; skin {} has no garage set", file(&name), f.skin(n)));
                    continue;
                }
            };
            for e in res.entries().to_vec() {
                let b = res.read(&e)?;
                put(&mut out, &mut clash, &e.name, &from_res(&name, &e.name), &b)?;
            }
            garages.push((name, res.entries().iter().map(|e| e.name.clone()).collect::<Vec<_>>()));
        }
    }
    // The cameras.
    for cam in CAMERAS {
        let got = src.cameras.as_ref().and_then(|r| res_find(r, cam)).map(|(m, b)| (m.clone(), from_res(CAMERA_SET, &m), b));
        let got = got.or_else(|| {
            let e = src.disc.find(cam).cloned()?;
            let b = src.disc.read(&e).ok()?;
            Some((e.name.clone(), e.name, b))
        });
        match got {
            Some((m, from, b)) => put(&mut out, &mut clash, &m, &from, &b)?,
            None => todo.push(format!("{}: not on the disc", leaf_of(cam))),
        }
    }

    // Every model's materials, then every material's textures (each shader pass's texture maps).
    let mut wanted: BTreeSet<String> = BTreeSet::new();
    for psg in out.ending(".psg") {
        match Model::parse(&out.read(&psg)?) {
            Ok(model) => wanted.extend(model.materials.iter().map(|m| m.to_ascii_lowercase())),
            Err(e) => todo.push(format!("{}: openmx's psg::Model::parse refuses it: {e}", file(&psg))),
        }
    }
    let mut no_material = Vec::new();
    for m in &wanted {
        let leaf = format!("{m}.psm");
        if out.has_leaf(&leaf) {
            continue;
        }
        match src.by_leaf(Which::Materials, "Veh\\Tex", &leaf) {
            Some((name, from, b)) => put(&mut out, &mut clash, &name, &from, &b)?,
            None => no_material.push(leaf),
        }
    }
    if !no_material.is_empty() {
        todo.push(format!("{} materials a model names are not on the disc: {}", no_material.len(), no_material.join(", ")));
    }
    let mut no_texture = Vec::new();
    for psm in out.ending(".psm") {
        let text = ini::text(&out.read(&psm)?);
        let name = leaf_of(&psm).rsplit_once('.').map(|(s, _)| s.to_string()).unwrap_or_default();
        // The material's own texture, the first of its forms found (as import-atv takes them).
        let own = game_render::world::texture_files(&name, &Material::parse(&text));
        if !own.is_empty() && !own.iter().any(|f| out.has_leaf(f)) {
            match own.iter().find_map(|f| src.by_leaf(Which::Textures, "Veh\\Tex", f)) {
                Some((n, from, b)) => put(&mut out, &mut clash, &n, &from, &b)?,
                None => no_texture.push(format!("{} ({})", leaf_of(&psm), own[0])),
            }
        }
        // Every other pass's maps too (the Kiowa's specular passes, the blade blurs).
        for s in ini::parse(&text) {
            for (k, v) in &s.keys {
                let v = v.trim().trim_matches('"');
                if !k.to_ascii_lowercase().starts_with("texturemap_") || v.is_empty() {
                    continue;
                }
                let f = format!("{v}.pst");
                if out.has_leaf(&f) || own.iter().any(|o| o.eq_ignore_ascii_case(&f)) {
                    continue;
                }
                match src.by_leaf(Which::Textures, "Veh\\Tex", &f) {
                    Some((n, from, b)) => put(&mut out, &mut clash, &n, &from, &b)?,
                    None => no_texture.push(format!("{} ({f})", leaf_of(&psm))),
                }
            }
        }
    }
    no_texture.sort();
    no_texture.dedup();
    if !no_texture.is_empty() {
        todo.push(format!("{} texture maps a material names are not on the disc: {}", no_texture.len(), no_texture.join(", ")));
    }

    // The sounds the fliers' sections of the three `.aud` name: `Filename`, `EngineLoop`, `SoundN`.
    // A name with no extension is the `.VAG` of that name under `Aud\`.
    let mut no_sound = Vec::new();
    for aud in [LOOP_ENGINE, HELI_ENGINE, MACHINE_SOUNDS] {
        let Ok(b) = out.read(aud) else { continue };
        for s in ini::parse(&ini::text(&b)) {
            let lower = s.name.to_ascii_lowercase();
            if !Flier::ALL.iter().any(|f| lower.starts_with(f.sound_prefix())) {
                continue;
            }
            for (k, v) in &s.keys {
                let k = k.to_ascii_lowercase();
                let numbered = k.strip_prefix("sound").is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
                if !(numbered || k == "filename" || k == "engineloop") {
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
                        put(&mut out, &mut clash, &e.name, &e.name, &b)?;
                    }
                    None => no_sound.push(format!("{} ({} [{}] {k})", file(&name), leaf_of(aud), s.name)),
                }
            }
        }
    }
    if !no_sound.is_empty() {
        todo.push(format!("{} sounds the fliers' .aud sections name are not on the disc: {}", no_sound.len(), no_sound.join(", ")));
    }

    // Every emitter the two particle lists name.
    for f in Flier::ALL {
        let list = f.particles();
        let Ok(b) = out.read(list) else {
            todo.push(format!("{}: not on the disc", file(list)));
            continue;
        };
        for e in game_render::biplane::emitters(&ini::text(&b)) {
            if e.emitter.is_empty() || out.has_leaf(&e.emitter) {
                continue;
            }
            match src.by_leaf(Which::Archive, "Part", &e.emitter) {
                Some((n, from, b)) => put(&mut out, &mut clash, &n, &from, &b)?,
                None => todo.push(format!("{}: [{}] names {}, which is not on the disc", file(list), e.name, e.emitter)),
            }
        }
    }

    // The skins: each one's model, garage model, collision pair and garage set.
    let mut skins = Vec::new();
    let spelt = |out: &Out, want: &str| out.members.iter().map(|(m, _)| m).find(|m| discx::normalise(m) == discx::normalise(want)).cloned();
    for f in Flier::ALL {
        for n in 1..=4 {
            let skin = f.skin(n);
            let need = |want: String, todo: &mut Vec<String>| match spelt(&out, &want) {
                Some(m) => file(&m),
                None => {
                    todo.push(format!("{}: skin {skin} names it, not on the disc", file(&want)));
                    file(&want)
                }
            };
            let psg = need(format!("Veh\\Anim\\{skin}.psg"), &mut todo);
            let ui = need(format!("Veh\\Anim\\{skin}_ui.psg"), &mut todo);
            let hull = need(format!("Veh\\Col\\{}F.col", f.col(n)), &mut todo);
            let rays = need(format!("Veh\\Col\\{}M.col", f.col(n)), &mut todo);
            let garage = f.garage(n);
            // The set is the skin's: its `_ui` model is the skin's own.
            if let Some((_, names)) = garages.iter().find(|(g, _)| *g == garage) {
                let want = format!("{skin}_ui.psg");
                if !names.iter().any(|m| leaf_of(m).eq_ignore_ascii_case(&want)) {
                    todo.push(format!("{}: holds no {want}; which skin it dresses is not known", file(&garage)));
                }
            }
            skins.push(Skin { machine: f, skin, model: model_of(n), psg, ui, hull, rays, garage: file(&garage) });
        }
    }

    if !clash.is_empty() {
        return Err(format!("{} paths two sources carry with different bytes: {}", clash.len(), clash.join("; ")));
    }
    todo.push(
        "the fliers' MX vs. ATV steps are openmx MD80b/MD80c and their pick MD80e: until those land the port loads this pack and flies none of it"
            .into(),
    );
    todo.push(format!("{ID}: not yet looked at beside MX vs. ATV in PCSX2 (T7)"));

    let json = mod_json(origin, &out.members, &skins, &garages.iter().map(|(g, _)| g.clone()).collect::<Vec<_>>(), &todo);
    std::fs::write(dir.join("mod.json"), json).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(Built { dir, members: out.members.into_iter().map(|(m, _)| m).collect(), skins, todo })
}

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

fn mod_json(origin: &Origin, members: &[(String, String)], skins: &[Skin], garages: &[String], todo: &[String]) -> String {
    let list = |items: Vec<String>, indent: &str| {
        if items.is_empty() {
            "[]".to_string()
        } else {
            format!("[\n{indent}  {}\n{indent}]", items.join(&format!(",\n{indent}  ")))
        }
    };
    // `source.members` (§7): the archive and the garage sets whole by name, any other `.res` member
    // as `OUTER:INNER`, every loose file by its PSR name.
    let mut psr: Vec<String> = Vec::new();
    for (_, from) in members {
        let m = match from.split_once(':') {
            Some((res, _)) if res.eq_ignore_ascii_case(ARCHIVE) || garages.iter().any(|g| g.eq_ignore_ascii_case(res)) => res.to_string(),
            _ => from.clone(),
        };
        if !psr.contains(&m) {
            psr.push(m);
        }
    }
    let skins_json: Vec<String> = skins
        .iter()
        .map(|s| {
            format!(
                "{{\"machine\": {}, \"skin\": {}, \"model\": {}, \"psg\": {}, \"ui\": {}, \"hull\": {}, \"rays\": {}, \"garage\": {}}}",
                json_str(s.machine.name()),
                json_str(&s.skin),
                s.model,
                json_str(&s.psg),
                json_str(&s.ui),
                json_str(&s.hull),
                json_str(&s.rays),
                json_str(&s.garage)
            )
        })
        .collect();
    let fliers: Vec<String> = Flier::ALL
        .iter()
        .map(|&f| {
            let (follow, replay) = f.cameras();
            format!(
                "\"{}\": {{\"phy\": {}, \"cameras\": {}, \"replay_cameras\": {}, \"particles\": {}, \"sounds\": {}}}",
                f.name(),
                json_str(&file(f.phy())),
                json_str(&file(follow)),
                json_str(&file(replay)),
                json_str(&file(f.particles())),
                json_str(f.sound_prefix()),
            )
        })
        .collect();
    let mut phy: Vec<String> = members.iter().map(|(m, _)| m).filter(|m| m.to_ascii_lowercase().ends_with(".phy")).map(|m| json_str(&file(m))).collect();
    phy.sort();
    let mut clips: Vec<String> = members
        .iter()
        .map(|(m, _)| m)
        .filter(|m| m.to_ascii_lowercase().starts_with("rider\\") && m.to_ascii_lowercase().ends_with(".mot"))
        .map(|m| json_str(&file(m)))
        .collect();
    clips.sort();
    format!(
        "{{\n  \"standard\": 1,\n  \"kind\": \"machine\",\n  \"id\": {},\n  \"name\": {},\n  \"machine\": {},\n  \"source\": {{\n    \"game\": {},\n    \"boot\": {},\n    \"psr\": {},\n    \"members\": {}\n  }},\n  \"version\": \"1.0.0\",\n  \"author\": {},\n  \"permission\": \"own-disc\",\n  \"archive\": {},\n  \"skins\": {},\n  \"fliers\": {{\n    {}\n  }},\n  \"phy\": {},\n  \"pilot_clips\": {},\n  \"sounds\": {{\"engine\": {}, \"helicopter\": {}, \"machine\": {}, \"events\": {}}},\n  \"particles\": {{\"list\": {}, \"kiowa\": {}, \"zones\": {}}},\n  \"todo\": {}\n}}\n",
        json_str(ID),
        json_str(NAME),
        json_str(MACHINE),
        json_str(&origin.game),
        json_str(&origin.boot),
        json_str(&origin.psr),
        list(psr.iter().map(|m| json_str(m)).collect(), "    "),
        json_str(AUTHOR),
        json_str(ARCHIVE),
        list(skins_json, "  "),
        fliers.join(",\n    "),
        list(phy, "  "),
        list(clips, "  "),
        json_str(&file(LOOP_ENGINE)),
        json_str(&file(HELI_ENGINE)),
        json_str(&file(MACHINE_SOUNDS)),
        json_str(&file(EVENT_SOUNDS)),
        json_str(&file(BIPLANE_PARTICLES)),
        json_str(&file(KIOWA_PARTICLES)),
        json_str(&file(PARTICLE_ZONES)),
        list(todo.iter().map(|t| json_str(t)).collect(), "  "),
    )
}

/// What [`check`] read back.
#[derive(Debug, Default)]
pub struct Checked {
    pub models: usize,
    pub collisions: usize,
    pub clips: usize,
    pub materials: usize,
    pub textures: usize,
    pub sounds: usize,
    pub unmarked_sounds: usize,
    pub emitters: usize,
    /// Each `FC_`/`RC_` camera file and its one-player 4:3 presets (`crate::camera_presets`).
    pub cameras: Vec<(String, usize)>,
    /// Each skin, its model, and each node openmx wants of it with the name the model gives it
    /// (`None`: not there).
    pub skins: Vec<(String, usize, Vec<(String, Option<String>)>)>,
    pub refused: Vec<(String, String)>,
}

/// The nodes openmx hangs a flier's moving parts and pilot on.
pub fn nodes_wanted(f: Flier) -> Vec<&'static str> {
    let mut v = match f {
        Flier::Biplane => vec![game_render::biplane::PROP_NODE],
        Flier::Kiowa => {
            let mut v = vec![game_render::kiowa::SPINDLE, game_render::kiowa::TAIL];
            v.extend(game_render::kiowa::BLADES);
            v
        }
    };
    v.extend(game_render::biplane::SEAT);
    v
}

/// A built pack read back from its folder: every model, collision file, material, texture, clip and
/// sound through openmx's readers, and each skin's model's flier nodes.
pub fn check(dir: &Path) -> Result<Checked, String> {
    let res = game_render::Res::folder(dir)?;
    let mut c = Checked::default();
    for e in res.entries() {
        let lower = e.name.to_ascii_lowercase();
        let bytes = res.read(&e.name)?;
        let mut refuse = |why: String| c.refused.push((file(&e.name), why));
        if lower.ends_with(".psg") {
            match Model::parse(&bytes) {
                Ok(_) => c.models += 1,
                Err(why) => refuse(why),
            }
        } else if lower.ends_with(".col") {
            // `<stem>F.col` is the hull the world meets, `<stem>M.col` the ray probe (openmx's
            // `flight::HULL` / `flight_search::BIPLANE_RAYS`, `kiowa::KIOWA_HULL` / `KIOWA_RAYS`).
            let got = match lower.ends_with("m.col") {
                true => game_sim::track::Rays::parse(&bytes).map(|_| ()),
                false => game_sim::track::Mesh::parse(&bytes).map(|_| ()),
            };
            match got {
                Ok(()) => c.collisions += 1,
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
        } else if let Some(n) = crate::camera_presets(&e.name, &bytes) {
            match n {
                0 => refuse("no 4x3/1p presets".into()),
                n => c.cameras.push((file(&e.name), n)),
            }
        } else if lower.ends_with("particles.dat") {
            c.emitters += game_render::biplane::emitters(&ini::text(&bytes)).len();
        }
    }
    for f in Flier::ALL {
        for n in 1..=4 {
            let skin = f.skin(n);
            let model = match res.read_suffix(&format!("\\{skin}.psg")).and_then(|b| Model::parse(&b)) {
                Ok(m) => m,
                Err(why) => {
                    c.refused.push((format!("{skin}.psg"), why));
                    continue;
                }
            };
            let wanted = nodes_wanted(f);
            // Each wanted node as the model spells it: Kiowa3/4 spell three of the seat's dummies with
            // a `01` on the end (`Dummy_Foot_R_pos01`), which is the name given here.
            let found: Vec<Option<String>> = wanted
                .iter()
                .map(|w| {
                    let spelt = |name: &str| model.nodes.iter().find(|nd| nd.name.eq_ignore_ascii_case(name)).map(|nd| nd.name.clone());
                    spelt(w).or_else(|| spelt(&format!("{w}01")))
                })
                .collect();
            c.skins.push((skin, model_of(n), wanted.iter().map(|w| w.to_string()).zip(found).collect()));
        }
    }
    Ok(c)
}
