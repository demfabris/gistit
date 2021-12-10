use clap_generate::{generate_to, generators};
use std::env;

const BIN_NAME: &str = "gistit";

include!("src/bin/gistit/cli.rs");

#[cfg(not(feature = "application"))]
fn main() {}

#[cfg(feature = "application")]
fn main() -> Result<(), String> {
    let mut app = app();
    let out_path =
        std::env::var_os("SHELL_COMPLETIONS_DIR").or_else(|| std::env::var_os("OUT_DIR"));

    let outdir = match out_path {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    generate_to(generators::Bash, &mut app, BIN_NAME, &outdir).map_err(|err| err.to_string())?;
    generate_to(generators::Zsh, &mut app, BIN_NAME, &outdir).map_err(|err| err.to_string())?;
    generate_to(generators::Fish, &mut app, BIN_NAME, &outdir).map_err(|err| err.to_string())?;
    generate_to(generators::PowerShell, &mut app, BIN_NAME, &outdir)
        .map_err(|err| err.to_string())?;
    println!(
        "cargo:warning=generated shell completion scripts at {:?}",
        outdir
    );

    Ok(())
}
