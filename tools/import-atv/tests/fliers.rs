//! The biplane and Kiowa's machine pack built off MX vs. ATV Unleashed's disc (openmx MD80d), proved
//! by modcheck byte for byte against the disc and read back through openmx's readers.
//!
//! `cargo test -p import-atv --release -- --ignored`. The image is found by boot ELF in `$DISCX_ISO`,
//! else `iso/` at the repo root (a worktree has none: set `DISCX_ISO`).

use import_atv::fliers::{self, Flier};
use std::path::PathBuf;

fn iso_dir() -> PathBuf {
    std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../iso"))
}

#[test]
#[ignore = "needs the MX vs. ATV Unleashed disc image"]
fn the_fliers_pack_is_the_discs_and_openmx_reads_it() {
    let image = discx::resolve_image(import_atv::GAME, &iso_dir()).unwrap();
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("import-fliers");
    let built = fliers::import(&image, &out).unwrap();
    // Four skins of each, skins 3 and 4 model two.
    assert_eq!(built.skins.len(), 8);
    for s in &built.skins {
        assert_eq!(s.model, if s.skin.ends_with('3') || s.skin.ends_with('4') { 2 } else { 1 }, "{}", s.skin);
    }
    // Nothing a model, material, .aud section or particle list names is missing: the two standing
    // entries are MD80b/c/e's and T7's.
    assert_eq!(built.todo.len(), 2, "{:#?}", built.todo);

    let mut discs = modcheck::Discs::new(Some(iso_dir()));
    let report = modcheck::check(&built.dir, &mut discs);
    assert!(report.fail.is_empty(), "{:#?}", report.fail);

    let c = fliers::check(&built.dir).unwrap();
    assert!(c.refused.is_empty(), "{:#?}", c.refused);
    // 16 collision files (every skin's F and M), the seven biplane loops, the Kiowa's engine, five
    // wreck clanks and the touchdown.
    assert_eq!(c.collisions, 16);
    assert_eq!(c.sounds, 14);
    assert_eq!(c.clips, 2);
    assert!(c.models >= 28 && c.textures >= 24 && c.emitters >= 20, "{c:?}");
    assert_eq!(c.skins.len(), 8);
    for (skin, _, nodes) in &c.skins {
        let lacks: Vec<&String> = nodes.iter().filter(|(_, got)| got.is_none()).map(|(w, _)| w).collect();
        assert!(lacks.is_empty(), "{skin} lacks {lacks:?}");
    }
    assert_eq!(fliers::nodes_wanted(Flier::Kiowa).len(), 10);
}
