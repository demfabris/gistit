use console::style;
use gistit::cli::app;
use gistit::dispatch::Dispatch;
use gistit::{dispatch_from_args, Result};

async fn run(action: &mut String) -> Result<()> {
    let matches = app().get_matches();
    action.push_str(matches.subcommand().0);
    match matches.subcommand() {
        ("send", Some(args)) => dispatch_from_args!(gistit::send, args),
        ("fetch", Some(args)) => dispatch_from_args!(gistit::fetch, args),
        _ => unimplemented!(),
    };
    Ok(())
}

#[tokio::main]
async fn main() {
    let mut action = String::new();
    if let Err(err) = run(&mut action).await {
        eprintln!(
            "{}: Something went wrong during {}{}: \n{:?}",
            style("error").red().bold(),
            style("gistit-").green().bold(),
            style(action).green().bold(),
            err
        )
    };
}
