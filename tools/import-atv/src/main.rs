use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
import-atv: MX vs. ATV Unleashed's quads as a machine pack (standard §7)

  import-atv [OUT]                    build OUT/mxvatv_atv/ (default out/mods/machines) off the disc
  import-atv --disc IMAGE --out OUT   the same off that image, and the fliers' pack beside it (build.sh)
  import-atv check DIR                read a built pack's models, clips, textures and sounds back
  import-atv fliers [OUT]             the biplane and the Kiowa: OUT/mxvatv_fliers/ (openmx MD80d)
  import-atv fliers --disc IMAGE --out OUT
  import-atv fliers check DIR         read the fliers' pack back: every model, .col, clip, texture, sound

The disc is looked for in iso/ ($DISCX_ISO overrides).";

fn iso_dir() -> PathBuf {
    std::env::var_os("DISCX_ISO").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("iso"))
}

fn one(image: &Path, out: &Path) -> Result<(), String> {
    let built = import_atv::import(image, out)?;
    println!("{}: {} files, {} builds", built.dir.display(), built.members.len(), built.builds.len());
    let mut chassis: Vec<&str> = built.builds.iter().map(|b| b.chassis.as_str()).collect();
    chassis.sort();
    chassis.dedup();
    println!("chassis: {}", chassis.join(", "));
    for t in &built.todo {
        println!("  todo: {t}");
    }
    Ok(())
}

fn fliers(image: &Path, out: &Path) -> Result<(), String> {
    let built = import_atv::fliers::import(image, out)?;
    println!("{}: {} files, {} skins", built.dir.display(), built.members.len(), built.skins.len());
    for t in &built.todo {
        println!("  todo: {t}");
    }
    Ok(())
}

fn fliers_check(dir: &Path) -> Result<(), String> {
    let c = import_atv::fliers::check(dir)?;
    println!(
        "{} models, {} collision files, {} clips, {} materials, {} textures, {} sounds ({} without VAGp), {} emitters read by openmx",
        c.models, c.collisions, c.clips, c.materials, c.textures, c.sounds, c.unmarked_sounds, c.emitters
    );
    for (skin, model, nodes) in &c.skins {
        let lacks: Vec<&str> = nodes.iter().filter(|(_, got)| got.is_none()).map(|(w, _)| w.as_str()).collect();
        let other: Vec<String> = nodes.iter().filter_map(|(w, got)| got.as_ref().filter(|g| *g != w).map(|g| format!("{w} as {g}"))).collect();
        match lacks.is_empty() {
            true => println!("{skin} (model {model}): every node{}", if other.is_empty() { String::new() } else { format!(", {}", other.join(", ")) }),
            false => println!("{skin} (model {model}): lacks {}", lacks.join(" ")),
        }
    }
    for (f, why) in &c.refused {
        println!("  refused {f}: {why}");
    }
    Ok(())
}

fn run(args: &[String]) -> Result<(), String> {
    match args {
        [f, cmd, dir] if f == "fliers" && cmd == "check" => fliers_check(Path::new(dir)),
        [f, a, image, b, out] if f == "fliers" && a == "--disc" && b == "--out" => fliers(Path::new(image), Path::new(out)),
        [f, rest @ ..] if f == "fliers" && rest.len() <= 1 => {
            let out = rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("out/mods/machines"));
            let image = discx::resolve_image(import_atv::GAME, &iso_dir())?;
            fliers(&image, &out)
        }
        // build.sh's form: both machine packs off the one disc.
        [a, image, b, out] if a == "--disc" && b == "--out" => {
            one(Path::new(image), Path::new(out))?;
            fliers(Path::new(image), Path::new(out))
        }
        [cmd, dir] if cmd == "check" => {
            let c = import_atv::check(Path::new(dir))?;
            println!(
                "{} models, {} clips, {} materials, {} textures, {} sounds ({} without VAGp), {} emitters read by openmx",
                c.models, c.clips, c.materials, c.textures, c.sounds, c.unmarked_sounds, c.emitters
            );
            for (model, nodes) in &c.chassis {
                println!("{model}: {}", nodes.join(" "));
            }
            for (f, why) in &c.refused {
                println!("  refused {f}: {why}");
            }
            Ok(())
        }
        [] | [_] => {
            let out = args.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("out/mods/machines"));
            let image = discx::resolve_image(import_atv::GAME, &iso_dir())?;
            one(&image, &out)
        }
        _ => Err("bad arguments".into()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|a| a == "-h" || a == "--help" || a == "help") {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("import-atv: {e}");
            ExitCode::FAILURE
        }
    }
}
