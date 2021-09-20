use std::sync::Arc;

use async_graphql::{Context, EmptySubscription, Object, Result, Schema};

use crate::{domains::counter::Counter, port::CounterRepository};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn find_counter(&self, ctx: &Context<'_>, name: String) -> Result<CounterResolver> {
        let counter_repo = ctx.data::<Arc<dyn CounterRepository>>()?;

        let counter = counter_repo.find_by_name(name).await?;

        Ok(CounterResolver { fetched: counter })
    }
}
struct CounterResolver {
    fetched: Counter,
}

#[Object]
impl CounterResolver {
    async fn count(&self) -> i32 {
        self.fetched.count
    }

    async fn name(&self) -> &str {
        &self.fetched.name
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn mutate_count(&self, ctx: &Context<'_>, name: String, count: i32) -> Result<bool> {
        let counter_repo = ctx.data::<Arc<dyn CounterRepository>>()?;

        Ok(counter_repo.mutate_count(name, count).await.is_ok())
    }
}

pub fn build_schema(
    counter_repo: Arc<dyn CounterRepository>,
) -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(counter_repo)
        .finish()
}
