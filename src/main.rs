use anyhow::{anyhow, Result};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use tempfile::tempdir;

include!(concat!(env!("OUT_DIR"), "/embedded_bins.rs"));

#[derive(Debug)]
struct App {
    args: String,
}

impl App {
    fn new() -> Self {
        Self { args: String::new() }
    }

    fn pick_binary(&self) -> Result<(&'static str, &'static [u8])> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        if os == "windows" {
            return Ok(("ciadpi.exe", ciadpi_exe));
        }

        let name = match arch {
            "x86_64" => ("ciadpi-x86_64", ciadpi_x86_64),
            "aarch64" => ("ciadpi-aarch64", ciadpi_aarch64),
            "mips" => ("ciadpi-mips", ciadpi_mips),
            "i686" => ("ciadpi-i686", ciadpi_i686),
            _ => return Err(anyhow!("unknown arch: {}", arch)),
        };

        Ok(name)
    }

    fn run(&self) -> Result<()> {
        let (name, data) = self.pick_binary()?;
        let dir = tempdir()?;
        let bin_path = dir.path().join(name);

        {
            let mut f = File::create(&bin_path)?;
            f.write_all(data)?;
        }

        #[cfg(unix)]
        fs::set_permissions(&bin_path, fs::Permissions::from_mode(0o755))?;

        let args: Vec<&str> = if self.args.trim().is_empty() {
            vec![]
        } else {
            self.args.split_whitespace().collect()
        };

        println!("running {} {:?}", name, args);

        let output = Command::new(&bin_path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.stdout.is_empty() {
            println!(
                "\nstdout:\n{}",
                String::from_utf8_lossy(&output.stdout)
            );
        }
        if !output.stderr.is_empty() {
            eprintln!(
                "\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        println!("\nexited: {:?}", output.status.code());

        Ok(())
    }
}

/// read user strategies from the home folder
fn read_user_strategies() -> Vec<(String, String)> {
    let mut result = vec![];
    let dir = dirs::home_dir()
        .unwrap_or(PathBuf::from("."))
        .join(".shootdpi/strategies");

    if dir.exists() && dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let name = path.file_name().unwrap().to_string_lossy().to_string();
                        result.push((name, content.trim().to_string()));
                    }
                }
            }
        }
    }

    result
}

/// were building a single list of strategies (built-in + custom)
fn get_all_strategies() -> Vec<(String, String)> {
    let mut strategies: Vec<(String, String)> = {
        let mut vec = vec![];
        let built_in = get_strategies();
        for (k, v) in built_in.iter() {
            vec.push((k.to_string(), v.to_string()));
        }
        vec
    };

    strategies.extend(read_user_strategies());
    strategies
}

/// we show the strategy menu and select
fn choose_strategy() -> String {
    let strategies = get_all_strategies();
    if strategies.is_empty() {
        println!("no strategies available, empty argument will be used");
        return "".into();
    }

    println!("choose a strategy:");
    for (i, (name, _)) in strategies.iter().enumerate() {
        println!("  {}. {}", i + 1, name);
    }

    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let choice = input.trim().parse::<usize>().unwrap_or(1);
    let choice = choice.saturating_sub(1).min(strategies.len() - 1);

    strategies[choice].1.clone()
}

fn main() -> Result<()> {
    println!("==== ciadpi launcher ====");
    println!("os: {}", std::env::consts::OS);
    println!("arch: {}", std::env::consts::ARCH);
    println!("hint: for builtin strategies, socks ip and port proxy would be: 127.0.0.1:1080");
    println!("hint #2: you can add your strategies: ~/.shootdpi/strategies/ (*nix) or %USERPROFILE%\\.shootdpi\\strategies\\ (Windows)");

    let args = choose_strategy();

    let mut app = App::new();
    app.args = args;

    app.run()?;

    Ok(())
}
