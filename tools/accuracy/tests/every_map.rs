//! Every imported MX vs. ATV Unleashed map against the game (M12): for each `out/mods/tracks/mxvatv_*`
//! with a RAM image of a Free Ride on it (`extracted/mxvatv/ram/<map>.bin`, `accuracy capture`):
//!
//! - the heights, surfaces, weights and (with MX vs. ATV's normal step) normals of the original's own
//!   terrain query, at 1,000 points in the map's own mode, equal to openmx's on the mod's `.trn`;
//! - the scene's `[SceneInfo]`, `[Fog]`, `[Environment]` and lights as the game holds them, equal to the
//!   mod's `.scn` as openmx reads its numbers;
//! - every file the game opened for the map in the mod.
//!
//! The images and mods are game data, not in git; a map without one is skipped, and the test says how
//! many it ran. notes/mxvatv.md has the table this proves.

use accuracy::heights::{self, Oracle};
use accuracy::loaded::{self, Have};
use accuracy::modfiles::TrackMod;
use accuracy::{ram, sceneram, source};
use game_sim::world::Ground;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn discx_boot() -> &'static str {
    source::MXVATV.boot
}

#[test]
fn every_imported_map_is_the_originals() {
    let tracks = root().join("out/mods/tracks");
    let Ok(dirs) = std::fs::read_dir(&tracks) else {
        eprintln!("skipped: no {}", tracks.display());
        return;
    };
    let mut dirs: Vec<PathBuf> = dirs.flatten().map(|e| e.path()).filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("mxvatv_"))).collect();
    dirs.sort();
    let mut ran = Vec::new();
    // Names the game asked for and fell back from are on no part of the disc; without the disc, every
    // name the mod lacks counts.
    let iso = std::env::var("DISCX_ISO").map(PathBuf::from).unwrap_or_else(|_| root().join("iso"));
    let psr = loaded::psr_names_in(&iso, discx_boot());
    for dir in dirs {
        let m = TrackMod::load(&dir).unwrap();
        let image = root().join("extracted/mxvatv/ram").join(format!("{}.bin", m.map.to_ascii_lowercase()));
        if !image.exists() {
            eprintln!("{}: skipped, no {}", m.id, image.display());
            continue;
        }
        let src = source::find(&m.boot).unwrap();
        let bytes = heights::load_ram(&image).unwrap();

        // The scene and the files first: they read the image as it is.
        let scn = std::fs::read_to_string(dir.join("mod.json")).unwrap();
        let json = modcheck::json::Json::parse(&scn).unwrap();
        let scn = json.get("lookups").and_then(|l| l.get("scene")).and_then(|s| s.str()).unwrap().to_owned();
        let text = String::from_utf8_lossy(&std::fs::read(dir.join(&scn)).unwrap()).into_owned();
        let fields = sceneram::compare(&bytes, &text).unwrap();
        assert!(!fields.iter().any(|f| f.differs()), "{}", sceneram::text(&m.id, &fields));
        let names = loaded::pool(&bytes, &m.map);
        assert!(!names.is_empty(), "{}: no name pool", m.id);
        let (paths, files) = loaded::mod_files(&dir);
        let mut rows = loaded::audit(&names, &paths, &files);
        if let Some(psr) = &psr {
            loaded::absent(&mut rows, psr);
        }
        let missing: Vec<_> = rows.into_iter().filter(|r| r.1 == Have::No).collect();
        assert!(missing.is_empty(), "{}: the game opened files the mod lacks: {missing:?}", m.id);

        let (key, scale) = m.root_key();
        let at = ram::find_terrain(&bytes, &src, &key, scale);
        assert_eq!(at.len(), 1, "{}: the terrain object: {at:x?}", m.id);
        let mut oracle = Oracle::new(bytes, src, at[0]);
        assert!(!oracle.packed_normals(), "{}: the packed normal table is on", m.id);
        let terrain = m.parse();
        let ground = Ground::new(&terrain, &m.layout);
        let field = ground.field();
        for (what, same) in heights::object_checks(&oracle, &src, &field) {
            // openmx's Ground sizes a one-level terrain (the arenas) at 16 samples a side where the game's
            // box is 256 (notes/openmx-asks.md); the queries do not clamp to the box, so the points agree.
            if what.contains("box") && terrain.levels == 1 {
                assert!(!same, "{}: the box now agrees; drop this exception", m.id);
                continue;
            }
            assert!(same, "{}: {what}", m.id);
        }
        let (lo, hi) = heights::inner_box(&oracle, &src);
        let points = heights::points(1000, 0x7acc_5eed, lo, hi, terrain.scale.0);
        let modes = vec![heights::modes(oracle.own_mode())[0]];
        let with_step = heights::compare(&mut oracle, &field, src.normals, &modes, &points, 0.0);
        assert_eq!(with_step.total_points(), 1000);
        assert_eq!(with_step.total_mismatches(), 0, "{}: {:?}", m.id, with_step.mismatches.first());
        ran.push(m.id);
    }
    eprintln!("{} maps against the game: {}", ran.len(), ran.join(" "));
}
