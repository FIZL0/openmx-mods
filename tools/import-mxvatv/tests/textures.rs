//! T11: every material of an imported map, looked up as openmx looks it up. The mod's folder is opened
//! with `disc::Res::folder` (MD2's loader), and each model the scene places, its ecosystem grows and
//! its toys drive is read with `read_suffix` and its materials resolved with `world::Assets::resolve`,
//! as the shell's `Assets::load` does it; the ground's `terrainscreenspacemesh.psm` passes as
//! `splat::Splat::load` reads them, and the sky as `sky::Sky::load` finds it.
//!
//! A miss is said with its cause: the model or the `.psm` is not in the folder, the `.psm` names a
//! texture no file has, the file is there but its reader refuses it, or the suffix lookup took another
//! file than the one asked for (`gate02.pst` answered by `startgate02.pst`).
//!
//! Ignored by default: the same over every track mod in `out/mods/tracks/` and every world's tiles
//! (`OPENMX_MODS_OUT` for another `out/`), printed as a table, and failing on any miss that the mod's
//! own `todo` does not name.

use game_render::disc::Res;
use game_render::{eco, ini, psg, pst::Texture, scene as rscene, sky, toy, world};
use modcheck::json::Json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// One thing the port draws without its texture, and why.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Miss {
    /// What is drawn wrong: `ground`, `sky`, `prop`, `plant`, `billboard`, `toy`.
    what: &'static str,
    /// The file the lookup wanted.
    file: String,
    /// The cause, in one line.
    why: String,
}

/// The base name, lower-cased, of a member path.
fn base(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_ascii_lowercase()
}

/// `read_suffix`, and, where the member it answered with is not the file asked for but one whose
/// name merely ends the same (the port's lookup has no path boundary), what that is.
fn read(res: &Res, name: &str) -> Option<(Vec<u8>, Option<String>)> {
    let e = res.find_suffix(name)?;
    let other = (base(&e.name) != base(name)).then(|| {
        match res.entries().iter().find(|x| base(&x.name) == base(name)) {
            Some(x) => format!("the suffix lookup answers with {} where {} is the file", e.name, x.name),
            None => format!("the map has no {name}; the suffix lookup answers with {}", e.name),
        }
    });
    let bytes = res.read(&e.name).ok()?;
    Some((bytes, other))
}

/// Every miss in one mod folder, by openmx's own readers.
fn audit(folder: &Path) -> Result<Vec<Miss>, String> {
    let res = Res::folder(folder)?;
    let scn = res
        .entries()
        .iter()
        .find(|e| e.name.to_ascii_lowercase().ends_with(".scn"))
        .ok_or("no .scn")?
        .name
        .clone();
    let text = ini::text(&res.read(&scn)?);
    let scene = rscene::Scene::parse(&text);
    let stem = base(&scn);
    let stem = stem.strip_suffix(".scn").unwrap_or(&stem).to_string();
    let mut misses = BTreeSet::new();
    let mut miss = |what: &'static str, file: &str, why: String| {
        misses.insert(Miss { what, file: file.to_string(), why });
    };

    // The sky.
    let cube = ini::parse(&text)
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case("Environment"))
        .and_then(|s| s.get("CubeFile").map(|c| c.trim().to_ascii_lowercase()));
    match cube {
        None => miss("sky", "CubeFile", "the scene names no sky".into()),
        Some(cube) => {
            let tagged = format!("{}.ps2.cub", cube.strip_suffix(".cub").unwrap_or(&cube));
            match res.read_suffix(&tagged).or_else(|_| res.read_suffix(&cube)) {
                Err(_) => miss("sky", &cube, format!("no file ending in {tagged} or {cube}")),
                Ok(b) => {
                    if let Err(e) = sky::Sky::parse(&b) {
                        miss("sky", &cube, format!("sky::Sky::parse: {e}"));
                    }
                }
            }
        }
    }

    // The ground: the screen-space mesh's passes, each a texture, as `Splat::load` reads them.
    if let Some((psm, _)) = read(&res, "terrainscreenspacemesh.psm") {
        for s in ini::parse(&ini::text(&psm)) {
            let name = s.name.to_ascii_lowercase();
            let Some(n) = name.strip_prefix("shaderpass_") else { continue };
            if n.contains('_') {
                continue;
            }
            let Some(map) = s.get("TextureMap_1").map(|t| t.to_ascii_lowercase()) else { continue };
            let file = format!("{map}.pst");
            match read(&res, &file) {
                None => miss("ground", &file, format!("terrainscreenspacemesh.psm [{}] names it, and no file ends in it", s.name)),
                Some((b, other)) => {
                    if let Some(o) = other {
                        miss("ground", &file, o);
                    } else if let Err(e) = Texture::parse(&b) {
                        miss("ground", &file, format!("pst::Texture::parse: {e}"));
                    }
                }
            }
        }
    }

    // The models: the scene's, the ecosystem's species and billboards, the toys' cars.
    let mut models: BTreeMap<String, &'static str> = BTreeMap::new();
    let mut boards: BTreeSet<String> = BTreeSet::new();
    for p in &scene.models {
        models.entry(p.model.clone()).or_insert("prop");
    }
    if let Some(file) = scene.ecosystem_file.as_deref() {
        match res.read_suffix(file) {
            Err(_) => miss("plant", file, "the scene's EcosystemFile finds no file".into()),
            Ok(b) => match eco::Ecosystem::parse(&b, 5760.0) {
                Err(e) => miss("plant", file, format!("eco::Ecosystem::parse: {e}")),
                Ok(e) => {
                    for s in &e.species {
                        models.entry(s.model_name()).or_insert("plant");
                        boards.insert(s.billboard.split('.').next().unwrap_or("").to_ascii_lowercase());
                    }
                }
            },
        }
    }
    if let Ok(b) = res.read_suffix(&format!("{stem}.toy")) {
        for car in toy::cars(&ini::text(&b)) {
            models.entry(car.stem().to_string()).or_insert("toy");
        }
    }

    let mut assets = world::Assets::default();
    for (name, what) in &models {
        let file = format!("{name}.psg");
        let Some((bytes, other)) = read(&res, &file) else {
            misses.insert(Miss { what, file, why: "the scene names the model, and no file ends in it".into() });
            continue;
        };
        if let Some(o) = &other {
            misses.insert(Miss { what, file: file.clone(), why: o.clone() });
        }
        match psg::Model::parse(&bytes) {
            Err(e) => {
                misses.insert(Miss { what, file, why: format!("psg::Model::parse: {e}") });
            }
            Ok(m) => {
                for mat in &m.materials {
                    material(&res, &mut assets, what, mat, &file, &mut misses);
                }
            }
        }
    }
    for b in &boards {
        material(&res, &mut assets, "billboard", b, "the ecosystem's billboard", &mut misses);
    }
    Ok(misses.into_iter().collect())
}

/// One material down to its texture, by `Assets::resolve`; a miss says which step failed.
fn material(res: &Res, assets: &mut world::Assets, what: &'static str, mat: &str, by: &str, misses: &mut BTreeSet<Miss>) {
    if mat.trim().is_empty() {
        return;
    }
    let psm_name = format!("{mat}.psm");
    let Some((psm, other)) = read(res, &psm_name) else {
        misses.insert(Miss { what, file: psm_name, why: format!("{by} names the material, and no file ends in it") });
        return;
    };
    if let Some(o) = other {
        misses.insert(Miss { what, file: psm_name.clone(), why: o.clone() });
    }
    let m = rscene::Material::parse(&ini::text(&psm));
    if m.texture.is_none() && m.animation.is_none() {
        return; // drawn as the .psm says, with no texture
    }
    let files = world::texture_files(mat, &m);
    let reader = |f: &str| res.read_suffix(f).ok();
    let found = files.iter().find_map(|f| read(res, f).map(|r| (f.clone(), r)));
    if assets.resolve(&reader, mat).0.is_some() {
        if let Some((f, (_, Some(o)))) = found {
            misses.insert(Miss { what, file: f, why: format!("{psm_name}'s texture: {o}") });
        }
        return;
    }
    let why = match found {
        Some((f, (b, _))) => {
            let e = if f.to_ascii_lowercase().ends_with(".atm") { Texture::parse_animation(&b) } else { Texture::parse(&b) };
            match e {
                Err(e) => format!("{psm_name}'s texture is there and its reader refuses it: {e}"),
                Ok(_) => format!("{psm_name}'s texture is there and decodes, and resolve still found none"),
            }
        }
        None => format!("{psm_name} wants {}, and no file ends in it", files.join(" or ")),
    };
    misses.insert(Miss { what, file: files.first().cloned().unwrap_or_default(), why });
}

// ---- over out/ ---------------------------------------------------------------------------------

/// The repo's `out/`, or `OPENMX_MODS_OUT`.
fn out_dir() -> PathBuf {
    std::env::var_os("OPENMX_MODS_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../out"))
}

fn kind(folder: &Path) -> Option<(String, Json)> {
    let text = std::fs::read_to_string(folder.join("mod.json")).ok()?;
    let j = Json::parse(&text).ok()?;
    Some((j.get("kind")?.str()?.to_string(), j))
}

/// Every track mod in `out/mods/tracks/`, and every world's tiles not also there, by id.
fn every_track(out: &Path) -> BTreeMap<String, PathBuf> {
    let mut found = BTreeMap::new();
    let tracks = out.join("mods/tracks");
    let mut list: Vec<PathBuf> = std::fs::read_dir(&tracks).into_iter().flatten().flatten().map(|e| e.path()).collect();
    list.sort();
    for d in &list {
        if let Some((k, _)) = kind(d)
            && k == "track"
        {
            found.insert(d.file_name().unwrap().to_string_lossy().into_owned(), d.clone());
        }
    }
    for d in &list {
        let mut tiles: Vec<PathBuf> = std::fs::read_dir(d.join("tiles")).into_iter().flatten().flatten().map(|e| e.path()).collect();
        tiles.sort();
        for t in tiles {
            let id = t.file_name().unwrap().to_string_lossy().into_owned();
            if matches!(kind(&t), Some((k, _)) if k == "track") {
                found.entry(id).or_insert(t);
            }
        }
    }
    found
}

/// Whether the mod's todo names this miss: an entry that mentions the file.
fn in_todo(j: &Json, m: &Miss) -> bool {
    let want = base(&m.file);
    let stem = want.rsplit_once('.').map_or(want.as_str(), |(s, _)| s).to_string();
    match j.get("todo") {
        Some(Json::Arr(items)) => items.iter().filter_map(|i| i.str()).any(|t| {
            let t = t.to_ascii_lowercase();
            t.contains(&want) || t.contains(&format!("{stem}.psm")) || t.contains(&format!("{stem}.psg"))
        }),
        _ => false,
    }
}

#[test]
#[ignore = "reads every mod in out/ (./build.sh first)"]
fn every_material_of_every_imported_map_resolves_in_openmx() {
    let out = out_dir();
    let all = every_track(&out);
    assert!(!all.is_empty(), "no track mods in {}/mods/tracks (./build.sh)", out.display());
    let mut unsaid = Vec::new();
    let mut total = 0;
    for (id, folder) in &all {
        let (_, j) = kind(folder).expect("read above");
        let misses = match audit(folder) {
            Ok(m) => m,
            Err(e) => {
                unsaid.push(format!("{id}: openmx refuses the folder: {e}"));
                continue;
            }
        };
        for m in &misses {
            total += 1;
            let said = in_todo(&j, m);
            println!("{id}\t{}\t{}\t{}\t{}", m.what, m.file, if said { "todo" } else { "UNSAID" }, m.why);
            if !said {
                unsaid.push(format!("{id}: {} {}: {}", m.what, m.file, m.why));
            }
        }
    }
    println!("{} mods, {total} misses, {} not in their mod's todo", all.len(), unsaid.len());
    assert!(unsaid.is_empty(), "{} misses no todo names:\n{}", unsaid.len(), unsaid.join("\n"));
}

// ---- a folder made here ------------------------------------------------------------------------

/// A one-pixel `.pst` in the layout `pst::Texture::parse` reads: header, palette, one mip.
fn pst() -> Vec<u8> {
    let mut d = Vec::new();
    d.extend_from_slice(&0x0002_0002u32.to_le_bytes());
    d.extend_from_slice(&0x28u32.to_le_bytes());
    d.extend(std::iter::repeat_n(0x80u8, 1024));
    for v in [1u32, 1, 1, 1, 0] {
        d.extend_from_slice(&v.to_le_bytes());
    }
    d.push(0);
    d
}

fn psm(texture: &str) -> Vec<u8> {
    format!("[General]\r\nTotalShaderPasses = 1\r\n[ShaderPass_1]\r\nTextureMap_1 = {texture}\r\n").into_bytes()
}

#[test]
fn each_material_is_resolved_as_openmx_resolves_it_and_a_miss_says_why() {
    let dir = std::env::temp_dir().join(format!("import-mxvatv-textures-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let files: Vec<(&str, Vec<u8>)> = vec![
        ("Tex/good.psm", psm("good")),
        ("Tex/good.pst", pst()),
        // `thq.psm` is there, and `BaleTHQ.psm` comes first in path order and ends the same.
        ("Tex/BaleTHQ.psm", psm("good")),
        ("Tex/thq.psm", psm("thq")),
        ("Tex/thq.pst", pst()),
        // Only the binary form, which openmx does not read.
        ("Tex/white.psm.bpb", b"BPB".to_vec()),
        // A texture named another way than the file.
        ("Tex/spaced.psm", psm("Nat12 _spaced")),
        ("Tex/Nat12_spaced.pst", pst()),
        // A page the reader refuses.
        ("Tex/broken.psm", psm("broken")),
        ("Tex/broken.pst", vec![0; 16]),
    ];
    for (path, bytes) in &files {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, bytes).unwrap();
    }
    let res = Res::folder(&dir).unwrap();
    let mut assets = world::Assets::default();
    let mut misses = BTreeSet::new();
    for m in ["good", "thq", "white", "spaced", "broken"] {
        material(&res, &mut assets, "prop", m, "a.psg", &mut misses);
    }
    let said: Vec<(String, String)> = misses.iter().map(|m| (m.file.clone(), m.why.clone())).collect();
    let _ = std::fs::remove_dir_all(&dir);
    let has = |file: &str, why: &str| said.iter().any(|(f, w)| f == file && w.contains(why));
    assert!(!said.iter().any(|(f, _)| f.starts_with("good")), "{said:?}");
    assert!(has("thq.psm", "answers with Tex\\BaleTHQ.psm where Tex\\thq.psm is the file"), "{said:?}");
    assert!(has("white.psm", "no file ends in it"), "{said:?}");
    assert!(has("Nat12 _spaced.pst", "no file ends in it"), "{said:?}");
    assert!(has("broken.pst", "its reader refuses it"), "{said:?}");
    assert_eq!(said.len(), 4, "{said:?}");
}
