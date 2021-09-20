use std::{any, sync::Arc};

use async_std::{channel::bounded, sync::RwLock, task};
use async_trait::async_trait;
use graphql_client::GraphQLQuery;
use worker::{wasm_bindgen::JsValue, Headers, Request, RequestInit, Stub};

mod gql;
use crate::port::CounterRepository;

use super::Context;
use anyhow::Result;

pub struct CounterProxy {
    pub shared: Arc<RwLock<super::Shared<Context>>>,
}

pub struct CounterGraphProxy {
    pub shared: Arc<RwLock<super::Shared<Context>>>,
}

async fn get_stub(shared: Arc<RwLock<super::Shared<Context>>>, name: &str) -> Result<Stub> {
    let handle = shared.read().await;

    let ns = handle
        .0
        .durable_object("COUNTER")
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    log::info!("getting do_stub");
    ns.id_from_name(name)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?
        .get_stub()
        .map_err(|e| anyhow::anyhow!("{:?}", e))
}

#[async_trait]
impl crate::port::CounterRepository for CounterProxy {
    async fn create_by_name(&self, name: String) -> Result<crate::domains::counter::Counter> {
        self.find_by_name(name).await
    }

    async fn find_by_name(&self, name: String) -> Result<crate::domains::counter::Counter> {
        let shared = self.shared.clone();

        let (tx, rx) = bounded(1);

        let handle = task::spawn_local(async move {
            log::info!("getting do namespace");

            let stub = get_stub(shared, &name).await?;

            let cmd_path = super::counter_document::restful::Commands::GetCount.query_string()?;

            log::info!("receive response");
            let mut resp = stub
                .fetch_with_str(&cmd_path)
                .await
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;

            let body: super::counter_document::restful::Payload<i32> =
                resp.json().await.map_err(|e| anyhow::anyhow!("{:?}", e))?;

            let counter = crate::domains::counter::Counter {
                count: body.data,
                name,
            };

            let _ = tx.send(counter).await;
            let rv: Result<()> = Ok(());
            rv
        });

        handle.await?;

        let counter = rx.recv().await?;

        Ok(counter)
    }

    async fn mutate_count(&self, name: String, val: i32) -> Result<()> {
        let shared = self.shared.clone();

        let (tx, rx) = bounded(1);

        let handle = task::spawn_local(async move {
            log::info!("getting do namespace");

            let stub = get_stub(shared, &name).await?;

            let cmd_path = super::counter_document::restful::Commands::MutateCount(Some(val))
                .query_string()?;

            log::info!("{}", cmd_path);

            log::info!("receive response");
            let mut resp = stub
                .fetch_with_str(&cmd_path)
                .await
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;

            let body: super::counter_document::restful::Payload<bool> =
                resp.json().await.map_err(|e| anyhow::anyhow!("{:?}", e))?;

            let _ = tx.send(body.data).await;
            let rv: Result<()> = Ok(());
            rv
        });

        handle.await?;

        match rx.recv().await? {
            true => Ok(()),
            false => Err(anyhow::anyhow!("unable to mutate")),
        }
    }
}

#[async_trait]
impl CounterRepository for CounterGraphProxy {
    async fn create_by_name(&self, name: String) -> Result<crate::domains::counter::Counter> {
        self.find_by_name(name).await
    }

    async fn find_by_name(&self, name: String) -> Result<crate::domains::counter::Counter> {
        let shared = self.shared.clone();

        let handle = task::spawn_local(async move {
            let stub = get_stub(shared, &name)
                .await
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;

            let rv: Result<crate::domains::counter::Counter>;

            let vars = gql::get_counter::Variables;
            let query = gql::GetCounter::build_query(vars);

            let mut init = RequestInit::new();
            let query_bytes = serde_json::to_vec(&query)?;
            let js_bytes = js_sys::Uint8Array::from(&query_bytes[..]);
            let js_val = JsValue::from(&js_bytes);

            init.with_method(worker::Method::Post)
                .with_body(Some(js_val));

            let req = Request::new_with_init("/graphql", &init)
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
            let mut resp = stub
                .fetch_with_request(req)
                .await
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;

            let graph_body: graphql_client::Response<gql::get_counter::ResponseData> =
                resp.json().await.map_err(|e| anyhow::anyhow!("{:?}", e))?;

            log::debug!(target: "graph", "{:?}", &graph_body);

            if let Some(err) = graph_body.errors {
                let msg = anyhow::anyhow!("{}", serde_json::to_string(&err).unwrap());
                return Err(msg);
            }

            rv = graph_body
                .data
                .map(|v| crate::domains::counter::Counter {
                    count: v.counter.count as i32,
                    name: name.to_string(),
                })
                .ok_or_else(|| anyhow::anyhow!("missing data"));

            rv
        });

        handle.await
    }

    async fn mutate_count(&self, name: String, val: i32) -> Result<()> {
        todo!()
    }
}
