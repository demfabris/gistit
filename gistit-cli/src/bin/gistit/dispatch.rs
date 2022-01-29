use async_trait::async_trait;

use crate::Result;

#[async_trait]
pub trait Dispatch {
    type InnerData;

    async fn prepare(&self) -> Result<Self::InnerData>;

    async fn dispatch(&self, payload: Self::InnerData) -> Result<()>;
}
