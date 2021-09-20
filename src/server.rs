mod gql;
use async_std::sync::{Arc, RwLock};
use worker::*;

use crate::{infra::persistent::counter_adapter::CounterGraphProxy, integration, port};

pub fn build_app(env: Env) -> tide::Server<()> {
    let wrapped = crate::infra::persistent::Shared(env);

    let shared = Arc::new(RwLock::new(wrapped));

    let counter_repo: Arc<dyn port::CounterRepository> = Arc::new(CounterGraphProxy { shared });

    let mut app = tide::new();

    integration::app_with_gql(
        &mut app,
        String::from("/graphql"),
        gql::build_schema(counter_repo),
    );

    app
}
