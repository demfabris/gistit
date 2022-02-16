use url::Url;

use gistit_project::{env, var};
use lazy_static::lazy_static;

lazy_static! {
    static ref SERVER_URL_BASE: Url = Url::parse(var::GISTIT_SERVER_URL_BASE).unwrap();
}

const SERVER_SUBPATH_GET: &str = "get";
const SERVER_SUBPATH_LOAD: &str = "load";
const SERVER_SUBPATH_TOKEN: &str = "token";

lazy_static! {
    pub static ref SERVER_URL_GET: Url = Url::parse(
        &std::env::var(env::GISTIT_SERVER_URL)
            .unwrap_or_else(|_| var::GISTIT_SERVER_URL_BASE.to_owned())
    )
    .expect("invalid `GISTIT_SERVER_URL` variable")
    .join(SERVER_SUBPATH_GET)
    .unwrap();
    pub static ref SERVER_URL_LOAD: Url = Url::parse(
        &std::env::var(env::GISTIT_SERVER_URL)
            .unwrap_or_else(|_| var::GISTIT_SERVER_URL_BASE.to_owned())
    )
    .expect("invalid `GISTIT_SERVER_URL` variable")
    .join(SERVER_SUBPATH_LOAD)
    .unwrap();
    pub static ref SERVER_URL_TOKEN: Url = Url::parse(
        &std::env::var(env::GISTIT_SERVER_URL)
            .unwrap_or_else(|_| var::GISTIT_SERVER_URL_BASE.to_owned())
    )
    .expect("invalid `GISTIT_SERVER_URL` variable")
    .join(SERVER_SUBPATH_TOKEN)
    .unwrap();
}
