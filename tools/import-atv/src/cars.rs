//! MX vs. ATV Unleashed's five cars as a machine pack (standard §7, openmx MD39c).
//!
//! SLUS_211.04 builds each car's archive as `%sVehicleType_%d.res`: types 4 to 8 are the buggy, the
//! monster truck, the baja, the golf cart and the sandrail (openmx MD39). Each archive holds the car's
//! `Veh\<car>.phy`, its chassis model `<car>.psg` and its collision objects `<car>m.col` and
//! `<car>f.col`, with whatever else the car draws with. Every member of the five archives is copied
//! whole, at its own path: the pack is what openmx needs to ride the cars on MX vs. ATV's step (the
//! `.phy`, the chassis model whose tire nodes place the tires and whose box is the mass properties'
//! half box). For openmx MD157, to draw them, follow them and hear them, it also brings:
//!
//! - **the textures**: every texture map of every material the archives hold (the archives carry the
//!   `.psm`s but none of their `.pst`s), out of `Veh\Vehicle_tex.res` or loose in `Veh\Tex\`;
//! - **the cameras**: each car's `Cam\FC_<kind>Default.pb` and `RC_<kind>Default.pb` out of
//!   `Cam\CameraPresets.res` ([`CAMERAS`], the names SLUS_211.04 builds from the car's class);
//! - **the engine**: `Aud\carengine.aud`, loose on the PSR, and every sample its five car profiles'
//!   loops name ([`ENGINE_PROFILES`]).
//!
//! `mod.json`'s `machine` is [`MACHINE`]; its `cars` lists each car once as openmx picks it: the
//! name, the `.phy`, the chassis model and the collision object.

use crate::{Origin, Out, file, from_res, json_str, leaf_of, open_res, res_find, res_leaf};
use discx::{Disc, ResFile};
use game_render::{ini, scene::Material};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const ID: &str = "mxvatv_cars";
pub const NAME: &str = "MX vs. ATV Unleashed cars";
/// The machine openmx is to ride it as.
pub const MACHINE: &str = "mxvatv_car";

/// The five cars: openmx's word for each (`car::MxvatvMachine`), and its archive's vehicle type.
pub const CARS: [(&str, u32); 5] = [("buggy", 4), ("monstertruck", 5), ("baja", 6), ("golfcart", 7), ("sandrail", 8)];

/// What openmx does not read off this pack yet.
pub const TODO: [&str; 2] = [
    "openmx draws each car's chassis, follows it on its own FC_ camera and plays its carengine.aud loops (MD157); the particle lists, the event sounds and the other liveries' models are carried, not read",
    "mxvatv_cars: not yet looked at beside MX vs. ATV in PCSX2 (T7)",
];

/// The camera archive, and each car's follow and race camera files in it, in [`CARS`]' order.
pub const CAMERA_SET: &str = "Cam\\CameraPresets.res";
pub const CAMERAS: [[&str; 2]; 5] = [
    ["Cam\\FC_BuggyDefault.pb", "Cam\\RC_BuggyDefault.pb"],
    ["Cam\\FC_MonsterDefault.pb", "Cam\\RC_MonsterDefault.pb"],
    ["Cam\\FC_BajaDefault.pb", "Cam\\RC_BajaDefault.pb"],
    ["Cam\\FC_GolfCartDefault.pb", "Cam\\RC_GolfCartDefault.pb"],
    ["Cam\\FC_SandrailDefault.pb", "Cam\\RC_SandrailDefault.pb"],
];
/// The cars' keyed-loop engine table, loose on the PSR, and its profile for each car in [`CARS`]'
/// order (the sections are spelt as the file spells them).
pub const ENGINE_SOUNDS: &str = "Aud\\carengine.aud";
pub const ENGINE_PROFILES: [&str; 5] = ["buggy", "monstertruck", "baja", "golfcart", "SandRail"];

/// The archive of vehicle type `n`.
pub fn archive(n: u32) -> String {
    format!("Veh\\vehicletype_{n}.res")
}

/// One car as built: its word, its archive, and its `.phy`, chassis model and collision object
/// as members of the pack.
#[derive(Clone, Debug)]
pub struct Car {
    pub name: String,
    pub archive: String,
    pub phy: String,
    pub psg: String,
    pub col: String,
}

#[derive(Clone, Debug)]
pub struct Built {
    pub dir: PathBuf,
    pub members: Vec<String>,
    pub cars: Vec<Car>,
    /// What the disc did not give (a texture, a camera, a sample).
    pub todo: Vec<String>,
}

/// Build `root/mxvatv_cars/` off the disc. A folder already there is replaced, but only if it is this
/// tool's pack.
pub fn import(image: &Path, root: &Path) -> Result<Built, String> {
    let mut disc = Disc::open_any(image)?;
    let game = disc.game.ok_or_else(|| format!("{}: not a disc this repo knows", image.display()))?;
    if game.key != crate::GAME {
        return Err(format!("{}: is {} ({}), not MX vs. ATV Unleashed", image.display(), game.title, game.boot));
    }
    let origin = Origin { game: game.title.to_string(), boot: game.boot.to_string(), psr: disc.psr_name.clone() };
    let mut archives = Vec::new();
    for (name, n) in CARS {
        let a = archive(n);
        let e = disc.find(&a).cloned().ok_or_else(|| format!("{a}: not on the disc"))?;
        archives.push((name, a, disc.res(&e)?));
    }
    let cameras = open_res(&mut disc, CAMERA_SET).ok();
    let textures = open_res(&mut disc, crate::TEXTURES).ok();
    build(&mut disc, &archives, cameras.as_ref(), textures.as_ref(), &origin, root)
}

/// A file by leaf name out of `res` (its own name, where it came from, its bytes), else loose on the
/// PSR under `dir`.
fn by_leaf(disc: &mut Disc, res: Option<(&ResFile, &str)>, dir: &str, leaf: &str) -> Option<(String, String, Vec<u8>)> {
    if let Some((res, name)) = res
        && let Some((member, b)) = res_leaf(res, leaf)
    {
        return Some((member.clone(), from_res(name, &member), b));
    }
    let e = disc.find(&format!("{dir}\\{leaf}")).cloned()?;
    let b = disc.read(&e).ok()?;
    Some((e.name.clone(), e.name, b))
}

fn build(
    disc: &mut Disc,
    archives: &[(&str, String, ResFile)],
    cameras: Option<&ResFile>,
    textures: Option<&ResFile>,
    origin: &Origin,
    root: &Path,
) -> Result<Built, String> {
    let dir = root.join(ID);
    clear(&dir)?;
    let mut out = Out { dir: dir.clone(), have: BTreeSet::new(), leaves: BTreeSet::new(), folders: Default::default(), members: Vec::new() };
    let mut todo: Vec<String> = Vec::new();
    let mut cars = Vec::new();
    for (name, a, res) in archives {
        let mut own = Vec::new();
        for e in res.entries().to_vec() {
            let b = res.read(&e)?;
            own.push(e.name.clone());
            out.put(&e.name, &from_res(a, &e.name), &b)?;
        }
        let find = |leaf: &str| {
            own.iter().find(|m| leaf_of(m).eq_ignore_ascii_case(leaf)).cloned().ok_or_else(|| format!("{a}: has no {leaf}"))
        };
        cars.push(Car {
            name: name.to_string(),
            archive: a.clone(),
            phy: find(&format!("{name}.phy"))?,
            psg: find(&format!("{name}.psg"))?,
            col: find(&format!("{name}m.col"))?,
        });
    }

    // Every material's texture maps: the material's own texture (the first of its forms found), then
    // every other pass's.
    let mut no_texture = Vec::new();
    for psm in out.ending(".psm") {
        let text = ini::text(&out.read(&psm)?);
        let name = leaf_of(&psm).rsplit_once('.').map(|(s, _)| s.to_string()).unwrap_or_default();
        let own = game_render::world::texture_files(&name, &Material::parse(&text));
        if !own.is_empty() && !own.iter().any(|f| out.has_leaf(f)) {
            match own.iter().find_map(|f| by_leaf(disc, textures.map(|r| (r, crate::TEXTURES)), "Veh\\Tex", f)) {
                Some((n, from, b)) => {
                    out.put(&n, &from, &b)?;
                }
                None => no_texture.push(format!("{} ({})", leaf_of(&psm), own[0])),
            }
        }
        for s in ini::parse(&text) {
            for (k, v) in &s.keys {
                let v = v.trim().trim_matches('"');
                if !k.to_ascii_lowercase().starts_with("texturemap_") || v.is_empty() {
                    continue;
                }
                let f = format!("{v}.pst");
                if out.has_leaf(&f) {
                    continue;
                }
                match by_leaf(disc, textures.map(|r| (r, crate::TEXTURES)), "Veh\\Tex", &f) {
                    Some((n, from, b)) => {
                        out.put(&n, &from, &b)?;
                    }
                    None => no_texture.push(format!("{} ({f})", leaf_of(&psm))),
                }
            }
        }
    }
    no_texture.sort();
    no_texture.dedup();
    if !no_texture.is_empty() {
        todo.push(format!("{} texture maps a material names are not on the disc, so they draw untextured: {}", no_texture.len(), no_texture.join(", ")));
    }

    // The cameras.
    for cam in CAMERAS.iter().flatten() {
        let got = cameras.and_then(|r| res_find(r, cam)).map(|(m, b)| (m.clone(), from_res(CAMERA_SET, &m), b));
        let got = got.or_else(|| {
            let e = disc.find(cam).cloned()?;
            let b = disc.read(&e).ok()?;
            Some((e.name.clone(), e.name, b))
        });
        match got {
            Some((m, from, b)) => {
                out.put(&m, &from, &b)?;
            }
            None => todo.push(format!("{}: not on the disc; that car follows on the bike's camera", leaf_of(cam))),
        }
    }

    // The engine table and the samples its car profiles' loops name. A name with no extension is the
    // `.VAG` of that name under `Aud\`.
    let mut engine = String::new();
    match disc.find(ENGINE_SOUNDS).cloned() {
        Some(e) => {
            let b = disc.read(&e)?;
            out.put(&e.name, &e.name, &b)?;
            engine = file(&e.name);
            let sections = ini::parse(&ini::text(&b));
            let mut no_sound = Vec::new();
            for profile in ENGINE_PROFILES {
                let lower = profile.to_ascii_lowercase();
                for s in sections.iter().filter(|s| s.name.to_ascii_lowercase().strip_prefix(&lower).is_some_and(|n| n.starts_with("loop"))) {
                    for (k, v) in &s.keys {
                        if !k.eq_ignore_ascii_case("filename") {
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
                        match tries.iter().find_map(|t| disc.find(t).cloned()) {
                            Some(e) => {
                                let b = disc.read(&e)?;
                                out.put(&e.name, &e.name, &b)?;
                            }
                            None => no_sound.push(format!("{} [{}]", file(&name), s.name)),
                        }
                    }
                }
            }
            if !no_sound.is_empty() {
                todo.push(format!("{} samples carengine.aud's car loops name are not on the disc: {}", no_sound.len(), no_sound.join(", ")));
            }
        }
        None => todo.push(format!("{}: not on the disc; the cars are silent", file(ENGINE_SOUNDS))),
    }

    std::fs::write(dir.join("mod.json"), mod_json(origin, &cars, &out.members, &engine, &todo)).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(Built { dir, members: out.members.into_iter().map(|(m, _)| m).collect(), cars, todo })
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

fn mod_json(origin: &Origin, cars: &[Car], members: &[(String, String)], engine: &str, missing: &[String]) -> String {
    let list = |items: Vec<String>| format!("[\n    {}\n  ]", items.join(",\n    "));
    // `source.members` (§7): the cars' archives whole by name, any other `.res` member as
    // `OUTER:INNER`, every loose file by its PSR name.
    let mut psr: Vec<String> = cars.iter().map(|c| c.archive.clone()).collect();
    for (_, from) in members {
        let m = match from.split_once(':') {
            Some((res, _)) if cars.iter().any(|c| c.archive.eq_ignore_ascii_case(res)) => res.to_string(),
            _ => from.clone(),
        };
        if !psr.contains(&m) {
            psr.push(m);
        }
    }
    let archives = psr.iter().map(|m| json_str(m)).collect::<Vec<_>>().join(", ");
    let phy = list(cars.iter().map(|c| json_str(&file(&c.phy))).collect());
    let cars_json = list(
        cars.iter()
            .map(|c| {
                format!(
                    "{{\"car\": {}, \"phy\": {}, \"psg\": {}, \"col\": {}}}",
                    json_str(&c.name),
                    json_str(&file(&c.phy)),
                    json_str(&file(&c.psg)),
                    json_str(&file(&c.col))
                )
            })
            .collect(),
    );
    let todo = list(TODO.iter().map(|t| json_str(t)).chain(missing.iter().map(|t| json_str(t))).collect());
    format!(
        "{{\n  \"standard\": 1,\n  \"kind\": \"machine\",\n  \"id\": {},\n  \"name\": {},\n  \"machine\": {},\n  \"source\": {{\n    \"game\": {},\n    \"boot\": {},\n    \"psr\": {},\n    \"members\": [{archives}]\n  }},\n  \"version\": \"1.0.0\",\n  \"author\": {},\n  \"permission\": \"own-disc\",\n  \"phy\": {phy},\n  \"cars\": {cars_json},\n  \"sounds\": {{\"engine\": {}}},\n  \"todo\": {todo}\n}}\n",
        json_str(ID),
        json_str(NAME),
        json_str(MACHINE),
        json_str(&origin.game),
        json_str(&origin.boot),
        json_str(&origin.psr),
        json_str(crate::AUTHOR),
        json_str(engine),
    )
}

/// A built pack read back: each car's `.phy` through openmx's car reader and its chassis model
/// through the quad's model reader, with the tire nodes the car's load needs. Each car's line.
pub fn check(dir: &Path) -> Result<Vec<String>, String> {
    let res = game_render::Res::folder(dir)?;
    let mut out = Vec::new();
    for (name, _) in CARS {
        let read = |m: String| res.read(&m.replace('/', "\\")).map_err(|e| format!("{m}: {e}"));
        let phy = read(format!("Veh/{name}.phy")).or_else(|_| {
            let m = res.entries().iter().find(|e| leaf_of(&e.name).eq_ignore_ascii_case(&format!("{name}.phy"))).map(|e| e.name.clone());
            m.ok_or(format!("no {name}.phy")).and_then(|m| read(m))
        })?;
        let psg = res
            .entries()
            .iter()
            .find(|e| leaf_of(&e.name).eq_ignore_ascii_case(&format!("{name}.psg")))
            .map(|e| e.name.clone())
            .ok_or(format!("no {name}.psg"))?;
        let ini = game_sim::phy::Ini::parse_bytes(&phy);
        let model = game_sim::quad::Model::parse(&read(psg.clone())?).map_err(|e| format!("{psg}: {e}"))?;
        for n in game_sim::car::TIRE_NODES {
            if !model.nodes.iter().any(|m| m.name == n) {
                return Err(format!("{psg}: no {n}"));
            }
        }
        let tires = ["FrontLeftTire", "FrontRightTire", "RearLeftTire", "RearRightTire"].iter().all(|t| ini.section(t).float("TireRadius").is_some());
        out.push(format!("{name}: {} nodes, tires {}", model.nodes.len(), if tires { "read" } else { "missing" }));
    }
    Ok(out)
}
