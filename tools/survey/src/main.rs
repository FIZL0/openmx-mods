//! `survey`: every map on a disc through openmx's readers, as markdown tables. `survey help` for usage.

use discx::{Disc, glob_match, resolve_image};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use survey::{ColRow, MapRow, col, map, map_res, md_row, table, wanted};

const USAGE: &str = r#"survey: every map on a disc, through openmx's readers, one markdown table row per map

  survey DISC              the map table, every refused .col, and every loose .col by folder
  survey res FILE.res      one map from a .res on disk

DISC is an image path, or a game (mxu, mxvatv, untamed, or its boot ELF) looked for in iso/
($DISCX_ISO overrides), as for discx. Maps are the .res under Tracks\ (mxu), Env\<MAP>\ (mxvatv) or
res\env\<MAP>\ (untamed) that hold a terrain, then any loose .trn not already covered."#;

/// Where each game keeps its map archives.
const MAP_RES: &[(&str, &str)] = &[
    ("mxu", r"Tracks\*.res"),
    ("mxvatv", r"Env\*\*.res"),
    ("untamed", r"res\env\*\*.res"),
];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("survey: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        [] | ["help" | "-h" | "--help"] => {
            println!("{USAGE}");
            Ok(())
        }
        ["res", file] => {
            let bytes = std::fs::read(file).map_err(|e| format!("{file}: {e}"))?;
            let name = stem(file);
            let row = map_res(&name, file, bytes, &mut |_| false)?;
            print!("{}", table(std::slice::from_ref(&row)));
            print_refused(std::slice::from_ref(&row));
            print_notes(std::slice::from_ref(&row));
            Ok(())
        }
        [disc] => survey_disc(disc),
        _ => Err(format!("unknown arguments\n\n{USAGE}")),
    }
}

fn stem(name: &str) -> String {
    let b = name.rsplit(['\\', '/']).next().unwrap_or(name);
    b.split('.').next().unwrap_or("").to_owned()
}

fn folder(name: &str) -> &str {
    name.rfind('\\').map_or("", |k| &name[..k])
}

fn survey_disc(arg: &str) -> Result<(), String> {
    let iso = std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("iso"));
    let path = resolve_image(arg, Path::new(&iso))?;
    let mut disc = Disc::open(&path)?;
    let game = disc.game.ok_or("survey knows the three MX discs only")?;
    let entries = disc.entries().to_vec();

    // Shared shapes: the disc's own top-level Col\ folder, by file name.
    let loose_cols: BTreeSet<String> = entries
        .iter()
        .filter(|e| glob_match(r"Col\*.col", &e.name))
        .map(|e| stem_ext(&e.name))
        .collect();
    let mut has_loose = |name: &str| loose_cols.contains(&name.to_ascii_lowercase());

    let pattern = MAP_RES
        .iter()
        .find(|(k, _)| *k == game.key)
        .map(|(_, p)| *p)
        .unwrap_or("");
    let mut rows: Vec<MapRow> = Vec::new();
    let mut not_maps = Vec::new();
    let (mut seen, mut twice) = (BTreeSet::new(), Vec::new());
    for e in entries.iter().filter(|e| glob_match(pattern, &e.name)) {
        let lower = e.name.to_ascii_lowercase();
        // MX.PSR lists a few archives twice under one name; the game finds the first.
        if !seen.insert(lower.clone()) {
            twice.push(e.name.clone());
            continue;
        }
        if ["_mat.res", "_mst.res", "_2.res"]
            .iter()
            .any(|s| lower.ends_with(s))
        {
            continue;
        }
        eprintln!("survey: {}", e.name);
        let bytes = disc.read(e)?;
        match map_res(&stem(&e.name), &e.name, bytes, &mut has_loose) {
            Ok(row) if row.terrain.as_ref().is_err_and(|w| w == "no terrain") => {
                not_maps.push(e.name.clone())
            }
            Ok(row) => rows.push(row),
            Err(why) => rows.push(MapRow {
                map: stem(&e.name),
                source: e.name.clone(),
                source_note: Some(why.clone()),
                res_padding: 0,
                terrain: Err(format!("the .res is refused: {why}")),
                scene: survey::SceneRow::None,
                cols: Vec::new(),
                extra: Vec::new(),
            }),
        }
    }

    // Loose terrains no map archive covered: surveyed from the files beside them.
    let covered: BTreeSet<String> = rows
        .iter()
        .filter_map(|r| {
            r.terrain
                .as_ref()
                .ok()
                .map(|t| stem(&t.file).to_ascii_lowercase())
        })
        .collect();
    let mut loose_maps = Vec::new();
    for e in entries
        .iter()
        .filter(|e| e.name.to_ascii_lowercase().ends_with(".trn"))
    {
        let s = stem(&e.name);
        if covered.contains(&s.to_ascii_lowercase()) {
            loose_maps.push(e.name.clone());
            continue;
        }
        let dir = folder(&e.name).to_owned();
        eprintln!("survey: loose {}", e.name);
        let mut files = Vec::new();
        for f in entries
            .iter()
            .filter(|f| folder(&f.name) == dir && wanted(&f.name))
        {
            files.push((f.name.clone(), disc.read(f)?));
        }
        let mut row = map(&s, &e.name, &files, &mut has_loose);
        row.source_note = Some("loose on the PSR, no map `.res` holds it".to_owned());
        rows.push(row);
    }
    rows.sort_by_key(|r| r.map.to_ascii_lowercase());

    // Every .col on the PSR itself, by folder.
    let mut by_folder: BTreeMap<String, Vec<ColRow>> = BTreeMap::new();
    for e in entries
        .iter()
        .filter(|e| e.name.to_ascii_lowercase().ends_with(".col"))
    {
        let b = disc.read(e)?;
        by_folder
            .entry(folder(&e.name).to_owned())
            .or_default()
            .push(col(&e.name, &b));
    }

    println!(
        "<!-- survey {}: {} ({}) -->\n",
        game.key, game.title, game.boot
    );
    print!("{}", table(&rows));
    let viewed = rows
        .iter()
        .filter(|r| r.terrain.as_ref().is_ok_and(|t| t.viewed.is_some()))
        .count();
    println!(
        "\n{} maps. † {viewed} terrains are refused by `Terrain::parse` as they are (openmx MD1); the \
         survey read their texture table itself and passed the reader a view without it, the blocks \
         moved up against the tree.",
        rows.len()
    );
    let in_maps: usize = rows.iter().map(|r| r.cols.len()).sum();
    let refused_in_maps: usize = rows.iter().map(|r| r.refused().count()).sum();
    println!(
        "`.col` in the map archives: {in_maps}, of which {refused_in_maps} refused by `Collision::parse`."
    );
    if !twice.is_empty() {
        println!(
            "Listed twice in the PSR, surveyed once (the first entry): {}.",
            twice.join(", ")
        );
    }
    if !not_maps.is_empty() {
        println!(
            "Archives matched but holding no terrain: {}.",
            not_maps.join(", ")
        );
    }
    if !loose_maps.is_empty() {
        println!(
            "{} loose `.trn` on the PSR are the same maps as an archive's and are not surveyed again.",
            loose_maps.len()
        );
    }
    print_refused(&rows);
    print_notes(&rows);

    println!("\n#### Every `.col` loose on the PSR\n");
    println!(
        "{}",
        md_row(&[
            "folder".into(),
            "`.col`".into(),
            "read".into(),
            "refused".into()
        ])
    );
    println!("|---|---|---|---|");
    for (f, cols) in &by_folder {
        let refused: Vec<String> = cols
            .iter()
            .filter(|c| c.read.is_err())
            .map(|c| stem_ext(&c.file))
            .collect();
        println!(
            "{}",
            md_row(&[
                if f.is_empty() {
                    "(root)".into()
                } else {
                    f.clone()
                },
                cols.len().to_string(),
                (cols.len() - refused.len()).to_string(),
                if refused.is_empty() {
                    "0".into()
                } else {
                    refused.join(" ")
                },
            ])
        );
    }
    let refused: Vec<&ColRow> = by_folder
        .values()
        .flatten()
        .filter(|c| c.read.is_err())
        .collect();
    if !refused.is_empty() {
        println!(
            "\n{}",
            md_row(&["loose `.col` refused".into(), "why".into()])
        );
        println!("|---|---|");
        for c in refused {
            println!("{}", md_row(&[c.file.clone(), c.read.clone().unwrap_err()]));
        }
    }
    Ok(())
}

fn stem_ext(name: &str) -> String {
    name.rsplit(['\\', '/'])
        .next()
        .unwrap_or(name)
        .to_ascii_lowercase()
}

fn print_refused(rows: &[MapRow]) {
    let refused: Vec<(&str, &str, &str)> = rows
        .iter()
        .flat_map(|r| r.refused().map(|(f, w)| (r.map.as_str(), f, w)))
        .collect();
    if refused.is_empty() {
        return;
    }
    println!(
        "\n{}",
        md_row(&["map".into(), "`.col` refused".into(), "why".into()])
    );
    println!("|---|---|---|");
    for (m, f, w) in refused {
        println!("{}", md_row(&[m.into(), f.into(), w.into()]));
    }
}

fn print_notes(rows: &[MapRow]) {
    // One line per distinct message, with the maps it is true of: most are the same on every map.
    let mut notes: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for r in rows {
        if let Some(n) = &r.source_note {
            notes.entry(n.clone()).or_default().push(&r.map);
        }
        if let Ok(t) = &r.terrain
            && let Some(why) = &t.viewed
        {
            notes
                .entry(format!("`Terrain::parse` on the file: {why}"))
                .or_default()
                .push(&r.map);
        }
        if !r.extra.is_empty() {
            notes
                .entry(format!("also holds {}, not surveyed", r.extra.join(", ")))
                .or_default()
                .push(&r.map);
        }
    }
    if !notes.is_empty() {
        println!();
        for (n, maps) in notes {
            println!("- {}: {n}", maps.join(", "));
        }
    }
    let padded: Vec<usize> = rows
        .iter()
        .map(|r| r.res_padding)
        .filter(|&p| p > 0)
        .collect();
    if let (Some(lo), Some(hi)) = (padded.iter().min(), padded.iter().max()) {
        println!("- The `.res` directory padding runs from {lo} to {hi} zero bytes.");
    }
}
