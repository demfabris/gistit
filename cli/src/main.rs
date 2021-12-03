use lib_gistit::cli::app;
use lib_gistit::dispatch::Dispatch;
use lib_gistit::{dispatch_from_args, gistit_error, list_bat_colorschemes, Result, CURRENT_ACTION};

async fn run() -> Result<()> {
    let matches = app().get_matches();
    CURRENT_ACTION
        .set(matches.subcommand().0.to_string())
        .expect("Internal error");
    println!("{:?}", matches);
    match matches.subcommand() {
        ("send", Some(args)) => dispatch_from_args!(lib_gistit::send, args),
        ("fetch", Some(args)) => dispatch_from_args!(lib_gistit::fetch, args),
        ("", None) => {
            // Global commands
            if matches.is_present("colorschemes") {
                list_bat_colorschemes();
                std::process::exit(0);
            }
            if matches.is_present("silent") {
                println!("WIP");
                std::process::exit(0);
            }
            app().print_help().expect("Couldn't write to stdout");
        }
        _ => (),
    };
    Ok(())
}

#[tokio::main]
async fn main() {
    // Top level error output
    if let Err(err) = run().await {
        gistit_error!(err);
    };
}
