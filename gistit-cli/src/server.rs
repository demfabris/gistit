use std::option_env;
use url::Url;

use lazy_static::lazy_static;

lazy_static! {
    static ref SERVER_URL_BASE: Url =
        Url::parse("https://us-central1-gistit-base.cloudfunctions.net/").unwrap();
}

const SERVER_SUBPATH_GET: &str = "get";
const SERVER_SUBPATH_LOAD: &str = "load";
const SERVER_SUBPATH_TOKEN: &str = "token";

lazy_static! {
    pub static ref SERVER_URL_GET: Url =
        Url::parse(option_env!("GISTIT_SERVER_URL").unwrap_or_else(|| SERVER_URL_BASE.as_str()))
            .expect("invalid `GISTIT_SERVER_URL` variable")
            .join(SERVER_SUBPATH_GET)
            .unwrap();
    pub static ref SERVER_URL_LOAD: Url =
        Url::parse(option_env!("GISTIT_SERVER_URL").unwrap_or_else(|| SERVER_URL_BASE.as_str()))
            .expect("invalid `GISTIT_SERVER_URL` variable")
            .join(SERVER_SUBPATH_LOAD)
            .unwrap();
    pub static ref SERVER_URL_TOKEN: Url =
        Url::parse(option_env!("GISTIT_SERVER_URL").unwrap_or_else(|| SERVER_URL_BASE.as_str()))
            .expect("invalid `GISTIT_SERVER_URL` variable")
            .join(SERVER_SUBPATH_TOKEN)
            .unwrap();
}
