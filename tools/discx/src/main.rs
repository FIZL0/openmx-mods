//! `discx`: list and extract PSR and `.res` members off a disc image. `discx help` for usage.

use discx::{Disc, Pattern, ResFile, res_member_path, resolve_image};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = r#"discx: PSR and .res members off a PS2 disc image (MX Unleashed, MX vs. ATV Unleashed, Untamed)

  discx discs [DIR]                   the images in DIR (default iso/) and their boot ELFs
  discx info  DISC                    which game, which PSR, how many members
  discx ls    DISC [PATTERN]          members and sizes
  discx x     DISC PATTERN OUTDIR     extract, keeping paths (\ becomes /)
  discx res   FILE.res [GLOB [OUTDIR]]  a loose .res: list its members, or extract them

DISC is an image path, or a game (mxu, mxvatv, untamed, or its boot ELF) looked for in iso/
($DISCX_ISO overrides). --psr NAME opens another PSR on the disc (Untamed's MX08AUD.PSR).

PATTERN: `*` stays within a path segment, `**` crosses them, `?` is one character; `\` and `/` are
both separators and case is ignored. `OUTER:INNER` reaches into .res members:
  discx ls mxvatv 'Env\*\*.res'
  discx ls mxvatv 'Env\Nat01\nat01.res:Col\*.col'
  discx x  mxvatv 'Env\Nat01\*' extracted/mxvatv/
A .res member is extracted to OUTDIR/<res path>.d/<member path>."#;

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let psr = take_flag(&mut args, "--psr");
    match run(&args, psr.as_deref()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("discx: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `println!` that stops quietly when the reader goes away (`discx ls ... | head`) instead of panicking.
macro_rules! out {
    ($($t:tt)*) => {{
        use std::io::Write;
        if let Err(e) = writeln!(std::io::stdout().lock(), $($t)*) {
            if e.kind() == std::io::ErrorKind::BrokenPipe {
                std::process::exit(0);
            }
            return Err(format!("stdout: {e}"));
        }
    }};
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> Option<String> {
    let k = args.iter().position(|a| a == flag)?;
    args.remove(k);
    (k < args.len()).then(|| args.remove(k))
}

fn iso_dir() -> PathBuf {
    std::env::var_os("DISCX_ISO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("iso"))
}

fn open(arg: &str, psr: Option<&str>) -> Result<Disc, String> {
    Disc::open_psr(&resolve_image(arg, &iso_dir())?, psr)
}

fn run(args: &[String], psr: Option<&str>) -> Result<(), String> {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    match a.as_slice() {
        ["discs"] | ["discs", _] => {
            let dir = a.get(1).map(PathBuf::from).unwrap_or_else(iso_dir);
            for (path, boot) in discx::images(&dir)? {
                let boot = boot.as_deref().unwrap_or("-");
                let game = discx::Game::by_boot(boot).map_or("(not one of ours)", |g| g.title);
                out!("{boot}\t{game}\t{}", path.display());
            }
            Ok(())
        }
        ["info", disc] => {
            let d = open(disc, psr)?;
            out!("image\t{}", d.path.display());
            out!("boot\t{}", d.boot.as_deref().unwrap_or("-"));
            out!("game\t{}", d.game.map_or("(not one of ours)", |g| g.title));
            out!("psr\t{} at {:#x}, {} bytes", d.psr_name, d.psr_at, d.psr_size());
            out!("members\t{}", d.entries().len());
            let others: Vec<&str> = d
                .root
                .iter()
                .filter(|f| f.name.to_ascii_uppercase().ends_with(".PSR") && f.name != d.psr_name)
                .map(|f| f.name.as_str())
                .collect();
            if !others.is_empty() {
                out!("other\t{}", others.join(", "));
            }
            Ok(())
        }
        ["ls", disc] => ls(&mut open(disc, psr)?, "**"),
        ["ls", disc, pattern] => ls(&mut open(disc, psr)?, pattern),
        ["x", disc, pattern, out] => extract(&mut open(disc, psr)?, pattern, Path::new(out)),
        ["res", file] => res(file, "**", None),
        ["res", file, glob] => res(file, glob, None),
        ["res", file, glob, out] => res(file, glob, Some(Path::new(out))),
        [] | ["help"] | ["-h"] | ["--help"] => {
            out!("{USAGE}");
            Ok(())
        }
        _ => Err(format!("cannot read that command line\n\n{USAGE}")),
    }
}

fn ls(d: &mut Disc, pattern: &str) -> Result<(), String> {
    let p = Pattern::parse(pattern);
    let outer: Vec<_> = d.matching(&p.outer).into_iter().cloned().collect();
    let mut n = 0;
    for e in &outer {
        match &p.inner {
            None => {
                out!("{}\t{}", e.size, e.name);
                n += 1;
            }
            Some(inner) => {
                let r = d.res(e)?;
                for m in r.matching(inner) {
                    out!("{}\t{}:{}", m.size, e.name, m.name);
                    n += 1;
                }
            }
        }
    }
    if n == 0 {
        return Err(format!("nothing in {} matches {pattern}", d.psr_name));
    }
    Ok(())
}

fn extract(d: &mut Disc, pattern: &str, out: &Path) -> Result<(), String> {
    let p = Pattern::parse(pattern);
    let outer: Vec<_> = d.matching(&p.outer).into_iter().cloned().collect();
    let (mut n, mut bytes) = (0usize, 0u64);
    for e in &outer {
        match &p.inner {
            None => {
                let path = out.join(e.path());
                let mut f = create(&path)?;
                d.copy(e, &mut f)?;
                n += 1;
                bytes += e.size as u64;
            }
            Some(inner) => {
                let r = d.res(e)?;
                for m in r.matching(inner) {
                    let data = r.read(m).map_err(|why| format!("{}:{}: {why}", e.name, m.name))?;
                    write(&out.join(res_member_path(e, m)), &data)?;
                    n += 1;
                    bytes += data.len() as u64;
                }
            }
        }
    }
    if n == 0 {
        return Err(format!("nothing in {} matches {pattern}", d.psr_name));
    }
    eprintln!("discx: {n} files, {bytes} bytes, into {}", out.display());
    Ok(())
}

fn res(file: &str, glob: &str, out: Option<&Path>) -> Result<(), String> {
    let r = ResFile::open(Path::new(file))?;
    let matched = r.matching(glob);
    if matched.is_empty() {
        return Err(format!("nothing in {file} matches {glob}"));
    }
    let Some(out) = out else {
        for m in matched {
            out!("{}\t{}", m.size, m.name);
        }
        return Ok(());
    };
    let mut bytes = 0u64;
    for m in &matched {
        let data = r.read(m).map_err(|why| format!("{file}:{}: {why}", m.name))?;
        write(&out.join(m.path()), &data)?;
        bytes += data.len() as u64;
    }
    eprintln!("discx: {} files, {bytes} bytes, into {}", matched.len(), out.display());
    Ok(())
}

fn create(path: &Path) -> Result<std::fs::File, String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn write(path: &Path, data: &[u8]) -> Result<(), String> {
    use std::io::Write;
    create(path)?
        .write_all(data)
        .map_err(|e| format!("{}: {e}", path.display()))
}
