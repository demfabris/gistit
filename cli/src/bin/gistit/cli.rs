//! Gistit command line interface

use clap::{crate_description, crate_version, App, AppSettings, Arg, SubCommand};

/// The gistit application
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn app() -> App<'static, 'static> {
    App::new("Gistit")
        .version(crate_version!())
        .global_setting(AppSettings::ColoredHelp)
        .about(crate_description!())
        .arg(
            Arg::with_name("colorschemes")
                .long("colorschemes")
                .help("List available colorschemes"),
        )
        .arg(
            Arg::with_name("silent")
                .long("silent")
                .help("Silent mode, omit stdout")
                .global(true),
        )
        .arg(
            Arg::with_name("config-init")
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
            SubCommand::with_name("send")
                .alias("s")
                .about("Send the gistit to the cloud")
                .arg(
                    Arg::with_name("file")
                        .long("file")
                        .short("f")
                        .help("The file to be sent [required]")
                        .required(true)
                        .multiple(false) // currently not supported
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("description")
                        .long("description")
                        .short("d")
                        .help("With a description")
                        .takes_value(true)
                        .requires("file"),
                )
                .arg(
                    Arg::with_name("author")
                        .long("author")
                        .short("a")
                        .help("With author information. Defaults to a random generated name")
                        .takes_value(true)
                        .requires("file"),
                )
                .arg(
                    Arg::with_name("lifespan")
                        .long("lifespan")
                        .short("l")
                        .help("With a custom lifespan")
                        .requires("file")
                        .takes_value(true)
                        .default_value("3600"),
                )
                .arg(
                    Arg::with_name("secret")
                        .long("secret")
                        .short("s")
                        .help("With password encryption")
                        .takes_value(true)
                        .requires("file"),
                )
                .arg(
                    Arg::with_name("theme")
                        .long("theme")
                        .short("t")
                        .requires("file")
                        .takes_value(true)
                        .help("The color scheme to apply syntax highlighting")
                        .long_help(
                            "The color scheme to apply syntax highlighting.
Run `gistit --colorschemes` to list available ones.",
                        ),
                )
                .arg(
                    Arg::with_name("clipboard")
                        .long("clipboard")
                        .short("c")
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
                    Arg::with_name("dry-run")
                        .long("dry-run")
                        .requires("file")
                        .short("r")
                        .help("Executes gistit-send in 'dry run' mode"),
                ),
        )
        .subcommand(
            SubCommand::with_name("fetch")
                .alias("f")
                .about("Fetch a gistit wherever it is")
                .arg(
                    Arg::with_name("hash")
                        .short("h")
                        .help("Fetch a gistit via it's hash")
                        .required_unless("url")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("preview")
                        .long("preview")
                        .help("Immediately preview the gistit after successfully fetching"),
                )
                .arg(
                    Arg::with_name("save")
                        .long("save")
                        .help("Save the gistit to local fs after successfully fetching")
                        .long_help(
                            "Save the gistit to local fs after successfully fetching.
Target directory defaults to 'XDG user directory' on Linux, 'Known Folder' system on Windows,
and 'Standard Directories' on MacOS.",
                        ),
                )
                .arg(
                    Arg::with_name("url")
                        .short("u")
                        .required_unless("hash")
                        .help("Fetch and open a gistit on your default browser")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("secret")
                        .short("s")
                        .help("The secret to decrypt the fetched gistit")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("theme")
                        .long("theme")
                        .short("t")
                        .takes_value(true)
                        .help("The color scheme to apply syntax highlighting")
                        .long_help(
                            "The color scheme to apply syntax highlighting.
Run `gistit --colorschemes` to list available ones.",
                        ),
                )
                .arg(
                    Arg::with_name("no-syntax-highlighting")
                        .long("no-syntax-highlighting")
                        .help("Disable syntax highlighting"),
                ),
        )
}
