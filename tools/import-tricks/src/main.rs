use import_tricks::{PACKS, Pack, describe, file, key_bits};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
import-tricks: MX vs. ATV Unleashed's and Untamed's tricks as trick packs (standard §6)

  import-tricks GAME|all [OUT]         build OUT/<id>/ (default out/mods/tricks) off the game's disc
  import-tricks --disc IMAGE --out OUT  the same, the pack chosen by the disc's boot ELF (build.sh)
  import-tricks table GAME             print each stunt table as the game's loader reads it
  import-tricks check DIR              read a built pack's tables and clips back

GAME is mxvatv or untamed (or the pack's id); its disc, and MX Unleashed's for the comparison, are
looked for in iso/ ($DISCX_ISO overrides), or GAME=path/to/image.iso picks the image. MX vs. ATV
Unleashed's PC install may stand in for its disc, here and as --disc: mxvatv=path/to/data (its
data/*.pak; a .pak, or several joined with `:`), or mxvatv=pc for $DISCX_PC (discx's pc_install).";

fn iso_dir() -> PathBuf {
    std::env::var_os("DISCX_ISO").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("iso"))
}

fn one(pack: &Pack, image: Option<&Path>, out: &Path) -> Result<(), String> {
    let image = match image {
        Some(p) => p.to_path_buf(),
        None => discx::resolve_image(pack.game, &iso_dir())?,
    };
    let built = import_tricks::import(&image, pack, &iso_dir(), out)?;
    for r in &built.tables {
        let openmx = match &r.openmx {
            Ok(n) => format!("openmx reads {n}"),
            Err(e) => format!("openmx refuses it: {e}"),
        };
        println!("{}: {} stunts; {openmx}", file(&r.member), r.table.stunts.len());
    }
    let bad = built.clips.iter().filter(|c| c.read.is_err()).count();
    println!("{} clips, {bad} refused by openmx's reader; new over MX Unleashed: {}", built.clips.len(), built.new.join(", "));
    println!("{}: {} files", built.dir.display(), built.members.len());
    for t in &built.todo {
        println!("  todo: {t}");
    }
    Ok(())
}

fn table(pack: &Pack, image: Option<&Path>) -> Result<(), String> {
    let image = match image {
        Some(p) => p.to_path_buf(),
        None => discx::resolve_image(pack.game, &iso_dir())?,
    };
    let mut disc = discx::Disc::open_any(&image)?;
    for &t in pack.tables {
        let e = disc.find(t).cloned().ok_or_else(|| format!("{t}: not on the disc"))?;
        let bytes = disc.read(&e)?;
        let table = import_tricks::Table::parse(&import_tricks::ini_text(t, &bytes)?, pack)?;
        println!("== {t}: {} stunts", table.stunts.len());
        for s in &table.stunts {
            let key = s.key.as_deref().map(|k| format!("{k} ({})", describe(key_bits(k)))).unwrap_or_default();
            println!(
                "{:<8} {:<18} {:<22} hold {:<5} safe {:<5} dur {:<5} pts {:<6} hold pts {:<6} bike {:<5} {key}",
                s.section,
                s.name,
                s.display_name,
                s.hold_time,
                s.safe_time,
                s.duration,
                s.point_value,
                s.hold_point_value,
                s.used_for_bike.map(|b| b.to_string()).unwrap_or_default()
            );
        }
        if let Some(d) = &table.depreciation {
            println!("[Depreciation] {} / restore one per {} / max after {}", d.pct, d.restore_one, d.max_depletion);
        }
    }
    Ok(())
}

fn run(args: &[String]) -> Result<(), String> {
    let pick = |game: &str| -> Result<(&'static Pack, Option<PathBuf>), String> {
        let (name, image) = match game.split_once('=') {
            Some((g, p)) => (g, Some(discx::resolve_source(p, &iso_dir())?)),
            None => (game, None),
        };
        let pack = Pack::by_game(name).ok_or_else(|| format!("{name}: not mxvatv or untamed"))?;
        Ok((pack, image))
    };
    match args {
        [a, image, b, out] if a == "--disc" && b == "--out" => {
            let disc = discx::Disc::open_any(Path::new(image))?;
            let key = disc.game.map(|g| g.key).unwrap_or_default();
            let pack = Pack::by_game(key)
                .ok_or_else(|| format!("{image}: {} has no trick pack here", disc.boot.as_deref().unwrap_or("no boot ELF")))?;
            one(pack, Some(Path::new(image)), Path::new(out))
        }
        [cmd, game] if cmd == "table" => {
            let (pack, image) = pick(game)?;
            table(pack, image.as_deref())
        }
        [cmd, dir] if cmd == "check" => {
            let (tables, clips) = import_tricks::check(Path::new(dir))?;
            for r in &tables {
                println!("{}: {} stunts, openmx: {:?}", file(&r.member), r.table.stunts.len(), r.openmx);
            }
            // Refusals grouped by reason, a few files each.
            let mut why: std::collections::BTreeMap<&str, Vec<&str>> = Default::default();
            for c in &clips {
                if let Err(e) = &c.read {
                    why.entry(e.as_str()).or_default().push(&c.member);
                }
            }
            let bad: usize = why.values().map(Vec::len).sum();
            println!("{} clips read by openmx's skin::Clip::parse, {bad} refused", clips.len() - bad);
            for (e, files) in &why {
                println!("  {} x {e}: {}{}", files.len(), files[..files.len().min(3)].join(", "), if files.len() > 3 { ", ..." } else { "" });
            }
            Ok(())
        }
        [game, rest @ ..] if rest.len() <= 1 => {
            let out = rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("out/mods/tricks"));
            if game == "all" {
                return PACKS.iter().try_for_each(|p| one(p, None, &out));
            }
            let (pack, image) = pick(game)?;
            one(pack, image.as_deref(), &out)
        }
        _ => Err("bad arguments".into()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" || args[0] == "help" {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("import-tricks: {e}");
            ExitCode::FAILURE
        }
    }
}
