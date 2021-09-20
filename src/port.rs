use async_trait::async_trait;

use crate::domains;
use anyhow::Result;

#[async_trait]
pub trait CounterRepository: Sync + Send {
    async fn create_by_name(&self, name: String) -> Result<domains::counter::Counter>;
    async fn find_by_name(&self, name: String) -> Result<domains::counter::Counter>;
    async fn mutate_count(&self, name: String, val: i32) -> Result<()>;
}

#[async_trait]
pub trait CounterDocument: Sync + Send {
    async fn get_count(&self) -> Result<i32>;
    async fn modify_count(&self, value: i32) -> Result<()>;
}
