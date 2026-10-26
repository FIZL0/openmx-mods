//! MX vs. ATV Unleashed's trick pack built off its PC paks against the one built off its disc (T12).
//!
//! `cargo test -p import-tricks --release -- --ignored`. The disc is found by boot ELF in
//! `$DISCX_ISO`, else `iso/` at the repo root; the PC install by `discx::pc_install` (`$DISCX_PC`, else
//! openmx's `context/mxvatv-pc/data`). Without either, the test says so on stderr and passes.

use import_tricks::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn iso_dir() -> PathBuf {
    std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../iso"))
}

fn sources() -> Option<(PathBuf, PathBuf)> {
    let image = discx::resolve_image("mxvatv", &iso_dir());
    let pc = discx::pc_install();
    match (image, pc) {
        (Ok(i), Some(p)) => Some((i, p)),
        (i, p) => {
            if let Err(e) = i {
                eprintln!("skipped: {e}");
            }
            if p.is_none() {
                eprintln!("skipped: no MX vs. ATV Unleashed PC install ($DISCX_PC, or openmx/context/mxvatv-pc/data)");
            }
            None
        }
    }
}

/// Every file under `dir`, by its path from `dir`.
fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(p.strip_prefix(dir).unwrap().to_string_lossy().into_owned(), std::fs::read(&p).unwrap());
            }
        }
    }
    out
}

/// Built off each source with the same reads recorded: every member the build reads off the disc
/// comes off the paks byte for byte, and the two packs are the same files but for what the paks hold
/// differently (notes/mxvatv.md, *The PC install*): `source.psr` names the paks, and the ATV bike's
/// NinjaNac clip, which the disc keeps under `Rider\Anim\` where the build does not look for it, is
/// in `Veh\Anim\` on the PC, so only that pack has it.
#[test]
#[ignore = "needs the MX vs. ATV Unleashed disc image and its PC install"]
fn the_pack_off_the_paks_is_the_discs() {
    let Some((image, data)) = sources() else { return };
    let pack = Pack::by_game("mxvatv").unwrap();
    let mxu = mxu_names_from(&iso_dir()).ok();
    let root = std::env::temp_dir().join(format!("import-tricks-pc-{}", std::process::id()));
    let mut reads: Vec<Vec<(String, Vec<u8>)>> = Vec::new();
    for (k, src) in [&image, &data].into_iter().enumerate() {
        let mut disc = discx::Disc::open_any(src).unwrap();
        let game = disc.game.unwrap();
        let origin = Origin { game: game.title.into(), boot: game.boot.into(), psr: disc.psr_name.clone() };
        let members: Vec<String> = disc.entries().iter().map(|e| e.name.clone()).collect();
        let mut got = Vec::new();
        let mut read = |name: &str| {
            let e = disc.find(name).cloned().ok_or_else(|| format!("{name}: not here"))?;
            let b = disc.read(&e)?;
            got.push((name.to_owned(), b.clone()));
            Ok(b)
        };
        build(pack, &origin, &members, &mut read, mxu.as_deref(), &root.join(k.to_string())).unwrap();
        reads.push(got);
    }
    let pc: BTreeMap<String, Vec<u8>> = reads[1].iter().map(|(n, b)| (discx::normalise(n), b.clone())).collect();
    for (name, bytes) in &reads[0] {
        assert_eq!(pc.get(&discx::normalise(name)), Some(bytes), "{name}: read off the disc, not the same off the paks");
    }
    assert!(reads[0].len() > 150, "{} reads", reads[0].len());

    let (a, b) = (tree(&root.join("0").join(pack.id)), tree(&root.join("1").join(pack.id)));
    let only_pc: Vec<&String> = b.keys().filter(|k| !a.contains_key(*k)).collect();
    assert_eq!(only_pc, ["Veh/Anim/ATV_bike_NinjaNac.mot"]);
    for (k, v) in &a {
        if k != "mod.json" {
            assert_eq!(b.get(k), Some(v), "{k}");
        }
    }
    let (ja, jb) = (String::from_utf8_lossy(&a["mod.json"]).into_owned(), String::from_utf8_lossy(&b["mod.json"]).into_owned());
    let want = ja
        .replace("\"psr\": \"MXVSATV.PSR\"", "\"psr\": \"data\\\\*.pak\"")
        .replace("      \"Veh\\\\Anim\\\\ATV_Bike_Varial.mot\",\n", "      \"Veh\\\\Anim\\\\ATV_Bike_Varial.mot\",\n      \"Veh\\\\Anim\\\\ATV_bike_NinjaNac.mot\",\n");
    assert_eq!(jb, want);
    std::fs::remove_dir_all(&root).unwrap();
}
