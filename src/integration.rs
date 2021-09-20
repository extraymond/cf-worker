use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    ObjectType, Schema, SubscriptionType,
};
use std::fmt::Debug;
use tide::http as http_types;
use worker::*;

use crate::integration::tide_gql_integration::endpoint;
mod tide_gql_integration;

async fn wreq_to_treq(mut req: Request) -> Result<http_types::Request> {
    let method = {
        match req.method() {
            Method::Head => http_types::Method::Head,
            Method::Get => http_types::Method::Get,
            Method::Post => http_types::Method::Post,
            Method::Put => http_types::Method::Put,
            Method::Patch => http_types::Method::Patch,
            Method::Delete => http_types::Method::Delete,
            Method::Options => http_types::Method::Options,
            Method::Connect => http_types::Method::Connect,
            Method::Trace => http_types::Method::Trace,
        }
    };

    let uri = req.url().map(|u| u.as_str().to_owned())?;
    let bytes = req.bytes().await?;

    let body = tide::Body::from_bytes(bytes);

    let mut t_req = http_types::Request::new(method, uri.as_str());

    for (name, value) in req.headers().into_iter() {
        t_req.insert_header(name.as_str(), value.as_str());
    }

    t_req.set_body(body);

    Ok(t_req)
}

async fn tresp_to_wrep(mut t_resp: http_types::Response) -> Result<worker::Response> {
    let bytes = t_resp
        .body_bytes()
        .await
        .map_err(|e| worker::Error::RustError(format!("{:?}", e)))?;
    log::info!("{:?}", serde_json::from_slice::<serde_json::Value>(&bytes));

    let mut resp: Response = {
        if let Ok(val) = serde_json::from_slice(&bytes) {
            Response::from_json::<serde_json::Value>(&val)?
        } else {
            Response::from_bytes(bytes)?
        }
    };

    let mut header = Headers::new();

    for key in t_resp.header_names() {
        header.append(key.as_str(), t_resp.header(key).unwrap().as_str())?;
    }

    resp = resp
        .with_headers(header)
        .with_status(t_resp.status() as u16);

    Ok(resp)
}

pub async fn respond_with_app<T: Sync + Send + Clone + 'static>(
    app: tide::Server<T>,
    req: Request,
) -> Result<Response> {
    let treq = wreq_to_treq(req).await?;

    let tresp = app
        .respond(treq)
        .await
        .map_err(|e| worker::Error::RustError(format!("{:?}", e)))?;

    Ok(tresp_to_wrep(tresp).await?)
}

pub struct CustomLogger;

#[async_trait::async_trait]
impl<State: Sync + Send + Clone + 'static + Debug> tide::Middleware<State> for CustomLogger {
    async fn handle(
        &self,
        request: tide::Request<State>,
        next: tide::Next<'_, State>,
    ) -> tide::Result {
        log::info!("{:?}", &request);

        Ok(next.run(request).await)
    }
}

pub fn app_with_gql<Q, M, S, T>(app: &mut tide::Server<T>, route: String, schema: Schema<Q, M, S>)
where
    T: Sync + Send + Clone + 'static,
    Q: ObjectType + 'static,
    M: ObjectType + 'static,
    S: SubscriptionType + 'static,
{
    let endpoint = endpoint(schema);

    app.at(&route).post(endpoint);
    app.at("/").get({
        move |_| {
            let r = route.clone();

            async move {
                let mut resp = tide::Response::new(tide::StatusCode::Ok);
                resp.set_body(tide::Body::from_string(playground_source(
                    GraphQLPlaygroundConfig::new(&r),
                )));
                resp.set_content_type(http_types::mime::HTML);
                Ok(resp)
            }
        }
    });
}
