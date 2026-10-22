//! Every `.bpb` on Untamed's disc: it parses, every section's and key's hash recomputes from its name,
//! the encoder gives back the same bytes, and every scene's INI is read by openmx's `Scene::parse` with
//! the values the BPB holds. The scenes inside the map archives (`Env\<MAP>\<MAP>.res`, read through
//! survey's view because `Res::parse` refuses them) are the same checks again.
//!
//! `cargo test -p bpb --release -- --ignored`. The image is found by boot ELF in `$DISCX_ISO`, else
//! `iso/` at the repo root (a worktree has none: set `DISCX_ISO`).

use std::collections::BTreeMap;
use std::path::PathBuf;

use bpb::{Bpb, Value, parse, write};
use discx::{Disc, resolve_image};
use game_sim::track::Scene;

fn iso_dir() -> PathBuf {
    std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../iso"))
}

/// The first value of `key` in the first section titled `title`, as text.
fn value(bpb: &Bpb, title: &str, key: &str) -> Option<String> {
    let s = bpb.sections.iter().find(|s| s.title().eq_ignore_ascii_case(title.as_bytes()))?;
    let k = s.keys.iter().find(|k| k.name.eq_ignore_ascii_case(key.as_bytes()))?;
    Some(String::from_utf8(k.value.text()).unwrap())
}

/// One file through every check; the scene checks too when it is a scene.
///
/// Three values on the disc end in a CR their source text carried (`StoreText=Twin Air\r` in
/// `UI\Store.dat.bpb`); they are written as held and counted in `kinds` under 0. A line feed, or a CR
/// anywhere else, would break the INI's lines and fails.
fn check(name: &str, b: &[u8], kinds: &mut BTreeMap<u16, usize>) -> Result<Option<Scene>, String> {
    let bpb = parse(b)?;
    let bad = bpb.bad_hashes();
    if !bad.is_empty() {
        return Err(format!("{} hashes are not their names', first {}", bad.len(), bad[0]));
    }
    let again = write(&bpb)?;
    if again != b {
        let at = again.iter().zip(b).position(|(x, y)| x != y).unwrap_or(again.len().min(b.len()));
        return Err(format!("re-encoded, differs at {at:#x} ({} against {} bytes)", again.len(), b.len()));
    }
    for s in &bpb.sections {
        for k in &s.keys {
            *kinds.entry(k.value.kind()).or_default() += 1;
            if let Value::Text(t) | Value::Short(t) = &k.value {
                let body = t.strip_suffix(b"\r").unwrap_or(t);
                if body.len() < t.len() {
                    *kinds.entry(0).or_default() += 1;
                }
                if body.iter().any(|&c| c == b'\r' || c == b'\n') {
                    return Err(format!("[{}] has a line break in a value", String::from_utf8_lossy(&s.title())));
                }
            }
        }
    }
    let ini = bpb.to_ini();
    if !name.to_ascii_lowercase().ends_with(".scn.bpb") {
        return Ok(None);
    }
    let scene = Scene::parse(&ini).map_err(|e| format!("Scene::parse: {e}"))?;
    // What openmx read is what the BPB holds.
    let want = |t: &str, k: &str| value(&bpb, t, k).unwrap_or_default();
    if scene.terrain_file != want("Environment", "TerrainFile") {
        return Err(format!("terrain {:?}, the BPB has {:?}", scene.terrain_file, want("Environment", "TerrainFile")));
    }
    if scene.terrain.width.to_string() != want("Environment", "TerrainWidth") {
        return Err(format!("TerrainWidth {}", scene.terrain.width));
    }
    for (key, got) in [("TerrainWidthScale", scene.terrain.width_scale), ("MinDivisionSize", scene.terrain.min_division_size)] {
        let held = bpb
            .sections
            .iter()
            .filter(|s| s.name == b"Environment")
            .flat_map(|s| &s.keys)
            .find(|k| k.name == key.as_bytes())
            .map(|k| k.value.clone());
        if let Some(Value::Float(f)) = held
            && got.0.to_bits() != f.to_bits()
        {
            return Err(format!("{key}: openmx read {}, the BPB holds {f}", got.0));
        }
    }
    let count = |prefix: &[u8], file: &[u8]| {
        bpb.sections
            .iter()
            .filter(|s| s.name.eq_ignore_ascii_case(prefix) && s.index.is_some())
            .filter(|s| s.keys.iter().any(|k| k.name.eq_ignore_ascii_case(file) && !k.value.text().is_empty()))
            .count()
    };
    if scene.models.len() != count(b"Model", b"SLT") || scene.collision.len() != count(b"Object", b"COL") {
        return Err(format!("{} models and {} collision placements read", scene.models.len(), scene.collision.len()));
    }
    Ok(Some(scene))
}

#[test]
#[ignore = "needs the Untamed disc image"]
fn every_bpb_on_the_disc_decodes() {
    let image = resolve_image("untamed", &iso_dir())
        .unwrap_or_else(|e| panic!("no Untamed disc in {}: {e}", iso_dir().display()));
    let mut disc = Disc::open(&image).expect("the disc opens");
    let entries: Vec<_> = disc.matching("**.bpb").into_iter().cloned().collect();
    let mut kinds = BTreeMap::new();
    let (mut failed, mut scenes, mut placements) = (Vec::new(), 0, 0);
    let mut loose_scenes = BTreeMap::new();
    for e in &entries {
        let b = disc.read(e).unwrap();
        match check(&e.name, &b, &mut kinds) {
            Ok(Some(s)) => {
                scenes += 1;
                placements += s.models.len() + s.collision.len();
                loose_scenes.insert(e.name.rsplit('\\').next().unwrap().to_ascii_lowercase(), b);
            }
            Ok(None) => {}
            Err(why) => failed.push(format!("{}: {why}", e.name)),
        }
    }
    eprintln!(
        "{} .bpb, {scenes} scenes ({placements} placements), values by type {kinds:?} (0: ending in a CR)",
        entries.len()
    );

    // The scenes in the map archives.
    let archives: Vec<_> = disc.matching(r"res\env\*\*.res").into_iter().cloned().collect();
    let (mut in_res, mut same) = (0, 0);
    for e in &archives {
        let view = survey::res_view(disc.read(e).unwrap()).unwrap_or_else(|why| panic!("{}: {why}", e.name));
        for m in view.res.entries.iter().filter(|m| m.name.to_ascii_lowercase().ends_with(".scn.bpb")) {
            let b = view.res.read(&view.bytes, m).unwrap();
            let name = format!("{}:{}", e.name, m.name);
            in_res += 1;
            if let Err(why) = check(&name, &b, &mut BTreeMap::new()) {
                failed.push(format!("{name}: {why}"));
            }
            let short = m.name.rsplit('\\').next().unwrap().to_ascii_lowercase();
            same += usize::from(loose_scenes.get(&short) == Some(&b));
        }
    }
    eprintln!("{in_res} scenes in the map archives, {same} the same bytes as the loose one");

    assert!(failed.is_empty(), "{} failed:\n{}", failed.len(), failed.join("\n"));
    assert_eq!(entries.len(), 7697, "the survey counted 7,697 .bpb members");
    assert_eq!(scenes, 50);
    assert!(in_res >= 48, "{in_res} scenes in the archives");
}
