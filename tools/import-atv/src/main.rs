use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
import-atv: MX vs. ATV Unleashed's quads as a machine pack (standard §7)

  import-atv [OUT]                    build OUT/mxvatv_atv/ (default out/mods/machines) off the disc
  import-atv --disc IMAGE --out OUT   the same off that image (build.sh)
  import-atv check DIR                read a built pack's models, clips, textures and sounds back

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

fn run(args: &[String]) -> Result<(), String> {
    match args {
        [a, image, b, out] if a == "--disc" && b == "--out" => one(Path::new(image), Path::new(out)),
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
