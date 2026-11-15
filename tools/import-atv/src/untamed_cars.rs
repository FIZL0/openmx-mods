//! MX vs. ATV Untamed's baja truck and golf cart as a machine pack (standard §7, openmx MD163, MD81e).
//!
//! SLUS-21701 builds each car's archive as `%sVehicleType_%d.res` under `res\veh\`, numbered as MX vs.
//! ATV Unleashed's (openmx MD81): 6 is the baja truck (the menus' TROPHY TRUCK), 7 the golf cart
//! (`golfcart.phy.bpb`: MaxAnalogThrottle 0.8 -> 0.1 and `[Preload]` off). Each archive holds the
//! car's `Veh\baja.phy.bpb` (the `.phy` compiled to a BPB, 51 keys retuned against Unleashed's: openmx
//! research/untamed-machines.md), its chassis model `baja.psg` and its collision objects, with whatever
//! else the car draws with. Every member is copied whole, at its own path, as [`crate::cars`] does for
//! Unleashed's five.
//!
//! The car runs Untamed's car step, not Unleashed's: `mod.json`'s `cars` entry carries
//! `"step": "untamed"` ([`STEP`]), which openmx reads as `machine_packs::CarFiles::untamed` and rides
//! as `atv_car::Load::untamed` (MD161). openmx offers it as `--mxvatv-car baja@untamed_cars`; a bare
//! `baja` stays Unleashed's (`mxvatv_cars`).

use crate::{Origin, file, json_str, leaf_of};
use discx::{Disc, ResFile};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const ID: &str = "untamed_cars";
pub const NAME: &str = "MX vs. ATV Untamed cars";
/// The machine openmx is to ride it as: the same as Unleashed's cars.
pub const MACHINE: &str = "mxvatv_car";
/// discx's key for the source.
pub const GAME: &str = "untamed";
/// The car entry's `step` word: openmx's `machine_packs::UNTAMED_STEP`.
pub const STEP: &str = "untamed";

/// The cars: openmx's word for each (`car::MxvatvMachine`), and its archive's vehicle type. The monster
/// truck (type 5) is openmx MD161b's.
pub const CARS: [(&str, u32); 2] = [("baja", 6), ("golfcart", 7)];

/// What openmx does not read off this pack yet.
pub const TODO: [&str; 2] = [
    "openmx rides the baja and the golf cart on MX vs. ATV Untamed's car step (MD161, MD163, MD81e) and draws none of it, nor plays its sounds or cameras: only the .phy.bpb and its chassis .psg are read",
    "untamed_cars: not yet looked at beside MX vs. ATV Untamed in PCSX2 (T7)",
];

/// The archive of vehicle type `n`.
pub fn archive(n: u32) -> String {
    format!("res\\veh\\VehicleType_{n}.res")
}

/// One car as built: its word, its archive, and its `.phy.bpb`, chassis model and collision object
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

/// Build `root/untamed_cars/` off the disc. A folder already there is replaced, but only if it is this
/// tool's pack.
pub fn import(image: &Path, root: &Path) -> Result<Built, String> {
    let mut disc = Disc::open_any(image)?;
    let game = disc.game.ok_or_else(|| format!("{}: not a disc this repo knows", image.display()))?;
    if game.key != GAME {
        return Err(format!("{}: is {} ({}), not MX vs. ATV Untamed", image.display(), game.title, game.boot));
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
    let mut members = Vec::new();
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
            members.push(e.name.clone());
        }
        let find = |leaves: &[String]| {
            leaves
                .iter()
                .find_map(|leaf| own.iter().find(|m| leaf_of(m).eq_ignore_ascii_case(leaf)).cloned())
                .ok_or_else(|| format!("{a}: has no {}", leaves.join(" or ")))
        };
        cars.push(Car {
            name: name.to_string(),
            archive: a.clone(),
            phy: find(&[format!("{name}.phy.bpb"), format!("{name}.phy")])?,
            psg: find(&[format!("{name}.psg")])?,
            col: find(&[format!("{name}m.col")])?,
        });
    }
    std::fs::write(dir.join("mod.json"), mod_json(origin, &cars)).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(Built { dir, members, cars })
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
                    "{{\"car\": {}, \"phy\": {}, \"psg\": {}, \"col\": {}, \"step\": {}}}",
                    json_str(&c.name),
                    json_str(&file(&c.phy)),
                    json_str(&file(&c.psg)),
                    json_str(&file(&c.col)),
                    json_str(STEP)
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

/// A built pack read back: each car's `.phy.bpb` through openmx's BPB reader and its chassis model
/// through the quad's model reader, with the tire nodes the car's load needs. Each car's line.
pub fn check(dir: &Path) -> Result<Vec<String>, String> {
    let res = game_render::Res::folder(dir)?;
    let mut out = Vec::new();
    for (name, _) in CARS {
        let find = |leaf: &str| res.entries().iter().find(|e| leaf_of(&e.name).eq_ignore_ascii_case(leaf)).map(|e| e.name.clone());
        let read = |m: &str| res.read(m).map_err(|e| format!("{m}: {e}"));
        let phy = find(&format!("{name}.phy.bpb")).ok_or(format!("no {name}.phy.bpb"))?;
        let psg = find(&format!("{name}.psg")).ok_or(format!("no {name}.psg"))?;
        let ini = game_sim::bpb::ini(&read(&phy)?).map_err(|e| format!("{phy}: {e}"))?;
        let model = game_sim::quad::Model::parse(&read(&psg)?).map_err(|e| format!("{psg}: {e}"))?;
        for n in game_sim::car::TIRE_NODES {
            if !model.nodes.iter().any(|m| m.name == n) {
                return Err(format!("{psg}: no {n}"));
            }
        }
        let tires = ["FrontLeftTire", "FrontRightTire", "RearLeftTire", "RearRightTire"].iter().all(|t| ini.section(t).float("TireRadius").is_some());
        let weight = ini.section("Mass Properties").float("Weight").map_or("none".to_string(), |w| w.0.to_string());
        out.push(format!("{name}: {} nodes, tires {}, Weight {weight}", model.nodes.len(), if tires { "read" } else { "missing" }));
    }
    Ok(out)
}
