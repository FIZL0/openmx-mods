//! The quad's machine pack built off MX vs. ATV Unleashed's disc, proved by modcheck byte for byte
//! against the disc and read back through openmx's readers.
//!
//! `cargo test -p import-atv --release -- --ignored`. The image is found by boot ELF in `$DISCX_ISO`,
//! else `iso/` at the repo root (a worktree has none: set `DISCX_ISO`).

use std::path::PathBuf;

fn iso_dir() -> PathBuf {
    std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../iso"))
}

#[test]
#[ignore = "needs the MX vs. ATV Unleashed disc image"]
fn the_pack_is_the_discs_and_openmx_reads_it() {
    let image = discx::resolve_image(import_atv::GAME, &iso_dir()).unwrap();
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("import-atv");
    let built = import_atv::import(&image, &out).unwrap();
    assert_eq!(built.builds.len(), 96);
    // Nothing the builds, models, motion sets, .aud or particle lists name is missing: the two
    // standing entries are MD37b/d's and T7's.
    assert_eq!(built.todo.len(), 2, "{:#?}", built.todo);

    let mut discs = modcheck::Discs::new(Some(iso_dir()));
    let report = modcheck::check(&built.dir, &mut discs);
    assert!(report.fail.is_empty(), "{:#?}", report.fail);

    let c = import_atv::check(&built.dir).unwrap();
    assert!(c.refused.is_empty(), "{:#?}", c.refused);
    assert_eq!(c.chassis.len(), 3);
    for (model, nodes) in &c.chassis {
        for want in ["SwingArm<", "WheelLeftFront<LeftHub", "WheelRightRear<RearAxle", "AarmUpperLeft<", "RightHub<AarmUpperRight"] {
            assert!(nodes.iter().any(|n| n.starts_with(want)), "{model}: no {want} in {nodes:?}");
        }
    }
    // The follow and race cameras (MD84), each read as openmx's shell reads a mod machine's.
    let cams: Vec<&str> = c.cameras.iter().map(|(f, _)| f.as_str()).collect();
    assert_eq!(cams, ["Cam/FC_ATVDefault.pb", "Cam/RC_ATVDefault.pb"], "{:?}", c.cameras);
    assert!(c.clips >= 80 && c.sounds >= 26 && c.textures >= 90, "{c:?}");
}

#[test]
#[ignore = "needs the MX vs. ATV Unleashed disc image"]
fn the_car_pack_is_the_discs_and_openmx_reads_it() {
    // openmx MD39c: the five cars' archives, whole, each car's .phy, chassis and collision found.
    let image = discx::resolve_image(import_atv::GAME, &iso_dir()).unwrap();
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("import-atv-cars");
    let built = import_atv::cars::import(&image, &out).unwrap();
    let names: Vec<&str> = built.cars.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["buggy", "monstertruck", "baja", "golfcart", "sandrail"]);

    let mut discs = modcheck::Discs::new(Some(iso_dir()));
    let report = modcheck::check(&built.dir, &mut discs);
    assert!(report.fail.is_empty(), "{:#?}", report.fail);

    let lines = import_atv::cars::check(&built.dir).unwrap();
    assert!(lines.iter().all(|l| l.ends_with("tires read")), "{lines:#?}");
}

#[test]
#[ignore = "needs the MX vs. ATV Untamed disc image"]
fn the_untamed_car_pack_is_the_discs_and_openmx_reads_it() {
    // openmx MD163: the baja truck's archive whole, its .phy.bpb, chassis and collision found, and
    // its car entry on Untamed's step; MD81e's golf cart and MD81f's sandrail beside it.
    let image = discx::resolve_image(import_atv::untamed_cars::GAME, &iso_dir()).unwrap();
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("import-atv-untamed-cars");
    let built = import_atv::untamed_cars::import(&image, &out).unwrap();
    let names: Vec<&str> = built.cars.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["baja", "golfcart", "sandrail"]);
    assert!(built.cars[0].phy.to_ascii_lowercase().ends_with("baja.phy.bpb"), "{:?}", built.cars[0]);
    assert!(built.cars[1].phy.to_ascii_lowercase().ends_with("golfcart.phy.bpb"), "{:?}", built.cars[1]);
    assert!(built.cars[2].phy.to_ascii_lowercase().ends_with("sandrail.phy.bpb"), "{:?}", built.cars[2]);
    let manifest = std::fs::read_to_string(built.dir.join("mod.json")).unwrap();
    assert!(manifest.contains("\"step\": \"untamed\""), "{manifest}");

    let mut discs = modcheck::Discs::new(Some(iso_dir()));
    let report = modcheck::check(&built.dir, &mut discs);
    assert!(report.fail.is_empty(), "{:#?}", report.fail);

    let lines = import_atv::untamed_cars::check(&built.dir).unwrap();
    assert_eq!(lines, ["baja: 43 nodes, tires read, Weight 3500", "golfcart: 47 nodes, tires read, Weight 1000", "sandrail: 43 nodes, tires read, Weight 2300"]);
}

#[test]
#[ignore = "needs the MX vs. ATV Untamed disc image"]
fn the_untamed_quad_pack_is_the_discs_and_openmx_reads_it() {
    // openmx MD144: the quad's archive whole, its two race quads' .phy.bpb listed, on Untamed's step,
    // and each builds a quad off the pack's chassis.
    let image = discx::resolve_image(import_atv::untamed_atv::GAME, &iso_dir()).unwrap();
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("import-atv-untamed-atv");
    let built = import_atv::untamed_atv::import(&image, &out).unwrap();
    assert_eq!(built.phy, ["Veh\\atv_Race250P.phy.bpb", "Veh\\atv_Pro250P.phy.bpb"]);
    let manifest = std::fs::read_to_string(built.dir.join("mod.json")).unwrap();
    assert!(manifest.contains("\"step\": \"untamed\"") && manifest.contains("\"machine\": \"atv\""), "{manifest}");
    // openmx MD144b: drawn. Untamed's ATV rider set and the clips, materials and textures its build
    // names, off the PSR's loose files; the compiled build reads as text with its parts.
    assert!(manifest.contains("\"rider\": {"), "{manifest}");
    for name in [import_atv::untamed_atv::RIDER_MOTIONS, import_atv::untamed_atv::RIDER_STUNTS, import_atv::untamed_atv::STUNT_TABLE] {
        assert!(built.dir.join(name.replace('\\', "/")).is_file(), "no {name}");
    }
    let sub = std::fs::read(built.dir.join("Veh/Anim").join(import_atv::untamed_atv::SUB)).unwrap();
    let text = game_render::ini::text_of(&sub);
    assert!(text.contains("[Objects]") && text.contains("[Material Changes]"), "{text}");
    assert!(built.dir.join("Veh/Tex/Striker_Plastics.psm.bpb").is_file());

    let mut discs = modcheck::Discs::new(Some(iso_dir()));
    let report = modcheck::check(&built.dir, &mut discs);
    assert!(report.fail.is_empty(), "{:#?}", report.fail);

    let lines = import_atv::untamed_atv::check(&built.dir).unwrap();
    assert_eq!(lines, ["atv_Race250P: built, Weight 420", "atv_Pro250P: built, Weight 420"]);
}
