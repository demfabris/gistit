//! The Dispatch trait

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::Result;

#[async_trait]
pub trait Dispatch {
    type InnerData;

    /// Perform the checks needed
    async fn prepare(&self) -> Result<Self::InnerData>;

    /// Execute the action
    async fn dispatch(&self, payload: Self::InnerData) -> Result<()>;
}

#[async_trait]
pub trait Hasheable {
    async fn hash(&self) -> Result<String>;
}

#[macro_export]
macro_rules! dispatch_from_args {
    ($mod:path, $args:expr) => {{
        use $mod as module;
        let action = module::Action::from_args($args)?;
        let payload = Dispatch::prepare(&*action).await?;
        Dispatch::dispatch(&*action, payload).await?;
    }};
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct GistitPayload {
    pub hash: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub colorscheme: String,
    pub lifespan: u16,
    pub secret: Option<String>,
    pub timestamp: String,
    pub gistit: GistitInner,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct GistitInner {
    pub name: String,
    pub lang: String,
    pub size: u64,
    pub data: String,
}
