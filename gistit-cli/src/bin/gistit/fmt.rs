use once_cell::sync::OnceCell;

use crate::Result;

pub static CURRENT_ACTION: OnceCell<String> = OnceCell::new();

pub fn set_action(action_name: &str) -> Result<()> {
    Ok(CURRENT_ACTION.set(action_name.to_owned())?)
}

#[macro_export]
macro_rules! errorln {
    ($err:expr) => {{
        use crate::fmt::CURRENT_ACTION;
        use console::style;

        eprintln!(
            "{}: Something went wrong during {}{}: \n    {}",
            style("error").red().bold(),
            style("gistit-").green().bold(),
            style(CURRENT_ACTION.get().unwrap_or(&"any".to_owned()))
                .green()
                .bold(),
            $err
        );
    }};
    ($msg:literal, $($rest:expr),* $(,)*) => {{
        use crate::fmt::CURRENT_ACTION;
        use console::style;

        let msg = format!($msg, $($rest,)*);
        println!("{}: Something went wrong during {}{}: \n    {}",
            style("error").red().bold(),
            style("gistit-").green().bold(),
            style(CURRENT_ACTION.get().unwrap_or("any"))
                .green()
                .bold(),
            msg
        );
    }};
}

#[macro_export]
macro_rules! warnln {
    ($warn:expr) => {{
        use crate::fmt::CURRENT_ACTION;
        use console::style;

        eprintln!(
            "{}: in {}{}: \n    {}",
            style("warning").yellow().bold(),
            style("gistit-").green().bold(),
            style(CURRENT_ACTION.get().unwrap_or(&"any".to_owned()))
                .green()
                .bold(),
            $warn
        );
    }};
    ($msg:literal, $($rest:expr),* $(,)*) => {{
        use crate::fmt::CURRENT_ACTION;
        use console::style;

        let msg = format!($msg, $($rest,)*);
        println!("{}: in {}{}: \n    {}",
            style("warning").yellow().bold(),
            style("gistit-").green().bold(),
            style(CURRENT_ACTION.get().unwrap_or(&"any".to_owned()))
                .green()
                .bold(),
            msg
        );
    }};
}

#[macro_export]
macro_rules! prettyln {
    ($msg:expr) => {{
        println!(
            "{}{}",
            console::Emoji("\u{2734}  ", "> "),
            console::style($msg).bold(),
        );
    }};
    ($msg:literal, $($rest:expr),* $(,)*) => {{
        let msg = format!($msg, $($rest,)*);
        println!("{}{}", console::Emoji("\u{2734}  ", "> "), console::style(msg).bold());
    }};
}
