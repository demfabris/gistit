//! The Dispatch trait

use crate::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Dispatch {
    type InnerData;

    /// Perform the checks needed
    async fn prepare(&self) -> Result<Self::InnerData>;

    /// Execute the action
    async fn dispatch(&self, payload: Self::InnerData) -> Result<()>;
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
