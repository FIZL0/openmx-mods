//! `renderprobe`: a map's render-side files through openmx game-render's readers. `renderprobe help`.

use discx::{Disc, resolve_image};
use game_render::eco::{Ecosystem, Grove};
use game_render::scene::{Material, Scene};
use game_render::sky::Sky;
use game_render::splat::Splat;
use game_render::world::{Assets, texture_files};
use game_render::{Model, Res, Texture, ini, toy};
use renderprobe::{base, page_pst, render_res};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = r#"renderprobe: a map's render-side files through openmx game-render's readers, as markdown

  renderprobe DISC MAP [RES...] [--png DIR]   one map: MAP's .res and _mat.res (or the RES named)
  renderprobe DISC skies                      every loose Skies\*.cub through Sky::parse
  renderprobe DISC pst GLOB                   every loose texture a glob matches through Texture::parse
  renderprobe DISC psg GLOB                   every loose model a glob matches through Model::parse

DISC as for discx (mxu, mxvatv, untamed, or an image path; $DISCX_ISO overrides iso/). MAP is the
map's folder name (Nat01); RES are PSR member names. --png writes the terrain's texture pages and
the sky's faces as PNGs into DIR, to be looked at (never into the repository)."#;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("renderprobe: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let mut png = None;
    let mut rest = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--png" {
            png = Some(PathBuf::from(it.next().ok_or("--png wants a directory")?));
        } else {
            rest.push(a.as_str());
        }
    }
    match rest[..] {
        [] | ["help" | "-h" | "--help"] => {
            println!("{USAGE}");
            Ok(())
        }
        [disc, "skies"] => skies(&mut open(disc)?),
        [disc, "pst", glob] => loose_textures(&mut open(disc)?, glob),
        [disc, "psg", glob] => loose_models(&mut open(disc)?, glob),
        [disc, map, ref res @ ..] => probe(&mut open(disc)?, map, res, png.as_deref()),
        _ => Err(format!("unknown arguments\n\n{USAGE}")),
    }
}

fn open(disc: &str) -> Result<Disc, String> {
    let iso = std::env::var_os("DISCX_ISO").map_or_else(|| PathBuf::from("iso"), PathBuf::from);
    Disc::open(&resolve_image(disc, &iso)?)
}

/// The PSR's loose members by last path segment, lower-cased: where a name the map's archives do not
/// hold may be found instead.
fn loose_index(disc: &Disc) -> HashMap<String, Vec<String>> {
    let mut m: HashMap<String, Vec<String>> = HashMap::new();
    for e in disc.entries() {
        m.entry(base(&e.name)).or_default().push(e.name.clone());
    }
    m
}

fn read_loose(disc: &mut Disc, name: &str) -> Result<Vec<u8>, String> {
    let e = disc.find(name).ok_or_else(|| format!("no {name}"))?.clone();
    disc.read(&e)
}

fn ext(name: &str) -> String {
    let b = base(name);
    match b.find('.') {
        Some(i) => b[i + 1..].to_string(),
        None => String::new(),
    }
}

fn tally<I: IntoIterator<Item = String>>(it: I) -> String {
    let mut m: BTreeMap<String, usize> = BTreeMap::new();
    for k in it {
        *m.entry(k).or_default() += 1;
    }
    m.iter()
        .map(|(k, n)| format!("{k} ×{n}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn some<T: AsRef<str>>(v: &[T], n: usize) -> String {
    let mut s = v
        .iter()
        .take(n)
        .map(|x| format!("`{}`", x.as_ref()))
        .collect::<Vec<_>>()
        .join(", ");
    if v.len() > n {
        s.push_str(&format!(", … ({} in all)", v.len()));
    }
    s
}

fn probe(disc: &mut Disc, map: &str, named: &[&str], png: Option<&Path>) -> Result<(), String> {
    let wanted: Vec<String> = if named.is_empty() {
        let low = map.to_ascii_lowercase();
        let mut v: Vec<String> = disc
            .entries()
            .iter()
            .filter(|e| {
                let b = base(&e.name);
                b == format!("{low}.res") || b == format!("{low}_mat.res")
            })
            .map(|e| e.name.clone())
            .collect();
        v.sort_by_key(|n| n.to_ascii_lowercase().contains("_mat"));
        v
    } else {
        named.iter().map(|s| s.to_string()).collect()
    };
    if wanted.is_empty() {
        return Err(format!("no {map}.res on the disc"));
    }
    let mut rs: Vec<Res> = Vec::new();
    for name in &wanted {
        let bytes = read_loose(disc, name)?;
        let view = survey::res_view(bytes).map_err(|e| format!("{name}: {e}"))?;
        rs.push(render_res(view.bytes)?);
    }
    let names = &wanted;
    let loose = loose_index(disc);
    let read = |suffix: &str| rs.iter().find_map(|a| a.read_suffix(suffix).ok());

    println!("## {map}\n");
    for (n, a) in names.iter().zip(&rs) {
        println!(
            "- `{n}`: {} members: {}",
            a.entries().len(),
            tally(a.entries().iter().map(|e| ext(&e.name)))
        );
    }
    println!();

    // ---- textures in the archives
    let mut ok = Vec::new();
    let mut bad = Vec::new();
    for a in &rs {
        for e in a.entries() {
            let x = ext(&e.name);
            if x == "pst" || x == "atm" {
                let b = a.read(&e.name)?;
                let r = if x == "atm" {
                    Texture::parse_animation(&b)
                } else {
                    Texture::parse(&b)
                };
                match r {
                    Ok(t) => ok.push(format!("{}×{}", t.width, t.height)),
                    Err(why) => bad.push(format!("{}: {why}", e.name)),
                }
            }
        }
    }
    println!("### Textures (`.pst`, `.atm`) — `Texture::parse`\n");
    println!(
        "{} read ({}), {} refused{}\n",
        ok.len(),
        tally(ok),
        bad.len(),
        if bad.is_empty() {
            String::new()
        } else {
            format!(": {}", some(&bad, 6))
        }
    );

    // ---- materials
    let mut programs = Vec::new();
    let mut multipass = 0;
    let (mut tex_here, mut tex_loose, mut tex_none, mut untextured) =
        (0, Vec::new(), Vec::new(), 0);
    let mut nmat = 0;
    for a in &rs {
        for e in a.entries() {
            if ext(&e.name) != "psm" {
                continue;
            }
            nmat += 1;
            let text = ini::text(&a.read(&e.name)?);
            let m = Material::parse(&text);
            let passes = ini::parse(&text)
                .iter()
                .filter(|s| s.name.to_ascii_lowercase().starts_with("shaderpass"))
                .count();
            if passes > 1 {
                multipass += 1;
            }
            programs.push(m.microprogram.clone().unwrap_or_else(|| "(none)".into()));
            let stem = base(&e.name);
            let stem = stem.trim_end_matches(".psm");
            let files = texture_files(stem, &m);
            if files.is_empty() {
                untextured += 1;
            } else if files.iter().any(|f| read(f).is_some()) {
                tex_here += 1;
            } else if let Some(f) = files
                .iter()
                .find(|f| loose.contains_key(&f.to_ascii_lowercase()))
            {
                tex_loose.push(loose[&f.to_ascii_lowercase()][0].clone());
            } else {
                tex_none.push(format!("{} → {}", e.name, files[0]));
            }
        }
    }
    println!("### Materials (`.psm`) — `Material::parse`\n");
    println!(
        "{nmat} materials; {multipass} with more than one shader pass (only the first is read). \
         Texture: {tex_here} in the map's archives, {} only loose on the PSR{}, {} nowhere{}, {untextured} name none.\n",
        tex_loose.len(),
        if tex_loose.is_empty() {
            String::new()
        } else {
            format!(" ({})", some(&tex_loose, 4))
        },
        tex_none.len(),
        if tex_none.is_empty() {
            String::new()
        } else {
            format!(" ({})", some(&tex_none, 6))
        },
    );
    println!("Microprograms: {}\n", tally(programs));

    // ---- models in the archives
    let mut ok = 0;
    let mut bad = Vec::new();
    let mut versions = Vec::new();
    let (mut meshes, mut groups) = (0usize, Vec::new());
    for a in &rs {
        for e in a.entries() {
            if ext(&e.name) != "psg" {
                continue;
            }
            let b = a.read(&e.name)?;
            if b.len() >= 8 {
                versions.push(format!("v{}", u32::from_le_bytes([b[4], b[5], b[6], b[7]])));
            }
            match Model::parse(&b) {
                Ok(m) => {
                    ok += 1;
                    meshes += m.meshes.len();
                    groups.push(format!("{}", m.groups));
                }
                Err(why) => bad.push(format!("{}: {why}", e.name)),
            }
        }
    }
    println!("### Models (`.psg`) — `Model::parse`\n");
    println!(
        "In the archives: {ok} read, {} refused{}. Versions: {}. LOD groups: {}. {meshes} meshes.\n",
        bad.len(),
        if bad.is_empty() {
            String::new()
        } else {
            format!(" ({})", some(&bad, 6))
        },
        tally(versions),
        tally(groups),
    );

    // ---- the scene
    let scn_name = rs
        .iter()
        .flat_map(|a| a.entries())
        .find(|e| ext(&e.name) == "scn")
        .map(|e| e.name.clone());
    let Some(scn_name) = scn_name else {
        println!("No `.scn` in the archives.\n");
        return Ok(());
    };
    let scene_text = ini::text(&read(&scn_name).ok_or("the scene went away")?);
    let scene = Scene::parse(&scene_text);
    let sections = ini::parse(&scene_text);
    let keys = |name: &str| {
        sections
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name))
            .map(|s| {
                s.keys
                    .iter()
                    .map(|(k, _)| k.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "(no section)".into())
    };
    let section_kinds = tally(sections.iter().map(|s| {
        s.name
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .to_string()
    }));
    println!("### Scene `{scn_name}` — game-render `Scene::parse`\n");
    println!(
        "`{}`: start {:?} facing {:?}; terrain `{}`; fog {:?}, {}..{} ft (vis {}, haze {}) → {:?}; \
         {} lights ({} directional); {} placed models; ecosystem {:?} (3D {}, 2D {}).\n",
        scene.name,
        [scene.start.x, scene.start.y, scene.start.z],
        [scene.start_dir.x, scene.start_dir.y, scene.start_dir.z],
        scene.terrain,
        scene.fog_color,
        scene.fog_near,
        scene.fog_far,
        scene.fog_visibility,
        scene.fog_haziness,
        scene.fog_range(),
        scene.lights.len(),
        scene.lights.iter().filter(|l| l.directional).count(),
        scene.models.len(),
        scene.ecosystem_file,
        scene.eco_distance_3d,
        scene.eco_distance_2d,
    );
    println!("Sections: {section_kinds}\n");
    println!("`[Environment]` keys: {}\n", keys("Environment"));
    println!("`[Fog]` keys: {}\n", keys("Fog"));

    // ---- the placed models, as the world loads them
    let mut assets = Assets::default();
    let unique: BTreeSet<String> = scene.models.iter().map(|p| p.model.clone()).collect();
    let mut loaded = 0;
    let mut missing_loose = Vec::new();
    let mut missing_nowhere = Vec::new();
    let mut refused = Vec::new();
    for name in &unique {
        if assets.load_placed(&rs, name) {
            loaded += 1;
            continue;
        }
        let file = format!("{}.psg", name.to_ascii_lowercase());
        match loose.get(&file) {
            Some(paths) => {
                let b = read_loose(disc, &paths[0])?;
                match Model::parse(&b) {
                    Ok(_) => missing_loose.push(paths[0].clone()),
                    Err(why) => refused.push(format!("{}: {why}", paths[0])),
                }
            }
            None => missing_nowhere.push(file),
        }
    }
    for m in assets.missing() {
        if m.contains(": ") {
            refused.push(m.clone());
        }
    }
    let (mut mats, mut no_tex) = (0, Vec::new());
    for (name, l) in &assets.models {
        for (i, t) in l.textures.iter().enumerate() {
            mats += 1;
            if t.is_none() {
                no_tex.push(format!("{name}:{}", l.model.materials[i]));
            }
        }
    }
    println!("### Placed models — `Assets::load_placed` over the map's archives\n");
    println!(
        "{} distinct models placed; {loaded} load from the map's archives; {} are only loose on the PSR \
         (and parse there){}; {} are on no part of the disc{}; {} refused{}.\n",
        unique.len(),
        missing_loose.len(),
        if missing_loose.is_empty() {
            String::new()
        } else {
            format!(": {}", some(&missing_loose, 6))
        },
        missing_nowhere.len(),
        if missing_nowhere.is_empty() {
            String::new()
        } else {
            format!(": {}", some(&missing_nowhere, 6))
        },
        refused.len(),
        if refused.is_empty() {
            String::new()
        } else {
            format!(": {}", some(&refused, 4))
        },
    );
    println!(
        "Of the loaded models' {mats} mesh materials, {} get no texture from the archives{}.\n",
        no_tex.len(),
        if no_tex.is_empty() {
            String::new()
        } else {
            format!(" ({})", some(&no_tex, 6))
        },
    );

    // ---- the sky
    let cube = sections
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case("Environment"))
        .and_then(|s| s.get("CubeFile"))
        .map(|s| s.trim().to_string());
    println!("### Sky — `Sky::load`, `Sky::parse`\n");
    match &cube {
        None => println!("The scene names no `CubeFile`.\n"),
        Some(c) => {
            let found = Sky::load(&rs, &scene_text);
            let stem = c.to_ascii_lowercase();
            let stem = stem.trim_end_matches(".cub");
            let member = rs
                .iter()
                .flat_map(|a| a.entries())
                .find(|e| base(&e.name) == format!("{stem}.ps2.cub"))
                .map(|e| e.name.clone());
            let detail = match &member {
                Some(m) => {
                    let b = read(m).unwrap_or_default();
                    let packets = packets(&b);
                    match Sky::parse(&b) {
                        Ok(sky) => {
                            if let Some(dir) = png {
                                write_sky(dir, map, &sky)?;
                            }
                            format!("`{m}`, {} bytes, {packets} packets: read", b.len())
                        }
                        Err(why) => format!("`{m}`, {} bytes, {packets} packets: {why}", b.len()),
                    }
                }
                None => "not in the map's archives".to_string(),
            };
            println!(
                "`CubeFile={c}`: {detail}. `Sky::load`: {}.\n",
                if found.is_some() { "a sky" } else { "none" }
            );
        }
    }

    // ---- the terrain and its pages
    let trn = rs
        .iter()
        .flat_map(|a| a.entries())
        .find(|e| ext(&e.name).ends_with("trn"))
        .map(|e| e.name.clone());
    let mut extent = None;
    let mut terrain_bytes: Option<Vec<u8>> = None;
    println!(
        "### Terrain — game-render `Terrain::parse`, and its texture pages through `Texture::parse`\n"
    );
    if let Some(trn) = &trn {
        let b = read(trn).ok_or("the terrain went away")?;
        match game_render::Terrain::parse(&b) {
            Ok(t) => println!(
                "game-render's `Terrain::parse` (`first-draw`'s reader) reads `{trn}`: {} × {}.\n",
                t.cols, t.rows
            ),
            Err(why) => println!(
                "game-render's `Terrain::parse` (`first-draw`'s reader) on `{trn}`: {why}.\n"
            ),
        }
        let row = survey::terrain(trn, &b)?;
        extent = Some(row.across.max(row.along) as f32 * 16.0 * row.spacing);
        let l = survey::layout(&b)?;
        let view;
        let tb: &[u8] = if row.viewed.is_some() {
            view = survey::terrain_view(&b, &l)?;
            &view
        } else {
            &b
        };
        let t = game_sim::track::Terrain::parse(tb)?;
        terrain_bytes = Some(tb.to_vec());
        let palette = t.palette().ok_or("the terrain has no palette")?.to_vec();
        let alphas = tally(palette.chunks(4).map(|c| format!("{:#04x}", c[3])));
        let mut decoded = Vec::new();
        let mut bad = Vec::new();
        for (i, p) in row.textures.pages.iter().enumerate() {
            let page = &b[p.at as usize..(p.at + p.bytes) as usize];
            match Texture::parse(&page_pst(&palette, page)) {
                Ok(tex) => decoded.push(tex),
                Err(why) => bad.push(format!("page {i}: {why}")),
            }
        }
        let used: BTreeSet<u8> = row
            .textures
            .pages
            .iter()
            .flat_map(|p| {
                b[(p.at as usize + 24)..(p.at + p.bytes) as usize]
                    .iter()
                    .copied()
            })
            .collect();
        println!(
            "Palette alphas: {alphas}. Pages: {} of {} decode as `.pst` through the terrain's palette{}; \
             the pages use {} of the 256 palette entries.\n",
            decoded.len(),
            row.textures.pages.len(),
            if bad.is_empty() {
                String::new()
            } else {
                format!(" ({})", some(&bad, 4))
            },
            used.len()
        );
        if let Some(dir) = png {
            write_pages(dir, map, &decoded)?;
        }
    } else {
        println!("No terrain in the archives.\n");
    }

    // ---- ground paint
    let stem = scn_name
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or("")
        .split('.')
        .next()
        .unwrap_or("")
        .to_string();
    let e = extent.unwrap_or(0.0);
    let splat = Splat::load(&rs, &stem, (e, e));
    let terrain_psm: Vec<String> = rs
        .iter()
        .flat_map(|a| a.entries())
        .filter(|en| base(&en.name).starts_with("terrain") && ext(&en.name) == "psm")
        .map(|en| en.name.clone())
        .collect();
    let loose_terrain_psm: Vec<String> = disc
        .entries()
        .iter()
        .filter(|en| {
            let b = base(&en.name);
            b.starts_with("terrain")
                && b.ends_with(".psm")
                && en
                    .name
                    .to_ascii_lowercase()
                    .contains(&format!("\\{}\\", map.to_ascii_lowercase()))
        })
        .map(|en| en.name.clone())
        .collect();
    let shadow = format!("{}_shadow_01.pst", stem.to_ascii_lowercase());
    let detail = format!("{}_d.pst", stem.to_ascii_lowercase());
    let detail_read = match loose.get(&detail) {
        Some(p) => {
            let b = read_loose(disc, &p[0])?;
            match Texture::parse(&b) {
                Ok(t) => format!("`{}` reads, {}×{}", p[0], t.width, t.height),
                Err(why) => format!("`{}`: {why}", p[0]),
            }
        }
        None => format!("no `{detail}`"),
    };
    println!("### Ground paint — `Splat::load`\n");
    println!(
        "`Splat::load`: {}. Terrain materials in the archives: {}; loose beside the map: {}. \
         `{shadow}`: {}. {detail_read}.\n",
        if splat.is_some() {
            "passes"
        } else {
            "none (no `terrainscreenspacemesh.psm`)"
        },
        some(&terrain_psm, 8),
        some(&loose_terrain_psm, 8),
        if read(&shadow).is_some() {
            "in the archives"
        } else if loose.contains_key(&shadow) {
            "loose"
        } else {
            "on no part of the disc"
        },
    );

    // ---- the ecosystem
    println!("### Ecosystem — `Ecosystem::parse`, `Grove::load`\n");
    match (&scene.ecosystem_file, extent) {
        (Some(f), Some(extent)) => match read(f) {
            None => println!("`{f}` is not in the map's archives.\n"),
            Some(b) => match Ecosystem::parse(&b, extent) {
                Err(why) => println!("`{f}`: {why}.\n"),
                Ok(eco) => {
                    let far = eco
                        .plants
                        .iter()
                        .map(|p| p.pos.x.max(p.pos.z))
                        .fold(0.0f32, f32::max);
                    let mut assets = Assets::default();
                    // Grove::load stands plants on the terrain (openmx V49): the sim's scene layout and terrain.
                    let grove = match (&terrain_bytes, read(&scn_name)) {
                        (Some(tb), Some(sb)) => game_sim::track::Scene::parse(&sb).and_then(|sim| {
                            let t = game_sim::track::Terrain::parse(tb)?;
                            Grove::load(&rs, &scene, &sim.terrain, &t, &mut assets)
                        }),
                        _ => Err("no terrain or scene for Grove::load".to_string()),
                    };
                    let missing: Vec<String> = assets.missing().to_vec();
                    let mut eco_loose = Vec::new();
                    for m in &missing {
                        if let Some(p) = loose.get(&m.to_ascii_lowercase()) {
                            eco_loose.push(p[0].clone());
                        }
                    }
                    let mut boards = 0;
                    for s in &eco.species {
                        let mat = s
                            .billboard
                            .split('.')
                            .next()
                            .unwrap_or("")
                            .to_ascii_lowercase();
                        if read(&format!("{mat}.psm")).is_some() {
                            boards += 1;
                        }
                    }
                    println!(
                        "`{f}` ({} bytes): version {}, {} species ({}), {} plants, farthest at {far:.0} of {extent:.0} ft, \
                         probability map {:?}, shrubbery {:?}. `Grove::load`: {}; species models missing from the \
                         archives: {}{}; billboard materials in the archives: {boards} of {}.\n",
                        b.len(),
                        eco.version,
                        eco.species.len(),
                        some(
                            &eco.species
                                .iter()
                                .map(|s| s.model.clone())
                                .collect::<Vec<_>>(),
                            4
                        ),
                        eco.plants.len(),
                        eco.probability,
                        eco.shrubbery,
                        match &grove {
                            Ok(Some(_)) => "a grove".to_string(),
                            Ok(None) => "none".to_string(),
                            Err(e) => e.clone(),
                        },
                        if missing.is_empty() {
                            "0".to_string()
                        } else {
                            format!("{} ({})", missing.len(), some(&missing, 6))
                        },
                        if eco_loose.is_empty() {
                            String::new()
                        } else {
                            format!(" (loose on the PSR: {})", some(&eco_loose, 4))
                        },
                        eco.species.len(),
                    );
                }
            },
        },
        (None, _) => println!("The scene names no `EcosystemFile`.\n"),
        (Some(f), None) => println!("`{f}`: no terrain to size it by.\n"),
    }

    // ---- the toys
    println!("### Toys (`.toy`) — `toy::cars`\n");
    let toy_name = rs
        .iter()
        .flat_map(|a| a.entries())
        .find(|e| ext(&e.name) == "toy")
        .map(|e| e.name.clone());
    match toy_name {
        None => println!("No `.toy` in the archives.\n"),
        Some(t) => {
            let text = ini::text(&read(&t).unwrap_or_default());
            let secs = ini::parse(&text);
            let kinds = tally(secs.iter().map(|s| {
                s.name
                    .trim_end_matches(|c: char| c.is_ascii_digit())
                    .to_string()
            }));
            let cars = toy::cars(&text);
            let mut where_ = Vec::new();
            for c in &cars {
                let model = format!("{}.psg", c.stem().to_ascii_lowercase());
                let mot = base(&c.mot);
                let m = if read(&model).is_some() {
                    "here"
                } else if loose.contains_key(&model) {
                    "loose"
                } else {
                    "nowhere"
                };
                let k = if read(&mot).is_some() {
                    "here"
                } else if loose.contains_key(&mot) {
                    "loose"
                } else {
                    "nowhere"
                };
                where_.push(format!(
                    "model {m}, clip {k}{}",
                    if c.has_physics { ", HasPhysics" } else { "" }
                ));
            }
            println!(
                "`{t}`: sections {kinds}. {} cars{}.\n",
                cars.len(),
                if where_.is_empty() {
                    String::new()
                } else {
                    format!(": {}", tally(where_))
                },
            );
        }
    }

    // ---- what game-render has no reader for
    let known = [
        "pst", "atm", "psm", "psg", "scn", "ps2.cub", "esb", "toy", "ps2.trn", "col",
    ];
    let other: Vec<String> = rs
        .iter()
        .flat_map(|a| a.entries())
        .map(|e| ext(&e.name))
        .filter(|x| !known.contains(&x.as_str()))
        .collect();
    println!("### No game-render reader\n");
    println!(
        "In the archives: {}.\n",
        if other.is_empty() {
            "nothing".into()
        } else {
            tally(other)
        }
    );
    Ok(())
}

/// The `00 00 00 00 09 02 00 00` packet heads `Sky::parse` cuts a `.cub` at.
fn packets(d: &[u8]) -> usize {
    (0..d.len().saturating_sub(8))
        .filter(|&i| d[i..i + 8] == [0, 0, 0, 0, 9, 2, 0, 0])
        .count()
}

fn write_rgba(path: &Path, t: &Texture) -> Result<(), String> {
    let rgb: Vec<u8> = t.rgba.chunks(4).flat_map(|p| [p[0], p[1], p[2]]).collect();
    game_render::png::write_rgb(path, t.width, t.height, &rgb)
}

fn write_sky(dir: &Path, map: &str, sky: &Sky) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    for (i, f) in sky.faces.iter().enumerate() {
        write_rgba(&dir.join(format!("{map}-sky-{i}.png")), f)?;
    }
    Ok(())
}

/// The pages in a mosaic, eight across, in table order.
fn write_pages(dir: &Path, map: &str, pages: &[Texture]) -> Result<(), String> {
    if pages.is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let (w, h) = (pages[0].width, pages[0].height);
    let across = 8.min(pages.len());
    let down = pages.len().div_ceil(across);
    let mut m = Texture {
        width: w * across,
        height: h * down,
        rgba: vec![0; w * across * h * down * 4],
    };
    for (i, p) in pages.iter().enumerate() {
        let (ox, oy) = ((i % across) * w, (i / across) * h);
        for y in 0..h.min(p.height) {
            let src = &p.rgba[y * p.width * 4..(y * p.width + w.min(p.width)) * 4];
            let at = ((oy + y) * m.width + ox) * 4;
            m.rgba[at..at + src.len()].copy_from_slice(src);
        }
    }
    write_rgba(&dir.join(format!("{map}-pages.png")), &m)
}

fn skies(disc: &mut Disc) -> Result<(), String> {
    let cubes: Vec<_> = disc.matching(r"Skies\*.cub").into_iter().cloned().collect();
    let mut bad = Vec::new();
    let mut packet_counts = Vec::new();
    for e in &cubes {
        let b = disc.read(e)?;
        packet_counts.push(format!("{}", packets(&b)));
        if let Err(why) = Sky::parse(&b) {
            bad.push(format!("{}: {why}", e.name));
        }
    }
    println!(
        "{} skies under `Skies\\`; {} read by `Sky::parse`, {} refused{}. Packet counts: {}.",
        cubes.len(),
        cubes.len() - bad.len(),
        bad.len(),
        if bad.is_empty() {
            String::new()
        } else {
            format!(": {}", some(&bad, 8))
        },
        tally(packet_counts)
    );
    Ok(())
}

fn loose_textures(disc: &mut Disc, glob: &str) -> Result<(), String> {
    let es: Vec<_> = disc.matching(glob).into_iter().cloned().collect();
    let mut ok = Vec::new();
    let mut bad = Vec::new();
    for e in &es {
        let b = disc.read(e)?;
        let r = if ext(&e.name) == "atm" {
            Texture::parse_animation(&b)
        } else {
            Texture::parse(&b)
        };
        match r {
            Ok(t) => ok.push(format!("{}×{}", t.width, t.height)),
            Err(why) => bad.push(format!("{}: {why}", e.name)),
        }
    }
    let heads = tally(es.iter().filter_map(|e| {
        let b = disc.read(e).ok()?;
        (b.len() >= 8).then(|| {
            format!(
                "{:#010x} {:#x}",
                u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
                u32::from_le_bytes([b[4], b[5], b[6], b[7]])
            )
        })
    }));
    println!(
        "`{glob}`: {} files; {} read ({}), {} refused{}. First two words: {heads}.",
        es.len(),
        ok.len(),
        tally(ok),
        bad.len(),
        if bad.is_empty() {
            String::new()
        } else {
            format!(": {}", some(&bad, 8))
        },
    );
    Ok(())
}

fn loose_models(disc: &mut Disc, glob: &str) -> Result<(), String> {
    let es: Vec<_> = disc.matching(glob).into_iter().cloned().collect();
    let (mut ok, mut bad, mut why) = (0, Vec::new(), Vec::new());
    for e in &es {
        let b = disc.read(e)?;
        match Model::parse(&b) {
            Ok(_) => ok += 1,
            Err(w) => {
                // The reason without its offsets, to count kinds of refusal.
                why.push(w.split(" at ").next().unwrap_or(&w).to_string());
                bad.push(format!("{}: {w}", e.name));
            }
        }
    }
    println!(
        "`{glob}`: {} files; {ok} read, {} refused{}. Why: {}.",
        es.len(),
        bad.len(),
        if bad.is_empty() {
            String::new()
        } else {
            format!(": {}", some(&bad, 8))
        },
        tally(why),
    );
    Ok(())
}
