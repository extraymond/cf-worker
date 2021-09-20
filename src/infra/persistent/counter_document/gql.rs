use std::sync::Arc;

use crate::console_log;
use crate::port::CounterDocument;
use async_graphql::{extensions, Context, EmptySubscription, Object, Result, Schema};

struct Counter;

#[Object]
impl Counter {
    async fn count(&self, ctx: &Context<'_>) -> Result<i32> {
        let ns = ctx.data::<Arc<dyn CounterDocument>>()?;
        let count = ns.get_count().await?;

        Ok(count)
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn counter<'ctx>(&self) -> Counter {
        Counter
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn add_counter(&self, ctx: &Context<'_>, val: i32) -> Result<bool> {
        let ns = ctx.data::<Arc<dyn CounterDocument>>()?;
        ns.modify_count(val).await?;

        Ok(true)
    }

    async fn minus_counter(&self, ctx: &Context<'_>, val: i32) -> Result<bool> {
        let ns = ctx.data::<Arc<dyn CounterDocument>>()?;
        ns.modify_count(-val).await?;

        Ok(true)
    }
}

pub fn schema_builder(
    counter_repo: Arc<dyn CounterDocument>,
) -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(counter_repo)
        .finish()
}

#[test]
fn test_build_schema() {
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish();

    std::fs::write(
        "./src/infra/persistent/counter_adapter/schemas/schema.graphql",
        schema.sdl(),
    )
    .unwrap();
}
