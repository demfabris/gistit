//! The Dispatch trait

use crate::Result;

#[async_trait::async_trait]
pub trait Dispatch {
    /// The payload to be dispatched
    type Payload;
    /// Execute the action
    async fn dispatch(&self, payload: Self::Payload) -> Result<()>;
    /// Perform the checks needed
    async fn prepare(&self) -> Result<Self::Payload>;
}
