//! `elevation`: real elevation to a track mod. `elevation help` for usage.

use elevation::geotiff::GeoTiff;
use elevation::sites::{DEM_URL, NAIP_TOKEN, NAIP_URL, SITES};
use elevation::{Built, Request, build_mod, write_mod};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = r#"elevation: a lidar DEM (USGS 3DEP GeoTIFF) and an aerial image (NAIP) to a track mod

  elevation --out STAGE [--from DIR] [--iso DIR] [--sky none]
                                      build every site (`elevation sites`) whose files are in DIR
                                      (default extracted/elevation: dem/ and naip/) into STAGE/<id>/
                                      (build.sh's data step), each checked with modcheck. Each takes its
                                      site's sky, fog and lights from MX vs. ATV Unleashed if that disc
                                      is in --iso (default $DISCX_ISO, else iso/); --sky none, or no
                                      disc, builds them without (public-domain)
  elevation sites                     the sites, their windows and files
  elevation surfaces [--iso DIR] [MAP...]
                                      MX Unleashed's freeride maps (free06-free11): each terrain
                                      sample's surface id against the painted layer that shows most
                                      there (M16; the ids the sites use come from it). Other maps
                                      named (nat01, sx05) get their ids counted
  elevation cover OUT.png SITE [--from DIR] [--green NDVI] [--picture RGB.png]
                                      a site's surfaces from its aerial picture, a pixel a sample
                                      (sand yellow, dirt brown, grass green, no picture red), with
                                      the shares and the picture's NDVI spread
  elevation fetch [--from DIR]        download the sites' files that are missing (curl; about 3.6 GB)
  elevation info FILE.tif...          where each GeoTIFF lies, its grid and units
  elevation relief FILE.tif... --cells N [--step M]
                                      the windows of N×N root cells with the most relief (to choose one)
  elevation preview OUT.png FILE.tif... [--every N] [--centre E,N --size PX]
                                      a shaded picture of the mosaic, one pixel every N (default 10), north up
  elevation build OUTDIR --id elev_x --name NAME --dem FILE.tif [--dem ...] --centre E,N
                  [--cells W[,D]] [--naip FILE.tif]... [--texture PX] [--surface ID]
                  [--ground dunes|desert|forest --green NDVI]
                  --data TEXT --date YYYY-MM-DD [--sky MAP,NAME [--iso DIR]]
                                      write the mod folder OUTDIR (out/mods/tracks/<id>), then check it
                                      with modcheck (the standard's §5)

--centre is in the DEM's projected coordinates (UTM easting, northing). --cells defaults to 8 (2049 samples,
2 km at 1 m), --texture to 1024, --surface to 0. --ground classifies each sample from the picture's
near infrared instead (sand 4, dirt 0, grass 3: notes/elevation.md, Surfaces). A map is 256 × cells + 1 DEM pixels a side, one sample a
pixel; nothing is resampled. --sky takes CubeFile, [Fog] and [Light<n>] and the sky's .cub from MX vs.
ATV Unleashed's Env\MAP (Nat03,Glamis). notes/elevation.md lists the sites, their tiles and where to fetch them.
"#;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("elevation: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Every value of `--name`, and the arguments that are not flags.
struct Args {
    flags: Vec<(String, String)>,
    plain: Vec<String>,
}

impl Args {
    fn new(a: &[String]) -> Result<Args, String> {
        let (mut flags, mut plain) = (Vec::new(), Vec::new());
        let mut i = 0;
        while i < a.len() {
            if let Some(f) = a[i].strip_prefix("--") {
                let v = a.get(i + 1).ok_or(format!("--{f} wants a value"))?;
                flags.push((f.to_owned(), v.clone()));
                i += 2;
            } else {
                plain.push(a[i].clone());
                i += 1;
            }
        }
        Ok(Args { flags, plain })
    }
    fn all(&self, name: &str) -> Vec<String> {
        self.flags
            .iter()
            .filter(|(f, _)| f == name)
            .map(|(_, v)| v.clone())
            .collect()
    }
    fn one(&self, name: &str) -> Option<String> {
        self.all(name).pop()
    }
    fn need(&self, name: &str) -> Result<String, String> {
        self.one(name)
            .ok_or(format!("--{name} is needed\n\n{USAGE}"))
    }
}

fn pair<T: std::str::FromStr + Copy>(s: &str) -> Result<(T, T), String> {
    let v: Vec<T> = s
        .split(',')
        .map(|p| p.trim().parse().map_err(|_| format!("{s:?}: not a number")))
        .collect::<Result<_, _>>()?;
    match v[..] {
        [a] => Ok((a, a)),
        [a, b] => Ok((a, b)),
        _ => Err(format!("{s:?}: one or two numbers")),
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let Some(cmd) = args.first() else {
        print!("{USAGE}");
        return Ok(());
    };
    if cmd == "--out" {
        return stage(&Args::new(args)?);
    }
    let a = Args::new(&args[1..])?;
    match cmd.as_str() {
        "sites" => {
            for s in SITES {
                println!(
                    "{}  {:?}  centre {:.0},{:.0}  {}×{} root cells",
                    s.id, s.name, s.centre.0, s.centre.1, s.cells.0, s.cells.1
                );
                for d in s.dem {
                    println!("  dem   {DEM_URL}/{}/TIFF/{}", d.project, d.file);
                }
                for n in s.naip {
                    println!("  naip  {NAIP_URL}/{}", n.path);
                }
            }
            Ok(())
        }
        "fetch" => fetch(&a),
        "info" => {
            for p in &a.plain {
                let t = GeoTiff::open(p.as_ref())?;
                let g = &t.geo;
                println!(
                    "{p}\n  {}×{} px, {} band(s), {}×{} a pixel, EPSG {:?}, model {}, units {:?} / vertical {:?}, nodata {:?}\n  \
                     west {:.3} east {:.3} north {:.3} south {:.3}",
                    g.width,
                    g.height,
                    g.bands,
                    g.dx,
                    g.dy,
                    g.epsg,
                    g.model,
                    g.linear_units,
                    g.vertical_units,
                    g.nodata,
                    g.left,
                    g.left + g.dx * f64::from(g.width),
                    g.top,
                    g.top - g.dy * f64::from(g.height)
                );
            }
            Ok(())
        }
        "surfaces" => surfaces(&a),
        "cover" => cover(&a),
        "relief" => relief(&a),
        "preview" => preview(&a),
        "build" => {
            let out = PathBuf::from(
                a.plain
                    .first()
                    .ok_or(format!("build wants OUTDIR\n\n{USAGE}"))?,
            );
            let mut req = Request {
                id: a.need("id")?,
                name: a.need("name")?,
                dem: a.all("dem").into_iter().map(PathBuf::from).collect(),
                naip: a.all("naip").into_iter().map(PathBuf::from).collect(),
                centre: pair(&a.need("centre")?)?,
                cells: pair(&a.one("cells").unwrap_or("8".into()))?,
                texture: a
                    .one("texture")
                    .map_or(Ok(1024), |s| s.parse().map_err(|_| "--texture".to_owned()))?,
                surface: a
                    .one("surface")
                    .map_or(Ok(0), |s| s.parse().map_err(|_| "--surface".to_owned()))?,
                ground: match a.one("ground") {
                    None => None,
                    Some(g) => Some((
                        elevation::surfaces::Ground::parse(&g).ok_or(format!("--ground {g:?}: dunes, desert or forest"))?,
                        a.need("green")?.parse().map_err(|_| "--green".to_owned())?,
                    )),
                },
                data: a.need("data")?,
                date: a.need("date")?,
                sky: None,
            };
            if let Some(m) = a.one("sky") {
                let (map, name) = m.split_once(',').unwrap_or((&m, &m));
                let iso = iso_dir(&a);
                let path = elevation::sky::find_disc(&iso).ok_or(format!(
                    "--sky: no {} disc in {}",
                    elevation::sky::BOOT,
                    iso.display()
                ))?;
                let mut disc = discx::Disc::open(&path)?;
                req.sky = Some(elevation::sky::Donor::take(&mut disc, map, name)?);
            }
            if req.dem.is_empty() {
                return Err(format!("--dem is needed\n\n{USAGE}"));
            }
            let b = build_mod(&req)?;
            write_mod(&out, &b)?;
            report(&out, &b)
        }
        "help" | "-h" | "--help" => {
            print!("{USAGE}");
            Ok(())
        }
        other => Err(format!("{other:?}: not a command\n\n{USAGE}")),
    }
}

/// The build's facts, then the standard's check of the folder (§5).
fn report(out: &std::path::Path, b: &Built) -> Result<(), String> {
    let f = &b.facts;
    println!(
        "{}: {}×{} samples at {} ft, corner (south-west sample) {:.3}, {:.3} (pixel {}, {}), datum {} , {:.2}..{:.2} ft, \
                 step {:.5} ft, read back within {:.5} ft; {} leaves, {} blocks; tile overlap {} px, {} differ (largest {}); start {:?}",
        out.display(),
        f.samples.0,
        f.samples.1,
        f.scale,
        f.corner.0,
        f.corner.1,
        f.corner_pixel.0,
        f.corner_pixel.1,
        f.datum,
        f.low_ft,
        f.high_ft,
        f.step_ft,
        f.worst_ft,
        f.leaves,
        f.blocks,
        f.overlap.pixels,
        f.overlap.differ,
        f.overlap.largest,
        f.spawn
    );
    let r = modcheck::check(&out, &mut modcheck::Discs::new(None));
    for n in &r.note {
        println!("  note: {n}");
    }
    for e in &r.fail {
        println!("  FAIL: {e}");
    }
    if r.ok() {
        Ok(())
    } else {
        Err(format!("modcheck: {} failures", r.fail.len()))
    }
}

/// Where the disc images are: `--iso`, else `$DISCX_ISO`, else `iso/` (build.sh's own default).
fn iso_dir(a: &Args) -> PathBuf {
    a.one("iso")
        .or_else(|| std::env::var("DISCX_ISO").ok())
        .unwrap_or("iso".into())
        .into()
}

/// build.sh's data step: every site whose files are here, into `STAGE/<id>/`. A site without its files
/// is skipped with a line naming them; a site that fails to build fails the step. With MX vs. ATV
/// Unleashed's disc each site takes its donor map's sky (M17); without it, or with `--sky none`, none.
fn stage(a: &Args) -> Result<(), String> {
    let stage = PathBuf::from(a.need("out")?);
    let from = PathBuf::from(a.one("from").unwrap_or("extracted/elevation".into()));
    let mut disc = match a.one("sky").as_deref() {
        Some("none") => None,
        Some(other) => return Err(format!("--sky {other:?}: the data step takes only none")),
        None => {
            let iso = iso_dir(a);
            match elevation::sky::find_disc(&iso) {
                Some(p) => Some(discx::Disc::open(&p)?),
                None => {
                    println!(
                        "elevation: no {} disc in {}: the sites are built without a sky, fog or lights",
                        elevation::sky::BOOT,
                        iso.display()
                    );
                    None
                }
            }
        }
    };
    let mut failed = Vec::new();
    for s in SITES {
        let mut req = match s.request(&from) {
            Ok(r) => r,
            Err(missing) => {
                println!(
                    "elevation: skip {}: not fetched ({}); `elevation fetch`",
                    s.id,
                    missing.join(", ")
                );
                continue;
            }
        };
        if let Some(disc) = disc.as_mut() {
            match elevation::sky::Donor::take(disc, s.sky.0, s.sky.1) {
                Ok(d) => req.sky = Some(d),
                Err(e) => {
                    println!("elevation: FAIL {}: its sky from {}: {e}", s.id, s.sky.0);
                    failed.push(s.id);
                    continue;
                }
            }
        }
        let out = stage.join(s.id);
        let done = build_mod(&req).and_then(|b| {
            write_mod(&out, &b)?;
            report(&out, &b)
        });
        if let Err(e) = done {
            println!("elevation: FAIL {}: {e}", s.id);
            failed.push(s.id);
        }
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!("{} failed", failed.join(", ")))
    }
}

/// One site's surfaces as a picture, for looking at the classification (M16).
fn cover(a: &Args) -> Result<(), String> {
    use elevation::surfaces::{Cover, ndvi};
    let [out, id] = &a.plain[..] else {
        return Err(format!("cover wants OUT.png and a site\n\n{USAGE}"));
    };
    let site = SITES
        .iter()
        .find(|s| s.id == id)
        .ok_or(format!("{id}: not a site (`elevation sites`)"))?;
    let from = PathBuf::from(a.one("from").unwrap_or("extracted/elevation".into()));
    let req = site
        .request(&from)
        .map_err(|m| format!("not fetched: {}", m.join(", ")))?;
    let (corner, pixel, samples, epsg) = elevation::window_of(&req)?;
    let seen = elevation::surfaces::seen(&req.naip, corner, pixel, samples, epsg)?;
    let (w, h) = (samples.0 as usize, samples.1 as usize);
    let green: f32 = a
        .one("green")
        .map_or(Ok(site.green), |s| s.parse().map_err(|_| "--green"))?;
    let covers = site.ground.classify(&seen, w, green);
    // The picture itself, beside the classes, when asked for.
    if let Some(p) = a.one("picture") {
        let mut rgb = vec![0u8; w * h * 3];
        for z in 0..h {
            for x in 0..w {
                let c = seen[x + z * w].map_or([200, 0, 0], |s| [s[0], s[1], s[2]].map(|v| v as u8));
                let at = 3 * (x + (h - 1 - z) * w);
                rgb[at..at + 3].copy_from_slice(&c);
            }
        }
        game_render::png::write_rgb(std::path::Path::new(&p), w, h, &rgb)?;
    }
    let mut rgb = vec![0u8; w * h * 3];
    let mut count = [0u64; 4];
    let mut hist = [0u64; 20];
    for z in 0..h {
        for x in 0..w {
            let s = seen[x + z * w].as_ref();
            if let Some(s) = s {
                hist[((ndvi(s) + 1.0) * 10.0).clamp(0.0, 19.0) as usize] += 1;
            }
            let (c, k) = match (s, covers[x + z * w]) {
                (None, _) => ([200, 0, 0], 3),
                (_, Cover::Sand) => ([230, 210, 140], 0),
                (_, Cover::Dirt) => ([120, 85, 50], 1),
                (_, Cover::Grass) => ([40, 150, 40], 2),
            };
            count[k] += 1;
            // Row 0 of the picture is north.
            let at = 3 * (x + (h - 1 - z) * w);
            rgb[at..at + 3].copy_from_slice(&c);
        }
    }
    game_render::png::write_rgb(std::path::Path::new(out), w, h, &rgb)?;
    let all = (w * h) as f64;
    println!(
        "{id} ({}): sand {:.1}%, dirt {:.1}%, grass {:.1}%, no picture {:.1}%",
        site.ground.name(),
        100.0 * count[0] as f64 / all,
        100.0 * count[1] as f64 / all,
        100.0 * count[2] as f64 / all,
        100.0 * count[3] as f64 / all
    );
    let line: Vec<String> = hist
        .iter()
        .enumerate()
        .filter(|(_, n)| **n > 0)
        .map(|(i, n)| format!("{:.1}: {:.1}%", i as f32 / 10.0 - 1.0, 100.0 * *n as f64 / all))
        .collect();
    println!("  NDVI from: {}", line.join(", "));
    Ok(())
}

/// MX Unleashed's freeride maps: surface id against painted layer, a table per map (M16).
fn surfaces(a: &Args) -> Result<(), String> {
    use elevation::surfaces::survey;
    use game_render::pst::Texture;
    let iso = iso_dir(a);
    let path = discx::images(&iso)?
        .into_iter()
        .find(|(_, b)| b.as_deref().is_some_and(|b| b.eq_ignore_ascii_case("SLUS_207.67")))
        .map(|(p, _)| p)
        .ok_or(format!("no MX Unleashed (SLUS_207.67) disc in {}", iso.display()))?;
    let mut disc = discx::Disc::open(&path)?;
    let free = ["free06", "free07", "free09", "free10", "free11"];
    let maps: Vec<String> = if a.plain.is_empty() {
        free.iter().map(|m| m.to_string()).collect()
    } else {
        a.plain.clone()
    };
    for map in maps.iter().map(String::as_str) {
        let e = disc
            .find(&format!("Tracks\\{map}.res"))
            .cloned()
            .ok_or(format!("no Tracks\\{map}.res"))?;
        let res = disc.res(&e)?;
        let read = |suffix: &str| -> Option<Vec<u8>> {
            let m = res.dir.entries.iter().find(|m| {
                let n = m.name.to_ascii_lowercase().replace('/', "\\");
                n.ends_with(&format!("\\{suffix}"))
            })?;
            res.read(m).ok()
        };
        let trn = read(&format!("{map}.ps2.trn")).ok_or("no .trn")?;
        if !free.contains(&map) {
            let n = elevation::surfaces::ids(&trn)?;
            let all: u64 = n.values().sum();
            let row: Vec<String> = n
                .iter()
                .map(|(id, c)| format!("{id}: {:.1}%", 100.0 * *c as f64 / all as f64))
                .collect();
            println!("{map}: {}", row.join(", "));
            continue;
        }
        let scn = game_sim::track::Scene::parse(&read(&format!("{map}.scn")).ok_or("no .scn")?)?;
        let psm = game_render::ini::text(&read("terrainscreenspacemesh.psm").ok_or("no .psm")?);
        let samples = scn.terrain.width * 256 + 1;
        if scn.terrain.breadth != scn.terrain.width {
            return Err(format!("{map}: not square"));
        }
        let s = survey(
            &trn,
            &psm,
            |n| Texture::parse(&read(&format!("{n}.pst"))?).ok(),
            samples,
            scn.terrain.width_scale.0,
        )?;
        println!("{map}: {samples}×{samples} samples at {} ft", scn.terrain.width_scale.0);
        print!("{}", s.table());
    }
    Ok(())
}

/// Download each site's files that are not there yet, with curl: the DEM tiles from USGS's bucket and
/// the NAIP quads from Planetary Computer with its anonymous read token.
fn fetch(a: &Args) -> Result<(), String> {
    use std::process::Command;
    let from = PathBuf::from(a.one("from").unwrap_or("extracted/elevation".into()));
    let curl = |url: &str, to: &std::path::Path| -> Result<(), String> {
        let part = to.with_extension("part");
        println!("fetch {}", to.display());
        let ok = Command::new("curl")
            .args(["-sSfL", "-o"])
            .arg(&part)
            .arg(url)
            .status()
            .map_err(|e| format!("curl: {e}"))?;
        if !ok.success() {
            return Err(format!("curl failed on {url}"));
        }
        std::fs::rename(&part, to).map_err(|e| e.to_string())
    };
    let mut token: Option<String> = None;
    for s in SITES {
        for d in s.dem {
            let to = from.join("dem").join(d.file);
            if !to.is_file() {
                std::fs::create_dir_all(to.parent().unwrap()).map_err(|e| e.to_string())?;
                curl(&format!("{DEM_URL}/{}/TIFF/{}", d.project, d.file), &to)?;
            }
        }
        for n in s.naip {
            let to = from.join("naip").join(n.file());
            if to.is_file() {
                continue;
            }
            if token.is_none() {
                let out = Command::new("curl")
                    .args(["-sSf", NAIP_TOKEN])
                    .output()
                    .map_err(|e| format!("curl: {e}"))?;
                let text = String::from_utf8_lossy(&out.stdout).replace("\\u0026", "&");
                let t = text
                    .split("\"token\":\"")
                    .nth(1)
                    .and_then(|r| r.split('"').next())
                    .ok_or(format!("no token in {text:?}"))?;
                token = Some(t.to_owned());
            }
            std::fs::create_dir_all(to.parent().unwrap()).map_err(|e| e.to_string())?;
            curl(
                &format!("{NAIP_URL}/{}?{}", n.path, token.as_deref().unwrap()),
                &to,
            )?;
        }
    }
    Ok(())
}

/// Slide an N×N-root-cell window over the mosaic in steps and print the ten with the largest relief
/// (high minus low) that have data everywhere, read from the file's full-resolution pixels.
fn relief(a: &Args) -> Result<(), String> {
    let cells: u32 = a.need("cells")?.parse().map_err(|_| "--cells")?;
    let n = 256 * cells + 1;
    let step: i64 = a
        .one("step")
        .map_or(Ok(256), |s| s.parse().map_err(|_| "--step"))?;
    let mut m = elevation::dem::Mosaic::open(&a.plain)?;
    // The mosaic's extent in the first raster's pixels.
    let (mut i1, mut j1) = (0i64, 0i64);
    for (r, (i, j)) in m.rasters.iter().zip(&m.at) {
        i1 = i1.max(i + i64::from(r.geo.width));
        j1 = j1.max(j + i64::from(r.geo.height));
    }
    let (i0, j0) = m.at.iter().fold((i64::MAX, i64::MAX), |(a, b), (i, j)| {
        (a.min(*i), b.min(*j))
    });
    // One strip of rows at a time, the whole width.
    let w = (i1 - i0) as u32;
    let mut found = Vec::new();
    let mut j = j0;
    while j + i64::from(n) <= j1 {
        let (px, _) = m.window(i0, j, w, n)?;
        let mut i = 0i64;
        while i + i64::from(n) <= i64::from(w) {
            let (mut lo, mut hi, mut bad) = (f32::INFINITY, f32::NEG_INFINITY, false);
            'rows: for r in 0..n as usize {
                for v in
                    &px[r * w as usize + i as usize..r * w as usize + (i + i64::from(n)) as usize]
                {
                    if v.is_nan() {
                        bad = true;
                        break 'rows;
                    }
                    lo = lo.min(*v);
                    hi = hi.max(*v);
                }
            }
            if !bad {
                let g = &m.geo;
                let centre = (
                    g.x((i0 + i) as f64 + f64::from(n / 2)),
                    g.y((j) as f64 + f64::from(n / 2)),
                );
                found.push((hi - lo, centre, lo, hi));
            }
            i += step;
        }
        j += step;
    }
    found.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (r, c, lo, hi) in found.iter().take(10) {
        println!(
            "centre {:.1},{:.1}  relief {r:.1} ({lo:.1}..{hi:.1})",
            c.0, c.1
        );
    }
    if found.is_empty() {
        println!("no window of {n} samples has data everywhere");
    }
    Ok(())
}

/// A hillshade of the mosaic at one pixel in `every`, north up, with no data in red: for choosing a
/// window by eye. Each pixel is the mosaic's own (no filter), lit from the north-west.
fn preview(a: &Args) -> Result<(), String> {
    let (out, files) = a
        .plain
        .split_first()
        .ok_or(format!("preview wants OUT.png and files\n\n{USAGE}"))?;
    let every: i64 = a
        .one("every")
        .map_or(Ok(10), |s| s.parse().map_err(|_| "--every"))?;
    let mut m = elevation::dem::Mosaic::open(files)?;
    let (mut i1, mut j1) = (0i64, 0i64);
    for (r, (i, j)) in m.rasters.iter().zip(&m.at) {
        i1 = i1.max(i + i64::from(r.geo.width));
        j1 = j1.max(j + i64::from(r.geo.height));
    }
    let (mut i0, mut j0) = m.at.iter().fold((i64::MAX, i64::MAX), |(a, b), (i, j)| {
        (a.min(*i), b.min(*j))
    });
    // Only a square of `--size` pixels around `--centre`, if given.
    if let Some(c) = a.one("centre") {
        let (x, y): (f64, f64) = pair(&c)?;
        let size: i64 = a.need("size")?.parse().map_err(|_| "--size")?;
        let (ic, jc) = m.pixel_of(x, y);
        (i0, j0, i1, j1) = (
            ic - size / 2,
            jc - size / 2,
            ic - size / 2 + size,
            jc - size / 2 + size,
        );
    }
    let (w, h) = (((i1 - i0) / every) as usize, ((j1 - j0) / every) as usize);
    let mut z = vec![f32::NAN; w * h];
    // A strip of `every` rows at a time; its first row is the one kept.
    for y in 0..h {
        let (px, _) = m.window(i0, j0 + y as i64 * every, (i1 - i0) as u32, 1)?;
        for x in 0..w {
            z[x + y * w] = px[x * every as usize];
        }
    }
    let d = (every as f64 * m.geo.dx) as f32;
    let mut rgb = vec![0u8; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let at = |x: usize, y: usize| z[x.min(w - 1) + y.min(h - 1) * w];
            let c = at(x, y);
            let px = &mut rgb[3 * (x + y * w)..3 * (x + y * w) + 3];
            let (gx, gy) = ((at(x + 1, y) - c) / d, (at(x, y + 1) - c) / d);
            if !(c.is_finite() && gx.is_finite() && gy.is_finite()) {
                px.copy_from_slice(&[200, 0, 0]);
                continue;
            }
            // Light from the north-west, 45 degrees up; +y in the picture is south.
            let n = (gx * gx + gy * gy + 1.0).sqrt();
            let l = ((gx - gy) * -0.5 + 0.7071) / n;
            px.fill((l.clamp(0.0, 1.0) * 255.0) as u8);
        }
    }
    game_render::png::write_rgb(std::path::Path::new(out), w, h, &rgb)?;
    println!(
        "{out}: {w}×{h}, one pixel every {every} ({} a pixel), west {:.0} north {:.0}",
        d,
        m.geo.x(i0 as f64) - m.geo.dx / 2.0,
        m.geo.y(j0 as f64) + m.geo.dy / 2.0
    );
    Ok(())
}
