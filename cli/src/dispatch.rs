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
