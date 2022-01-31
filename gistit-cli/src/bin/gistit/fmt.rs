#[macro_export]
macro_rules! errorln {
    ($err:expr) => {{
        use console::style;

        eprintln!(
            "{} {}",
            style("error:").red().bold(),
            $err
        );
    }};

    ($msg:literal, $($rest:expr),* $(,)*) => {{
        use console::style;

        let msg = format!($msg, $($rest,)*);
        println!("{} {}",
            style("error:").red().bold(),
            msg
        );
    }};
}

#[macro_export]
macro_rules! warnln {
    ($warn:expr) => {{
        use console::style;

        eprintln!(
            "{} {}",
            style("warning:").yellow().bold(),
            $warn
        );
    }};

    ($msg:literal, $($rest:expr),* $(,)*) => {{
        use console::style;

        let msg = format!($msg, $($rest,)*);
        println!("{} {}",
            style("warning:").yellow().bold(),
            msg
        );
    }};
}

#[macro_export]
macro_rules! prettyln {
    ($msg:expr) => {{
        use console::style;

        println!("{}", style($msg).green().bold());
    }};

    ($msg:literal, $($rest:expr),* $(,)*) => {{
        use console::style;

        let msg = format!($msg, $($rest,)*);
        println!("{}", style(msg).green().bold());
    }};
}
