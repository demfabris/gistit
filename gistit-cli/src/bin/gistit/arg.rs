/// Gistit command line interface
use clap::{crate_authors, crate_description, crate_version, App, Arg, ArgGroup, ValueHint};

/// The gistit application
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn app() -> App<'static> {
    let random_name = Box::leak(Box::new(names::Generator::default().next().unwrap()));

    App::new("gistit-cli")
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .after_help(
            "Note: `gistit -h` prints a short and concise overview while `gistit --help` gives all \
                 details.",
        )
        .arg(
            Arg::new("FILE")
                .help("File to send/upload.")
                .allow_invalid_utf8(true)
                .takes_value(true)
                .value_hint(ValueHint::FilePath)
        )
        .arg(
            Arg::new("description")
                .long("description")
                .short('d')
                .help("With a description")
                .takes_value(true)
        )
        .arg(
            Arg::new("author")
                .long("author")
                .short('a')
                .help("With author information. Defaults to a random generated name")
                .takes_value(true)
                .default_value(random_name)
                .value_hint(ValueHint::Username),
        )
        .arg(
            Arg::new("clipboard")
                .long("clipboard")
                .short('c')
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
            Arg::new("list-colorschemes")
                .long("list-colorschemes")
                .conflicts_with("FILE")
                .help("List available colorschemes"),
        )
        .subcommand(
            App::new("fetch")
                .alias("f")
                .about("Fetch a gistit wherever it is")
                .arg(
                    Arg::new("HASH")
                        .help("Fetch a gistit via it's hash")
                        .takes_value(true)
                        .required(true),
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
                    Arg::new("colorscheme")
                        .long("colorscheme")
                        .takes_value(true)
                        .help("The colorscheme to apply syntax highlighting")
                        .long_help(
                            "The colorscheme to apply syntax highlighting.
Run `gistit --colorschemes` to list available ones.",
                        ),
                )
        )
        .subcommand(
            App::new("node")
                .alias("n")
                .about("Start a p2p gistit node for file transfer")
                .group(ArgGroup::new("daemon_cmd"))
                .group(ArgGroup::new("other"))
                .arg(
                    Arg::new("start")
                        .long("start")
                        .help("Start encrypted private network node.")
                        .group("daemon_cmd")
                        .long_help(
                            "Spawn the gistit network node background process to enable peer 
to peer file sharing.")
                        .conflicts_with_all(&["join", "stop", "status"]),
                )
                .arg(
                    Arg::new("stop")
                        .long("stop")
                        .group("daemon_cmd")
                        .help("Stop gistit node background process")
                        .conflicts_with_all(&["start", "status"]),
                )
                .arg(
                    Arg::new("status")
                        .long("status")
                        .group("daemon_cmd")
                        .help("Display the status of your gistit network node process")
                        .conflicts_with_all(&["start", "stop"]),
                )
                .arg(
                    Arg::new("host")
                        .long("host")
                        .help("The Ipv4 address used to listen for inbound connections. Defaults to '127.0.0.1'")
                        .takes_value(true)
                        .value_name("ivp4-address")
                        .default_value("0.0.0.0")
                        .value_hint(ValueHint::Hostname)
                        .conflicts_with_all(&["stop", "status"]),
                )
                .arg(
                    Arg::new("port")
                        .long("port")
                        .help("The port to listen for connections. Defaults to 0 (random open port)")
                        .takes_value(true)
                        .value_name("port")
                        .default_value("0")
                        .conflicts_with_all(&["stop", "status"]),
                )
                .arg(
                    Arg::new("join")
                        .long("join")
                        .help("Join a private network.")
                        .takes_value(true)
                        .value_name("address")
                        .group("other")
                        .conflicts_with_all(&["start", "stop", "status"]),
                )
                .arg(
                    Arg::new("clipboard")
                        .long("clipboard")
                        .short('c')
                        .requires("start")
                        .help("Attempt to copy your gistit node id into clipboard")
                        .conflicts_with_all(&["stop", "status"]),
                )
        )
}