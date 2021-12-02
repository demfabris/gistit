use console::style;

use lib_gistit::cli::app;
use lib_gistit::dispatch::Dispatch;
use lib_gistit::{dispatch_from_args, gistit_error, Result, CURRENT_ACTION};

async fn run() -> Result<()> {
    let matches = app().get_matches();
    CURRENT_ACTION
        .set(matches.subcommand().0.to_string())
        .expect("Internal error");
    match matches.subcommand() {
        ("send", Some(args)) => dispatch_from_args!(lib_gistit::send, args),
        ("fetch", Some(args)) => dispatch_from_args!(lib_gistit::fetch, args),
        _ => unimplemented!(),
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
