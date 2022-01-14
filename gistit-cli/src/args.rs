/// Gistit command line interface
use clap::{crate_authors, crate_description, crate_version, App, Arg, ArgGroup, ValueHint};

/// The gistit application
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn app() -> App<'static> {
    App::new("gistit-cli")
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .arg(
            Arg::new("colorschemes")
                .long("colorschemes")
                .help("List available colorschemes"),
        )
        .arg(
            Arg::new("silent")
                .long("silent")
                .help("Silent mode, omit stdout")
                .global(true),
        )
        .arg(
            Arg::new("config-init")
                .long("config-init")
                .help("Initialize the default settings.yaml file into the project config directory")
                .long_help(
                    "Initialize the default settings.yaml file into the project config directory.
This flag can be also used to reset settings to default.
Beware to mistakenly overwriting your settings.",
                )
                .global(true),
        )
        .subcommand(
            App::new("send")
                .alias("s")
                .about("Send the gistit to the cloud")
                .arg(
                    Arg::new("file")
                        .long("file")
                        .short('f')
                        .allow_invalid_utf8(true)
                        .help("The file to be sent [required]")
                        .required(true)
                        .multiple_occurrences(false) // currently not supported
                        .takes_value(true)
                        .value_hint(ValueHint::FilePath),
                )
                .arg(
                    Arg::new("description")
                        .long("description")
                        .short('d')
                        .help("With a description")
                        .takes_value(true)
                        .requires("file")
                )
                .arg(
                    Arg::new("author")
                        .long("author")
                        .short('a')
                        .help("With author information. Defaults to a random generated name")
                        .takes_value(true)
                        .requires("file")
                        .value_hint(ValueHint::Username),
                )
                .arg(
                    Arg::new("lifespan")
                        .long("lifespan")
                        .short('l')
                        .help("With a custom lifespan")
                        .requires("file")
                        .takes_value(true)
                        .default_value("3600"),
                )
                .arg(
                    Arg::new("secret")
                        .long("secret")
                        .short('s')
                        .help("With password encryption")
                        .takes_value(true)
                        .requires("file"),
                )
                .arg(
                    Arg::new("theme")
                        .long("theme")
                        .short('t')
                        .requires("file")
                        .takes_value(true)
                        .help("The color scheme to apply syntax highlighting")
                        .long_help(
                            "The color scheme to apply syntax highlighting.
Run `gistit --colorschemes` to list available ones.",
                        ),
                )
                .arg(
                    Arg::new("clipboard")
                        .long("clipboard")
                        .short('c')
                        .requires("file")
                        .help("Copies the result hash to the system clipboard")
                        .long_help(
                            "Copies the result hash to the system clipboard.
This program will attempt to find a suitable clipboard program in your system and use it.
If none was found it defaults to ANSI escape sequence OSC52.
This is our best efforts at persisting the hash into the system clipboard after the program exits.
",
                        ),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .requires("file")
                        .short('r')
                        .help("Executes gistit-send in 'dry run' mode"),
                ),
        )
        .subcommand(
            App::new("fetch")
                .alias("f")
                .about("Fetch a gistit wherever it is")
                .arg(
                    Arg::new("hash")
                        .long("hash")
                        .short('x')
                        .help("Fetch a gistit via it's hash")
                        .required_unless_present("url")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("preview")
                        .long("preview")
                        .help("Immediately preview the gistit after successfully fetching"),
                )
                .arg(
                    Arg::new("save")
                        .long("save")
                        .help("Save the gistit to local fs after successfully fetching")
                        .long_help(
                            "Save the gistit to local fs after successfully fetching.
Target directory defaults to 'XDG user directory' on Linux, 'Known Folder' system on Windows,
and 'Standard Directories' on MacOS.",
                        ),
                )
                .arg(
                    Arg::new("url")
                        .long("url")
                        .short('u')
                        .required_unless_present("hash")
                        .help("Fetch and open a gistit on your default browser")
                        .takes_value(true)
                        .value_hint(ValueHint::Url),
                )
                .arg(
                    Arg::new("secret")
                        .long("secret")
                        .short('s')
                        .help("The secret to decrypt the fetched gistit")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("theme")
                        .long("theme")
                        .short('t')
                        .takes_value(true)
                        .help("The color scheme to apply syntax highlighting")
                        .long_help(
                            "The color scheme to apply syntax highlighting.
Run `gistit --colorschemes` to list available ones.",
                        ),
                )
        )
        .subcommand(
            App::new("host")
                .alias("h")
                .about("Host a gistit for p2p transfer")
                .group(ArgGroup::new("process_cmd").required(true))
                .arg(
                    Arg::new("status")
                        .long("status")
                        .help("Display the status of your gistit network node process")
                        .group("process_cmd")
                        .conflicts_with_all(&["secret", "file", "start", "stop"]),
                )
                .arg(
                    Arg::new("start")
                        .long("start")
                        .help("Start encrypted private network node.")
                        .long_help(
                            "Spawn the gistit network node background process to enable peer 
to peer file sharing.",
                        )
                        .group("process_cmd")
                        .conflicts_with_all(&["secret", "file", "stop", "status"]),
                )
                .arg(
                    Arg::new("seed")
                        .long("seed")
                        .help("Seed to derive your ed25519 keypair under peer to peer connections.")
                        .long_help("Seed to derive your ed25519 keypair under peer to peer connections.
Use this to have a peristing keypair that enables your peers to recognize you in future connections, provided you entered the same seed.")
                        .takes_value(true)
                        .value_name("seed")
                        .requires("start")
                        .conflicts_with_all(&["secret", "file", "stop", "status"]),
                )
                .arg(
                    Arg::new("clipboard")
                        .long("clipboard")
                        .requires("start")
                        .help("Attempt to copy your gistit node hash into clipboard")
                        .conflicts_with_all(&["secret", "file", "stop", "status"]),
                )
                .arg(
                    Arg::new("listen")
                        .long("listen")
                        .help("The Ipv4 address used to listen for inbound connections. Defaults to '127.0.0.1:0'")
                        .long_help("The Ipv4 address used to listen for inbound connections. 
Defaults to '127.0.0.1:0', which means (localhost:random_port)")
                        .takes_value(true)
                        .value_name("address:port")
                        .default_value("127.0.0.1:0")
                        .value_hint(ValueHint::Hostname)
                        .conflicts_with_all(&["secret", "file", "stop", "status"]),
                )
                .arg(
                    Arg::new("stop")
                        .long("stop")
                        .group("process_cmd")
                        .help("Stop gistit node background process")
                        .conflicts_with_all(&["start", "secret", "file", "status"]),
                )
                .arg(
                    Arg::new("secret")
                        .long("secret")
                        .short('s')
                        .help("Encrypts the target file with a secret.")
                        .takes_value(true)
                        .conflicts_with_all(&["stop", "status"]),
                )
                .arg(
                    Arg::new("file")
                        .long("file")
                        .short('f')
                        .allow_invalid_utf8(true)
                        .help("Appends this file to your hosted gistits")
                        .multiple_occurrences(false)
                        .takes_value(true)
                        .value_hint(ValueHint::FilePath)
                        .conflicts_with_all(&["stop", "status"]),
                ),
        )
}
