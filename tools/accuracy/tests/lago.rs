//! The one map run against its source game (T7): MX vs. ATV Unleashed's Lago (`Nat05`), imported by
//! tools/import-mxvatv, against a RAM image of the game in a Free Ride on it.
//!
//! The image is game data and is not in git: `extracted/mxvatv/ram/nat05.bin`, made with
//! `accuracy capture` (notes/accuracy.md). The mod is `out/mods/tracks/mxvatv_nat05` (./build.sh). Where
//! either is missing the test says so and passes, as the disc tests elsewhere in this workspace do.

use accuracy::heights::{self, Oracle};
use accuracy::modfiles::TrackMod;
use accuracy::{ram, source};
use game_sim::world::Ground;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn lagos_heights_all_equal_the_originals() {
    let (image, dir) = (root().join("extracted/mxvatv/ram/nat05.bin"), root().join("out/mods/tracks/mxvatv_nat05"));
    if !image.exists() || !dir.join("mod.json").exists() {
        eprintln!("skipped: needs {} and {}", image.display(), dir.display());
        return;
    }
    let m = TrackMod::load(&dir).unwrap();
    let src = source::find(&m.boot).unwrap();
    assert_eq!(src, source::MXVATV);
    let bytes = heights::load_ram(&image).unwrap();
    let (key, scale) = m.root_key();
    let at = ram::find_terrain(&bytes, &src, &key, scale);
    assert_eq!(at.len(), 1, "the terrain object: {at:x?}");
    let mut oracle = Oracle::new(bytes, src, at[0]);
    assert!(!oracle.packed_normals());
    let terrain = m.parse();
    let ground = Ground::new(&terrain, &m.layout);
    let field = ground.field();
    for (what, same) in heights::object_checks(&oracle, &src, &field) {
        assert!(same, "{what}");
    }
    let (lo, hi) = heights::inner_box(&oracle, &src);
    let points = heights::points(1500, 0x7acc_5eed, lo, hi, terrain.scale.0);
    let modes = heights::modes(oracle.own_mode());
    assert_eq!(modes[0], heights::Mode { coarse: false, split_one_way: true }, "Lago's own setting");

    // openmx as it is: every height, surface and weight the original's; the normals not quite.
    let as_is = heights::compare(&mut oracle, &field, source::Normals::Raw, &modes, &points, 0.0);
    assert_eq!(as_is.total_points(), 6000);
    assert!(as_is.modes.iter().all(|(_, t)| t.cells == t.points && t.rides == t.points));
    for f in ["height", "ride height", "four surfaces", "weights", "ride surface"] {
        assert_eq!(as_is.differ(f), 0, "{f}: {:?}", as_is.mismatches.iter().find(|x| x.what == f));
    }
    assert!(as_is.differ("normal") > 0, "the cell reader's normals over 127 (notes/accuracy.md)");

    // With MX vs. ATV's normal step, everything.
    let with_step = heights::compare(&mut oracle, &field, src.normals, &modes, &points, 0.0);
    assert_eq!(with_step.total_mismatches(), 0, "{:?}", with_step.mismatches.first());
}
