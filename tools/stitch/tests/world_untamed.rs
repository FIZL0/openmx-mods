//! C3: MX vs. ATV Untamed's giant map, `worlds/world_untamed.json`.
//!
//! The spec names every open map M4 imports (notes/mxvatv-later.md, *The giant map (C3)*): the 80×80-leaf
//! amateur, BT, national and supermoto maps (SM_HS 56×80), and none of the 896 ft stadiums (SX, FS,
//! MT_A). The ignored test derives that set off the disc and builds and checks the world.

use modcheck::json::Json;
use std::path::{Path, PathBuf};
use stitch::build::{Params, build};

fn spec() -> Json {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("worlds/world_untamed.json");
    Json::parse(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn tiles(j: &Json) -> Vec<String> {
    j.get("tiles").and_then(Json::arr).unwrap().iter().filter_map(Json::str).map(str::to_owned).collect()
}

#[test]
fn the_spec_names_every_open_map_once_and_no_stadium() {
    let j = spec();
    let mut got = tiles(&j);
    let sky = j.get("sky").and_then(Json::str).unwrap().to_owned();
    assert!(got.contains(&sky), "the sky {sky} is not one of the tiles");
    got.sort();
    let mut want: Vec<String> = [
        "am_cs", "am_gfuc", "am_jy", "bt_a", "bt_b", "nat_az", "nat_co", "nat_dvca", "nat_in", "nat_ma", "nat_md",
        "nat_nc", "nat_ny", "nat_or", "nat_pa", "nat_sdca", "nat_ut", "sm_as", "sm_cf", "sm_hs", "sm_mb", "sm_of",
        "sm_s1",
    ]
    .map(|m| format!("untamed_{m}"))
    .into();
    want.sort();
    assert_eq!(got, want);
    // Built at stitch's defaults: nothing raised, so no spec todo is owed (C10).
    for k in ["max_slope", "min_gap", "fade", "chunk", "todo"] {
        assert!(j.get(k).is_none(), "the spec sets {k}");
    }
}

// ---- against the user's disc (`cargo test --release -p stitch --test world_untamed -- --ignored`) ----

#[test]
#[ignore = "needs the MX vs. ATV Untamed disc (SLUS_217.01) in iso/ or $DISCX_ISO; about 2.5 minutes in release"]
fn the_world_of_every_open_map_builds_and_checks() {
    use import_mxvatv::untamed;
    let dir = std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../iso"));
    let (image, _) = discx::images(&dir)
        .unwrap()
        .into_iter()
        .find(|(_, b)| b.as_deref().is_some_and(|b| b.eq_ignore_ascii_case(untamed::BOOT)))
        .unwrap_or_else(|| panic!("no MX vs. ATV Untamed disc in {}", dir.display()));
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("stitch-world-untamed");
    let _ = std::fs::remove_dir_all(&root);
    let mods = root.join("tracks");
    let mut disc = discx::Disc::open(&image).unwrap();
    let master = untamed::master(&mut disc).unwrap();
    // M4's maps, and of them the open ones: terrain more than C2's 1280 ft a side (the stadiums are 896).
    let mut open = Vec::new();
    for (f, pick) in untamed::chosen(&disc, &master.1) {
        if pick.is_err() {
            continue;
        }
        let map = untamed::load_map(&mut disc, &f, &master).unwrap();
        let m = import_mxvatv::import(&map, &mods).unwrap();
        let t = stitch::Tile::load(&m.dir).unwrap();
        if f64::from(t.n.0.min(t.n.1)) * t.spacing > 1280.0 {
            open.push(m.id);
        }
    }
    open.sort();
    let j = spec();
    let names = tiles(&j);
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(sorted, open, "the spec's tiles are not the disc's open maps");

    let mut p = Params::new(j.get("id").and_then(Json::str).unwrap(), j.get("name").and_then(Json::str).unwrap());
    p.sky = j.get("sky").and_then(Json::str).map(str::to_owned);
    let dirs: Vec<PathBuf> = names.iter().map(|t| mods.join(t)).collect();
    let built = build(&dirs, &root.join("out"), &p).unwrap();
    let r = stitch::check::check(&built.dir, Some(&mods), Some(dir));
    assert!(r.ok(), "check failed:\n{}", r.fail.join("\n"));
    assert!(r.seam_points > 200_000, "{} seam points", r.seam_points);
}
