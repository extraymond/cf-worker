use super::{ContextDo, Shared};
use crate::{
    integration::{self, respond_with_app},
    port::CounterDocument,
};
use anyhow::Result;
use async_std::sync::{Arc, RwLock};
use async_std::task;
use async_trait::async_trait;
use worker::*;

mod gql;
pub mod restful;

#[durable_object]
#[derive(Clone)]
pub struct Counter {
    shared: Arc<RwLock<Shared<ContextDo>>>,
}

#[durable_object]
impl DurableObject for Counter {
    fn new(state: State, env: Env) -> Self {
        let _ = console_log::init();
        log::info!("ready to go");
        let shared = Arc::new(RwLock::new(Shared((state, env))));

        Self { shared }
    }

    async fn fetch(&self, req: Request) -> worker::Result<Response> {
        let _ = console_log::init();
        log::info!("acquire data from Counter graphql interface");

        let repo: Arc<dyn CounterDocument> = Arc::new(Counter {
            shared: self.shared.clone(),
        });

        let mut app = tide::new();

        app.with(integration::CustomLogger);

        integration::app_with_gql(
            &mut app,
            String::from("/graphql"),
            gql::schema_builder(repo),
        );

        // let schema = gql::schema_builder(repo);
        // integration::app_with_gql(
        //     &mut app,
        //     String::from("/graphql"),
        //     gql::schema_builder(repo),
        // );

        // app.at(restful::Commands::GetCount.handler_path()).get(
        //     |req: tide::Request<Arc<dyn CounterDocument>>| async move {
        //         let repo = req.state().clone();

        //         let rs = task::spawn_local(async move {
        //             let count = repo.get_count().await.map_err(tide::Error::from_debug)?;

        //             let rv: tide::Result<_> = Ok(count);
        //             rv
        //         });

        //         let count = rs.await?;

        //         let builder = tide::Response::builder(tide::StatusCode::Ok);
        //         let payload = restful::Payload { data: count };
        //         let body = tide::Body::from_json(&payload)?;

        //         Ok(builder.body(body).build())
        //     },
        // );

        // app.at(restful::Commands::MutateCount(None).handler_path())
        //     .get(|req: tide::Request<Arc<dyn CounterDocument>>| async move {
        //         let repo = req.state().clone();

        //         let rs = task::spawn_local(async move {
        //             let count: i32 = req.param("value")?.parse()?;

        //             let rv = repo.modify_count(count).await.is_ok();

        //             let rv: tide::Result<bool> = Ok(rv);
        //             rv
        //         });

        //         let rv = rs.await?;

        //         let builder = tide::Response::builder(tide::StatusCode::Ok);
        //         let payload = restful::Payload { data: rv };
        //         let body = tide::Body::from_json(&payload)?;

        //         Ok(builder.body(body).build())
        //     });

        respond_with_app(app, req).await
    }
}

#[async_trait]
impl crate::port::CounterDocument for Counter {
    async fn get_count(&self) -> Result<i32> {
        let shared = self.shared.clone();

        let handle = task::spawn_local(async move {
            let inner = shared.read().await;

            let storage = inner.0 .0.storage();
            let rs = storage.get("count").await.unwrap_or(0);

            rs
        });

        Ok(handle.await)
    }

    async fn modify_count(&self, value: i32) -> Result<()> {
        let old = self.get_count().await?;

        let shared = self.shared.clone();

        let handle = task::spawn_local(async move {
            let inner = shared.read().await;
            let mut storage = inner.0 .0.storage();
            storage
                .put("count", old + value)
                .await
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;

            let rv: Result<()> = Ok(());
            rv
        });

        handle.await
    }
}
