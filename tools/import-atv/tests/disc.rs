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

    // Each kind's own chase and replay cameras, and the engine sounds with their loops.
    for kind in ["Buggy", "Monster", "Baja", "GolfCart", "Sandrail"] {
        for side in ["FC", "RC"] {
            let cam = built.dir.join(format!("Cam/{side}_{kind}Default.pb"));
            assert!(cam.is_file(), "{}", cam.display());
        }
    }
    assert!(built.dir.join("Aud/carengine.aud").is_file());
    let vags = std::fs::read_dir(built.dir.join("Aud/Engine")).unwrap().count();
    assert!(vags > 0, "no engine loops");
    let json = std::fs::read_to_string(built.dir.join("mod.json")).unwrap();
    assert!(json.contains("\"engine\""), "{json}");
}
