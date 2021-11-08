use gistit::cli::app;
use gistit::dispatch::Dispatch;
use gistit::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let matches = app().get_matches();
    match matches.subcommand() {
        // TODO: Improve this
        ("send", Some(args)) => {
            let action = gistit::send::Action::from_args(args)?;
            let payload = Dispatch::prepare(&*action).await?;
            Dispatch::dispatch(&*action, payload).await?;
        }
        ("fetch", Some(args)) => {
            let action = gistit::fetch::Action::from_args(args)?;
            let payload = Dispatch::prepare(&*action).await?;
            Dispatch::dispatch(&*action, payload).await?;
        }
        _ => unimplemented!(),
    };
    Ok(())
}
