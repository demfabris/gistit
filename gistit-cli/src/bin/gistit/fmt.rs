use std::sync::{Arc, Mutex};

use indicatif::{ProgressBar, ProgressStyle};

#[macro_export]
macro_rules! errorln {
    ($err:expr) => {{
        use console::style;

        eprintln!(
            "{}: {}",
            style("error").red().bold(),
            $err
        );
    }};

    ($msg:literal, $($rest:expr),* $(,)*) => {{
        use console::style;

        let msg = format!($msg, $($rest,)*);
        println!("{}: {}",
            style("error").red().bold(),
            msg
        );
    }};
}

#[macro_export]
macro_rules! warnln {
    ($warn:expr) => {{
        use console::style;
        use crate::fmt::PROGRESS;

        PROGRESS.println(format!( "{}: {}",
            style("warning").yellow().bold(),
            $warn
        ));
    }};

    ($msg:literal, $($rest:expr),* $(,)*) => {{
        use console::style;
        use crate::fmt::PROGRESS;

        let msg = format!($msg, $($rest,)*);
        PROGRESS.println(format!("{}: {}",
            style("warning").yellow().bold(),
            msg
        ));
    }};
}

#[macro_export]
macro_rules! progress {
    ($msg:expr) => {{
        use crate::fmt::{PROGRESS, STATUS};
        let mut status = STATUS.lock().unwrap();
        PROGRESS.set_message($msg);
        *status = Box::leak(Box::new($msg));
    }};

    ($msg:literal, $($rest:expr),* $(,)*) => {{
        use crate::fmt::{PROGRESS, STATUS};
        let mut status = STATUS.lock().unwrap();
        let msg = format!($msg, $($rest,)*);
        PROGRESS.set_message(msg.clone());
        *status = Box::leak(Box::new(msg));
    }};
}

#[macro_export]
macro_rules! updateln {
    ($msg:expr) => {{
        use console::{style, Emoji};
        use crate::fmt::PROGRESS;
        PROGRESS.println(format!("{} {}", style(Emoji("✔️ ", "> ")).green(), $msg));
    }};

    ($msg:literal, $($rest:expr),* $(,)*) => {{
        use crate::fmt::PROGRESS;
        use console::{style, Emoji};
        let msg = format!($msg, $($rest,)*);
        PROGRESS.println(format!("{} {}", style(Emoji("✔️ ", "> ")).green(), msg));
    }};
}

#[macro_export]
macro_rules! finish {
    ($msg:expr) => {{
        use crate::fmt::PROGRESS;

        PROGRESS.println(format!("{}", $msg));
        PROGRESS.finish_and_clear();
    }};
}

#[macro_export]
macro_rules! interruptln {
    () => {{
        use crate::fmt::{PROGRESS, STATUS};
        use console::{style, Emoji};
        let status = STATUS.lock().unwrap();

        PROGRESS.println(format!("{} {}", style(Emoji("❌", "x ")).red(), status));
        PROGRESS.finish_and_clear();
    }};
}

lazy_static::lazy_static! {
    pub static ref PROGRESS: ProgressBar = {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&[
                "◜",
                "◠",
                "◝",
                "◞",
                "◡",
                "◟",
                "✔️",
            ])
            .template("{spinner:.blue}  {msg}"),
        );
        pb.enable_steady_tick(100);
        pb
    };

    pub static ref STATUS: Arc<Mutex<&'static str>> = Arc::new(Mutex::new(""));
}
