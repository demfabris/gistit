use std::env;
use std::fs;
use std::io::Write;
use std::thread;
use std::time::Duration;

use rand::{distributions::Alphanumeric, Rng};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use url::Url;

use gistit_reference::project;

use crate::patch::webbrowser::{self, BrowserOptions};
use crate::server::SERVER_URL_TOKEN;
use crate::{Error, Result};

pub const GITHUB_OAUTH_CLIENT_ID: &str = "265cd618948a2e58042e";
pub const GITHUB_OAUTH_BASE_URL: &str = "https://github.com/login/oauth/authorize";
pub const GITHUB_GISTS_API_URL: &str = "https://api.github.com/gists";

#[derive(Clone, Debug, Serialize)]
pub struct Oauth {
    pub state: String,
    pub token: Option<Token>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateResponse {
    pub url: String,
    pub forks_url: String,
    pub commits_url: String,
    pub id: String,
    pub node_id: String,
    pub git_pull_url: String,
    pub git_push_url: String,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub description: String,
    pub comments: i32,
    pub comments_url: String,
}

impl Oauth {
    /// Looks for token in project config dir and initializes state.
    /// Will not fail if token file is missing.
    ///
    /// # Errors
    ///
    /// Fails if cannot read token file
    pub fn new() -> Result<Self> {
        let config = project::path::config()?;
        let token_path = config.join("github");
        let state = unguessable_state();

        let token = if fs::metadata(&token_path).is_ok() {
            Some(serde_json::from_str(&fs::read_to_string(&token_path)?)?)
        } else {
            None
        };

        Ok(Self { state, token })
    }

    /// Attempts to open a web browser and authorize GitHub OAuth
    ///
    /// # Errors
    ///
    /// Fails if cannot find a suitable browser or outside a display environment
    pub fn authorize(&self) -> Result<()> {
        let url = Url::parse_with_params(
            GITHUB_OAUTH_BASE_URL,
            &[
                ("client_id", GITHUB_OAUTH_CLIENT_ID),
                ("state", &self.state),
                ("scope", "gist"),
            ],
        )?;

        // Can't open browser under ssh
        if env::var("SSH_CLIENT").is_ok() {
            return Err(Error::OAuth(url.to_string()));
        }

        webbrowser::open_browser_with_options(BrowserOptions {
            url: String::from(url.as_str()),
            suppress_output: Some(true),
            browser: Some(webbrowser::Browser::Default),
        })
        .map_err(|_| Error::OAuth(url.to_string()))?;

        Ok(())
    }

    /// Polls server for authenticated token every 2 seconds
    ///
    /// # Errors
    ///
    /// Fails after 3 retries
    pub async fn poll_token(&mut self) -> Result<()> {
        let mut retry = 0_usize;
        let token: Token = loop {
            let response = reqwest::Client::new()
                .post(SERVER_URL_TOKEN.to_string())
                .json(self)
                .send()
                .await?;

            match response.status() {
                StatusCode::NOT_FOUND => {
                    if retry < 7 {
                        thread::sleep(Duration::from_secs(3));
                        retry += 1;
                    } else {
                        return Err(Error::OAuth("could not authorize".to_owned()));
                    }
                }
                StatusCode::OK => {
                    break response.json().await?;
                }
                _ => return Err(Error::Server("unexpected response")),
            }
        };

        let config = project::path::config()?;
        fs::File::create(config.join("github"))?.write_all(&serde_json::to_vec(&token)?)?;

        self.token = Some(token);

        Ok(())
    }

    #[must_use]
    pub const fn token(&self) -> Option<&Token> {
        self.token.as_ref()
    }
}

#[must_use]
pub fn unguessable_state() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}
