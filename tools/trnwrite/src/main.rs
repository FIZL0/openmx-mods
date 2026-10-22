//! `trnwrite`: a height grid → `<stem>.ps2.trn` and `<stem>.scn`; and a terrain checked or rewritten.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use game_sim::track::Terrain;
use trnwrite::scene::SceneText;
use trnwrite::{Grid, Options, Page, build, openmx_view, read, write};

const USAGE: &str = "\
usage:
  trnwrite build STEM OUTDIR --size W,D (--feet FILE | --raw FILE --height-scale H) --scale FEET
                 [--surfaces FILE] [--normals FILE] [--palette FILE] [--page FILE]...
                 [--page-map FILE --tiles-x N] [--start X,Y,Z] [--direction X,Y,Z]
  trnwrite check FILE.trn
  trnwrite rewrite IN.trn OUT.trn

build writes OUTDIR/STEM.ps2.trn and OUTDIR/STEM.scn. Grids are W×D samples, rows along x then z,
little-endian: --feet f32 world heights (quantised so the highest is 65535), --raw u16 with its
--height-scale (world y = scale × height scale × raw), --surfaces one byte a sample, --normals three
signed bytes a sample (else computed). --palette is 1024 bytes RGBA, --page 65536 bytes of 8-bit pixels
(one 256×256 page each, in order), --page-map one i16 a 64-sample tile (-1 for none), N tiles wide.
The start defaults to the middle of the grid, 6 ft above it, facing +z.

check reads a terrain with openmx's reader; rewrite decodes one and writes it again.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let run = match args.first().map(String::as_str) {
        Some("build") => build_cmd(&args[1..]),
        Some("check") if args.len() == 2 => check_cmd(Path::new(&args[1])),
        Some("rewrite") if args.len() == 3 => rewrite_cmd(Path::new(&args[1]), Path::new(&args[2])),
        _ => Err(USAGE.to_owned()),
    };
    match run {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("trnwrite: {e}");
            ExitCode::FAILURE
        }
    }
}

fn load(p: &str) -> Result<Vec<u8>, String> {
    std::fs::read(p).map_err(|e| format!("{p}: {e}"))
}

fn numbers<const N: usize>(s: &str) -> Result<[f32; N], String> {
    let v: Vec<f32> = s
        .split(',')
        .map(|x| x.trim().parse::<f32>())
        .collect::<Result<_, _>>()
        .map_err(|e| format!("{s:?}: {e}"))?;
    v.try_into().map_err(|_| format!("{s:?}: want {N} numbers"))
}

fn build_cmd(args: &[String]) -> Result<(), String> {
    let mut pos = Vec::new();
    let mut opt: Vec<(String, String)> = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if let Some(k) = a.strip_prefix("--") {
            let v = it.next().ok_or_else(|| format!("--{k} wants a value"))?;
            opt.push((k.to_owned(), v.clone()));
        } else {
            pos.push(a.clone());
        }
    }
    let [stem, outdir] = pos.as_slice() else {
        return Err(USAGE.to_owned());
    };
    let get = |k: &str| {
        opt.iter()
            .rev()
            .find(|(n, _)| n == k)
            .map(|(_, v)| v.as_str())
    };
    for (k, _) in &opt {
        if ![
            "size",
            "feet",
            "raw",
            "height-scale",
            "scale",
            "surfaces",
            "normals",
            "palette",
            "page",
            "page-map",
            "tiles-x",
            "start",
            "direction",
        ]
        .contains(&k.as_str())
        {
            return Err(format!("unknown option --{k}"));
        }
    }
    let [w, d] = numbers::<2>(get("size").ok_or("--size W,D is required")?)?;
    let (w, d) = (w as u32, d as u32);
    let scale: f32 = get("scale")
        .ok_or("--scale is required")?
        .parse()
        .map_err(|e| format!("--scale: {e}"))?;
    let n = (w * d) as usize;

    let (mut g, height_scale) = match (get("feet"), get("raw")) {
        (Some(f), None) => {
            let b = load(f)?;
            let feet: Vec<f32> = b
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                .collect();
            if b.len() != 4 * n {
                return Err(format!(
                    "{f}: {} bytes, want {} ({w}×{d} f32)",
                    b.len(),
                    4 * n
                ));
            }
            Grid::from_feet(w, d, &feet, scale)?
        }
        (None, Some(f)) => {
            let b = load(f)?;
            if b.len() != 2 * n {
                return Err(format!(
                    "{f}: {} bytes, want {} ({w}×{d} u16)",
                    b.len(),
                    2 * n
                ));
            }
            let hs: f32 = get("height-scale")
                .ok_or("--raw needs --height-scale")?
                .parse()
                .map_err(|e| format!("--height-scale: {e}"))?;
            let heights = b
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            (
                Grid {
                    width: w,
                    depth: d,
                    heights,
                    surfaces: Vec::new(),
                    normals: Vec::new(),
                },
                hs,
            )
        }
        _ => return Err("one of --feet or --raw".into()),
    };
    if let Some(f) = get("surfaces") {
        g.surfaces = load(f)?;
    }
    if let Some(f) = get("normals") {
        g.normals = load(f)?
            .chunks_exact(3)
            .map(|c| [c[0] as i8, c[1] as i8, c[2] as i8])
            .collect();
    }
    let mut o = Options::new(scale, height_scale);
    if let Some(f) = get("palette") {
        let p: [u8; 1024] = load(f)?
            .try_into()
            .map_err(|_| format!("{f}: a palette is 1024 bytes"))?;
        o.palette = Some(Box::new(p));
    }
    for (_, f) in opt.iter().filter(|(k, _)| k == "page") {
        o.pages
            .push(Page::psmt8(load(f)?).map_err(|e| format!("{f}: {e}"))?);
    }
    if let Some(f) = get("page-map") {
        o.page_map = load(f)?
            .chunks_exact(2)
            .map(|c| u16::try_from(i16::from_le_bytes([c[0], c[1]])).ok())
            .collect();
        o.tiles_x = get("tiles-x")
            .ok_or("--page-map needs --tiles-x")?
            .parse()
            .map_err(|e| format!("--tiles-x: {e}"))?;
    }

    let t = build(&g, &o)?;
    let (bytes, written) = write(&t)?;
    // Proved before it is kept: openmx's reader takes it, and it decodes to what was built.
    let view = openmx_view(&bytes)?;
    Terrain::parse(&view).map_err(|e| format!("openmx refuses the result: {e}"))?;
    if read(&bytes)? != t {
        return Err("the written terrain decodes differently (a writer bug)".into());
    }

    let mid = ((w / 2) + (d / 2) * w) as usize;
    let ground = scale * height_scale * f32::from(g.heights[mid]);
    let start = match get("start") {
        Some(s) => numbers::<3>(s)?,
        None => [scale * (w / 2) as f32, ground + 6.0, scale * (d / 2) as f32],
    };
    let direction = match get("direction") {
        Some(s) => numbers::<3>(s)?,
        None => [0.0, 0.0, 1.0],
    };
    let scn = SceneText::for_terrain(&t, stem, start, direction).to_text();

    let dir = PathBuf::from(outdir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{outdir}: {e}"))?;
    let trn = dir.join(format!("{stem}.ps2.trn"));
    std::fs::write(&trn, &bytes).map_err(|e| format!("{}: {e}", trn.display()))?;
    let scn_path = dir.join(format!("{stem}.scn"));
    std::fs::write(&scn_path, scn).map_err(|e| format!("{}: {e}", scn_path.display()))?;
    println!(
        "{}: {} levels, {} leaves over {} blocks, {} pages, {} bytes; {}",
        trn.display(),
        t.levels,
        written.leaves,
        written.blocks,
        written.pages,
        bytes.len(),
        scn_path.display()
    );
    Ok(())
}

fn check_cmd(p: &Path) -> Result<(), String> {
    let b = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
    let view = openmx_view(&b)?;
    let t = Terrain::parse(&view)?;
    let m = read(&b)?;
    let (nodes, leaves) = m.count();
    let (cw, cd) = m.root_cells();
    println!(
        "{}: {:?} scale {} levels {} root cells {cw}×{cd}, {nodes} nodes, {leaves} leaves, {} blocks, {} pages{}",
        p.display(),
        t.name,
        t.scale.to_f32(),
        t.levels,
        t.blocks,
        m.pages.len(),
        if matches!(view, std::borrow::Cow::Owned(_)) {
            " (openmx reads it through the MD1 view)"
        } else {
            ""
        }
    );
    Ok(())
}

fn rewrite_cmd(src: &Path, dst: &Path) -> Result<(), String> {
    let b = std::fs::read(src).map_err(|e| format!("{}: {e}", src.display()))?;
    let m = read(&b)?;
    let (out, w) = write(&m)?;
    if read(&out)? != m {
        return Err("the rewritten terrain decodes differently (a writer bug)".into());
    }
    std::fs::write(dst, &out).map_err(|e| format!("{}: {e}", dst.display()))?;
    println!(
        "{}: {} leaves, {} blocks, {} pages, {} bytes",
        dst.display(),
        w.leaves,
        w.blocks,
        w.pages,
        out.len()
    );
    Ok(())
}
