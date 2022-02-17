use clap_complete::{generate_to, Shell};

const BIN_NAME: &str = "gistit";

include!("src/arg.rs");

fn main() -> Result<(), String> {
    let mut app = app();
    let out_path =
        std::env::var_os("SHELL_COMPLETIONS_DIR").or_else(|| std::env::var_os("OUT_DIR"));

    let outdir = match out_path {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    generate_to(Shell::Bash, &mut app, BIN_NAME, &outdir).map_err(|err| err.to_string())?;
    generate_to(Shell::Zsh, &mut app, BIN_NAME, &outdir).map_err(|err| err.to_string())?;
    generate_to(Shell::Fish, &mut app, BIN_NAME, &outdir).map_err(|err| err.to_string())?;
    generate_to(Shell::PowerShell, &mut app, BIN_NAME, &outdir).map_err(|err| err.to_string())?;
    println!(
        "cargo:warning=generated shell completion scripts at {:?}",
        outdir
    );

    Ok(())
}
