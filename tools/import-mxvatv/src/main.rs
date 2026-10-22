//! `import-mxvatv`: MX vs. ATV Unleashed's and Untamed's maps as track mods. `import-mxvatv --help`
//! for usage.

use discx::Disc;
use import_mxvatv::{BOOT, Map, import, load_map, map_folders, untamed};
use modcheck::{Discs, check};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "import-mxvatv: MX vs. ATV Unleashed's and Untamed's maps as track mods (standard §2)

  import-mxvatv --disc IMAGE --out STAGE [--map NAME,...] [--no-check]

MX vs. ATV Unleashed (SLUS_211.04): writes STAGE/mxvatv_<map>/ for every map on the disc
(Env\\<MAP>\\<map>.res, with its _mat and _mst archives). MX vs. ATV Untamed (SLUS_217.01): writes
STAGE/untamed_<map>/ for every map (res\\env\\<MAP>\\<MAP>.res, _mat, _mst) that Env\\Master.atyList
gives a Free Ride or a national, its binary scene decoded beside it. --map takes any map archive's
folder instead. Each mod holds the archives' members at their own paths, the loose PSR members the
game opens for the map, and a mod.json. Each mod is then read back with modcheck against the same
disc (the image's folder is searched for it by boot ELF); --no-check skips that. A map that fails is
removed from STAGE and named; the exit is 1 if any did. Any other disc is skipped with a line.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help" || a == "help") {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let (mut disc_path, mut out, mut only, mut prove) = (None, None, None::<Vec<String>>, true);
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--disc" => disc_path = it.next().map(PathBuf::from),
            "--out" => out = it.next().map(PathBuf::from),
            "--map" => only = it.next().map(|m| m.split(',').map(|s| s.trim().to_ascii_lowercase()).collect()),
            "--no-check" => prove = false,
            // build.sh's `after` steps get --mods; this is a disc step, so it has no use for it.
            "--mods" => {
                it.next();
            }
            other => {
                eprintln!("import-mxvatv: unknown argument {other} (--help)");
                return ExitCode::from(2);
            }
        }
    }
    let (Some(disc_path), Some(out)) = (disc_path, out) else {
        eprintln!("import-mxvatv: --disc and --out are both needed (--help)");
        return ExitCode::from(2);
    };
    // The boot ELF first: another game's disc has no MXVSATV.PSR to open.
    let iso_dir = disc_path.parent().map(|p| if p.as_os_str().is_empty() { PathBuf::from(".") } else { p.to_path_buf() });
    let boot = iso_dir
        .as_deref()
        .and_then(|d| discx::images(d).ok())
        .and_then(|all| all.into_iter().find(|(p, _)| same_file(p, &disc_path)))
        .and_then(|(_, b)| b)
        .unwrap_or_default();
    let is_untamed = boot.eq_ignore_ascii_case(untamed::BOOT);
    if !boot.eq_ignore_ascii_case(BOOT) && !is_untamed {
        println!(
            "import-mxvatv: {} is {}, not {BOOT} or {}; nothing imported",
            disc_path.display(),
            if boot.is_empty() { "no disc this repo knows" } else { &boot },
            untamed::BOOT
        );
        return ExitCode::SUCCESS;
    }
    let mut disc = match Disc::open(&disc_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("import-mxvatv: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("import-mxvatv: {}: {e}", out.display());
        return ExitCode::FAILURE;
    }
    let master = if is_untamed {
        match untamed::master(&mut disc) {
            Ok(m) => Some(m),
            Err(e) => {
                eprintln!("import-mxvatv: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        None
    };
    let mut folders = match &master {
        None => map_folders(&disc),
        Some(_) => untamed::map_folders(&disc),
    };
    if let (Some(m), None) = (&master, &only) {
        folders.clear();
        for (f, pick) in untamed::chosen(&disc, &m.1) {
            match pick {
                Ok(()) => folders.push(f),
                Err(why) => println!("skip {f}: {why}"),
            }
        }
    }
    if let Some(only) = &only {
        for m in only {
            if !folders.iter().any(|f| f.eq_ignore_ascii_case(m)) {
                eprintln!("import-mxvatv: no map {m} on the disc");
                return ExitCode::FAILURE;
            }
        }
        folders.retain(|f| only.contains(&f.to_ascii_lowercase()));
    }
    let mut discs = Discs::new(iso_dir);
    let (mut ok, mut failed) = (0, 0);
    for folder in &folders {
        let loaded: Result<Map, String> = match &master {
            None => load_map(&mut disc, folder),
            Some(m) => untamed::load_map(&mut disc, folder, m),
        };
        let result = loaded.and_then(|m| import(&m, &out));
        match result {
            Err(e) => {
                println!("FAIL {folder}: {e}");
                failed += 1;
            }
            Ok(m) => {
                if prove {
                    let r = check(&m.dir, &mut discs);
                    if !r.ok() {
                        println!("FAIL {} ({folder}): the read-back failed; removed", m.id);
                        for f in &r.fail {
                            println!("    fail: {f}");
                        }
                        let _ = std::fs::remove_dir_all(&m.dir);
                        failed += 1;
                        continue;
                    }
                }
                println!(
                    "ok   {} {:?}: {} files, {} todo{}{}",
                    m.id,
                    m.name,
                    m.files,
                    m.todo.len(),
                    if m.spawn.is_some() { ", spawn from the activity list" } else { "" },
                    if prove { ", read back" } else { "" }
                );
                ok += 1;
            }
        }
    }
    println!("import-mxvatv: {ok} maps imported, {failed} failed");
    if failed > 0 { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

fn same_file(a: &std::path::Path, b: &std::path::Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}
