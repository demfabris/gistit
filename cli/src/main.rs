use std::convert::TryFrom;

use gistit::cli::MainArgs;
use gistit::dispatch::Dispatch;
use gistit::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args: MainArgs = argh::from_env();
    dbg!("{}", &args);
    let action = gistit::send::Action::try_from(&args)?;
    let _prep = Dispatch::prepare(&action).await?;
    let _dispatch = Dispatch::dispatch(&action).await?;
    Ok(())
}
