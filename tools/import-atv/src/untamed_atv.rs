//! MX vs. ATV Untamed's race quad as a machine pack (standard §7, openmx MD144).
//!
//! SLUS-21701 builds the quad's archive as `res\veh\VehicleType_1.res`, numbered as MX vs. ATV
//! Unleashed's (openmx MD81). It holds the quads' `.phy` compiled to BPBs (`Veh\atv_Race250P.phy.bpb`,
//! `Veh\atv_Pro250P.phy.bpb`, …), the race chassis `250r_atv_chasis.psg`, its collision objects and the
//! wreck's (`WreckATVm.col`), with whatever else the quads draw with. Every member is copied whole, at
//! its own path, as [`crate::untamed_cars`] does for the cars.
//!
//! The quad runs Untamed's step, not Unleashed's: `mod.json` carries `"step": "untamed"` ([`STEP`]),
//! which openmx reads as `machine_packs::Pack::untamed` and rides through `quad::Ride::untamed`
//! (MD144: Untamed's preload, backflip, 360, hazard clock and stunt ids). Only the two race quads are
//! listed (`phy`), the ones MD147 captured the original on; openmx offers them as
//! `--atv atv_Race250P@untamed_atv`, and a bare `atv_Race250P` stays Unleashed's (`mxvatv_atv`).

use crate::{Origin, file, json_str, leaf_of};
use discx::{Disc, ResFile};
use std::path::{Path, PathBuf};

pub const ID: &str = "untamed_atv";
pub const NAME: &str = "MX vs. ATV Untamed quads";
/// The machine openmx is to ride it as: the same as Unleashed's quads.
pub const MACHINE: &str = "atv";
/// discx's key for the source.
pub const GAME: &str = "untamed";
/// The pack's `step` word: openmx's `machine_packs::UNTAMED_STEP`.
pub const STEP: &str = "untamed";
/// The quad's archive.
pub const ARCHIVE: &str = "res\\veh\\VehicleType_1.res";

/// The quads listed: the race chassis' two handling files.
pub const QUADS: [&str; 2] = ["atv_Race250P", "atv_Pro250P"];

/// The race chassis' model and its motion set, the build openmx draws the quad by, and the collision
/// objects openmx rides it with.
pub const CHASSIS: &str = "250r_atv_chasis";
pub const MCF: &str = "ATV_R.mcf";
/// The build's `.sub` (compiled, as the `.phy`): the player's 250 (`quad_draw::build` takes a player's).
pub const SUB: &str = "ATV_R_250_KTM.sub.bpb";
pub const COLS: [&str; 2] = ["ATV_R_ChasisM.col", "WreckATVm.col"];

/// What openmx does not read off this pack yet.
pub const TODO: [&str; 2] = [
    "openmx rides the race quads on MX vs. ATV Untamed's step (MD144) off the .phy.bpb, the chassis .psg and its two collision objects only; it does not draw the pack's parts, play its sounds or use its cameras",
    "untamed_atv: not yet looked at beside MX vs. ATV Untamed in PCSX2 (T7)",
];

#[derive(Clone, Debug)]
pub struct Built {
    pub dir: PathBuf,
    pub members: Vec<String>,
    /// The listed `.phy.bpb`s as members of the pack.
    pub phy: Vec<String>,
}

/// Build `root/untamed_atv/` off the disc. A folder already there is replaced, but only if it is this
/// tool's pack.
pub fn import(image: &Path, root: &Path) -> Result<Built, String> {
    let mut disc = Disc::open_any(image)?;
    let game = disc.game.ok_or_else(|| format!("{}: not a disc this repo knows", image.display()))?;
    if game.key != GAME {
        return Err(format!("{}: is {} ({}), not MX vs. ATV Untamed", image.display(), game.title, game.boot));
    }
    let origin = Origin { game: game.title.to_string(), boot: game.boot.to_string(), psr: disc.psr_name.clone() };
    let e = disc.find(ARCHIVE).cloned().ok_or_else(|| format!("{ARCHIVE}: not on the disc"))?;
    let res = disc.res(&e)?;
    build(&res, &origin, root)
}

fn build(res: &ResFile, origin: &Origin, root: &Path) -> Result<Built, String> {
    let dir = root.join(ID);
    clear(&dir)?;
    let mut members = Vec::new();
    for e in res.entries().to_vec() {
        let b = res.read(&e)?;
        let path = dir.join(file(&e.name));
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| format!("{}: {e}", path.display()))?;
        std::fs::write(&path, &b).map_err(|e| format!("{}: {e}", path.display()))?;
        members.push(e.name.clone());
    }
    let find = |leaf: &str| members.iter().find(|m| leaf_of(m).eq_ignore_ascii_case(leaf)).cloned().ok_or_else(|| format!("{ARCHIVE}: has no {leaf}"));
    let phy = QUADS.iter().map(|q| find(&format!("{q}.phy.bpb"))).collect::<Result<Vec<_>, _>>()?;
    let (mcf, sub) = (find(MCF)?, find(SUB)?);
    find(&format!("{CHASSIS}.psg"))?;
    for c in COLS {
        find(c)?;
    }
    std::fs::write(dir.join("mod.json"), mod_json(origin, &phy, &sub, &mcf)).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(Built { dir, members, phy })
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

fn mod_json(origin: &Origin, phy: &[String], sub: &str, mcf: &str) -> String {
    let list = |items: Vec<String>| format!("[\n    {}\n  ]", items.join(",\n    "));
    let phy_json = list(phy.iter().map(|p| json_str(&file(p))).collect());
    let builds = format!("[\n    {{\"sub\": {}, \"mcf\": {}, \"chassis\": {}}}\n  ]", json_str(&file(sub)), json_str(&file(mcf)), json_str(CHASSIS));
    let todo = list(TODO.iter().map(|t| json_str(t)).collect());
    format!(
        "{{\n  \"standard\": 1,\n  \"kind\": \"machine\",\n  \"id\": {},\n  \"name\": {},\n  \"machine\": {},\n  \"source\": {{\n    \"game\": {},\n    \"boot\": {},\n    \"psr\": {},\n    \"members\": [{}]\n  }},\n  \"version\": \"1.0.0\",\n  \"author\": {},\n  \"permission\": \"own-disc\",\n  \"step\": {},\n  \"builds\": {builds},\n  \"phy\": {phy_json},\n  \"todo\": {todo}\n}}\n",
        json_str(ID),
        json_str(NAME),
        json_str(MACHINE),
        json_str(&origin.game),
        json_str(&origin.boot),
        json_str(&origin.psr),
        json_str(ARCHIVE),
        json_str(crate::AUTHOR),
        json_str(STEP),
    )
}

/// A built pack read back: each listed quad's `.phy.bpb` through openmx's BPB reader, the chassis model
/// through the quad's model reader, and a quad built off them at the origin
/// (`quad::Load::read_model`). Each quad's line.
pub fn check(dir: &Path) -> Result<Vec<String>, String> {
    let res = game_render::Res::folder(dir)?;
    let find = |leaf: &str| res.entries().iter().find(|e| leaf_of(&e.name).eq_ignore_ascii_case(leaf)).map(|e| e.name.clone()).ok_or(format!("no {leaf}"));
    let read = |m: &str| res.read(m).map_err(|e| format!("{m}: {e}"));
    let psg = find(&format!("{CHASSIS}.psg"))?;
    let model = game_sim::quad::Model::parse(&read(&psg)?).map_err(|e| format!("{psg}: {e}"))?;
    for c in COLS {
        let m = find(c)?;
        game_sim::track::Rays::parse(&read(&m)?).map_err(|e| format!("{m}: {e}"))?;
    }
    let curve = game_sim::camera::CurveTable::build();
    let mut out = Vec::new();
    for q in QUADS {
        let phy = find(&format!("{q}.phy.bpb"))?;
        let ini = game_sim::bpb::ini(&read(&phy)?).map_err(|e| format!("{phy}: {e}"))?;
        let (at, heading) = (ps2_float::Vf::new(0.0, 0.0, 0.0, 1.0), ps2_float::Vf::new(0.0, 0.0, 1.0, 1.0));
        game_sim::quad::Load::read_model(&ini, &model, at, heading, &curve).map_err(|e| format!("{phy}: {e}"))?;
        let weight = ini.section("Mass Properties").float("Weight").map_or("none".to_string(), |w| w.0.to_string());
        out.push(format!("{q}: built, Weight {weight}"));
    }
    Ok(out)
}
