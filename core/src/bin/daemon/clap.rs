use clap::{crate_authors, crate_description, crate_version, App, Arg, ValueHint};
use std::env::temp_dir;

#[must_use]
pub fn app() -> App<'static> {
    App::new("gistit_daemon")
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .arg(
            Arg::new("host")
                .long("host")
                .help("The ipv4 address to listen for connections")
                .default_value("127.0.0.1")
                .takes_value(true)
                .value_name("ipv4")
                .value_hint(ValueHint::Hostname)
                .validator(|input| input.parse::<std::net::Ipv4Addr>()),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .help("The port that will be used for connections")
                .default_value("0")
                .takes_value(true)
                .value_name("port")
                .validator(|input| input.parse::<u16>()),
        )
        .arg(
            Arg::new("runtime-dir")
                .long("runtime-dir")
                .help("Directory to cache peers")
                .required(true)
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
        .arg(Arg::new("persist").long("persist").help("Persist peers"))
}
