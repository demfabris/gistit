use std::env;
use std::process::Output;

use rand::{distributions::Alphanumeric, Rng};
use url::Url;

use crate::{Error, Result};

pub const GITHUB_OAUTH_CLIENT_ID: &str = "265cd618948a2e58042e";
pub const GITHUB_OAUTH_BASE_URL: &str = "https://github.com/login/oauth/authorize";

/// Attempts to open webbrowser and authorize GitHub OAuth
///
/// # Errors
///
/// Fails if cannot find a suitable browser or outside a display environment
pub fn request_oauth() -> Result<Output> {
    let url = Url::parse_with_params(
        GITHUB_OAUTH_BASE_URL,
        &[("client_id", GITHUB_OAUTH_CLIENT_ID), ("state", &entropy())],
    )?;

    // Can't open browser under ssh
    if env::var("SSH_CLIENT").is_err() {
        return Err(Error::OAuth(url.to_string()));
    }

    webbrowser::open(url.as_str()).map_err(|_| Error::OAuth(url.to_string()))
}

#[must_use]
pub fn entropy() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}
