use async_std::task;
use integration::respond_with_app;
use serde_wasm_bindgen as serde_wasm;
use server::build_app;
use worker::{wasm_bindgen::JsValue, *};

mod domains;
mod infra;
mod integration;
mod port;
mod server;
mod utils;

mod service;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    log::info!("worker started started");

    let _ = console_log::init_with_level(log::Level::Debug);

    log::info!("build app");
    let app = build_app(env);

    respond_with_app(app, req).await
}
