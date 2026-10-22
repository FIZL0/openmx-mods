use import_music::{Pack, PACKS};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
import-music: MX vs. ATV Unleashed's and Untamed's songs as music packs (standard §4)

  import-music GAME|all [OUT]   build OUT/<id>/ (default out/mods/music) off the game's disc
  import-music --disc IMAGE --out OUT   the same, the pack chosen by the disc's boot ELF (build.sh)
  import-music check DIR        read every song of a built pack back with openmx's reader

GAME is mxvatv or untamed (or the pack's id); its disc is looked for in iso/ ($DISCX_ISO overrides),
or GAME=path/to/image.iso picks the image.";

fn iso_dir() -> PathBuf {
    std::env::var_os("DISCX_ISO").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("iso"))
}

fn one(pack: &Pack, image: Option<&Path>, out: &Path) -> Result<(), String> {
    let image = match image {
        Some(p) => p.to_path_buf(),
        None => discx::resolve_image(pack.game, &iso_dir())?,
    };
    let built = import_music::import(&image, pack, out)?;
    for p in &built.songs {
        let r = &p.read;
        println!(
            "{:<52} {:>9} bytes  {} Hz x{}  {:>7.2} s  L/R {:.2}  {} - {}",
            p.song.file(),
            p.bytes,
            r.rate,
            r.channels,
            r.seconds(),
            r.correlation,
            p.song.artist,
            p.song.title
        );
    }
    println!("{}: {} songs", built.dir.display(), built.songs.len());
    for t in &built.todo {
        println!("  todo: {t}");
    }
    Ok(())
}

fn run(args: &[String]) -> Result<(), String> {
    match args {
        [a, image, b, out] if a == "--disc" && b == "--out" => {
            let disc = discx::Disc::open(Path::new(image))?;
            let key = disc.game.map(|g| g.key).unwrap_or_default();
            let pack = Pack::by_game(key).ok_or_else(|| format!("{image}: {} has no music pack here", disc.boot.as_deref().unwrap_or("no boot ELF")))?;
            one(pack, Some(Path::new(image)), Path::new(out))
        }
        [cmd, dir] if cmd == "check" => {
            for (file, r) in import_music::check(Path::new(dir))? {
                let bad = r.problems();
                println!("{file}: {} Hz x{}, {:.2} s{}", r.rate, r.channels, r.seconds(), if bad.is_empty() { String::new() } else { format!("; {}", bad.join("; ")) });
            }
            Ok(())
        }
        [game, rest @ ..] if rest.len() <= 1 => {
            let out = rest.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("out/mods/music"));
            let (name, image) = match game.split_once('=') {
                Some((g, p)) => (g, Some(Path::new(p))),
                None => (game.as_str(), None),
            };
            if name == "all" && image.is_none() {
                return PACKS.iter().try_for_each(|p| one(p, None, &out));
            }
            let pack = Pack::by_game(name).ok_or_else(|| format!("{name}: not mxvatv, untamed or all"))?;
            one(pack, image, &out)
        }
        _ => Err("bad arguments".into()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("import-music: {e}\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}
