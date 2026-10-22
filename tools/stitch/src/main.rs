//! stitch: track mods joined into one world mod (standard §3).
//!
//! ```text
//! stitch build OUT --id world_x --name NAME [--sky TILE] [--max-slope X] [--min-gap FT] [--chunk N]
//!              [--fade FT] [--threads N] [--no-prune] TILE_DIR...
//! stitch check WORLD_DIR [--mods DIR] [--iso DIR]
//! stitch profile TILE_DIR [SAMPLES]       (each side's median height by distance from the edge)
//! stitch map WORLD_DIR OUT.pgm [FEET]       (the world's heights as a greyscale picture, shaded)
//! stitch edges [--max-slope X] [--fade FT] TILE_DIR...  (each edge's relief and seam gate, C9)
//! stitch --mods OUT/mods --out STAGE        (build.sh: every world in tools/stitch/worlds/*.json)
//! ```
//!
//! A world spec (`worlds/<id>.json`): `{"id": "world_x", "name": "...", "tiles": ["mxvatv_nat05", ...]}`
//! with optional `sky`, `max_slope`, `min_gap`, `chunk`, `fade`, and `todo` (lines the world's mod.json todo
//! carries, e.g. why the spec raises `max_slope`). Its tiles are looked up in
//! `--mods`/tracks/. An `mxu_` tile is never there (MX Unleashed's tracks are only a world's tiles,
//! §1, so no step packages them): it is imported off the MX Unleashed disc in `$DISCX_ISO` (or `iso/`)
//! into `<STAGE>.mxu/`, beside the stage, so build.sh does not package it as a track (C4).

use modcheck::json::Json;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use stitch::build::{Params, build};

fn usage() -> ExitCode {
    eprintln!("{}", include_str!("main.rs").lines().skip(3).take(8).map(|l| l.trim_start_matches("//! ")).collect::<Vec<_>>().join("\n"));
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("build") => cmd_build(&args[1..]),
        Some("check") => cmd_check(&args[1..]),
        Some("profile") => cmd_profile(&args[1..]),
        Some("map") => cmd_map(&args[1..]),
        Some("edges") => cmd_edges(&args[1..]),
        Some("--mods") | Some("--out") => cmd_specs(&args),
        _ => usage(),
    }
}

fn take(args: &[String], i: &mut usize, flag: &str) -> Result<String, String> {
    *i += 1;
    args.get(*i).cloned().ok_or_else(|| format!("{flag} needs a value"))
}

fn num(v: &str, flag: &str) -> Result<f64, String> {
    v.parse::<f64>().map_err(|_| format!("{flag}: {v:?} is not a number"))
}

fn report(b: &stitch::build::Built) {
    println!(
        "stitch: {} ({} tiles, {} join pieces at {} ft, {} placements dropped; join slope {:.3} away from seams, {:.3} near them)",
        b.dir.display(),
        b.plan.placed.len(),
        b.pieces,
        b.plan.spacing,
        b.drops,
        b.far_slope,
        b.near_slope
    );
}

fn cmd_build(args: &[String]) -> ExitCode {
    let run = || -> Result<(), String> {
        let mut out = None;
        let mut p = Params::new("", "");
        let mut tiles = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let a = args[i].as_str();
            match a {
                "--id" => p.id = take(args, &mut i, a)?,
                "--name" => p.name = take(args, &mut i, a)?,
                "--sky" => p.sky = Some(take(args, &mut i, a)?),
                "--max-slope" => p.max_slope = num(&take(args, &mut i, a)?, a)?,
                "--min-gap" => p.min_gap = num(&take(args, &mut i, a)?, a)?,
                "--fade" => p.fade = num(&take(args, &mut i, a)?, a)?,
                "--chunk" => p.chunk = num(&take(args, &mut i, a)?, a)? as u32,
                "--threads" => p.threads = num(&take(args, &mut i, a)?, a)? as usize,
                "--no-prune" => p.prune = false,
                _ if a.starts_with("--") => return Err(format!("unknown flag {a}")),
                _ if out.is_none() => out = Some(PathBuf::from(a)),
                _ => tiles.push(PathBuf::from(a)),
            }
            i += 1;
        }
        let out = out.ok_or("no OUT folder")?;
        if p.id.is_empty() || p.name.is_empty() {
            return Err("--id and --name are needed".into());
        }
        let b = build(&tiles, &out, &p)?;
        report(&b);
        Ok(())
    };
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("stitch: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_check(args: &[String]) -> ExitCode {
    let mut dir = None;
    let (mut mods, mut iso) = (None, std::env::var_os("DISCX_ISO").map(PathBuf::from));
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--mods" => {
                i += 1;
                mods = args.get(i).map(PathBuf::from);
            }
            "--iso" => {
                i += 1;
                iso = args.get(i).map(PathBuf::from);
            }
            a => dir = Some(PathBuf::from(a)),
        }
        i += 1;
    }
    let Some(dir) = dir else { return usage() };
    let r = stitch::check::check(&dir, mods.as_deref(), iso);
    for n in &r.note {
        println!("note: {n}");
    }
    for f in &r.fail {
        println!("FAIL: {f}");
    }
    println!(
        "stitch check {}: {} seam points (worst height step {:.5} ft, slope step {:.6}), {} where joins of two spacings meet \
         (worst {:.5} ft), join slope {:.3} away from the seams: {}",
        dir.display(),
        r.seam_points,
        r.worst_height,
        r.worst_slope,
        r.join_seam_points,
        r.join_seam_height,
        r.far_slope,
        if r.ok() { "ok" } else { "FAILED" }
    );
    if r.ok() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

/// build.sh's step: every spec in tools/stitch/worlds/ whose tiles are all built.
fn cmd_specs(args: &[String]) -> ExitCode {
    let (mut mods, mut out) = (None, None);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--mods" => mods = args.get(i + 1).map(PathBuf::from),
            "--out" => out = args.get(i + 1).map(PathBuf::from),
            _ => {}
        }
        i += 2;
    }
    let (Some(mods), Some(out)) = (mods, out) else { return usage() };
    let specs = Path::new(env!("CARGO_MANIFEST_DIR")).join("worlds");
    let mut names: Vec<PathBuf> = std::fs::read_dir(&specs)
        .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "json")).collect())
        .unwrap_or_default();
    names.sort();
    if names.is_empty() {
        println!("stitch: no world specs in {}: nothing to build", specs.display());
        return ExitCode::SUCCESS;
    }
    let mut failed = false;
    for spec in names {
        let run = || -> Result<Option<stitch::build::Built>, String> {
            let j = Json::parse(&std::fs::read_to_string(&spec).map_err(|e| e.to_string())?)?;
            let g = |k: &str| j.get(k).and_then(Json::str).map(str::to_owned);
            let mut p = Params::new(&g("id").ok_or("no id")?, &g("name").ok_or("no name")?);
            p.sky = g("sky");
            p.todo = j.get("todo").and_then(Json::arr).unwrap_or_default().iter().filter_map(Json::str).map(str::to_owned).collect();
            if let Some(v) = j.get("max_slope").and_then(Json::num) {
                p.max_slope = v;
            }
            if let Some(v) = j.get("min_gap").and_then(Json::num) {
                p.min_gap = v;
            }
            if let Some(v) = j.get("fade").and_then(Json::num) {
                p.fade = v;
            }
            if let Some(v) = j.get("chunk").and_then(Json::num) {
                p.chunk = v as u32;
            }
            let names: Vec<&str> = j.get("tiles").and_then(Json::arr).unwrap_or_default().iter().filter_map(Json::str).collect();
            let mxu_stage = out.with_extension("mxu");
            let mxu: Vec<&str> = names.iter().copied().filter(|t| t.starts_with("mxu_") && !mods.join("tracks").join(t).join("mod.json").exists()).collect();
            if !mxu.is_empty() {
                if let Err(e) = mxu_tiles(&mxu, &mxu_stage) {
                    println!("stitch: skip {}: its MX Unleashed tiles: {e}", p.id);
                    return Ok(None);
                }
            }
            let tiles: Vec<PathBuf> = names
                .iter()
                .map(|t| if mxu.contains(t) { mxu_stage.join(t) } else { mods.join("tracks").join(t) })
                .collect();
            if let Some(missing) = tiles.iter().find(|t| !t.join("mod.json").exists()) {
                println!("stitch: skip {}: {} is not built", p.id, missing.display());
                return Ok(None);
            }
            build(&tiles, &out, &p).map(Some)
        };
        match run() {
            Ok(Some(b)) => report(&b),
            Ok(None) => {}
            Err(e) => {
                eprintln!("stitch: {}: {e}", spec.display());
                failed = true;
            }
        }
    }
    if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

/// MX Unleashed's tracks as tiles, off its disc into `stage` (import-mxvatv's MXU path, read back as
/// tiles), for the spec's `mxu_` tiles that no step built. Each is imported once per stage.
fn mxu_tiles(ids: &[&str], stage: &Path) -> Result<(), String> {
    use import_mxvatv::mxu;
    let iso = std::env::var_os("DISCX_ISO").map_or_else(|| PathBuf::from("iso"), PathBuf::from);
    let (image, _) = discx::images(&iso)?
        .into_iter()
        .find(|(_, b)| b.as_deref().is_some_and(|b| b.eq_ignore_ascii_case(mxu::BOOT)))
        .ok_or_else(|| format!("no disc with boot ELF {} in {}", mxu::BOOT, iso.display()))?;
    let mut disc = discx::Disc::open(&image)?;
    let tracks = mxu::tracks(&disc);
    let mut discs = modcheck::Discs::new(Some(iso));
    for id in ids {
        if stage.join(id).join("mod.json").exists() {
            continue;
        }
        let want = &id["mxu_".len()..];
        let t = tracks.iter().find(|t| t.eq_ignore_ascii_case(want)).ok_or_else(|| format!("{id}: no Tracks\\{want}.res on the disc"))?;
        let m = import_mxvatv::import(&mxu::load_map(&mut disc, t)?, stage)?;
        let r = modcheck::check_tile(&m.dir, &mut discs);
        if !r.ok() {
            let _ = std::fs::remove_dir_all(&m.dir);
            return Err(format!("{id}: the read-back failed: {}", r.fail.join("; ")));
        }
        println!("stitch: {id} imported off {} into {}", image.display(), stage.display());
    }
    Ok(())
}

/// Each side's edge profile and the cut find_keep makes, to see a map's berm.
fn cmd_profile(args: &[String]) -> ExitCode {
    let Some(dir) = args.first() else { return usage() };
    let last: usize = args.get(1).and_then(|v| v.parse().ok()).unwrap_or(200);
    let t = match stitch::Tile::load(Path::new(dir)) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("stitch: {e}");
            return ExitCode::FAILURE;
        }
    };
    let terrain = t.terrain();
    let skip = (stitch::SHOOTOFF / t.spacing).ceil() as u32;
    let p: Vec<Vec<f64>> = (0..4).map(|side| t.profile(&terrain, side, last, skip)).collect();
    println!("{}: {}×{} samples at {} ft, ground {:?}, keep {:?}", t.id, t.n.0, t.n.1, t.spacing, t.ground, t.find_keep());
    // The whole map, coarse: one height every 1/20 of a side (rows are z).
    let at = |x: u32, z: u32| {
        let (cx, cz) = (x.min(t.n.0 - 1), z.min(t.n.1 - 1));
        terrain.cell(cx, cz, 1).map_or(f64::NAN, |q| f64::from((t.scale * q.height[((x - cx) + 2 * (z - cz)) as usize]).0))
    };
    for k in 0..=20 {
        let z = t.n.1 * k / 20;
        println!("z={z:5} {}", (0..=20).map(|i| format!("{:4.0}", at(t.n.0 * i / 20, z))).collect::<Vec<_>>().join(""));
    }
    println!("samples\tfeet\tx=x0+d\tz=z0+d\tx=x1-d\tz=z1-d\t(the ground's sides)");
    for d in 0..=last {
        println!("{d}\t{:.1}\t{:.2}\t{:.2}\t{:.2}\t{:.2}", d as f64 * t.spacing, p[0][d], p[1][d], p[2][d], p[3][d]);
    }
    ExitCode::SUCCESS
}

/// Each tile's edges as the joiner sees them: the cut, each edge's height range and steepest cell, and
/// the seam gate its extension needs at this `max_slope` (C9).
fn cmd_edges(args: &[String]) -> ExitCode {
    let run = || -> Result<(), String> {
        let mut p = Params::new("", "");
        let mut dirs = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let a = args[i].as_str();
            match a {
                "--max-slope" => p.max_slope = num(&take(args, &mut i, a)?, a)?,
                "--fade" => p.fade = num(&take(args, &mut i, a)?, a)?,
                _ if a.starts_with("--") => return Err(format!("unknown flag {a}")),
                _ => dirs.push(PathBuf::from(a)),
            }
            i += 1;
        }
        println!("tile\tedge\tlow\thigh\tsteepest\tgate");
        for d in dirs {
            let t = stitch::Tile::load(&d)?;
            let src = stitch::build::lone_source(&t, &p);
            for (e, g) in src.gates.iter().enumerate() {
                let edge = &src.ring.edges[e];
                let (lo, hi) = edge.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), q| (a.min(q.h), b.max(q.h)));
                let steep = edge.iter().map(|q| q.gx.hypot(q.gz)).fold(0.0, f64::max);
                println!("{}\t{e}\t{lo:.1}\t{hi:.1}\t{steep:.2}\t{g:.0}", t.id);
            }
        }
        Ok(())
    };
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("stitch: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The world's ground as a PGM: height as grey, lit from the north-west so slopes show.
fn cmd_map(args: &[String]) -> ExitCode {
    let (Some(dir), Some(out)) = (args.first(), args.get(1)) else { return usage() };
    let step: f64 = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(18.0);
    let (w, h, z) = match stitch::check::heightmap(Path::new(dir), step) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("stitch: {e}");
            return ExitCode::FAILURE;
        }
    };
    let (lo, hi) = z.iter().filter(|v| v.is_finite()).fold((f64::INFINITY, f64::NEG_INFINITY), |a, v| (a.0.min(*v), a.1.max(*v)));
    let mut img = format!("P5\n{w} {h}\n255\n").into_bytes();
    for r in 0..h {
        for c in 0..w {
            let v = z[c + r * w];
            let px = if !v.is_finite() {
                0
            } else {
                let base = (v - lo) / (hi - lo).max(1.0);
                let n = if c > 0 && r > 0 { z[c - 1 + (r - 1) * w] } else { v };
                let shade = if n.is_finite() { ((v - n) / step).clamp(-1.0, 1.0) * 0.35 } else { 0.0 };
                ((0.15 + 0.6 * base + shade).clamp(0.0, 1.0) * 255.0) as u8
            };
            img.push(px);
        }
    }
    if let Err(e) = std::fs::write(out, img) {
        eprintln!("stitch: {out}: {e}");
        return ExitCode::FAILURE;
    }
    println!("stitch: {out}: {w}×{h} at {step} ft, {lo:.1} to {hi:.1} ft");
    ExitCode::SUCCESS
}
