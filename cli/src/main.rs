use std::convert::TryFrom;

use gistit::cli::app;
use gistit::dispatch::Dispatch;
use gistit::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let matches = app().get_matches();
    let action = match matches.subcommand() {
        ("send", Some(args)) => gistit::send::Action::try_from(args)?,
        ("fetch", args) => todo!(),
        _ => todo!(),
    };
    let payload = Dispatch::prepare(&action).await?;
    Dispatch::dispatch(&action, payload).await?;
    Ok(())
}
