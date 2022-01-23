use clap::{crate_authors, crate_description, crate_version, App, Arg, ValueHint};
use std::env::temp_dir;

#[must_use]
pub fn app() -> App<'static> {
    App::new("gistit_daemon")
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .arg(
            Arg::new("runtime-dir")
                .long("runtime-dir")
                .help("Directory to cache peers")
                .takes_value(true)
                .value_name("directory")
                .default_value_os({
                    let path = Box::leak(Box::new(temp_dir()));
                    path.as_os_str()
                })
                .value_hint(ValueHint::DirPath)
                .allow_invalid_utf8(true),
        )
        .arg(
            Arg::new("seed")
                .long("seed")
                .help("Seed to derive keypair")
                .takes_value(true)
                .value_name("seed")
                .default_value("none"),
        )
}
