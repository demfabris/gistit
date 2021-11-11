use gistit::cli::app;
use gistit::dispatch::Dispatch;
use gistit::{dispatch_from_args, Result};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let matches = app().get_matches();
    match matches.subcommand() {
        ("send", Some(args)) => dispatch_from_args!(gistit::send, args),
        ("fetch", Some(args)) => dispatch_from_args!(gistit::fetch, args),
        _ => unimplemented!(),
    };
    Ok(())
}
