//! C2: MX vs. ATV Unleashed's giant map, `worlds/world_mxvatv.json`.
//!
//! The spec names every open map of M8's table (notes/mxvatv.md, *The 56 maps*): the 80×80-leaf
//! nationals, supermoto and machine maps (Nat04 56×80), and none of the 896–1280 ft arenas (SX, FS,
//! MC06–08). The ignored test derives that set off the disc and builds and checks the world.

use modcheck::json::Json;
use std::path::{Path, PathBuf};
use stitch::build::{Params, build};

fn spec() -> Json {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("worlds/world_mxvatv.json");
    Json::parse(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn tiles(j: &Json) -> Vec<String> {
    j.get("tiles").and_then(Json::arr).unwrap().iter().filter_map(Json::str).map(str::to_owned).collect()
}

#[test]
fn the_spec_names_every_open_map_once_and_no_arena() {
    let j = spec();
    let mut got = tiles(&j);
    let sky = j.get("sky").and_then(Json::str).unwrap().to_owned();
    assert!(got.contains(&sky), "the sky {sky} is not one of the tiles");
    got.sort();
    let mut want: Vec<String> = (1..=17)
        .map(|n| format!("mxvatv_nat{n:02}"))
        .chain((1..=3).map(|n| format!("mxvatv_sm{n:02}")))
        .chain([1, 2, 3, 4, 5, 9, 10].map(|n| format!("mxvatv_mc{n:02}")))
        .collect();
    want.sort();
    assert_eq!(got, want);
    // Built at stitch's defaults: nothing raised, so no spec todo is owed (C10).
    for k in ["max_slope", "min_gap", "fade", "chunk", "todo"] {
        assert!(j.get(k).is_none(), "the spec sets {k}");
    }
}

// ---- against the user's disc (`cargo test --release -p stitch --test world_mxvatv -- --ignored`) ----

#[test]
#[ignore = "needs the MX vs. ATV Unleashed disc (SLUS_211.04) in iso/ or $DISCX_ISO; about 2 minutes in release"]
fn the_world_of_every_open_map_builds_and_checks() {
    let dir = std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../iso"));
    let (image, _) = discx::images(&dir)
        .unwrap()
        .into_iter()
        .find(|(_, b)| b.as_deref().is_some_and(|b| b.eq_ignore_ascii_case("SLUS_211.04")))
        .unwrap_or_else(|| panic!("no MX vs. ATV Unleashed disc in {}", dir.display()));
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("stitch-world-mxvatv");
    let _ = std::fs::remove_dir_all(&root);
    let mods = root.join("tracks");
    let mut disc = discx::Disc::open(&image).unwrap();
    // M8's open maps: every map whose terrain is more than an arena's 1280 ft a side.
    let mut open = Vec::new();
    for f in import_mxvatv::map_folders(&disc) {
        let map = import_mxvatv::load_map(&mut disc, &f).unwrap();
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
    p.wrap = j.get("wrap").and_then(Json::str).map(str::to_owned);
    let dirs: Vec<PathBuf> = names.iter().map(|t| mods.join(t)).collect();
    let built = build(&dirs, &root.join("out"), &p).unwrap();
    let r = stitch::check::check(&built.dir, Some(&mods), Some(dir));
    assert!(r.ok(), "check failed:\n{}", r.fail.join("\n"));
    assert!(r.seam_points > 250_000, "{} seam points", r.seam_points);
}
