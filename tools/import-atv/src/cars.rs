//! MX vs. ATV Unleashed's five cars as a machine pack (standard §7, openmx MD39c).
//!
//! SLUS_211.04 builds each car's archive as `%sVehicleType_%d.res`: types 4 to 8 are the buggy, the
//! monster truck, the baja, the golf cart and the sandrail (openmx MD39). Each archive holds the car's
//! `Veh\<car>.phy`, its chassis model `<car>.psg` and its collision objects `<car>m.col` and
//! `<car>f.col`, with whatever else the car draws with. Every member of the five archives is copied
//! whole, at its own path, and nothing else: the pack is what openmx needs to ride the cars on MX vs.
//! ATV's step (the `.phy`, the chassis model whose tire nodes place the tires and whose box is the
//! mass properties' half box). Drawing, sound, cameras and particles read only what the archives
//! hold, and openmx reads none of them for the cars yet ([`TODO`]).
//!
//! `mod.json`'s `machine` is [`MACHINE`]; its `cars` lists each car once as openmx picks it: the
//! name, the `.phy`, the chassis model and the collision object.

use crate::{Origin, file, json_str, leaf_of};
use discx::{Disc, ResFile};
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
    "openmx rides the cars on MX vs. ATV's step (MD39c) and draws none of them, nor plays their sounds or cameras: only each .phy and its chassis .psg are read",
    "mxvatv_cars: not yet looked at beside MX vs. ATV in PCSX2 (T7)",
];

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
    build(&archives, &origin, root)
}

fn build(archives: &[(&str, String, ResFile)], origin: &Origin, root: &Path) -> Result<Built, String> {
    let dir = root.join(ID);
    clear(&dir)?;
    let mut have = BTreeSet::new();
    let mut members: Vec<(String, String)> = Vec::new();
    let mut cars = Vec::new();
    for (name, a, res) in archives {
        let mut own = Vec::new();
        for e in res.entries().to_vec() {
            let b = res.read(&e)?;
            own.push(e.name.clone());
            if !have.insert(discx::normalise(&e.name)) {
                continue;
            }
            let path = dir.join(file(&e.name));
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| format!("{}: {e}", path.display()))?;
            std::fs::write(&path, &b).map_err(|e| format!("{}: {e}", path.display()))?;
            members.push((e.name.clone(), a.clone()));
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
    std::fs::write(dir.join("mod.json"), mod_json(origin, &cars)).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(Built { dir, members: members.into_iter().map(|(m, _)| m).collect(), cars })
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

fn mod_json(origin: &Origin, cars: &[Car]) -> String {
    let list = |items: Vec<String>| format!("[\n    {}\n  ]", items.join(",\n    "));
    let archives = cars.iter().map(|c| json_str(&c.archive)).collect::<Vec<_>>().join(", ");
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
    let todo = list(TODO.iter().map(|t| json_str(t)).collect());
    format!(
        "{{\n  \"standard\": 1,\n  \"kind\": \"machine\",\n  \"id\": {},\n  \"name\": {},\n  \"machine\": {},\n  \"source\": {{\n    \"game\": {},\n    \"boot\": {},\n    \"psr\": {},\n    \"members\": [{archives}]\n  }},\n  \"version\": \"1.0.0\",\n  \"author\": {},\n  \"permission\": \"own-disc\",\n  \"phy\": {phy},\n  \"cars\": {cars_json},\n  \"todo\": {todo}\n}}\n",
        json_str(ID),
        json_str(NAME),
        json_str(MACHINE),
        json_str(&origin.game),
        json_str(&origin.boot),
        json_str(&origin.psr),
        json_str(crate::AUTHOR),
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
