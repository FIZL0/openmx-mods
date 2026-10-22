//! MX vs. ATV Untamed's maps as track mods: M2's importer with M10's BPB decoder in front.
//!
//! ```text
//! MX08PS2.PSR ── res\env\Nat_AZ\Nat_AZ.res ─────┐
//!             ── res\env\Nat_AZ\Nat_AZ_mat.res ─┼─ unpacked (directory padding cut) ──> untamed_nat_az/
//!             ── res\env\Nat_AZ\Nat_AZ_mst.res ─┘    Nat_AZ.scn.bpb kept, Nat_AZ.scn decoded beside it
//!             ── Env\Master.atyList.bpb   [Nat_AZ General], [Nat_AZ Free Ride]: name, start
//! ```
//!
//! Everything else is M2's: the same copy, the same read-back and the same todo. What differs is said
//! where it is done:
//!
//! - the map archives sit under `res\env\`, and `Res::parse` refuses every one of them for the zero
//!   bytes that pad its directory, so they are opened through `discx::ResFile::unpadded`;
//! - the scene is BPB, decoded to `<MAP>.scn` (§2a), and so are the materials and toys, which the
//!   import decodes only to look at them;
//! - one activity list, `Env\Master.atyList.bpb`, holds every map's `[<MAP> General]`;
//! - the loose PSR members the game opens for a map are followed through SLUS_217.01's loaders
//!   ([`pull_loose`]).

use crate::{FreeRide, Map, Opener, Origin, Pulled, base, free_ride_of, load, with_ext, NOT_FILES};
use discx::{Disc, normalise};
use game_render::{eco, ini};
use game_sim::track::Scene;

pub const BOOT: &str = "SLUS_217.01";
pub const GAME: &str = "MX vs. ATV Untamed";
pub const UNTAMED: Origin = Origin { game: GAME, boot: BOOT, prefix: "untamed_", archives: r"res\env" };
/// The one activity list, read by SLUS_217.01 as `%sMaster.atyList` (`0x002cb110` reads an
/// activity's keys).
pub const MASTER: &str = r"Env\Master.atyList.bpb";

/// Every map archive on the disc: each folder `res\env\<MAP>\` with a `<MAP>.res`, in PSR order.
pub fn map_folders(disc: &Disc) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for e in disc.matching(r"res\env\*\*.res") {
        let parts: Vec<&str> = e.name.split(['\\', '/']).collect();
        let (folder, file) = (parts[2], parts[3]);
        if file.eq_ignore_ascii_case(&format!("{folder}.res")) && !out.iter().any(|f| f.eq_ignore_ascii_case(folder)) {
            out.push(folder.to_string());
        }
    }
    out
}

/// `Env\Master.atyList.bpb` decoded: (member, INI text).
pub fn master(disc: &mut Disc) -> Result<(String, Vec<u8>), String> {
    let e = disc.find(MASTER).cloned().ok_or_else(|| format!("{MASTER}: not on the disc"))?;
    let b = bpb::parse(&disc.read(&e)?).map_err(|err| format!("{}: bpb: {err}", e.name))?;
    let bad = b.bad_hashes();
    if !bad.is_empty() {
        return Err(format!("{}: names whose hash does not recompute: {}", e.name, bad.join(", ")));
    }
    Ok((e.name, b.to_ini()))
}

/// The activity types the list gives a map (`free ride`, `national`, …), in its order; empty when it
/// has no `[<MAP> General]`.
pub fn activity_types(master: &[u8], folder: &str) -> Vec<String> {
    let sections = ini::parse(&ini::text(master));
    let unquote = |v: &str| v.trim().trim_matches('"').trim().to_string();
    let Some(general) = sections.iter().find(|s| s.name.eq_ignore_ascii_case(&format!("{folder} General"))) else {
        return Vec::new();
    };
    general
        .keys
        .iter()
        .filter(|(k, _)| k.eq_ignore_ascii_case("Activity"))
        .filter_map(|(_, a)| sections.iter().find(|s| s.name.eq_ignore_ascii_case(&unquote(a))))
        .filter_map(|s| s.get("Type").map(|t| unquote(t).to_ascii_lowercase()))
        .collect()
}

/// The maps M4 imports: those whose activities include a Free Ride or a national. Every map archive
/// on the disc is listed with `Err(why)` when it is left out.
pub fn chosen(disc: &Disc, master: &[u8]) -> Vec<(String, Result<(), String>)> {
    map_folders(disc)
        .into_iter()
        .map(|f| {
            let types = activity_types(master, &f);
            let pick = types.iter().any(|t| t == "free ride" || t == "national");
            let why = if types.is_empty() {
                format!("{MASTER} has no [{f} General]")
            } else {
                format!("its activities are {}: no free ride or national", types.join(", "))
            };
            (f, if pick { Ok(()) } else { Err(why) })
        })
        .collect()
}

/// Read one map's archives, its Free Ride from the master list, and the loose members its loaders
/// reach.
pub fn load_map(disc: &mut Disc, folder: &str, master: &(String, Vec<u8>)) -> Result<Map, String> {
    let mut map = load(disc, folder, &UNTAMED)?;
    map.ride = Some(free_ride(&master.0, &master.1, folder)?);
    map.pulled = pull_loose(disc, &map.archives)?;
    Ok(map)
}

/// The map's Free Ride in the master list: its `[<MAP> General]` `DisplayName` and the activity of
/// `Type = free ride` that it lists. The game reads an activity's start from `StartPosition` and
/// `StartDirection` only (`0x002cb110`); `DefaultStartPosition` is the scene's key (`0x0030e490`),
/// so one in an activity (FS_A's) is not a start, and the todo says so.
pub fn free_ride(member: &str, ini_text: &[u8], folder: &str) -> Result<FreeRide, String> {
    let mut fr = free_ride_of(member, ini_text, &format!("{folder} General"))?;
    let sections = ini::parse(&ini::text(ini_text));
    if let Some(s) = sections.iter().find(|s| {
        s.get("Type").is_some_and(|t| t.trim().trim_matches('"').eq_ignore_ascii_case("free ride"))
            && s.name.to_ascii_lowercase().starts_with(&format!("{} ", folder.to_ascii_lowercase()))
    }) && let Some(p) = s.get("DefaultStartPosition")
    {
        fr.todo.push(format!(
            "{member} [{}] DefaultStartPosition={}: the game reads an activity's start only from StartPosition (0x002cb110), so this key starts nothing",
            s.name,
            p.trim()
        ));
    }
    Ok(fr)
}

/// The loose PSR members SLUS_217.01 opens for a map because its archives lack them
/// (notes/mxvatv-later.md, *Loose PSR members*).
///
/// Untamed's resource manager is MX vs. ATV Unleashed's (M15): `0x0011e950` and `0x0011eff0` look a
/// name up among the loaded archives' members (`0x0011ec20`) and on a miss call `0x0011c7f0`, which
/// opens the PSR member at that path (`0x001bc760`). Its off-switch, flag `0x1000`, is set only by
/// `0x0011f250` from the `.res` loader `0x00222050` when its fifth argument is non-zero, and none of
/// its 43 callers passes one. The requests followed, each as the code builds the name:
///
/// - the placed models, `Geo\<slt>.psg` for each distinct `SLT=` (`0x00186bd0`, lowercased, with the
///   model folder that the level load `0x001c27d0` sets to `Geo\`);
/// - collision, `Col\<name>.col` for each distinct `COL=` the scene places (`0x00114f20`);
/// - the sky, `Skies\<CubeFile less its extension>.ps2.cub` (`0x0030c4d0`, `%s.ps2.cub` then
///   `0x0018f230`'s `Skies\`);
/// - the ecosystem's species, `Eco\<model>.psg` (the `Eco\` model folder, as M15 read it in MX vs.
///   ATV Unleashed's GAME_REL.BIN; Untamed's overlay was not opened);
/// - the map's material table `Env\<MAP>\<MAP>.mst` (`0x001c2d20`), each line a material read by the
///   BPB loader (`0x00168ff0` → `0x001425c0`): `<line>.psm.bpb`, then `<line>.psm` as text;
/// - those materials' textures: the `.psm`'s folder and each pass's `TextureMap_<n>` (`0x00163930`,
///   `%s%s`), or the material's own name when it has no `.psm`, tried as `.pst`, then `.atm`
///   (`0x0016cba0`).
///
/// A path an archive holds is the archive's. One only the PSR holds, loose, is pulled.
pub fn pull_loose(disc: &mut Disc, archives: &[(String, discx::ResFile)]) -> Result<Vec<Pulled>, String> {
    let mut o = Opener::new(disc, archives);
    let scenes: Vec<String> = o.held.keys().filter(|k| k.ends_with(".scn.bpb")).cloned().collect();
    let [scn] = scenes.as_slice() else {
        return Ok(Vec::new()); // the import refuses the map, and says why
    };
    let stem = base(scn).trim_end_matches(".scn.bpb").to_string();
    let text = bpb::parse(&o.held_bytes(scn)?).map_err(|e| format!("{scn}: bpb: {e}"))?.to_ini();
    let sections = ini::parse(&ini::text(&text));

    // The placed models, then the collision.
    if let Ok(scene) = Scene::parse(&text) {
        let mut seen = std::collections::BTreeSet::new();
        for p in &scene.models {
            if seen.insert(p.file.to_ascii_lowercase()) {
                o.open(&with_ext(&format!("Geo\\{}", p.file), ".psg"), &format!("SLT={} in {}", p.file, base(scn)))?;
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for p in &scene.collision {
            if seen.insert(p.file.to_ascii_lowercase()) {
                o.open(&with_ext(&format!("Col\\{}", p.file), ".col"), &format!("COL={} in {}", p.file, base(scn)))?;
            }
        }
    }

    // The sky.
    let env = sections.iter().find(|s| s.name.eq_ignore_ascii_case("Environment"));
    if let Some(cube) = env.and_then(|e| e.get("CubeFile")).map(str::trim).filter(|c| !c.is_empty()) {
        let stem = cube.rfind('.').map_or(cube, |d| &cube[..d]);
        o.open(&format!("Skies\\{stem}.ps2.cub"), &format!("CubeFile={cube} in {}", base(scn)))?;
    }

    // The ecosystem's species.
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
    let mst = format!("env/{}/{}.mst", stem.to_ascii_lowercase(), stem.to_ascii_lowercase());
    if o.held.contains_key(&mst) {
        let table = ini::text(&o.held_bytes(&mst)?);
        for line in table.lines() {
            let m = line.split(';').next().unwrap_or("").trim();
            if m.is_empty() || m.starts_with('#') {
                continue;
            }
            let why = format!("material {m} in {mst}");
            let psm = with_ext(m, ".psm");
            let mut found = None;
            if let Some(b) = o.open(&format!("{psm}.bpb"), &why)? {
                found = Some(bpb::parse(&b).map_err(|e| format!("{psm}.bpb: bpb: {e}"))?.to_ini());
            } else if let Some(b) = o.open(&psm, &why)? {
                found = Some(b);
            }
            let mut textures = Vec::new();
            match found {
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
                for ext in [".pst", ".atm"] {
                    if o.open(&with_ext(&t, ext), &format!("texture {t} of {why}"))?.is_some() {
                        break;
                    }
                }
            }
        }
    }
    Ok(o.pulled)
}
