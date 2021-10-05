//! The Dispatch trait

use crate::Result;

#[async_trait::async_trait]
pub trait Dispatch {
    /// Execute the action
    async fn dispatch(&self) -> Result<()>;
    /// Perform the checks needed
    async fn prepare(&self) -> Result<()>;
}
