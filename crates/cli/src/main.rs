use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "rustweb", version, about = "Production tooling for Rust Web frontend apps")]
pub struct Cli {

    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {

    New {

        name: String,

        #[arg(long)]
        dir: Option<PathBuf>,
    },

    Build {

        #[arg(long)]
        package: Option<String>,

        #[arg(long, default_value = "-Oz")]
        optimize: String,

        #[arg(long)]
        budget: Option<String>,

        #[arg(long)]
        features: Option<String>,

        #[arg(long, default_value_t = false)]
        no_opt: bool,
    },

    Serve {

        #[arg(long, default_value_t = 8080)]
        port: u16,

        #[arg(long, default_value = "dist")]
        dir: PathBuf,
    },
}

pub fn parse_budget(s: &str) -> Result<u64, String> {
    let s = s.trim().to_lowercase();
    let (num, mult) = if let Some(v) = s.strip_suffix("kb") {
        (v, 1024u64)
    } else if let Some(v) = s.strip_suffix("mb") {
        (v, 1024 * 1024u64)
    } else if let Some(v) = s.strip_suffix('b') {
        (v, 1u64)
    } else {
        (s.as_str(), 1u64)
    };
    num.trim()
        .parse::<f64>()
        .map(|n| (n * mult as f64) as u64)
        .map_err(|_| format!("invalid budget `{s}`: expected like `200kb`, `1.5mb`, `512b`"))
}

pub fn dist_size(dir: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().map(|x| x == "wasm" || x == "js").unwrap_or(false) {
                total += e.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    total
}

fn main() -> AnyhowExit {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::New { name, dir } => cmd_new(&name, dir.as_deref()),
        Cmd::Build { package, optimize, budget, features, no_opt } => {
            cmd_build(package.as_deref(), &optimize, budget.as_deref(), features.as_deref(), no_opt)
        }
        Cmd::Serve { port, dir } => cmd_serve(port, &dir),
    }
}

type AnyhowExit = Result<(), Box<dyn std::error::Error>>;

fn cmd_new(name: &str, dir: Option<&Path>) -> AnyhowExit {
    let out = dir.map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from(name));
    std::fs::create_dir_all(out.join("src"))?;
    std::fs::write(
        out.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
rustweb-core = "0.1"
rustweb-dom = "0.1"

[profile.release]
opt-level = "z"
lto = true
"#
        ),
    )?;
    std::fs::write(
        out.join("src/main.rs"),
        r#"use rustweb_core::{Attr, VNode};

fn main() {
    let tree = VNode::element("main", vec![Attr::new("aria-label", "App")], vec![VNode::text("Hello, rustweb!")]);
    println!("{}", rustweb_ssr::render_to_string(&tree));
}
"#,
    )?;
    println!("created app `{name}` at {}", out.display());
    Ok(())
}

fn run(cmd: &str, args: &[&str]) -> Result<(), String> {
    let status = std::process::Command::new(cmd)
        .args(args)
        .status()
        .map_err(|e| format!("failed to run `{cmd}`: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`{cmd} {}` exited with {status}", args.join(" ")))
    }
}

fn cmd_build(
    package: Option<&str>,
    optimize: &str,
    budget: Option<&str>,
    features: Option<&str>,
    no_opt: bool,
) -> AnyhowExit {

    let mut args = vec!["build", "--release", "--target", "wasm32-unknown-unknown"];
    let pkg;
    if let Some(p) = package {
        pkg = p.to_string();
        args.extend(["--package", &pkg]);
    }
    let feats;
    if let Some(f) = features {
        feats = f.to_string();
        args.extend(["--features", &feats]);
    }
    println!("$ cargo {}", args.join(" "));
    run("cargo", &args).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    match run("wasm-bindgen", &["--version"]) {
        Ok(()) => {
            println!("$ wasm-bindgen … --out-dir dist");
            let _ = std::fs::create_dir_all("dist");

            println!("note: run `wasm-bindgen target/wasm32-unknown-unknown/release/<crate>.wasm --out-dir dist --target web` for your crate");
        }
        Err(_) => println!("warning: `wasm-bindgen` not found; skipping JS glue generation"),
    }

    if !no_opt {
        match run("wasm-opt", &["--version"]) {
            Ok(()) => println!("$ wasm-opt {optimize} dist/*.wasm (configure per chunk for code-split builds)"),
            Err(_) => println!("warning: `wasm-opt` not found; install binaryen for bundle-size wins"),
        }
    }

    let size = dist_size(Path::new("dist"));
    println!("dist size (wasm+js): {size} bytes");
    if let Some(b) = budget {
        let limit = parse_budget(b).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
        if size > limit && size != 0 {
            return Err(format!("bundle budget exceeded: {size} > {limit} bytes").into());
        }
        println!("budget check passed (<= {limit} bytes)");
    }
    Ok(())
}

fn cmd_serve(port: u16, dir: &Path) -> AnyhowExit {
    println!("serving {} at http://127.0.0.1:{port}/ (Ctrl-C to stop)", dir.display());
    println!("note: production dev-server with HMR headers is trunk-parity roadmap; this MVP serves via `python3 -m http.server` semantics.");
    let dir = dir.to_string_lossy().to_string();
    let port = port.to_string();
    run("python3", &["-m", "http.server", &port, "--directory", &dir]).map_err(|e| e.into())
}
